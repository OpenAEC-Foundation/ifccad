use ocdraw::ifccad::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    let mut value: Value = serde_json::from_slice(include_bytes!(
        "../examples/ifccad/hello-nested-blocks.ifcx"
    ))
    .unwrap();
    for node in value["data"].as_array_mut().unwrap() {
        for role in ["ifccad::layout", "ifccad::blockDefinition"] {
            if let Some(fields) = node["attributes"]
                .get_mut(role)
                .and_then(Value::as_object_mut)
            {
                fields.remove("bounds");
                fields.remove("boundsQuality");
            }
        }
    }
    value
}
fn read(value: &Value) -> Result<ValidatedIfccad, IfccadReadError> {
    load_ifccad_bytes(&serde_json::to_vec(value).unwrap(), Default::default())
}

#[test]
fn ordinary_anonymous_block_metadata_survives_native_production_readback() {
    let mut value = fixture();
    let block = value["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find_map(|n| n["attributes"].get_mut("ifccad::blockDefinition"))
        .unwrap();
    block["name"] = json!("*U42");
    block["description"] = json!("Reusable anonymous definition");
    block["anonymous"] = json!(true);
    block["explodable"] = json!(false);
    block["uniformScaling"] = json!(false);
    let loaded = read(&value).unwrap();
    let encoded = encode_ifccad_document(loaded.document()).unwrap();
    assert_eq!(
        load_ifccad_bytes(encoded.bytes(), Default::default())
            .unwrap()
            .document(),
        loaded.document()
    );
    let raw: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    let block = raw["data"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|n| {
            n["attributes"]
                .get("ifccad::blockDefinition")
                .filter(|b| b["name"] == "*U42")
        })
        .unwrap();
    assert_eq!(block["description"], "Reusable anonymous definition");
    assert_eq!(block["anonymous"], json!(true));
    assert_eq!(block["explodable"], json!(false));
}

#[test]
fn uniform_policy_checks_signed_components_for_every_nested_instance() {
    for (scale, valid) in [
        ([-2., -2., -2.], true),
        ([2., 2., 2.], true),
        ([-2., 2., 2.], false),
        ([2., 2., 3.], false),
    ] {
        let mut value = fixture();
        for node in value["data"].as_array_mut().unwrap() {
            if let Some(block) = node["attributes"].get_mut("ifccad::blockDefinition") {
                block["uniformScaling"] = json!(true);
            }
            if let Some(instance) = node["attributes"].get_mut("ifccad::blockInstance") {
                instance["transform"]["scale"] = json!(scale);
            }
        }
        assert_eq!(read(&value).is_ok(), valid, "{scale:?}");
    }
}

#[test]
fn new_block_fields_reject_nulls_unknowns_and_wrong_types() {
    for (key, invalid) in [
        ("description", Value::Null),
        ("anonymous", Value::Null),
        ("explodable", Value::Null),
        ("uniformScaling", Value::Null),
        ("unknown", json!(true)),
        ("anonymous", json!(1)),
    ] {
        let mut value = fixture();
        let block = value["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find_map(|n| n["attributes"].get_mut("ifccad::blockDefinition"))
            .unwrap();
        block[key] = invalid;
        assert!(read(&value).is_err(), "{key}");
    }
}
