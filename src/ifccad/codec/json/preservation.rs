//! Canonical IFCX paths and bytes stay at the physical mapping boundary.
use super::*;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};

fn problem(message: impl Into<String>) -> IfccadReport {
    crate::ifccad::diagnostics::failure("IFCCAD-PRESERVATION-001", "/", message)
}
fn target_path(t: IfccadPreservationTarget, prefix: &str) -> String {
    use IfccadPreservationTarget::*;
    match t {
        Drawing => prefix.into(),
        Entity(id) => format!("{prefix}/e{id}"),
        Layer(id) => format!("{prefix}/layer/{id}"),
        LinePattern(id) => format!("{prefix}/linePattern/{}", id.0),
        Layout(id) => format!("{prefix}/layout/{id}"),
        BlockDefinition(id) => format!("{prefix}/block/{id}"),
        Record(id) => format!("{prefix}/preservation/r{}", id.0),
    }
}
fn encode_target(t: IfccadPreservationTarget, prefix: &str) -> Value {
    let v = serde_json::to_value(t).expect("target serialization");
    json!({"role":v["role"],"path":target_path(t,prefix)})
}
fn decode_target(v: &Value, prefix: &str) -> Result<Value, IfccadReport> {
    let obj = v
        .as_object()
        .ok_or_else(|| problem("target must be an object"))?;
    if obj.len() != 2 {
        return Err(problem("target requires only role and path"));
    }
    let role = v["role"]
        .as_str()
        .ok_or_else(|| problem("target role must be a string"))?;
    let path = v["path"]
        .as_str()
        .ok_or_else(|| problem("target path must be a string"))?;
    if role == "drawing" {
        return if path == prefix {
            Ok(json!({"role":"drawing"}))
        } else {
            Err(problem("wrong drawing target"))
        };
    }
    let suffix = match role {
        "entity" => "/e",
        "layer" => "/layer/",
        "linePattern" => "/linePattern/",
        "layout" => "/layout/",
        "blockDefinition" => "/block/",
        "record" => "/preservation/r",
        _ => return Err(problem("unknown target role")),
    };
    let id = super::validate::numbered(path, &format!("{prefix}{suffix}"))?;
    if role == "record" && id == 0 {
        return Err(problem("record ID must be positive"));
    }
    Ok(json!({"role":role,"value":id}))
}
fn decode_bytes(v: &Value) -> Result<Value, IfccadReport> {
    let text = v
        .as_str()
        .ok_or_else(|| problem("payload/baseline requires Base64 string"))?;
    let bytes = STANDARD
        .decode(text)
        .map_err(|e| problem(format!("invalid Base64: {e}")))?;
    if STANDARD.encode(&bytes) != text {
        return Err(problem("noncanonical Base64"));
    }
    Ok(json!(bytes))
}
pub(super) fn encode_record(r: &IfccadPreservationRecord, prefix: &str) -> Value {
    let mut v = serde_json::to_value(r).expect("record serialization");
    v.as_object_mut().unwrap().remove("id");
    if let Some(t) = r.subject {
        v["subject"] = encode_target(t, prefix);
    }
    for (v, b) in v["bindings"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .zip(&r.bindings)
    {
        v["target"] = encode_target(b.target, prefix);
    }
    for (v, c) in v["conditions"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .zip(&r.conditions)
    {
        v["target"] = encode_target(c.target, prefix);
        v["baseline"] = json!(STANDARD.encode(&c.baseline));
    }
    v["payload"]["bytes"] = json!(STANDARD.encode(&r.payload.bytes));
    v
}
pub(super) fn decode_record(
    v: &Value,
    id: u64,
    prefix: &str,
) -> Result<IfccadPreservationRecord, IfccadReport> {
    if v.get("id").is_some() {
        return Err(problem("record ID belongs in its canonical path"));
    }
    let mut v = v.clone();
    let obj = v
        .as_object_mut()
        .ok_or_else(|| problem("record must be an object"))?;
    obj.insert("id".into(), json!(id));
    if let Some(t) = v.get("subject") {
        v["subject"] = decode_target(t, prefix)?;
    }
    for b in v["bindings"]
        .as_array_mut()
        .ok_or_else(|| problem("bindings array required"))?
    {
        b["target"] = decode_target(&b["target"], prefix)?;
    }
    for c in v["conditions"]
        .as_array_mut()
        .ok_or_else(|| problem("conditions array required"))?
    {
        c["target"] = decode_target(&c["target"], prefix)?;
        c["baseline"] = decode_bytes(&c["baseline"])?;
    }
    v["payload"]["bytes"] = decode_bytes(&v["payload"]["bytes"])?;
    serde_json::from_value(v).map_err(|e| problem(format!("invalid preservation record: {e}")))
}
pub(super) fn decode_opaque(
    v: &Value,
    id: u64,
    prefix: &str,
    patterns: &std::collections::BTreeMap<String, IfccadLinePatternId>,
) -> Result<IfccadEntity, IfccadReport> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Opaque {
        preservation_record: String,
        visible: bool,
        native_layer: Option<String>,
        native_appearance: Option<Appearance>,
    }
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Appearance {
        appearance: super::wire::EntityAppearance,
        line_pattern_scale: f64,
    }
    if ["nativeLayer", "nativeAppearance"]
        .into_iter()
        .any(|k| v.get(k).is_some_and(Value::is_null))
    {
        return Err(problem("omit absent native common fields"));
    }
    if let Some(a) = v.get("nativeAppearance") {
        super::validate::appearance_modes(&a["appearance"], prefix)?;
    }
    let value: Opaque = serde_json::from_value(v.clone())
        .map_err(|e| problem(format!("invalid opaque entity: {e}")))?;
    let record = super::validate::numbered(
        &value.preservation_record,
        &format!("{prefix}/preservation/r"),
    )?;
    let layer = value
        .native_layer
        .as_deref()
        .map(|path| super::validate::numbered(path, &format!("{prefix}/layer/")))
        .transpose()?;
    let appearance = value
        .native_appearance
        .map(|a| {
            a.appearance
                .typed(patterns)
                .map(|appearance| IfccadOpaqueAppearance {
                    appearance,
                    line_pattern_scale: a.line_pattern_scale,
                })
        })
        .transpose()?;
    Ok(IfccadEntity::Opaque(IfccadOpaqueEntity {
        id,
        preservation_record_id: IfccadPreservationRecordId(record),
        layer_id: layer,
        appearance,
        visible: value.visible,
    }))
}
