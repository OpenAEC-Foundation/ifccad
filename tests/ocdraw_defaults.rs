use ocdraw::ocdraw::*;
use serde_json::{json, Value};
#[path = "support/drawing_defaults.rs"]
mod support;

fn fixture() -> Value {
    serde_json::from_slice(include_bytes!(
        "fixtures/ocdraw/state-and-storage.ocdraw.json"
    ))
    .unwrap()
}
fn read(v: &Value) -> OcdrawDocument {
    load_ocdraw_bytes(&serde_json::to_vec(v).unwrap())
        .unwrap()
        .into_document()
}
fn invalid(v: &Value) -> bool {
    load_ocdraw_bytes(&serde_json::to_vec(v).unwrap()).is_err()
}

#[test]
fn layer_omission_matches_expanded_input() {
    let mut v = fixture();
    let expected = read(&v);
    for key in [
        "visible",
        "frozen",
        "locked",
        "plottable",
        "frozenInNewViewports",
        "opacity",
    ] {
        v["layers"][0].as_object_mut().unwrap().remove(key);
    }
    support::assert_ocdraw_eq(&read(&v), &expected);
}

#[test]
fn layer_explicit_false_and_zero_are_retained() {
    let mut v = fixture();
    v["layers"][0]["visible"] = json!(false);
    v["layers"][0]["plottable"] = json!(false);
    v["layers"][0]["opacity"] = json!(0.);
    let doc = read(&v);
    assert!(!doc.layers[0].visible);
    assert!(!doc.layers[0].plottable);
    assert_eq!(doc.layers[0].opacity, 0.);
}

#[test]
fn polyline_absence_means_open_and_preserves_dormant_bulge() {
    let mut v = fixture();
    v["streams"]["spatialPolylineStream"]["closed"] = json!([false]);
    *v["streams"]["planarPolylineStream"]["bulge"]
        .as_array_mut()
        .unwrap()
        .last_mut()
        .unwrap() = json!(0.5);
    let expected = read(&v);
    for stream in ["planarPolylineStream", "spatialPolylineStream"] {
        v["streams"][stream]
            .as_object_mut()
            .unwrap()
            .remove("closed");
    }
    support::assert_ocdraw_eq(&read(&v), &expected);
}

#[test]
fn null_wrong_type_and_short_columns_remain_invalid() {
    for key in [
        "visible",
        "frozen",
        "locked",
        "plottable",
        "frozenInNewViewports",
        "opacity",
    ] {
        for bad in [Value::Null, json!("wrong")] {
            let mut v = fixture();
            v["layers"][0][key] = bad;
            assert!(invalid(&v), "{key}");
        }
    }
    for bad in [Value::Null, json!([null]), json!([]), json!([false, false])] {
        let mut v = fixture();
        v["streams"]["planarPolylineStream"]["closed"] = bad;
        assert!(invalid(&v));
    }
    for (stream, pool) in [
        ("spatialPolylineStream", "z"),
        ("planarPolylineStream", "bulge"),
    ] {
        let mut v = fixture();
        v["streams"][stream].as_object_mut().unwrap().remove(pool);
        assert!(invalid(&v));
        let mut v = fixture();
        v["streams"][stream][pool] = json!([]);
        assert!(invalid(&v));
    }
    let mut v = fixture();
    v["streams"]["lineStream"]["z1"] = json!([0.]);
    assert!(invalid(&v));
}

#[test]
fn missing_closed_active_clip_is_rejected() {
    let mut v: Value = serde_json::from_slice(include_bytes!(
        "../conformance/next/ocdraw/invalid/viewport-open-clip.ocdraw.json"
    ))
    .unwrap();
    v["streams"]["planarPolylineStream"]
        .as_object_mut()
        .unwrap()
        .remove("closed");
    assert!(invalid(&v));
}

#[test]
fn compact_and_expanded_candidates_have_equal_native_semantics() {
    let expanded: Value = serde_json::from_slice(include_bytes!(
        "../conformance/next/ocdraw/valid/defaults-expanded.ocdraw.json"
    ))
    .unwrap();
    let compact: Value = serde_json::from_slice(include_bytes!(
        "../conformance/next/ocdraw/valid/defaults-compact.ocdraw.json"
    ))
    .unwrap();
    support::assert_ocdraw_eq(&read(&compact), &read(&expanded));
}
