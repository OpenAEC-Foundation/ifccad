//! Profile-owned closed marker values and canonical complete local references.
use crate::{line_pattern::*, *};
use serde_json::{json, Value};

pub(crate) fn encode_pattern_elements(
    elements: &[IfccadLinePatternElement],
    prefix: &str,
) -> Value {
    json!(elements.iter().map(|e| {
        let Some(marker)=&e.marker else {return json!(e.length);};
        let (mut v,t)=match marker {
            IfccadLinePatternMarker::Text {text,text_style_id,transform}=>(json!({"kind":"text","text":text,"textStyle":format!("{prefix}/textStyle/{}",text_style_id.0)}),transform),
            IfccadLinePatternMarker::Symbol {symbol_source_id,shape_number,transform}=>(json!({"kind":"symbol","symbolSource":format!("{prefix}/symbolSource/{}",symbol_source_id.0),"shapeNumber":shape_number}),transform),
        };
        if !super::defaults::is_one(t.scale) { v["scale"]=json!(t.scale); }
        if !t.offset.iter().all(|value| super::defaults::is_positive_zero(*value)) { v["offset"]=json!(t.offset); }
        match t.rotation {
            LinePatternRotation::Relative(angle) if super::defaults::is_positive_zero(angle) => {},
            LinePatternRotation::Relative(angle) => { v["rotation"]=json!({"mode":"relative","angle":angle}); },
            LinePatternRotation::Absolute(angle) => { v["rotation"]=json!({"mode":"absolute","angle":angle}); },
            LinePatternRotation::Upright(angle) => { v["rotation"]=json!({"mode":"upright","angle":angle}); },
        }
        json!({"length":e.length,"marker":v})
    }).collect::<Vec<_>>())
}

fn local_id(value: &Value, prefix: &str, role: &str) -> Option<u64> {
    let raw = value.as_str()?.strip_prefix(&format!("{prefix}/{role}/"))?;
    let id = raw.parse::<u64>().ok()?;
    (raw == id.to_string()).then_some(id)
}
pub(crate) fn decode_pattern_elements(
    value: &Value,
    prefix: &str,
    location: &str,
) -> Result<Vec<IfccadLinePatternElement>, IfccadReport> {
    let failure = |i| {
        crate::ifccad::diagnostics::failure(
            "IFCCAD-PATTERN-002",
            &format!("{location}/{i}"),
            "invalid physical marker fields or local reference path",
        )
    };
    value
        .as_array()
        .ok_or_else(|| failure(0))?
        .iter()
        .enumerate()
        .map(|(i, v)| decode_element(v, prefix).ok_or_else(|| failure(i)))
        .collect()
}
fn decode_element(value: &Value, prefix: &str) -> Option<IfccadLinePatternElement> {
    if let Some(length) = value.as_f64() {
        return Some(LinePatternElement::simple(length));
    }
    let m = value.get("marker")?;
    let scale = match m.get("scale") {
        None => 1.,
        Some(v) => v.as_f64()?,
    };
    let offset = match m.get("offset") {
        None => [0.; 2],
        Some(v) => [v.get(0)?.as_f64()?, v.get(1)?.as_f64()?],
    };
    let rotation = match m.get("rotation") {
        None => LinePatternRotation::Relative(0.),
        Some(v) => match v.get("mode")?.as_str()? {
            "relative" => LinePatternRotation::Relative(v.get("angle")?.as_f64()?),
            "absolute" => LinePatternRotation::Absolute(v.get("angle")?.as_f64()?),
            "upright" => LinePatternRotation::Upright(v.get("angle")?.as_f64()?),
            _ => return None,
        },
    };
    let transform = LinePatternTransform {
        scale,
        offset,
        rotation,
    };
    let marker = match m.get("kind")?.as_str()? {
        "text" => IfccadLinePatternMarker::Text {
            text: m.get("text")?.as_str()?.into(),
            text_style_id: IfccadTextStyleId(local_id(m.get("textStyle")?, prefix, "textStyle")?),
            transform,
        },
        "symbol" => IfccadLinePatternMarker::Symbol {
            symbol_source_id: IfccadSymbolSourceId(local_id(
                m.get("symbolSource")?,
                prefix,
                "symbolSource",
            )?),
            shape_number: u32::try_from(m.get("shapeNumber")?.as_u64()?).ok()?,
            transform,
        },
        _ => return None,
    };
    Some(LinePatternElement {
        length: value.get("length")?.as_f64()?,
        marker: Some(marker),
    })
}
pub(crate) fn encode_source(value: &SymbolSource) -> Value {
    match value {
        SymbolSource::Shx { file_name } => json!({"kind":"shx","fileName":file_name}),
    }
}
pub(crate) fn decode_source(value: &Value, location: &str) -> Result<SymbolSource, IfccadReport> {
    super::supplemental::validate_value("ifccad::symbolSource", value, location)?;
    let Some(file_name) = value.get("fileName").and_then(Value::as_str) else {
        return Err(crate::ifccad::diagnostics::failure(
            "IFCCAD-SYMBOL-001",
            location,
            "source filename required",
        ));
    };
    Ok(SymbolSource::Shx {
        file_name: file_name.into(),
    })
}
