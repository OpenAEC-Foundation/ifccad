//! Registered Text rows; IDs and scope ownership remain drawing-local.
use super::{
    decode_appearance::appearance,
    encode_geometry::{appearance_columns, placement_frame},
};
use crate::{ocdraw::*, text::*};
use serde_json::{json, Value};

pub(crate) fn encode_text(entities: &[DrawingTextEntity]) -> Value {
    let mut stream = json!({"count": entities.len(),
        "entityId": entities.iter().map(|t| t.id).collect::<Vec<_>>(),
        "layerId": entities.iter().map(|t| t.layer_id).collect::<Vec<_>>(),
        "styleId": entities.iter().map(|t| t.style_id.0).collect::<Vec<_>>(),
        "placement": entities.iter().map(|t| placement_frame(t.placement)).collect::<Vec<_>>(),
        "visible": entities.iter().map(|t| t.visible).collect::<Vec<_>>(),
        "rotation": entities.iter().map(|t| t.rotation).collect::<Vec<_>>(),
        "backward": entities.iter().map(|t| t.backward).collect::<Vec<_>>(),
        "upsideDown": entities.iter().map(|t| t.upside_down).collect::<Vec<_>>(),
        "obliqueAngle": entities.iter().map(|t| t.oblique_angle).collect::<Vec<_>>(),
        "thickness": entities.iter().map(|t| t.thickness).collect::<Vec<_>>(),
        "layout": entities.iter().map(|t| encode_layout(t.layout)).collect::<Vec<_>>(),
        "content": entities.iter().map(|t| t.content.iter().map(|r| json!({"text": r.text, "underline": r.underline, "overline": r.overline, "strikeThrough": r.strike_through})).collect::<Vec<_>>()).collect::<Vec<_>>()});
    appearance_columns(
        stream.as_object_mut().expect("object stream"),
        &entities.iter().map(|t| &t.appearance).collect::<Vec<_>>(),
    );
    stream
}

pub(crate) fn decode_text(value: &Value) -> Result<Vec<DrawingTextEntity>, OcdrawDiagnostic> {
    let Some(stream) = value["streams"].get("textStream") else {
        return Ok(Vec::new());
    };
    let count = stream["count"]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| failure(0))?;
    (0..count)
        .map(|row| decode_row(stream, row).ok_or_else(|| failure(row)))
        .collect()
}
fn failure(row: usize) -> OcdrawDiagnostic {
    OcdrawDiagnostic {
        code: "TEXT_ENCODING",
        location: format!("/streams/textStream/{row}"),
        message: "invalid registered Text row, exact ID backing or placement".into(),
    }
}
fn decode_row(stream: &Value, row: usize) -> Option<DrawingTextEntity> {
    let column = |name| stream.get(name)?.get(row);
    let placement = match column("placement").filter(|v| !v.is_null()) {
        None => CoordinateFrame3::default(),
        Some(v) => super::frame_from_json(v)?,
    };
    Some(DrawingTextEntity {
        id: column("entityId")?.as_u64()?,
        layer_id: u32::try_from(column("layerId")?.as_u64()?).ok()?,
        style_id: OcdrawTextStyleId(u32::try_from(column("styleId")?.as_u64()?).ok()?),
        appearance: appearance(stream, row)?,
        visible: column("visible").and_then(Value::as_bool).unwrap_or(true),
        placement,
        rotation: column("rotation").and_then(Value::as_f64).unwrap_or(0.0),
        backward: column("backward").and_then(Value::as_bool).unwrap_or(false),
        upside_down: column("upsideDown")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        oblique_angle: column("obliqueAngle")
            .and_then(Value::as_f64)
            .unwrap_or(0.0),
        thickness: column("thickness").and_then(Value::as_f64).unwrap_or(0.0),
        layout: decode_layout(column("layout")?)?,
        content: column("content")?
            .as_array()?
            .iter()
            .map(|r| {
                Some(TextRun {
                    text: r.get("text")?.as_str()?.into(),
                    underline: r.get("underline").and_then(Value::as_bool).unwrap_or(false),
                    overline: r.get("overline").and_then(Value::as_bool).unwrap_or(false),
                    strike_through: r
                        .get("strikeThrough")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                })
            })
            .collect::<Option<Vec<_>>>()?,
    })
}

fn encode_layout(layout: TextLayout) -> Value {
    match layout {
        TextLayout::Anchored {
            horizontal,
            vertical,
            height,
            width_factor,
        } => {
            json!({"kind": "anchored", "horizontal": match horizontal { TextHorizontalAlignment::Left => "left", TextHorizontalAlignment::Center => "center", TextHorizontalAlignment::Right => "right" }, "vertical": match vertical { TextVerticalAlignment::Baseline => "baseline", TextVerticalAlignment::Bottom => "bottom", TextVerticalAlignment::Middle => "middle", TextVerticalAlignment::Top => "top" }, "height": height, "widthFactor": width_factor})
        }
        TextLayout::WholeTextMiddle {
            height,
            width_factor,
        } => json!({"kind": "wholeTextMiddle", "height": height, "widthFactor": width_factor}),
        TextLayout::Aligned {
            length,
            width_factor,
        } => json!({"kind": "aligned", "length": length, "widthFactor": width_factor}),
        TextLayout::Fit { length, height } => {
            json!({"kind": "fit", "length": length, "height": height})
        }
    }
}
fn decode_layout(v: &Value) -> Option<TextLayout> {
    Some(match v.get("kind")?.as_str()? {
        "anchored" => TextLayout::Anchored {
            horizontal: match v.get("horizontal")?.as_str()? {
                "left" => TextHorizontalAlignment::Left,
                "center" => TextHorizontalAlignment::Center,
                "right" => TextHorizontalAlignment::Right,
                _ => return None,
            },
            vertical: match v.get("vertical")?.as_str()? {
                "baseline" => TextVerticalAlignment::Baseline,
                "bottom" => TextVerticalAlignment::Bottom,
                "middle" => TextVerticalAlignment::Middle,
                "top" => TextVerticalAlignment::Top,
                _ => return None,
            },
            height: v.get("height")?.as_f64()?,
            width_factor: v.get("widthFactor")?.as_f64()?,
        },
        "wholeTextMiddle" => TextLayout::WholeTextMiddle {
            height: v.get("height")?.as_f64()?,
            width_factor: v.get("widthFactor")?.as_f64()?,
        },
        "aligned" => TextLayout::Aligned {
            length: v.get("length")?.as_f64()?,
            width_factor: v.get("widthFactor")?.as_f64()?,
        },
        "fit" => TextLayout::Fit {
            length: v.get("length")?.as_f64()?,
            height: v.get("height")?.as_f64()?,
        },
        _ => return None,
    })
}
