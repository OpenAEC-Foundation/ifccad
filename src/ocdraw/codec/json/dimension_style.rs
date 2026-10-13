use super::dimension_values::*;
use crate::{dimension::DimensionKind, ocdraw::*};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StyleRow {
    id: u32,
    name: String,
    settings: WireSettings,
    #[serde(default)]
    type_overrides: BTreeMap<DimensionKind, WireOverrides>,
}
pub(crate) fn encode_dimension_styles(rows: &[DrawingDimensionStyle]) -> Value {
    serde_json::to_value(
        rows.iter()
            .map(|s| StyleRow {
                id: s.id.0,
                name: s.name.clone(),
                settings: encode_settings(&s.settings),
                type_overrides: s
                    .type_overrides
                    .iter()
                    .map(|(k, v)| (*k, encode_overrides(v)))
                    .collect(),
            })
            .collect::<Vec<_>>(),
    )
    .expect("validated finite style values")
}
pub(crate) fn decode_dimension_styles(
    v: &Value,
) -> Result<Vec<DrawingDimensionStyle>, OcdrawDiagnostic> {
    let Some(rows) = v.get("dimensionStyles") else {
        return Ok(vec![]);
    };
    let rows: Vec<StyleRow> =
        serde_json::from_value(rows.clone()).map_err(|e| OcdrawDiagnostic {
            code: "DIMENSION_STYLE_ENCODING",
            location: "/dimensionStyles".into(),
            message: e.to_string(),
        })?;
    Ok(rows
        .into_iter()
        .map(|s| DrawingDimensionStyle {
            id: OcdrawDimensionStyleId(s.id),
            name: s.name,
            settings: decode_settings(&s.settings),
            type_overrides: s
                .type_overrides
                .into_iter()
                .map(|(k, v)| (k, decode_overrides(&v)))
                .collect(),
        })
        .collect())
}
