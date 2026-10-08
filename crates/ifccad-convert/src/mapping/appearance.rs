use crate::diagnostics::{diagnostic, modification};
use crate::IfccadDiagnostic;
use ocdraw::ifccad::{IfccadColor, IfccadEntityAppearance, IfccadLayerAppearance, IfccadMode};
use opencadcodec::entities::EntityCommon;
use opencadcodec::{Color, Layer, LineWeight, Transparency};

pub(super) fn native_color(value: cad_presentation_convert::CadColorValue) -> IfccadColor {
    IfccadColor {
        rgb: value.rgb,
        indexed: value.indexed,
        named: value.named,
    }
}
pub(super) fn scalar_color(value: &IfccadColor) -> cad_presentation_convert::CadColorValue {
    cad_presentation_convert::CadColorValue {
        rgb: value.rgb,
        indexed: value.indexed.clone(),
        named: value.named.clone(),
    }
}
pub(super) fn cad_color(
    value: &IfccadColor,
    loc: &str,
    issues: &mut Vec<IfccadDiagnostic>,
) -> cad_presentation_convert::CadColorMapping {
    let mapped = cad_presentation_convert::color_to_cad(&scalar_color(value))
        .expect("validated native color");
    for loss in &mapped.losses {
        issues.push(modification(
            "appearance",
            format!("{loc}.color"),
            format!("indexed color identity omitted ({loss:?}); RGB retained"),
        ));
    }
    mapped
}
pub(super) fn cad_weight(value: f64, loc: &str, issues: &mut Vec<IfccadDiagnostic>) -> LineWeight {
    let mapped =
        cad_presentation_convert::lineweight_to_cad(value).expect("validated native lineweight");
    if mapped.changed {
        issues.push(modification(
            "appearance",
            format!("{loc}.line_weight"),
            format!(
                "lineweight {value} mm quantized to {} mm",
                mapped.roundtrip_mm
            ),
        ));
    }
    mapped.weight
}
pub(super) fn cad_opacity(
    value: f64,
    loc: &str,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Transparency {
    let mapped = cad_presentation_convert::opacity_to_cad(value).expect("validated native opacity");
    if mapped.changed {
        issues.push(modification(
            "appearance",
            format!("{loc}.opacity"),
            format!("opacity {value} quantized to {}", mapped.roundtrip),
        ));
    }
    mapped.transparency
}
fn source_color(
    value: Color,
    loc: &str,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Option<IfccadColor> {
    match cad_presentation_convert::explicit_color_from_cad(value) {
        Ok(value) => Some(native_color(value)),
        Err(_) => {
            issues.push(diagnostic(
                "appearance",
                format!("{loc}.color"),
                "unsupported required concrete color; dependent item omitted",
            ));
            None
        }
    }
}
fn source_weight(value: LineWeight, loc: &str, issues: &mut Vec<IfccadDiagnostic>) -> Option<f64> {
    match value {
        LineWeight::Default => Some(0.25),
        LineWeight::Value(v) if v >= 0 => Some(f64::from(v) / 100.0),
        _ => {
            issues.push(diagnostic(
                "appearance",
                format!("{loc}.line_weight"),
                "unsupported required concrete lineweight; dependent item omitted",
            ));
            None
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
        Explicit(v) => {
            let mapped = cad_color(v, loc, issues);
            if let Some((catalog, name)) = mapped.named {
                if catalog.contains('$') {
                    issues.push(modification("appearance",format!("{loc}.color"),"named catalog contains entity-name delimiter; RGB/index retained without named identity"));
                } else {
                    c.color_name = Some(format!("{catalog}${name}"));
                }
            }
            mapped.color
        }
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
    patterns: &crate::mapping::line_pattern::SourcePatterns,
    loc: &str,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Option<IfccadEntityAppearance> {
    use IfccadMode::*;
    let mut color = match c.color {
        Color::ByLayer => ByLayer,
        Color::ByBlock => ByBlock,
        v => Explicit(source_color(v, loc, issues)?),
    };
    if let (Explicit(target), Some(name)) = (&mut color, &c.color_name) {
        if let Some((catalog, name)) = name
            .split_once('$')
            .filter(|(catalog, name)| !catalog.is_empty() && !name.is_empty())
        {
            target.named = Some((catalog.to_owned(), name.to_owned()));
        }
    }
    let opacity = match c.transparency {
        Transparency::ByLayer => ByLayer,
        Transparency::ByBlock => ByBlock,
        Transparency::Explicit(a) => Explicit(1. - f64::from(a) / 255.),
    };
    let line_weight = match c.line_weight {
        LineWeight::ByLayer => ByLayer,
        LineWeight::ByBlock => ByBlock,
        v => Explicit(source_weight(v, loc, issues)?),
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
    if matches!(&color, Explicit(value) if value.named.is_some()) {
        r.color_name = None;
    }
    r.line_weight = b.line_weight;
    r.linetype = b.linetype.clone();
    r.transparency = b.transparency;
    r.linetype_handle = None;
    r.linetype_scale = b.linetype_scale;
    r.invisible = b.invisible;
    r.entity_mode = None;
    r.raw_record = None;
    r.has_ds_data = false;
    if r != b {
        issues.push(diagnostic(
            "entity-common",
            loc,
            "unsupported common properties omitted (including XDATA, style or material)",
        ));
    }
    Some(IfccadEntityAppearance {
        color,
        opacity,
        line_pattern,
        line_weight,
    })
}
pub(crate) fn to_layer(
    name: &str,
    a: &IfccadLayerAppearance,
    pattern_name: &str,
    loc: &str,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Layer {
    let mut layer = Layer::new(name);
    let mapped = cad_color(&a.color, loc, issues);
    layer.color = mapped.color;
    if let Some((catalog, name)) = mapped.named {
        if catalog.contains('$') {
            issues.push(modification("appearance",format!("{loc}.color"),"named catalog contains the DXF color-book delimiter; concrete color retained without named identity"));
        } else {
            layer.book_name = Some(catalog);
            layer.color_name = Some(name);
        }
    }
    layer.transparency = cad_opacity(a.opacity, loc, issues);
    layer.line_weight = cad_weight(a.line_weight, loc, issues);
    layer.line_type = pattern_name.into();
    layer
}
pub(crate) fn from_layer(
    layer: &Layer,
    patterns: &crate::mapping::line_pattern::SourcePatterns,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Option<IfccadLayerAppearance> {
    let loc = format!("layer/{}", layer.name);
    let mut color = source_color(layer.color, &loc, issues)?;
    match (&layer.book_name, &layer.color_name) {
        (Some(catalog), Some(name)) if !catalog.is_empty() && !name.is_empty() => {
            color.named = Some((catalog.clone(), name.clone()));
        }
        (None, None) => (),
        _ => issues.push(modification(
            "appearance",
            format!("{loc}.color"),
            "incomplete named-color identity omitted; concrete RGB/index retained",
        )),
    }
    let opacity = match layer.transparency {
        Transparency::Explicit(a) => 1. - f64::from(a) / 255.,
        // The codec uses this layer mode for the opaque default, not inheritance.
        Transparency::ByLayer => 1.,
        v => {
            issues.push(diagnostic(
                "appearance",
                format!("{loc}.opacity"),
                format!("unsupported required layer opacity {v:?}; layer omitted"),
            ));
            return None;
        }
    };
    let line_weight = source_weight(layer.line_weight, &loc, issues)?;
    let line_pattern = patterns.layer(&layer.line_type);
    Some(IfccadLayerAppearance {
        color,
        opacity,
        line_weight,
        line_pattern,
    })
}
