//! Closed JSON pattern values; identities use exact integer backings.
use crate::line_pattern::{
    LinePatternElement, LinePatternRotation, LinePatternTransform, SymbolSource,
};
use crate::ocdraw::{
    DrawingLinePatternElement, DrawingLinePatternMarker, DrawingSymbolSource, OcdrawDiagnostic,
    OcdrawTextStyleId, SymbolSourceId,
};
use serde_json::{json, Value};

pub(crate) fn encode_pattern_elements(elements: &[DrawingLinePatternElement]) -> Value {
    Value::Array(elements.iter().map(|element| {
        let Some(marker) = &element.marker else { return json!(element.length); };
        let (mut value, transform) = match marker {
            DrawingLinePatternMarker::Text {text, text_style_id, transform} => (json!({"kind":"text","text":text,"textStyleId":text_style_id.0}), transform),
            DrawingLinePatternMarker::Symbol {symbol_source_id, shape_number, transform} => (json!({"kind":"symbol","symbolSourceId":symbol_source_id.0,"shapeNumber":shape_number}), transform),
        };
        value["scale"] = json!(transform.scale);
        value["offset"] = json!(transform.offset);
        value["rotation"] = match transform.rotation {
            LinePatternRotation::Relative(angle) => json!({"mode":"relative","angle":angle}),
            LinePatternRotation::Absolute(angle) => json!({"mode":"absolute","angle":angle}),
        };
        json!({"length":element.length,"marker":value})
    }).collect())
}

fn failure(location: &str) -> OcdrawDiagnostic {
    OcdrawDiagnostic {
        code: "LINE_PATTERN_ENCODING",
        location: location.into(),
        message: "pattern fields need exact physical types and registered marker variants".into(),
    }
}
fn id(value: &Value) -> Option<u32> {
    u32::try_from(value.as_u64()?).ok()
}

pub(crate) fn decode_pattern_elements(
    value: &Value,
    location: &str,
) -> Result<Vec<DrawingLinePatternElement>, OcdrawDiagnostic> {
    value
        .as_array()
        .ok_or_else(|| failure(location))?
        .iter()
        .enumerate()
        .map(|(i, value)| decode_element(value).ok_or_else(|| failure(&format!("{location}/{i}"))))
        .collect()
}
fn decode_element(value: &Value) -> Option<DrawingLinePatternElement> {
    if let Some(length) = value.as_f64() {
        return Some(LinePatternElement::simple(length));
    }
    let marker = value.get("marker")?;
    let defaults = LinePatternTransform::default();
    let scale = match marker.get("scale") {
        None => defaults.scale,
        Some(v) => v.as_f64()?,
    };
    let offset = match marker.get("offset") {
        None => defaults.offset,
        Some(v) => [v.get(0)?.as_f64()?, v.get(1)?.as_f64()?],
    };
    let rotation = match marker.get("rotation") {
        None => defaults.rotation,
        Some(v) => match v.get("mode")?.as_str()? {
            "relative" => LinePatternRotation::Relative(v.get("angle")?.as_f64()?),
            "absolute" => LinePatternRotation::Absolute(v.get("angle")?.as_f64()?),
            _ => return None,
        },
    };
    let transform = LinePatternTransform {
        scale,
        offset,
        rotation,
    };
    let marker = match marker.get("kind")?.as_str()? {
        "text" => DrawingLinePatternMarker::Text {
            text: marker.get("text")?.as_str()?.into(),
            text_style_id: OcdrawTextStyleId(id(marker.get("textStyleId")?)?),
            transform,
        },
        "symbol" => DrawingLinePatternMarker::Symbol {
            symbol_source_id: SymbolSourceId(id(marker.get("symbolSourceId")?)?),
            shape_number: id(marker.get("shapeNumber")?)?,
            transform,
        },
        _ => return None,
    };
    Some(LinePatternElement {
        length: value.get("length")?.as_f64()?,
        marker: Some(marker),
    })
}
pub(crate) fn encode_symbol_sources(rows: &[DrawingSymbolSource]) -> Value {
    Value::Array(
        rows.iter()
            .map(|row| match &row.source {
                SymbolSource::Shx { file_name } => {
                    json!({"id":row.id.0,"kind":"shx","fileName":file_name})
                }
            })
            .collect(),
    )
}
pub(crate) fn decode_symbol_sources(
    value: &Value,
) -> Result<Vec<DrawingSymbolSource>, OcdrawDiagnostic> {
    let Some(rows) = value.get("symbolSources") else {
        return Ok(vec![]);
    };
    rows.as_array()
        .ok_or_else(|| failure("/symbolSources"))?
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let decode = || {
                Some(DrawingSymbolSource {
                    id: SymbolSourceId(id(row.get("id")?)?),
                    source: match row.get("kind")?.as_str()? {
                        "shx" => SymbolSource::Shx {
                            file_name: row.get("fileName")?.as_str()?.into(),
                        },
                        _ => return None,
                    },
                })
            };
            decode().ok_or_else(|| failure(&format!("/symbolSources/{i}")))
        })
        .collect()
}
