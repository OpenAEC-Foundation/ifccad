use super::{
    decode_appearance::appearance,
    dimension_values::*,
    encode_geometry::{appearance_columns, placement_frame},
    frame_from_json,
};
use crate::ocdraw::*;
use serde_json::{json, Value};

pub(crate) fn encode_dimensions(rows: &[DrawingDimensionEntity]) -> Value {
    let mut stream = json!({"count":rows.len(),"entityId":rows.iter().map(|e|e.id).collect::<Vec<_>>(),"layerId":rows.iter().map(|e|e.layer_id).collect::<Vec<_>>(),
        "visible":rows.iter().map(|e|e.visible).collect::<Vec<_>>(),"placement":rows.iter().map(|e|placement_frame(e.placement)).collect::<Vec<_>>(),
        "styleId":rows.iter().map(|e|e.style_id.0).collect::<Vec<_>>(),"definition":rows.iter().map(|e|&e.definition).collect::<Vec<_>>(),
        "textPlacement":rows.iter().map(|e|&e.text_placement).collect::<Vec<_>>(),"label":rows.iter().map(|e|&e.label).collect::<Vec<_>>(),
        "overrides":rows.iter().map(|e|encode_overrides(&e.overrides)).collect::<Vec<_>>(),"associations":rows.iter().map(|e|&e.associations).collect::<Vec<_>>()});
    appearance_columns(
        stream.as_object_mut().unwrap(),
        &rows.iter().map(|e| &e.appearance).collect::<Vec<_>>(),
    );
    stream
}
pub(crate) fn decode_dimensions(
    v: &Value,
) -> Result<Vec<DrawingDimensionEntity>, OcdrawDiagnostic> {
    let Some(s) = v["streams"].get("dimensionStream") else {
        return Ok(vec![]);
    };
    let fail = |row: usize| OcdrawDiagnostic {
        code: "DIMENSION_ENCODING",
        location: format!("/streams/dimensionStream/{row}"),
        message: "invalid closed Dimension row or identity backing".into(),
    };
    let count = s["count"]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| fail(0))?;
    let mut rows = Vec::new();
    for row in 0..count {
        let get = |key: &str| {
            s.get(key)
                .and_then(Value::as_array)
                .and_then(|v| v.get(row))
        };
        let required = |key: &str| get(key).cloned().ok_or_else(|| fail(row));
        let over: WireOverrides =
            serde_json::from_value(required("overrides")?).map_err(|_| fail(row))?;
        rows.push(DrawingDimensionEntity {
            id: get("entityId")
                .and_then(Value::as_u64)
                .ok_or_else(|| fail(row))?,
            layer_id: get("layerId")
                .and_then(Value::as_u64)
                .and_then(|v| u32::try_from(v).ok())
                .ok_or_else(|| fail(row))?,
            visible: get("visible")
                .map(|v| v.as_bool().ok_or_else(|| fail(row)))
                .transpose()?
                .unwrap_or(true),
            appearance: appearance(s, row).ok_or_else(|| fail(row))?,
            placement: frame_from_json(&required("placement")?).ok_or_else(|| fail(row))?,
            style_id: OcdrawDimensionStyleId(
                get("styleId")
                    .and_then(Value::as_u64)
                    .and_then(|v| u32::try_from(v).ok())
                    .ok_or_else(|| fail(row))?,
            ),
            definition: serde_json::from_value(required("definition")?).map_err(|_| fail(row))?,
            text_placement: serde_json::from_value(required("textPlacement")?)
                .map_err(|_| fail(row))?,
            label: serde_json::from_value(required("label")?).map_err(|_| fail(row))?,
            overrides: decode_overrides(&over),
            associations: serde_json::from_value(required("associations")?)
                .map_err(|_| fail(row))?,
        });
    }
    Ok(rows)
}
