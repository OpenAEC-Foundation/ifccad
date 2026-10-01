use crate::outcome::diagnostic;
use crate::IfcxCadDiagnostic;
use cadcodec::entities::EntityCommon;
use cadcodec::{Color, Layer, LineWeight, Transparency};
use ocdraw::ifcx_cad::IfcxCadLayerAppearance;
use ocdraw::ifcx_cad::{IfcxCadEntityAppearance, IfcxCadMode};

pub(crate) fn color(value: &str) -> Color {
    Color::from_rgb(
        u8::from_str_radix(&value[1..3], 16).unwrap(),
        u8::from_str_radix(&value[3..5], 16).unwrap(),
        u8::from_str_radix(&value[5..7], 16).unwrap(),
    )
}
pub(crate) fn to_common(
    a: &IfcxCadEntityAppearance,
    layer: &str,
    loc: &str,
    issues: &mut Vec<IfcxCadDiagnostic>,
) -> EntityCommon {
    use IfcxCadMode::*;
    let mut c = EntityCommon::with_layer(layer);
    c.color = match &a.color {
        ByLayer => Color::ByLayer,
        ByBlock => Color::ByBlock,
        Explicit(v) => color(v),
    };
    c.transparency = match a.opacity {
        ByLayer => Transparency::ByLayer,
        ByBlock => Transparency::ByBlock,
        Explicit(v) => opacity(v).unwrap_or_else(|| {
            issues.push(diagnostic(
                "appearance",
                loc,
                "opacity is not exactly representable",
            ));
            Transparency::OPAQUE
        }),
    };
    c.line_weight = match a.line_weight {
        ByLayer => LineWeight::ByLayer,
        ByBlock => LineWeight::ByBlock,
        Explicit(v) => weight(v).unwrap_or_else(|| {
            issues.push(diagnostic(
                "appearance",
                loc,
                "line weight is not exactly representable",
            ));
            LineWeight::Value(25)
        }),
    };
    c.linetype = match &a.line_pattern {
        ByLayer => String::new(),
        ByBlock => "ByBlock".into(),
        Explicit(v) => {
            if v != "Continuous" {
                issues.push(diagnostic(
                    "appearance",
                    loc,
                    "only Continuous is supported",
                ));
            }
            v.clone()
        }
    };
    c
}
pub(crate) fn from_common(
    c: &EntityCommon,
    loc: &str,
    issues: &mut Vec<IfcxCadDiagnostic>,
) -> IfcxCadEntityAppearance {
    use IfcxCadMode::*;
    let color = match c.color {
        Color::ByLayer => ByLayer,
        Color::ByBlock => ByBlock,
        v => Explicit(rgb(v).unwrap_or_else(|| {
            issues.push(diagnostic(
                "appearance",
                loc,
                "indexed color identity is unsupported",
            ));
            "#FFFFFF".into()
        })),
    };
    let opacity = match c.transparency {
        Transparency::ByLayer => ByLayer,
        Transparency::ByBlock => ByBlock,
        Transparency::Explicit(a) => Explicit(1. - f64::from(a) / 255.),
    };
    let line_weight = match c.line_weight {
        LineWeight::ByLayer => ByLayer,
        LineWeight::ByBlock => ByBlock,
        LineWeight::Value(v) if weight(f64::from(v) / 100.).is_some() => {
            Explicit(f64::from(v) / 100.)
        }
        _ => {
            issues.push(diagnostic(
                "appearance",
                loc,
                "default or unsupported line weight",
            ));
            Explicit(0.25)
        }
    };
    let line_pattern = if c.linetype.is_empty() || c.linetype.eq_ignore_ascii_case("ByLayer") {
        ByLayer
    } else if c.linetype.eq_ignore_ascii_case("ByBlock") {
        ByBlock
    } else {
        if c.linetype != "Continuous" {
            issues.push(diagnostic(
                "appearance",
                loc,
                "only Continuous is supported",
            ));
        }
        Explicit(c.linetype.clone())
    };
    // Compare skipped-serde fields as well. Only mapped values, identity and
    // storage caches are reset; authored common semantics remain detectable.
    let mut r = c.clone();
    let b = EntityCommon::new();
    r.handle = b.handle;
    r.owner_handle = b.owner_handle;
    r.layer = b.layer.clone();
    r.color = b.color;
    r.line_weight = b.line_weight;
    r.linetype = b.linetype.clone();
    r.transparency = b.transparency;
    r.linetype_handle = None;
    r.entity_mode = None;
    r.raw_record = None;
    r.has_ds_data = false;
    if r != b {
        issues.push(diagnostic(
            "entity-common",
            loc,
            "unsupported common properties (including XDATA, style, visibility or material)",
        ));
    }
    IfcxCadEntityAppearance {
        color,
        opacity,
        line_pattern,
        line_weight,
    }
}
pub(crate) fn rgb(value: Color) -> Option<String> {
    match value {
        Color::Rgb { r, g, b } => Some(format!("#{r:02X}{g:02X}{b:02X}")),
        _ => None,
    }
}
pub(crate) fn opacity(value: f64) -> Option<Transparency> {
    // Select a byte by its decoded value; the codec's percent constructor can
    // otherwise ceil a value one byte further because of floating-point error.
    (0..=255)
        .find(|a| 1. - f64::from(*a) / 255. == value)
        .map(Transparency::Explicit)
}
pub(crate) fn weight(value: f64) -> Option<LineWeight> {
    [
        0, 5, 9, 13, 15, 18, 20, 25, 30, 35, 40, 50, 53, 60, 70, 80, 90, 100, 106, 120, 140, 158,
        200, 211,
    ]
    .into_iter()
    .find(|v| f64::from(*v) / 100. == value)
    .map(LineWeight::Value)
}
pub(crate) fn to_layer(
    name: &str,
    a: &IfcxCadLayerAppearance,
    location: &str,
    issues: &mut Vec<IfcxCadDiagnostic>,
) -> Layer {
    let mut layer = Layer::new(name);
    layer.color = color(&a.color);
    match opacity(a.opacity) {
        Some(t) => layer.transparency = t,
        None => issues.push(diagnostic(
            "appearance",
            location,
            "opacity is not an exactly representable CAD byte",
        )),
    }
    match weight(a.line_weight) {
        Some(w) => layer.line_weight = w,
        None => issues.push(diagnostic(
            "appearance",
            location,
            "line weight is not a supported CAD hundredth-mm value",
        )),
    }
    if a.line_pattern != "Continuous" {
        issues.push(diagnostic(
            "appearance",
            location,
            "only Continuous is supported",
        ));
    }
    layer
}
pub(crate) fn from_layer(
    layer: &Layer,
    issues: &mut Vec<IfcxCadDiagnostic>,
) -> IfcxCadLayerAppearance {
    let loc = format!("layer/{}", layer.name);
    let color = rgb(layer.color).unwrap_or_else(|| {
        issues.push(diagnostic(
            "appearance",
            &loc,
            "indexed or inherited layer color is not represented",
        ));
        "#FFFFFF".into()
    });
    let opacity = match layer.transparency {
        Transparency::Explicit(a) => 1. - f64::from(a) / 255.,
        _ => {
            issues.push(diagnostic(
                "appearance",
                &loc,
                "inherited layer opacity is unsupported",
            ));
            1.
        }
    };
    let line_weight = match layer.line_weight {
        LineWeight::Value(v) if weight(f64::from(v) / 100.).is_some() => f64::from(v) / 100.,
        _ => {
            issues.push(diagnostic(
                "appearance",
                &loc,
                "default/inherited or invalid layer weight is unsupported",
            ));
            0.25
        }
    };
    if layer.line_type != "Continuous" {
        issues.push(diagnostic(
            "appearance",
            &loc,
            "only Continuous is supported",
        ));
    }
    IfcxCadLayerAppearance {
        color,
        opacity,
        line_weight,
        line_pattern: layer.line_type.clone(),
    }
}
