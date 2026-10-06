use crate::ocdraw::*;
use serde_json::{json, Value};

fn selection<T>(s: &AppearanceSelection<T>, encode: impl Fn(&T) -> Value) -> Value {
    match s {
        AppearanceSelection::ByLayer => json!({"mode":"ByLayer"}),
        AppearanceSelection::ByBlock => json!({"mode":"ByBlock"}),
        AppearanceSelection::Explicit(v) => json!({"mode":"Explicit","value":encode(v)}),
    }
}
fn appearance(a: &EntityAppearance) -> Value {
    json!({"color":selection(&a.color, super::encode_color),"opacity":selection(&a.opacity, |v| json!(v)),
        "linePattern":selection(&a.line_pattern, |v| json!(v.0)),"lineWeight":selection(&a.line_weight, |v| json!(v)),"linePatternScale":a.line_pattern_scale})
}
fn parse_selection<T>(
    v: &Value,
    decode: impl Fn(&Value) -> Option<T>,
) -> Option<AppearanceSelection<T>> {
    Some(match v["mode"].as_str()? {
        "ByLayer" => AppearanceSelection::ByLayer,
        "ByBlock" => AppearanceSelection::ByBlock,
        "Explicit" => AppearanceSelection::Explicit(decode(v.get("value")?)?),
        _ => return None,
    })
}
fn parse_appearance(v: &Value) -> Option<EntityAppearance> {
    Some(EntityAppearance {
        color: parse_selection(&v["color"], super::decode_appearance::color)?,
        opacity: parse_selection(&v["opacity"], Value::as_f64)?,
        line_pattern: parse_selection(&v["linePattern"], |v| {
            Some(LinePatternId(u32::try_from(v.as_u64()?).ok()?))
        })?,
        line_weight: parse_selection(&v["lineWeight"], Value::as_f64)?,
        line_pattern_scale: v["linePatternScale"].as_f64()?,
    })
}

pub(crate) fn encode_opaque_entities(rows: &[DrawingOpaqueEntity]) -> Value {
    json!({"count":rows.len(),"id":rows.iter().map(|r| r.id).collect::<Vec<_>>(),
        "preservationRecordId":rows.iter().map(|r| r.preservation_record_id.0).collect::<Vec<_>>(),
        "visible":rows.iter().map(|r| r.visible).collect::<Vec<_>>(),"layerId":rows.iter().map(|r| r.layer_id).collect::<Vec<_>>(),
        "appearance":rows.iter().map(|r| r.appearance.as_ref().map(appearance)).collect::<Vec<_>>()})
}
pub(crate) fn decode_opaque_entities(
    value: &Value,
) -> Result<Vec<DrawingOpaqueEntity>, OcdrawDiagnostic> {
    let Some(s) = value["streams"].get("opaqueEntityStream") else {
        return Ok(vec![]);
    };
    let decode = || {
        (0..usize::try_from(s["count"].as_u64()?).ok()?)
            .map(|i| {
                Some(DrawingOpaqueEntity {
                    id: s["id"][i].as_u64()?,
                    preservation_record_id: OcdrawPreservationRecordId(
                        s["preservationRecordId"][i].as_u64()?,
                    ),
                    visible: s["visible"][i].as_bool()?,
                    layer_id: if s["layerId"][i].is_null() {
                        None
                    } else {
                        Some(u32::try_from(s["layerId"][i].as_u64()?).ok()?)
                    },
                    appearance: if s["appearance"][i].is_null() {
                        None
                    } else {
                        Some(parse_appearance(&s["appearance"][i])?)
                    },
                })
            })
            .collect::<Option<Vec<_>>>()
    };
    decode().ok_or_else(|| {
        crate::ocdraw::read::diagnostic(
            "OPAQUE_DECODE",
            "/streams/opaqueEntityStream",
            "opaque entity IDs and native properties must use their exact domains",
        )
    })
}
