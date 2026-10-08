use ocdraw::ifccad::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    serde_json::from_slice(include_bytes!("../examples/ifccad/hello-text.ifcx")).unwrap()
}
fn role<'a>(value: &'a mut Value, key: &str) -> &'a mut Value {
    &mut value["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"].get(key).is_some())
        .unwrap()["attributes"][key]
}
fn read(value: &Value) -> Result<ValidatedIfccad, IfccadReadError> {
    load_ifccad_bytes(&serde_json::to_vec(value).unwrap(), Default::default())
}
fn canonical(value: &Value) -> Value {
    let loaded = read(value).unwrap();
    let encoded = encode_ifccad_document(loaded.document()).unwrap();
    assert_eq!(
        load_ifccad_bytes(encoded.bytes(), Default::default())
            .unwrap()
            .document(),
        loaded.document()
    );
    serde_json::from_slice(encoded.bytes()).unwrap()
}

#[test]
fn omitted_status_fields_keep_the_document_visible_plottable_and_unrestricted() {
    let value = fixture();
    let loaded = read(&value).unwrap();
    for layer in &loaded.document().layers {
        assert!(layer.visible && layer.plottable);
        assert!(!layer.frozen && !layer.locked && !layer.frozen_in_new_viewports);
        assert!(layer.description.is_none());
    }
    for entity in &loaded.document().model.entities {
        assert!(entity.as_native().unwrap().visible);
    }
    canonical(&value);
}

#[test]
fn layer_status_fields_and_description_are_independent_authored_values() {
    let mut value = fixture();
    let layer = role(&mut value, "ifccad::layer");
    for (field, setting) in [
        ("visible", false),
        ("frozen", true),
        ("locked", true),
        ("plottable", false),
        ("frozenInNewViewports", true),
    ] {
        layer[field] = json!(setting);
    }
    layer["description"] = json!("A hidden locked layer that does not plot");
    let mut written = canonical(&value);
    let layer = role(&mut written, "ifccad::layer");
    for (field, expected) in [
        ("visible", false),
        ("frozen", true),
        ("locked", true),
        ("plottable", false),
        ("frozenInNewViewports", true),
    ] {
        assert_eq!(layer[field], json!(expected), "{field}");
    }
    assert_eq!(
        layer["description"],
        "A hidden locked layer that does not plot"
    );
}

#[test]
fn hidden_geometry_text_and_instances_keep_content_order_and_bounds() {
    let mut value = fixture();
    let before = read(&value).unwrap().into_document();
    let mut count = 0;
    for node in value["data"].as_array_mut().unwrap() {
        if let Some(entity) = node["attributes"].get_mut("ifccad::entity") {
            entity["visible"] = json!(false);
            count += 1;
        }
    }
    assert!(count >= 4);
    let after = read(&value).unwrap().into_document();
    assert_eq!(before.model.bounds, after.model.bounds);
    assert_eq!(
        before
            .model
            .entities
            .iter()
            .map(IfccadEntity::id)
            .collect::<Vec<_>>(),
        after
            .model
            .entities
            .iter()
            .map(IfccadEntity::id)
            .collect::<Vec<_>>()
    );
    let written = canonical(&value);
    assert_eq!(
        written["data"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["attributes"].get("ifccad::entity").is_some())
            .count(),
        count
    );
    for node in written["data"].as_array().unwrap() {
        if let Some(entity) = node["attributes"].get("ifccad::entity") {
            assert_eq!(entity["visible"], json!(false));
        }
    }
}

#[test]
fn status_nulls_and_unknown_fields_are_not_silently_ignored() {
    for key in [
        "visible",
        "frozen",
        "locked",
        "plottable",
        "frozenInNewViewports",
        "description",
        "unexpected",
    ] {
        let mut value = fixture();
        role(&mut value, "ifccad::layer")[key] = Value::Null;
        assert!(read(&value).is_err(), "layer {key}");
    }
    let mut value = fixture();
    role(&mut value, "ifccad::entity")["visible"] = Value::Null;
    assert!(read(&value).is_err());
}

#[test]
fn viewport_visibility_belongs_to_common_entity_state() {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("../examples/ifccad/hello-viewports.ifcx")).unwrap();
    let node = value["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"].get("ifccad::viewport").is_some())
        .unwrap();
    node["attributes"]["ifccad::viewport"]
        .as_object_mut()
        .unwrap()
        .remove("visible");
    node["attributes"]["ifccad::entity"]["visible"] = json!(false);
    let mut written = canonical(&value);
    let viewport = role(&mut written, "ifccad::viewport");
    assert!(viewport.get("visible").is_none());
    let hidden = written["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["attributes"].get("ifccad::viewport").is_some())
        .unwrap();
    assert_eq!(
        hidden["attributes"]["ifccad::entity"]["visible"],
        json!(false)
    );
    let mut invalid = value;
    role(&mut invalid, "ifccad::viewport")["visible"] = json!(false);
    assert!(read(&invalid).is_err());
}
