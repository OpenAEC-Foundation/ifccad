use super::decode_appearance::color;
use crate::ocdraw::logical::DrawingLayer;
use serde_json::Value;

pub(crate) fn decode_layers(value: &Value) -> Option<Vec<DrawingLayer>> {
    value
        .get("layers")
        .and_then(Value::as_array)
        .map(|layers| {
            layers
                .iter()
                .map(|row| {
                    Some(DrawingLayer {
                        id: u32::try_from(row.get("id")?.as_u64()?).ok()?,
                        name: row.get("name")?.as_str()?.to_owned(),
                        description: row
                            .get("description")
                            .and_then(Value::as_str)
                            .map(str::to_owned),
                        visible: row
                            .get("visible")
                            .map(Value::as_bool)
                            .unwrap_or(Some(true))?,
                        frozen: row
                            .get("frozen")
                            .map(Value::as_bool)
                            .unwrap_or(Some(false))?,
                        locked: row
                            .get("locked")
                            .map(Value::as_bool)
                            .unwrap_or(Some(false))?,
                        plottable: row
                            .get("plottable")
                            .map(Value::as_bool)
                            .unwrap_or(Some(true))?,
                        frozen_in_new_viewports: row
                            .get("frozenInNewViewports")
                            .map(Value::as_bool)
                            .unwrap_or(Some(false))?,
                        color: color(row.get("color")?)?,
                        opacity: row.get("opacity").map(Value::as_f64).unwrap_or(Some(1.0))?,
                        line_pattern_id: crate::ocdraw::LinePatternId(
                            row.get("linePatternId")?.as_u64()? as u32,
                        ),
                        line_weight: row.get("lineWeight")?.as_f64()?,
                    })
                })
                .collect()
        })
        .unwrap_or_else(|| Some(Vec::new()))
}
