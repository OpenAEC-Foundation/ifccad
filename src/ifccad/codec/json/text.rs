//! IFCCAD text payload mapping; symbolic font values use the shared text contract.
use super::{text_layout as layout, text_values as values};
use crate::ifccad::{
    IfccadEntityKind, IfccadMText, IfccadPlacement, IfccadText, IfccadTextStyleId,
};
use crate::ifccad::{IfccadReport, IfccadTextStyle};
use crate::text::{FontRequest, TextStyleProperties};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub(super) fn decode_entity(
    v: &Value,
    placement: &Value,
    key: &str,
    path: &str,
    styles: &BTreeMap<String, IfccadTextStyleId>,
) -> Result<IfccadEntityKind, IfccadReport> {
    let kind = if key == "ifccad::text" {
        "text"
    } else {
        "mText"
    };
    super::supplemental::validate_value(key, v, path)?;
    let failure = || {
        crate::ifccad::diagnostics::failure(
            super::supplemental::rule_id(key),
            &format!("{path}/{key}"),
            format!("invalid {kind} fields"),
        )
    };
    let placement: IfccadPlacement = serde_json::from_value(placement.clone()).map_err(|e| {
        crate::ifccad::diagnostics::failure(
            "IFCCAD-GEOMETRY-001",
            &format!("{path}/ifccad::geom::placement"),
            e.to_string(),
        )
    })?;
    let style_id = *styles
        .get(v["style"].as_str().ok_or_else(failure)?)
        .ok_or_else(|| {
            crate::ifccad::diagnostics::failure(
                "IFCCAD-TEXT-004",
                &format!("{path}/{key}/style"),
                "unresolved drawing-local text style",
            )
        })?;
    let rotation = v.get("rotation").and_then(Value::as_f64).unwrap_or(0.);
    let backward = v.get("backward").and_then(Value::as_bool).unwrap_or(false);
    let upside_down = v
        .get("upsideDown")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if kind == "text" {
        let content = v["content"]
            .as_array()
            .ok_or_else(failure)?
            .iter()
            .map(|r| {
                Some(crate::text::TextRun {
                    text: r["text"].as_str()?.into(),
                    underline: r["underline"].as_bool().unwrap_or(false),
                    overline: r["overline"].as_bool().unwrap_or(false),
                    strike_through: r["strikeThrough"].as_bool().unwrap_or(false),
                })
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(failure)?;
        Ok(IfccadEntityKind::Text(IfccadText {
            style_id,
            placement,
            rotation,
            backward,
            upside_down,
            layout: layout::decode_layout(&v["layout"]).ok_or_else(failure)?,
            content,
            oblique_angle: v.get("obliqueAngle").and_then(Value::as_f64).unwrap_or(0.),
            thickness: v.get("thickness").and_then(Value::as_f64).unwrap_or(0.),
        }))
    } else {
        Ok(IfccadEntityKind::MText(Box::new(IfccadMText {
            style_id,
            placement,
            rotation,
            backward,
            upside_down,
            height: v["height"].as_f64().ok_or_else(failure)?,
            attachment: values::optional(v.get("attachment"), values::read_attachment)
                .ok_or_else(failure)?
                .unwrap_or_default(),
            flow: values::optional(v.get("flow"), values::read_flow)
                .ok_or_else(failure)?
                .unwrap_or_default(),
            wrap_width: values::optional(v.get("wrapWidth"), Value::as_f64).ok_or_else(failure)?,
            columns: values::optional(v.get("columns"), values::read_columns)
                .ok_or_else(failure)?,
            background: values::optional(v.get("background"), values::read_background)
                .ok_or_else(failure)?,
            character_format: values::optional(v.get("characterFormat"), values::read_character)
                .ok_or_else(failure)?
                .unwrap_or_default(),
            paragraph_format: values::optional(v.get("paragraphFormat"), values::read_paragraph)
                .ok_or_else(failure)?
                .unwrap_or_default(),
            content: values::read_content(&v["content"]).ok_or_else(failure)?,
        })))
    }
}

pub(super) fn encode_entity(
    kind: &IfccadEntityKind,
    prefix: &str,
    attrs: &mut serde_json::Map<String, Value>,
) {
    let (key, mut v, id, placement, rotation, backward, upside_down) = match kind {
        IfccadEntityKind::Text(t) => (
            "ifccad::text",
            json!({"layout":layout::encode_layout(t.layout),"content":t.content.iter().map(|r|json!({"text":r.text,"underline":r.underline,"overline":r.overline,"strikeThrough":r.strike_through})).collect::<Vec<_>>(),"obliqueAngle":t.oblique_angle,"thickness":t.thickness}),
            t.style_id,
            &t.placement,
            t.rotation,
            t.backward,
            t.upside_down,
        ),
        IfccadEntityKind::MText(t) => {
            let mut v = json!({"height":t.height,"attachment":values::attachment(t.attachment),"flow":values::flow(t.flow),"content":values::content(&t.content),"characterFormat":values::character(&t.character_format),"paragraphFormat":values::paragraph(&t.paragraph_format)});
            if let Some(width) = t.wrap_width {
                v["wrapWidth"] = json!(width);
            }
            if let Some(columns) = &t.columns {
                v["columns"] = values::columns(columns);
            }
            if let Some(background) = &t.background {
                v["background"] = values::background(background);
            }
            (
                "ifccad::mText",
                v,
                t.style_id,
                &t.placement,
                t.rotation,
                t.backward,
                t.upside_down,
            )
        }
        _ => unreachable!("text kind required"),
    };
    v["style"] = json!(format!("{prefix}/textStyle/{}", id.0));
    v["rotation"] = json!(rotation);
    v["backward"] = json!(backward);
    v["upsideDown"] = json!(upside_down);
    attrs.insert(key.into(), v);
    attrs.insert("ifccad::geom::placement".into(), json!(placement));
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FontValue {
    family: Option<String>,
    cad_font_name: Option<String>,
    big_font_name: Option<String>,
    bold: Option<bool>,
    italic: Option<bool>,
    charset: Option<u16>,
    pitch: Option<u8>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StyleValue {
    name: String,
    font: FontValue,
    #[serde(default = "crate::ifccad::logical::patterns::one")]
    width_factor: f64,
    #[serde(default)]
    oblique_angle: f64,
    #[serde(default)]
    vertical: bool,
    creation_height: Option<f64>,
    last_used_height: Option<f64>,
    #[serde(default)]
    creation_backward: bool,
    #[serde(default)]
    creation_upside_down: bool,
}

fn reject_nulls(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Object(o) => o.values().all(reject_nulls),
        Value::Array(a) => a.iter().all(reject_nulls),
        _ => true,
    }
}

pub(super) fn decode_style(
    value: &Value,
    path: &str,
) -> Result<(String, TextStyleProperties), IfccadReport> {
    super::supplemental::validate_value("ifccad::textStyle", value, path)?;
    if !reject_nulls(value) {
        return Err(crate::ifccad::diagnostics::failure(
            "IFCCAD-TEXT-001",
            path,
            format!("{path} textStyle cannot contain null"),
        ));
    }
    let s: StyleValue = serde_json::from_value(value.clone()).map_err(|e| {
        crate::ifccad::diagnostics::failure(
            "IFCCAD-TEXT-001",
            &format!("{path}/ifccad::textStyle"),
            e.to_string(),
        )
    })?;
    Ok((
        s.name,
        TextStyleProperties {
            font: FontRequest {
                family: s.font.family,
                cad_font_name: s.font.cad_font_name,
                big_font_name: s.font.big_font_name,
                bold: s.font.bold,
                italic: s.font.italic,
                charset: s.font.charset,
                pitch: s.font.pitch,
            },
            width_factor: s.width_factor,
            oblique_angle: s.oblique_angle,
            vertical: s.vertical,
            creation_height: s.creation_height,
            last_used_height: s.last_used_height,
            creation_backward: s.creation_backward,
            creation_upside_down: s.creation_upside_down,
        },
    ))
}

pub(super) fn encode_style(style: &IfccadTextStyle) -> Value {
    let p = &style.properties;
    let mut font = json!({});
    macro_rules! fields { ($($field:ident=>$key:literal),*) => { $(if let Some(v)=&p.font.$field { font[$key]=json!(v); })* }; }
    fields!(family=>"family",cad_font_name=>"cadFontName",big_font_name=>"bigFontName",bold=>"bold",italic=>"italic",charset=>"charset",pitch=>"pitch");
    let mut value = json!({"name": style.name, "font":font, "widthFactor":p.width_factor, "obliqueAngle":p.oblique_angle, "vertical":p.vertical, "creationBackward":p.creation_backward, "creationUpsideDown":p.creation_upside_down});
    if let Some(v) = p.creation_height {
        value["creationHeight"] = json!(v);
    }
    if let Some(v) = p.last_used_height {
        value["lastUsedHeight"] = json!(v);
    }
    value
}
