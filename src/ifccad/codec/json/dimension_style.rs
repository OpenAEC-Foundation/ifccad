use super::dimension_values::*;
use crate::{dimension::DimensionKind, *};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StyleValue {
    #[serde(default)]
    annotative: bool,
    name: String,
    settings: WireSettings,
    #[serde(default)]
    type_overrides: BTreeMap<DimensionKind, WireOverrides>,
}
pub(super) fn encode_style(s: &IfccadDimensionStyle, prefix: &str) -> Value {
    let mut value = json!({"name":s.name,"settings":encode_settings(&s.settings,prefix),"typeOverrides":s.type_overrides.iter().map(|(k,v)|(*k,encode_overrides(v,prefix))).collect::<BTreeMap<_,_>>()});
    if s.annotative {
        value["annotative"] = json!(true);
    }
    value
}
pub(super) fn decode_style(
    v: &Value,
    id: IfccadDimensionStyleId,
    prefix: &str,
    path: &str,
) -> Result<IfccadDimensionStyle, IfccadReport> {
    super::supplemental::validate_value("ifccad::dimensionStyle", v, path)?;
    let v: StyleValue = serde_json::from_value(v.clone()).map_err(|e| {
        crate::ifccad::diagnostics::failure("IFCCAD-DIMENSION-002", path, e.to_string())
    })?;
    let mut types = BTreeMap::new();
    for (k, over) in v.type_overrides {
        types.insert(k, decode_overrides(&over, prefix, path)?);
    }
    Ok(IfccadDimensionStyle {
        annotative: v.annotative,
        id,
        name: v.name,
        settings: decode_settings(&v.settings, prefix, path)?,
        type_overrides: types,
    })
}
