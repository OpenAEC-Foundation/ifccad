use super::hatch_values::BoundaryValue;
use super::*;
use crate::geometry_kernel::hatch::{HatchAreaRule, DEFAULT_HATCH_JOIN_TOLERANCE};
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
    sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    provenance: Option<ProvenanceValue>,
    #[serde(default, skip_serializing_if = "is_false")]
    text_island: bool,
}
fn rule() -> String {
    "normal".into()
}
fn tolerance() -> f64 {
    DEFAULT_HATCH_JOIN_TOLERANCE
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct HatchValue {
    #[serde(default, deserialize_with = "super::presentation::present")]
    classic_settings: Option<Value>,
    #[serde(default, deserialize_with = "super::presentation::present")]
    boundary_construction: Option<Value>,
    #[serde(default)]
    association_enabled: bool,
    loops: Vec<LoopValue>,
    #[serde(default = "rule")]
    area_rule: String,
    #[serde(default = "tolerance")]
    join_tolerance: f64,
    fill: Value,
    #[serde(default, deserialize_with = "super::presentation::present")]
    gradient: Option<Value>,
}
pub(super) fn decode_hatch(
    value: &Value,
    placement: &Value,
    prefix: &str,
    path: &str,
) -> Result<IfccadEntityKind, IfccadReport> {
    super::supplemental::validate_value("ifccad::hatch", value, path)?;
    super::supplemental::validate_value("ifccad::geom::placement", placement, path)?;
    let error = |msg: &str| IfccadReport::one(format!("{path}: {msg}"));
    let raw = value["loops"]
        .as_array()
        .ok_or_else(|| error("Hatch loops required"))?;
    for l in raw {
        if ["sources", "provenance", "textIsland"]
            .iter()
            .any(|k| l.get(k).is_some_and(Value::is_null))
            || l["boundary"].get("bulges").is_some_and(Value::is_null)
        {
            return Err(error(
                "Hatch optional reference/array must be absent, not null",
            ));
        }
    }
    let v: HatchValue = serde_json::from_value(value.clone()).map_err(|e| error(&e.to_string()))?;
    let fill = super::hatch_pattern::decode(&v.fill).map_err(|e| error(&e.to_string()))?;
    let area_rule = match v.area_rule.as_str() {
        "normal" => HatchAreaRule::Normal,
        "outer" => HatchAreaRule::Outer,
        "ignore" => HatchAreaRule::Ignore,
        _ => return Err(error("invalid Hatch area rule")),
    };
    let source_prefix = format!("{prefix}/e");
    let mut loops = Vec::new();
    for l in v.loops {
        let mut ids = Vec::new();
        for source in l.sources {
            let tail = source
                .strip_prefix(&source_prefix)
                .ok_or_else(|| error("Hatch source needs a complete same-drawing entity path"))?;
            let id = tail
                .parse::<u64>()
                .map_err(|_| error("invalid Hatch source identity"))?;
            if tail != id.to_string() {
                return Err(error("noncanonical Hatch source identity"));
            }
            ids.push(id);
        }
        loops.push(IfccadHatchLoop {
            boundary: l.boundary.into(),
            source_entity_ids: ids,
            provenance: l.provenance.map(Into::into),
            text_island: l.text_island,
        });
    }
    let placement: IfccadPlacement =
        serde_json::from_value(placement.clone()).map_err(|e| error(&e.to_string()))?;
    Ok(IfccadEntityKind::Hatch(IfccadHatch {
        classic_settings: v
            .classic_settings
            .as_ref()
            .map(super::hatch_editing::decode_classic)
            .transpose()
            .map_err(|e| error(&e.to_string()))?,
        boundary_construction: v
            .boundary_construction
            .as_ref()
            .map(super::hatch_editing::decode_construction)
            .transpose()
            .map_err(|e| error(&e.to_string()))?,
        association_enabled: v.association_enabled,
        gradient: v
            .gradient
            .as_ref()
            .map(super::hatch_gradient::decode)
            .transpose()
            .map_err(|e| error(&e.to_string()))?,
        placement,
        loops,
        area_rule,
        join_tolerance: v.join_tolerance,
        fill,
    }))
}
pub(super) fn encode_hatch(h: &IfccadHatch, prefix: &str) -> Value {
    let fill = super::hatch_pattern::encode(&h.fill);
    let mut value = json!({
        "loops":h.loops.iter().map(|l|LoopValue{boundary:l.boundary.clone().into(),sources:l.source_entity_ids.iter().map(|id|format!("{prefix}/e{id}")).collect(),provenance:l.provenance.as_ref().map(Into::into),text_island:l.text_island}).collect::<Vec<_>>(),
        "areaRule":match h.area_rule{HatchAreaRule::Normal=>"normal",HatchAreaRule::Outer=>"outer",HatchAreaRule::Ignore=>"ignore"},
        "joinTolerance":h.join_tolerance,"fill":fill
    });
    if h.association_enabled {
        value["associationEnabled"] = json!(true);
    }
    if let Some(c) = &h.classic_settings {
        value["classicSettings"] = super::hatch_editing::encode_classic(c);
    }
    if let Some(c) = &h.boundary_construction {
        value["boundaryConstruction"] = super::hatch_editing::encode_construction(c);
    }
    if let Some(g) = &h.gradient {
        value["gradient"] = super::hatch_gradient::encode(g);
    }
    value
}
