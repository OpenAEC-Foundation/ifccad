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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LoopValue {
    boundary: BoundaryValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_entity_id: Option<u64>,
}
pub(crate) fn encode_hatch(rows: &[DrawingHatchEntity]) -> Value {
    let mut v = json!({"count":rows.len(),"entityId":rows.iter().map(|h|h.id).collect::<Vec<_>>(),
        "layerId":rows.iter().map(|h|h.layer_id).collect::<Vec<_>>(),
        "visible":rows.iter().map(|h|h.visible).collect::<Vec<_>>(),
        "placement":rows.iter().map(|h|placement_frame(h.placement)).collect::<Vec<_>>(),
        "areaRule":rows.iter().map(|h|match h.area_rule{HatchAreaRule::Normal=>"normal",HatchAreaRule::Outer=>"outer",HatchAreaRule::Ignore=>"ignore"}).collect::<Vec<_>>(),
        "joinTolerance":rows.iter().map(|h|h.join_tolerance).collect::<Vec<_>>(),
        "fill":rows.iter().map(|h|match &h.fill{HatchFill::Solid=>json!({"kind":"solid"})}).collect::<Vec<_>>(),
        "loops":rows.iter().map(|h|h.loops.iter().map(|l|LoopValue{boundary:l.boundary.clone().into(),source_entity_id:l.source_entity_id}).collect::<Vec<_>>()).collect::<Vec<_>>()
    });
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
            if l.get("sourceEntityId").is_some_and(Value::is_null)
                || l["boundary"].get("bulges").is_some_and(Value::is_null)
            {
                return Err(fail(row));
            }
            let l: LoopValue = serde_json::from_value(l.clone()).map_err(|_| fail(row))?;
            loops.push(OcdrawHatchLoop {
                boundary: l.boundary.into(),
                source_entity_id: l.source_entity_id,
            });
        }
        let fill = get("fill").ok_or_else(|| fail(row))?;
        if fill.as_object().is_none_or(|o| o.len() != 1) || fill["kind"] != "solid" {
            return Err(fail(row));
        }
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
            fill: HatchFill::Solid,
        });
    }
    Ok(rows)
}
