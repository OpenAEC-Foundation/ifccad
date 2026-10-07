//! Physical text-style records; semantic checks belong to logical validation.
use crate::{
    ocdraw::{DrawingTextStyle, OcdrawDiagnostic, OcdrawTextStyleId},
    text::{FontRequest, TextStyleProperties},
};
use serde_json::{json, Value};

pub(crate) fn encode_text_styles(styles: &[DrawingTextStyle]) -> Value {
    Value::Array(styles.iter().map(|style| {
        let p = &style.properties;
        let mut font = json!({});
        macro_rules! optional_font {
            ($($field:ident => $key:literal),*) => { $(if let Some(value) = &p.font.$field { font[$key] = json!(value); })* };
        }
        optional_font!(family => "family", cad_font_name => "cadFontName", big_font_name => "bigFontName", bold => "bold", italic => "italic", charset => "charset", pitch => "pitch");
        let mut row = json!({"id": style.id.0, "name": style.name, "font": font,
            "widthFactor": p.width_factor, "obliqueAngle": p.oblique_angle,
            "vertical": p.vertical, "creationBackward": p.creation_backward,
            "creationUpsideDown": p.creation_upside_down});
        if let Some(height) = p.creation_height { row["creationHeight"] = json!(height); }
        if let Some(height) = p.last_used_height { row["lastUsedHeight"] = json!(height); }
        row
    }).collect())
}

pub(crate) fn decode_text_styles(value: &Value) -> Result<Vec<DrawingTextStyle>, OcdrawDiagnostic> {
    let Some(rows) = value.get("textStyles") else {
        return Ok(Vec::new());
    };
    rows.as_array()
        .ok_or_else(|| failure("/textStyles"))?
        .iter()
        .enumerate()
        .map(|(i, row)| decode_style(row).ok_or_else(|| failure(&format!("/textStyles/{i}"))))
        .collect()
}
fn failure(location: &str) -> OcdrawDiagnostic {
    OcdrawDiagnostic {
        code: "TEXT_STYLE_ENCODING",
        location: location.into(),
        message: "text style fields need registered physical types and exact ID backings".into(),
    }
}
fn decode_style(row: &Value) -> Option<DrawingTextStyle> {
    let font = row.get("font")?;
    let optional_string = |key| optional(font.get(key), |v| v.as_str().map(str::to_owned));
    let optional_bool = |key| optional(font.get(key), Value::as_bool);
    Some(DrawingTextStyle {
        id: OcdrawTextStyleId(u32::try_from(row.get("id")?.as_u64()?).ok()?),
        name: row.get("name")?.as_str()?.into(),
        properties: TextStyleProperties {
            font: FontRequest {
                family: optional_string("family")?,
                cad_font_name: optional_string("cadFontName")?,
                big_font_name: optional_string("bigFontName")?,
                bold: optional_bool("bold")?,
                italic: optional_bool("italic")?,
                charset: optional(font.get("charset"), |v| u16::try_from(v.as_u64()?).ok())?,
                pitch: optional(font.get("pitch"), |v| u8::try_from(v.as_u64()?).ok())?,
            },
            width_factor: optional(row.get("widthFactor"), Value::as_f64)?.unwrap_or(1.0),
            oblique_angle: optional(row.get("obliqueAngle"), Value::as_f64)?.unwrap_or(0.0),
            vertical: optional(row.get("vertical"), Value::as_bool)?.unwrap_or(false),
            creation_height: optional(row.get("creationHeight"), Value::as_f64)?,
            last_used_height: optional(row.get("lastUsedHeight"), Value::as_f64)?,
            creation_backward: optional(row.get("creationBackward"), Value::as_bool)?
                .unwrap_or(false),
            creation_upside_down: optional(row.get("creationUpsideDown"), Value::as_bool)?
                .unwrap_or(false),
        },
    })
}
fn optional<T>(
    value: Option<&Value>,
    parse: impl FnOnce(&Value) -> Option<T>,
) -> Option<Option<T>> {
    match value {
        None => Some(None),
        Some(value) => parse(value).map(Some),
    }
}
