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
                        visible: row.get("visible")?.as_bool()?,
                        frozen: row.get("frozen")?.as_bool()?,
                        locked: row.get("locked")?.as_bool()?,
                        plottable: row.get("plottable")?.as_bool()?,
                        frozen_in_new_viewports: row.get("frozenInNewViewports")?.as_bool()?,
                        color: color(row.get("color")?)?,
                        opacity: row.get("opacity")?.as_f64()?,
                        line_pattern: row.get("linePattern")?.as_str()?.to_owned(),
                        line_weight: row.get("lineWeight")?.as_f64()?,
                    })
                })
                .collect()
        })
        .unwrap_or_else(|| Some(Vec::new()))
}
