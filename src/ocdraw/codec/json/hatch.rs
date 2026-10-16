use super::{
    decode_appearance::appearance,
    encode_geometry::{appearance_columns, placement_frame},
    frame_from_json,
    hatch_values::BoundaryValue,
};
use crate::geometry_kernel::hatch::*;
use crate::ocdraw::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProvenanceValue {
    #[serde(default, skip_serializing_if = "is_false")]
    from_source_objects: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    derived_from_pick_point: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    outermost_hint: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    text_box: bool,
}
fn is_false(value: &bool) -> bool {
    !*value
}
impl From<ProvenanceValue> for crate::geometry_kernel::hatch::HatchLoopProvenance {
    fn from(p: ProvenanceValue) -> Self {
        Self {
            from_source_objects: p.from_source_objects,
            derived_from_pick_point: p.derived_from_pick_point,
            outermost_hint: p.outermost_hint,
            text_box: p.text_box,
        }
    }
}
impl From<&crate::geometry_kernel::hatch::HatchLoopProvenance> for ProvenanceValue {
    fn from(p: &crate::geometry_kernel::hatch::HatchLoopProvenance) -> Self {
        Self {
            from_source_objects: p.from_source_objects,
            derived_from_pick_point: p.derived_from_pick_point,
            outermost_hint: p.outermost_hint,
            text_box: p.text_box,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LoopValue {
    boundary: BoundaryValue,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    source_entity_ids: Vec<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    provenance: Option<ProvenanceValue>,
    #[serde(default, skip_serializing_if = "is_false")]
    text_island: bool,
}
pub(crate) fn encode_hatch(rows: &[DrawingHatchEntity]) -> Value {
    let mut v = json!({"count":rows.len(),"entityId":rows.iter().map(|h|h.id).collect::<Vec<_>>(),
        "layerId":rows.iter().map(|h|h.layer_id).collect::<Vec<_>>(),
        "visible":rows.iter().map(|h|h.visible).collect::<Vec<_>>(),
        "placement":rows.iter().map(|h|placement_frame(h.placement)).collect::<Vec<_>>(),
        "areaRule":rows.iter().map(|h|match h.area_rule{HatchAreaRule::Normal=>"normal",HatchAreaRule::Outer=>"outer",HatchAreaRule::Ignore=>"ignore"}).collect::<Vec<_>>(),
        "joinTolerance":rows.iter().map(|h|h.join_tolerance).collect::<Vec<_>>(),
        "fill":rows.iter().map(|h|super::hatch_pattern::encode(&h.fill)).collect::<Vec<_>>(),
        "loops":rows.iter().map(|h|h.loops.iter().map(|l|LoopValue{boundary:l.boundary.clone().into(),source_entity_ids:l.source_entity_ids.clone(),provenance:l.provenance.as_ref().map(Into::into),text_island:l.text_island}).collect::<Vec<_>>()).collect::<Vec<_>>()
    });
    if rows.iter().any(|h| h.association_enabled) {
        v["associationEnabled"] = json!(rows
            .iter()
            .map(|h| h.association_enabled)
            .collect::<Vec<_>>());
    }
    if rows.iter().any(|h| h.gradient.is_some()) {
        v["gradient"] = Value::Array(
            rows.iter()
                .map(|h| {
                    h.gradient
                        .as_ref()
                        .map(super::hatch_gradient::encode)
                        .unwrap_or(Value::Null)
                })
                .collect(),
        );
    }
    appearance_columns(
        v.as_object_mut().expect("stream"),
        &rows.iter().map(|h| &h.appearance).collect::<Vec<_>>(),
    );
    v
}
pub(crate) fn decode_hatch(value: &Value) -> Result<Vec<DrawingHatchEntity>, OcdrawDiagnostic> {
    let Some(s) = value["streams"].get("hatchStream") else {
        return Ok(vec![]);
    };
    let fail = |row: usize| OcdrawDiagnostic {
        code: "HATCH_ENCODING",
        location: format!("/streams/hatchStream/{row}"),
        message: "invalid closed Hatch row or exact identity".into(),
    };
    let count = s["count"]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| fail(0))?;
    let mut rows = Vec::new();
    for row in 0..count {
        let get = |key: &str| {
            s.get(key)
                .and_then(|v| v.as_array())
                .and_then(|v| v.get(row))
        };
        let raw_loops = get("loops")
            .and_then(Value::as_array)
            .ok_or_else(|| fail(row))?;
        let mut loops = Vec::new();
        for l in raw_loops {
            if ["sourceEntityIds", "provenance", "textIsland"]
                .iter()
                .any(|k| l.get(k).is_some_and(Value::is_null))
                || l["boundary"].get("bulges").is_some_and(Value::is_null)
            {
                return Err(fail(row));
            }
            let l: LoopValue = serde_json::from_value(l.clone()).map_err(|_| fail(row))?;
            loops.push(OcdrawHatchLoop {
                boundary: l.boundary.into(),
                source_entity_ids: l.source_entity_ids,
                provenance: l.provenance.map(Into::into),
                text_island: l.text_island,
            });
        }
        let fill = get("fill").ok_or_else(|| fail(row))?;
        let fill = super::hatch_pattern::decode(fill).map_err(|_| fail(row))?;
        let rule = match get("areaRule").and_then(Value::as_str) {
            None if get("areaRule").is_none() => HatchAreaRule::Normal,
            Some("normal") => HatchAreaRule::Normal,
            Some("outer") => HatchAreaRule::Outer,
            Some("ignore") => HatchAreaRule::Ignore,
            _ => return Err(fail(row)),
        };
        let tolerance = match get("joinTolerance") {
            None => DEFAULT_HATCH_JOIN_TOLERANCE,
            Some(v) => v.as_f64().ok_or_else(|| fail(row))?,
        };
        rows.push(DrawingHatchEntity {
            association_enabled: get("associationEnabled")
                .map(|v| v.as_bool().ok_or_else(|| fail(row)))
                .transpose()?
                .unwrap_or(false),
            gradient: get("gradient")
                .filter(|v| !v.is_null())
                .map(super::hatch_gradient::decode)
                .transpose()
                .map_err(|_| fail(row))?,
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
            placement: frame_from_json(get("placement").ok_or_else(|| fail(row))?)
                .ok_or_else(|| fail(row))?,
            loops,
            area_rule: rule,
            join_tolerance: tolerance,
            fill,
        });
    }
    Ok(rows)
}
