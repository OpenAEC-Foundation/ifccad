use ocdraw::ocdraw::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    serde_json::from_slice(include_bytes!(
        "../conformance/next/ocdraw/valid/paper-viewport.ocdraw.json"
    ))
    .unwrap()
}
#[test]
fn viewport_mode_is_independent_of_layout_quality_and_roundtrips_as_a_mode() {
    for mode in ["AsDisplayed", "Wireframe", "Hidden", "Rendered"] {
        let mut value = fixture();
        value["streams"]["viewportStream"]["plotShadingOverride"][0] = json!(mode);
        let loaded = load_ocdraw_bytes(&serde_json::to_vec(&value).unwrap()).unwrap();
        let encoded = encode_ocdraw_document(loaded.document()).unwrap();
        let restored = load_ocdraw_bytes(encoded.bytes()).unwrap();
        assert_eq!(restored.document().viewports, loaded.document().viewports);
        let raw: Value = serde_json::from_slice(encoded.bytes()).unwrap();
        assert_eq!(
            raw["streams"]["viewportStream"]["plotShadingOverride"][0],
            mode
        );
    }
}
#[test]
fn legacy_combined_viewport_quality_objects_are_rejected_in_the_active_contract() {
    let mut value = fixture();
    value["streams"]["viewportStream"]["plotShadingOverride"][0] =
        json!({"mode":"Hidden","quality":{"mode":"Custom","dpi":600}});
    assert!(load_ocdraw_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
}
