use ocdraw::ocdraw::*;
use serde_json::{json, Value};

fn model() -> Value {
    serde_json::from_slice(include_bytes!(
        "../examples/ocdraw/state-and-storage.ocdraw.json"
    ))
    .unwrap()
}
fn paper() -> Value {
    let mut value: Value = serde_json::from_slice(include_bytes!(
        "../conformance/next/ocdraw/valid/paper-viewport.ocdraw.json"
    ))
    .unwrap();
    let source = model();
    let window = &source["modelWindows"][0];
    let scope = value["layouts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["kind"] == "paper")
        .unwrap()["scopeId"]
        .clone();
    value["paperCanvases"] = json!([{"scopeId":scope,"view":window["view"],"grid":window["grid"],"snap":window["snap"],"storedUcs":{"kind":"World"},"currentUcs":{"kind":"World"},"activeContext":{"kind":"Canvas"}}]);
    value
}
fn read(value: &Value) -> Result<ValidatedOcdraw, OcdrawReadError> {
    load_ocdraw_bytes(&serde_json::to_vec(value).unwrap())
}
fn roundtrip(value: &Value) -> Value {
    let loaded = read(value).unwrap();
    validate_ocdraw_document(loaded.document()).unwrap();
    let encoded = encode_ocdraw_document(loaded.document()).unwrap();
    load_ocdraw_bytes(encoded.bytes())
        .unwrap()
        .as_value()
        .clone()
}

#[test]
fn windows_survive_without_a_selected_window() {
    let mut value = model();
    value["drawingViewState"]
        .as_object_mut()
        .unwrap()
        .remove("activeModelWindowId");
    let back = roundtrip(&value);
    assert_eq!(back["modelWindows"], value["modelWindows"]);
    assert!(back["drawingViewState"]
        .get("activeModelWindowId")
        .is_none());
    value.as_object_mut().unwrap().remove("drawingViewState");
    assert!(roundtrip(&value).get("drawingViewState").is_none());
}
#[test]
fn model_ucs_survives_without_window_snapshot() {
    let mut value = model();
    value.as_object_mut().unwrap().remove("modelWindows");
    value["drawingViewState"]
        .as_object_mut()
        .unwrap()
        .remove("activeModelWindowId");
    let back = roundtrip(&value);
    assert_eq!(
        back["drawingViewState"]["currentModelUcs"],
        value["drawingViewState"]["currentModelUcs"]
    );
    assert!(back.get("modelWindows").is_none());
}
#[test]
fn unknown_canvas_context_stays_unknown() {
    let mut value = paper();
    let canvas = value["paperCanvases"][0].as_object_mut().unwrap();
    canvas.remove("activeContext");
    canvas.remove("currentUcs");
    let back = roundtrip(&value);
    assert!(back["paperCanvases"][0].get("activeContext").is_none());
    assert!(back["paperCanvases"][0].get("currentUcs").is_none());
    assert_eq!(
        back["paperCanvases"][0]["storedUcs"],
        json!({"kind":"World"})
    );
}
fn change_current_ucs(value: &mut Value) {
    value["paperCanvases"][0]["currentUcs"] = json!({"kind":"Unnamed","frame":{"origin":{"x":1,"y":0,"z":0},"X":{"x":1,"y":0,"z":0},"Y":{"x":0,"y":1,"z":0}}});
}
#[test]
fn disabled_activation_preserves_different_current_ucs() {
    let mut value = paper();
    change_current_ucs(&mut value);
    value["paperCanvases"][0]["useStoredUcs"] = json!(false);
    let back = roundtrip(&value);
    assert_eq!(back["paperCanvases"][0]["currentUcs"]["kind"], "Unnamed");
    assert_eq!(
        back["paperCanvases"][0]["currentUcs"]["frame"]["origin"]["x"].as_f64(),
        Some(1.)
    );
    assert_eq!(back["paperCanvases"][0]["useStoredUcs"], false);
    value["paperCanvases"][0]["useStoredUcs"] = json!(true);
    assert!(read(&value).is_err());
}
#[test]
fn missing_canvas_activation_preserves_existing_true_rule() {
    let mut value = paper();
    assert_eq!(roundtrip(&value)["paperCanvases"][0]["useStoredUcs"], true);
    change_current_ucs(&mut value);
    assert!(read(&value).is_err());
}
#[test]
fn canvas_frame_does_not_change_geometry_bounds() {
    let mut value = paper();
    let before = roundtrip(&value);
    value["paperCanvases"][0]["frame"] =
        json!({"center":{"x":100,"y":-50,"z":3},"width":10,"height":8});
    value["paperCanvases"][0]["snap"]["enabled"] = json!(false);
    value["paperCanvases"][0]["snap"]["spacing"] = json!({"x":0,"y":0});
    let back = roundtrip(&value);
    assert_eq!(back["scopes"], before["scopes"]);
    assert_eq!(back["streams"], before["streams"]);
    assert_eq!(
        back["paperCanvases"][0]["frame"]["center"]["z"].as_f64(),
        Some(3.)
    );
    value["paperCanvases"][0]["snap"]["enabled"] = json!(true);
    assert!(read(&value).is_err());
}
#[test]
fn malformed_optional_state_is_not_replaced_with_absence() {
    let mut value = model();
    value["drawingViewState"]["activeModelWindowId"] = Value::Null;
    assert!(read(&value).is_err());
    let mut value = paper();
    value["paperCanvases"][0]["activeContext"] = Value::Null;
    assert!(read(&value).is_err());
    value = paper();
    value["paperCanvases"][0]
        .as_object_mut()
        .unwrap()
        .remove("activeContext");
    assert!(read(&value).is_err());
    value = paper();
    value["paperCanvases"][0]["frame"] = json!({"center":{"x":0,"y":0,"z":0},"width":0,"height":1});
    assert!(read(&value).is_err());
}

#[test]
fn viewport_workspace_can_exist_without_canvas() {
    let mut value = paper();
    let canvas = value["paperCanvases"][0].clone();
    value.as_object_mut().unwrap().remove("paperCanvases");
    let id = value["streams"]["viewportStream"]["entityId"][0].clone();
    value["viewportWorkspaces"] = json!([{"viewportEntityId":id,"grid":canvas["grid"],"snap":canvas["snap"],"storedUcs":{"kind":"World"},"useStoredUcs":false}]);
    let back = roundtrip(&value);
    assert_eq!(back["viewportWorkspaces"][0]["viewportEntityId"], id);
    assert!(back.get("paperCanvases").is_none());
}
