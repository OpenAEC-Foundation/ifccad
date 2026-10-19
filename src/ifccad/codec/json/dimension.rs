use super::dimension_values::*;
use crate::{dimension::*, *};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Association {
    role: DimensionRole,
    source: String,
    selector: DimensionPointSelector,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DimensionValue {
    style: String,
    definition: DimensionDefinition,
    text_placement: DimensionTextPlacement,
    label: DimensionLabel,
    overrides: WireOverrides,
    associations: Vec<Association>,
}
pub(super) fn encode(v: &IfccadDimension, prefix: &str) -> Value {
    json!({"style":format!("{prefix}/dimensionStyle/{}",v.style_id.0),"definition":v.definition,"textPlacement":v.text_placement,"label":v.label,"overrides":encode_overrides(&v.overrides,prefix),"associations":v.associations.iter().map(|a|Association{role:a.role,source:format!("{prefix}/e{}",a.source_entity_id),selector:a.selector}).collect::<Vec<_>>()})
}
pub(super) fn decode(
    v: &Value,
    placement: &Value,
    prefix: &str,
    path: &str,
) -> Result<IfccadEntityKind, IfccadReport> {
    super::supplemental::validate_value("ifccad::dimension", v, path)?;
    let fail = |e: serde_json::Error| {
        crate::ifccad::diagnostics::failure("IFCCAD-DIMENSION-001", path, e.to_string())
    };
    let v: DimensionValue = serde_json::from_value(v.clone()).map_err(fail)?;
    let placement: IfccadPlacement = serde_json::from_value(placement.clone()).map_err(fail)?;
    let mut associations = Vec::new();
    for a in v.associations {
        associations.push(IfccadDimensionAssociation {
            role: a.role,
            source_entity_id: reference(prefix, "e", &a.source, path)?,
            selector: a.selector,
        });
    }
    Ok(IfccadEntityKind::Dimension(Box::new(IfccadDimension {
        placement,
        style_id: IfccadDimensionStyleId(reference(prefix, "dimensionStyle/", &v.style, path)?),
        definition: v.definition,
        text_placement: v.text_placement,
        label: v.label,
        overrides: decode_overrides(&v.overrides, prefix, path)?,
        associations,
    })))
}
