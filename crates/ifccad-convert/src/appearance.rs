use crate::diagnostics::{diagnostic, modification};
use crate::IfccadDiagnostic;
use ocdraw::ifccad::{IfccadEntityAppearance, IfccadLayerAppearance, IfccadMode};
use opencadcodec::entities::EntityCommon;
use opencadcodec::{Color, Layer, LineWeight, Transparency};

const WEIGHTS: [i16; 24] = [
    0, 5, 9, 13, 15, 18, 20, 25, 30, 35, 40, 50, 53, 60, 70, 80, 90, 100, 106, 120, 140, 158, 200,
    211,
];

pub(crate) fn color(value: &str) -> Color {
    Color::from_rgb(
        u8::from_str_radix(&value[1..3], 16).unwrap(),
        u8::from_str_radix(&value[3..5], 16).unwrap(),
        u8::from_str_radix(&value[5..7], 16).unwrap(),
    )
}
pub(crate) fn weight(value: f64) -> Option<LineWeight> {
    WEIGHTS
        .into_iter()
        .find(|v| f64::from(*v) / 100. == value)
        .map(LineWeight::Value)
}
pub(crate) fn opacity(value: f64) -> Option<Transparency> {
    (0..=255)
        .find(|a| 1. - f64::from(*a) / 255. == value)
        .map(Transparency::Explicit)
}
fn cad_weight(value: f64, loc: &str, issues: &mut Vec<IfccadDiagnostic>) -> LineWeight {
    if let Some(w) = weight(value) {
        return w;
    }
    let nearest = WEIGHTS
        .into_iter()
        .min_by(|a, b| {
            (f64::from(*a) / 100. - value)
                .abs()
                .total_cmp(&(f64::from(*b) / 100. - value).abs())
        })
        .unwrap();
    issues.push(modification(
        "appearance",
        format!("{loc}.line_weight"),
        format!(
            "line weight {value} mm rounded to {} mm",
            f64::from(nearest) / 100.
        ),
    ));
    LineWeight::Value(nearest)
}
fn cad_opacity(value: f64, loc: &str, issues: &mut Vec<IfccadDiagnostic>) -> Transparency {
    if let Some(t) = opacity(value) {
        return t;
    }
    let nearest = (0u8..=255)
        .min_by(|a, b| {
            ((1. - f64::from(*a) / 255.) - value)
                .abs()
                .total_cmp(&((1. - f64::from(*b) / 255.) - value).abs())
                .then_with(|| b.cmp(a))
        })
        .unwrap();
    issues.push(modification(
        "appearance",
        format!("{loc}.opacity"),
        format!(
            "opacity {value} quantized to {} (transparency byte {nearest})",
            1. - f64::from(nearest) / 255.
        ),
    ));
    Transparency::Explicit(nearest)
}
fn source_color(value: Color, loc: &str, issues: &mut Vec<IfccadDiagnostic>) -> String {
    let (r, g, b) = value.rgb().unwrap_or((255, 255, 255));
    let target = format!("#{r:02X}{g:02X}{b:02X}");
    if !matches!(value, Color::Rgb { .. }) {
        issues.push(modification("appearance",format!("{loc}.color"),format!("color {value:?} mapped to {target}; indexed/inherited/none identity is not retained")));
    }
    target
}
fn source_weight(value: LineWeight, loc: &str, issues: &mut Vec<IfccadDiagnostic>) -> f64 {
    match value {
        LineWeight::Value(v) if v >= 0 => cad_weight(f64::from(v) / 100., loc, issues)
            .millimeters()
            .unwrap(),
        _ => {
            issues.push(modification("appearance",format!("{loc}.line_weight"),format!("weight {value:?} replaced with explicit 0.25 mm; no drawing-default weight is available in this profile")));
            0.25
        }
    }
}
pub(crate) fn to_common(
    a: &IfccadEntityAppearance,
    pattern: Option<(String, opencadcodec::Handle)>,
    layer: &str,
    loc: &str,
    issues: &mut Vec<IfccadDiagnostic>,
) -> EntityCommon {
    use IfccadMode::*;
    let mut c = EntityCommon::with_layer(layer);
    c.color = match &a.color {
        ByLayer => Color::ByLayer,
        ByBlock => Color::ByBlock,
        Explicit(v) => color(v),
    };
    c.transparency = match a.opacity {
        ByLayer => Transparency::ByLayer,
        ByBlock => Transparency::ByBlock,
        Explicit(v) => cad_opacity(v, loc, issues),
    };
    c.line_weight = match a.line_weight {
        ByLayer => LineWeight::ByLayer,
        ByBlock => LineWeight::ByBlock,
        Explicit(v) => cad_weight(v, loc, issues),
    };
    c.linetype = match &a.line_pattern {
        ByLayer => String::new(),
        ByBlock => "ByBlock".into(),
        Explicit(_) => pattern.as_ref().unwrap().0.clone(),
    };
    c.linetype_handle = pattern.map(|p| p.1);
    c
}
pub(crate) fn from_common(
    c: &EntityCommon,
    patterns: &crate::patterns::SourcePatterns,
    loc: &str,
    issues: &mut Vec<IfccadDiagnostic>,
) -> IfccadEntityAppearance {
    use IfccadMode::*;
    let color = match c.color {
        Color::ByLayer => ByLayer,
        Color::ByBlock => ByBlock,
        v => Explicit(source_color(v, loc, issues)),
    };
    let opacity = match c.transparency {
        Transparency::ByLayer => ByLayer,
        Transparency::ByBlock => ByBlock,
        Transparency::Explicit(a) => Explicit(1. - f64::from(a) / 255.),
    };
    let line_weight = match c.line_weight {
        LineWeight::ByLayer => ByLayer,
        LineWeight::ByBlock => ByBlock,
        v => Explicit(source_weight(v, loc, issues)),
    };
    let line_pattern = patterns.entity(c);
    // Typed comparison also covers public fields skipped by serde.
    let mut r = c.clone();
    let b = EntityCommon::new();
    r.handle = b.handle;
    r.owner_handle = b.owner_handle;
    r.layer = b.layer.clone();
    // source::inspect has qualified the handle/name association; native layers
    // retain that association with newly constructed CAD identities.
    r.layer_handle = None;
    r.color = b.color;
    r.line_weight = b.line_weight;
    r.linetype = b.linetype.clone();
    r.transparency = b.transparency;
    r.linetype_handle = None;
    r.linetype_scale = b.linetype_scale;
    r.entity_mode = None;
    r.raw_record = None;
    r.has_ds_data = false;
    if r != b {
        issues.push(diagnostic("entity-common",loc,"unsupported common properties omitted (including XDATA, style, visibility or material)"));
    }
    IfccadEntityAppearance {
        color,
        opacity,
        line_pattern,
        line_weight,
    }
}
pub(crate) fn to_layer(
    name: &str,
    a: &IfccadLayerAppearance,
    pattern_name: &str,
    loc: &str,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Layer {
    let mut layer = Layer::new(name);
    layer.color = color(&a.color);
    layer.transparency = cad_opacity(a.opacity, loc, issues);
    layer.line_weight = cad_weight(a.line_weight, loc, issues);
    layer.line_type = pattern_name.into();
    layer
}
pub(crate) fn from_layer(
    layer: &Layer,
    patterns: &crate::patterns::SourcePatterns,
    issues: &mut Vec<IfccadDiagnostic>,
) -> IfccadLayerAppearance {
    let loc = format!("layer/{}", layer.name);
    let color = source_color(layer.color, &loc, issues);
    let opacity = match layer.transparency {
        Transparency::Explicit(a) => 1. - f64::from(a) / 255.,
        v => {
            issues.push(modification(
                "appearance",
                format!("{loc}.opacity"),
                format!("layer opacity {v:?} replaced with explicit 1"),
            ));
            1.
        }
    };
    let line_weight = source_weight(layer.line_weight, &loc, issues);
    let line_pattern = patterns.layer(&layer.line_type);
    IfccadLayerAppearance {
        color,
        opacity,
        line_weight,
        line_pattern,
    }
}
