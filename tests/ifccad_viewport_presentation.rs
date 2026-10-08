use ocdraw::ifccad::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    serde_json::from_slice(include_bytes!("../examples/ifccad/hello-viewports.ifcx")).unwrap()
}
fn viewport(v: &mut Value) -> &mut Value {
    &mut v["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"].get("ifccad::viewport").is_some())
        .unwrap()["attributes"]["ifccad::viewport"]
}
fn read(v: &Value) -> Result<ValidatedIfccad, IfccadReadError> {
    load_ifccad_bytes(&serde_json::to_vec(v).unwrap(), Default::default())
}
fn row() -> Value {
    json!({"layer":"/cad/d1/layer/0","frozen":true,"color":{"rgb":[255,0,0],"indexedColor":{"system":"ACI","index":1},"namedColor":{"catalog":"Example","name":"Red"}},"opacity":0.5,"linePattern":"/cad/d1/linePattern/0","lineWeight":0.225})
}

#[test]
fn relational_overrides_and_plotmode_survive_native_readback_independently_of_render_mode() {
    let mut v = fixture();
    viewport(&mut v)["layerOverrides"]
        .as_array_mut()
        .unwrap()
        .push(row());
    viewport(&mut v)["plotShadingOverride"] = json!("Hidden");
    let source = read(&v).unwrap();
    let encoded = encode_ifccad_document(source.document()).unwrap();
    let restored = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    assert_eq!(source.document(), restored.document());
    let mut raw: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    let view = viewport(&mut raw);
    assert!(view.get("frozenLayers").is_none());
    assert_eq!(view["plotShadingOverride"], "Hidden");
    assert_eq!(view["renderMode"], "Wireframe");
    assert_eq!(view["layerOverrides"][0], row());
}

#[test]
fn duplicate_dangling_and_invalid_override_values_are_rejected() {
    let base = fixture();
    assert!(read(&base).is_ok());
    for bad in [
        json!([{"layer":"/cad/d1/layer/0","frozen":false}]),
        json!([row(), row()]),
        json!([{"layer":"/cad/d1/layer/999"}]),
        json!([{"layer":"/cad/d2/layer/0"}]),
        json!([{"layer":"/cad/d1/layer/0","linePattern":"/cad/d1/linePattern/999"}]),
        json!([{"layer":"/cad/d1/layer/0","opacity":2}]),
        json!([{"layer":"/cad/d1/layer/0","lineWeight":-1}]),
        json!([{"layer":"/cad/d1/layer/0","color":null}]),
        json!([{"layer":"/cad/d1/layer/0","frozen":null}]),
        json!([{"layer":"/cad/d1/layer/0","unknown":true}]),
        Value::Null,
    ] {
        let mut v = base.clone();
        viewport(&mut v)["layerOverrides"] = bad.clone();
        assert!(read(&v).is_err(), "accepted {bad}");
    }
}

#[test]
fn viewport_plotmode_rejects_quality_objects_nulls_and_removed_frozen_lists() {
    for bad in [
        json!({"mode":"Hidden","quality":{"mode":"Custom","dpi":600}}),
        Value::Null,
        json!("unknown"),
    ] {
        let mut v = fixture();
        viewport(&mut v)["plotShadingOverride"] = bad;
        assert!(read(&v).is_err());
    }
    let mut v = fixture();
    viewport(&mut v)["frozenLayers"] = json!([]);
    assert!(read(&v).is_err());
}
