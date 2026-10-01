//! Stream recognition belongs to the versioned JSON codec, not the logical model.
use serde_json::Value;
use std::sync::OnceLock;
fn registry() -> &'static Value {
    static VALUE: OnceLock<Value> = OnceLock::new();
    VALUE.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../../schemas/ocdraw/registry-0.1.0.json"
        ))
        .expect("bundled registry")
    })
}
pub(super) fn mapping() -> &'static Value {
    static VALUE: OnceLock<Value> = OnceLock::new();
    VALUE.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../../../schemas/ocdraw/json-mapping-0.1.0.json"
        ))
        .expect("bundled mapping")
    })
}
/// Selects only present object streams by the fixed version's registry and mapping.
pub(super) fn object_streams(value: &Value) -> impl Iterator<Item = (&'static str, &Value)> {
    registry()["streams"]
        .as_array()
        .expect("bundled registry")
        .iter()
        .filter(|stream| stream["role"] == "object")
        .filter_map(move |stream| {
            let map = mapping()["streams"]
                .as_array()
                .expect("bundled mapping")
                .iter()
                .find(|map| map["name"] == stream["name"])?;
            let payload = map["payload"]
                .as_str()
                .expect("stream payload")
                .strip_prefix("streams.")
                .expect("stream path");
            value["streams"]
                .get(payload)
                .map(|stream| (payload, stream))
        })
}
