use ocdraw::ifccad::*;
use serde_json::{json, Value};
#[path = "support/ifccad_preservation.rs"]
mod preservation_support;

fn migrate_fixture_colors(value: &mut Value) {
    match value {
        Value::String(s) if s.len() == 7 && s.starts_with('#') => {
            if let Ok(rgb) = u32::from_str_radix(&s[1..], 16) {
                *value = json!({"rgb": [(rgb >> 16) & 255, (rgb >> 8) & 255, rgb & 255]});
            }
        }
        Value::Object(fields) => fields.values_mut().for_each(migrate_fixture_colors),
        Value::Array(values) => values.iter_mut().for_each(migrate_fixture_colors),
        _ => {}
    }
}
fn fixture(bytes: &[u8]) -> Value {
    let mut value: Value = serde_json::from_slice(bytes).unwrap();
    migrate_fixture_colors(&mut value);
    value
}
fn color() -> Value {
    json!({"rgb":[255,0,0],"indexedColor":{"system":"ACI","index":1},"namedColor":{"catalog":"Example","name":"Red"}})
}
fn role<'a>(value: &'a mut Value, name: &str) -> &'a mut Value {
    value["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|node| node["attributes"].get(name).is_some())
        .unwrap()
}
fn read(value: &Value) -> Result<ValidatedIfccad, IfccadReadError> {
    load_ifccad_bytes(&serde_json::to_vec(value).unwrap(), Default::default())
}
fn canonical(value: &Value) -> Value {
    let loaded = read(value).unwrap();
    let bytes = encode_ifccad_document(loaded.document()).unwrap();
    let restored = load_ifccad_bytes(bytes.bytes(), Default::default()).unwrap();
    assert_eq!(loaded.document(), restored.document());
    serde_json::from_slice(bytes.bytes()).unwrap()
}

#[test]
fn layer_and_entity_colors_keep_rgb_indexed_and_named_identity() {
    let mut value = fixture(include_bytes!("../examples/ifccad/hello-cad.ifcx"));
    role(&mut value, "ifccad::layer")["attributes"]["ifccad::layer"]["appearance"]["color"] =
        color();
    role(&mut value, "ifccad::entity")["attributes"]["ifccad::entity"]["appearance"]["color"] =
        json!({"mode":"Explicit","value":color()});
    let mut written = canonical(&value);
    assert_eq!(
        role(&mut written, "ifccad::layer")["attributes"]["ifccad::layer"]["appearance"]["color"],
        color()
    );
    assert_eq!(
        role(&mut written, "ifccad::entity")["attributes"]["ifccad::entity"]["appearance"]["color"]
            ["value"],
        color()
    );
}

#[test]
fn color_objects_reject_removed_shapes_unknown_fields_and_explicit_nulls() {
    let base = fixture(include_bytes!("../examples/ifccad/hello-cad.ifcx"));
    assert!(read(&base).is_ok());
    for bad in [
        json!("#FF0000"),
        json!({"rgb":[0,0]}),
        json!({"rgb":[0,0,0,0]}),
        json!({"rgb":[-1,0,0]}),
        json!({"rgb":[256,0,0]}),
        json!({"rgb":[0.5,0,0]}),
        json!({"rgb":[0,0,0],"unknown":true}),
        json!({"rgb":[0,0,0],"indexedColor":null}),
        json!({"rgb":[0,0,0],"namedColor":null}),
        json!({"rgb":[0,0,0],"indexedColor":{"system":"","index":1}}),
        json!({"rgb":[0,0,0],"namedColor":{"catalog":"Example","name":""}}),
        json!({"rgb":[0,0,0],"indexedColor":{"system":"ACI","index":1,"unknown":true}}),
    ] {
        let mut value = base.clone();
        role(&mut value, "ifccad::layer")["attributes"]["ifccad::layer"]["appearance"]["color"] =
            bad.clone();
        assert!(read(&value).is_err(), "accepted {bad}");
    }
}

#[test]
fn native_generic_color_indices_keep_exact_uint64_values() {
    let mut value = fixture(include_bytes!("../examples/ifccad/hello-cad.ifcx"));
    let expected = json!({"rgb":[12,34,56],"indexedColor":{"system":"custom","index":u64::MAX}});
    role(&mut value, "ifccad::layer")["attributes"]["ifccad::layer"]["appearance"]["color"] =
        expected.clone();
    let mut written = canonical(&value);
    assert_eq!(
        role(&mut written, "ifccad::layer")["attributes"]["ifccad::layer"]["appearance"]["color"],
        expected
    );
}

#[test]
fn composed_replacement_does_not_retain_stale_color_metadata() {
    let mut value = fixture(include_bytes!("../examples/ifccad/hello-cad.ifcx"));
    let layer = role(&mut value, "ifccad::layer");
    layer["attributes"]["ifccad::layer"]["appearance"]["color"] = color();
    let mut replacement = layer.clone();
    replacement["attributes"]["ifccad::layer"]["appearance"]["color"] = json!({"rgb":[12,34,56]});
    value["data"].as_array_mut().unwrap().push(replacement);
    let mut written = canonical(&value);
    assert_eq!(
        role(&mut written, "ifccad::layer")["attributes"]["ifccad::layer"]["appearance"]["color"],
        json!({"rgb":[12,34,56]})
    );
}

#[test]
fn mtext_inline_and_background_colors_use_the_same_concrete_color_contract() {
    let mut value = fixture(include_bytes!("../examples/ifccad/hello-text.ifcx"));
    let text = &mut role(&mut value, "ifccad::mText")["attributes"]["ifccad::mText"];
    text["characterFormat"]["color"] = json!({"kind":"explicit","color":color()});
    text["background"] = json!({"fill":{"kind":"color","color":color()}});
    text["content"][0]["inlines"][0]["characterFormat"]["color"] =
        json!({"kind":"explicit","color":color()});
    let mut written = canonical(&value);
    let text = &role(&mut written, "ifccad::mText")["attributes"]["ifccad::mText"];
    assert_eq!(text["characterFormat"]["color"]["color"], color());
    assert_eq!(text["background"]["fill"]["color"], color());
    assert_eq!(
        text["content"][0]["inlines"][0]["characterFormat"]["color"]["color"],
        color()
    );
}

#[test]
fn editable_opaque_color_keeps_snapshot_payload_unchanged() {
    let original = preservation_support::with_opaque();
    let mut value: Value =
        serde_json::from_slice(encode_ifccad_document(&original).unwrap().bytes()).unwrap();
    migrate_fixture_colors(&mut value);
    let appearance =
        role(&mut value, "ifccad::entity")["attributes"]["ifccad::entity"]["appearance"].clone();
    let opaque =
        &mut role(&mut value, "ifccad::opaqueEntity")["attributes"]["ifccad::opaqueEntity"];
    opaque["nativeLayer"] = json!("/cad/d1/layer/0");
    opaque["nativeAppearance"] = json!({"appearance":appearance,"linePatternScale":1.0});
    opaque["nativeAppearance"]["appearance"]["color"] = json!({"mode":"Explicit","value":color()});
    let loaded = read(&value).unwrap();
    assert_eq!(
        loaded.document().preservation.as_ref().unwrap().records[0].payload,
        original.preservation.as_ref().unwrap().records[0].payload
    );
    let mut written = canonical(&value);
    assert_eq!(
        role(&mut written, "ifccad::opaqueEntity")["attributes"]["ifccad::opaqueEntity"]
            ["nativeAppearance"]["appearance"]["color"]["value"],
        color()
    );
}
