use super::hatch_values::BoundaryValue;
use super::*;
use crate::geometry_kernel::hatch::{HatchAreaRule, DEFAULT_HATCH_JOIN_TOLERANCE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LoopValue {
    boundary: BoundaryValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source: Option<String>,
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
    loops: Vec<LoopValue>,
    #[serde(default = "rule")]
    area_rule: String,
    #[serde(default = "tolerance")]
    join_tolerance: f64,
    fill: Value,
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
        if l.get("source").is_some_and(Value::is_null)
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
        let id = if let Some(source) = l.source {
            let tail = source
                .strip_prefix(&source_prefix)
                .ok_or_else(|| error("Hatch source needs a complete same-drawing entity path"))?;
            let id = tail
                .parse::<u64>()
                .map_err(|_| error("invalid Hatch source identity"))?;
            if tail != id.to_string() {
                return Err(error("noncanonical Hatch source identity"));
            }
            Some(id)
        } else {
            None
        };
        loops.push(IfccadHatchLoop {
            boundary: l.boundary.into(),
            source_entity_id: id,
        });
    }
    let placement: IfccadPlacement =
        serde_json::from_value(placement.clone()).map_err(|e| error(&e.to_string()))?;
    Ok(IfccadEntityKind::Hatch(IfccadHatch {
        placement,
        loops,
        area_rule,
        join_tolerance: v.join_tolerance,
        fill,
    }))
}
pub(super) fn encode_hatch(h: &IfccadHatch, prefix: &str) -> Value {
    let fill = super::hatch_pattern::encode(&h.fill);
    json!({
        "loops":h.loops.iter().map(|l|LoopValue{boundary:l.boundary.clone().into(),source:l.source_entity_id.map(|id|format!("{prefix}/e{id}"))}).collect::<Vec<_>>(),
        "areaRule":match h.area_rule{HatchAreaRule::Normal=>"normal",HatchAreaRule::Outer=>"outer",HatchAreaRule::Ignore=>"ignore"},
        "joinTolerance":h.join_tolerance,"fill":fill
    })
}
