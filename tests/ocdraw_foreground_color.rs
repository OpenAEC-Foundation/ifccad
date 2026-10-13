use ocdraw::ocdraw::{encode_ocdraw_document, load_ocdraw_bytes};
use serde_json::{json, Value};

fn base() -> Value {
    serde_json::from_slice(include_bytes!("../examples/ocdraw/text.ocdraw.json")).unwrap()
}

#[test]
fn editable_opaque_foreground_preserves_source_payload_bytes() {
    let mut v: Value = serde_json::from_slice(include_bytes!(
        "../examples/ocdraw/preserved-spline.ocdraw.json"
    ))
    .unwrap();
    let payload = v["preservation"].clone();
    v["streams"]["opaqueEntityStream"]["appearance"][0]["color"] =
        json!({"mode":"Explicit","value":{"kind":"Foreground"}});
    let loaded = load_ocdraw_bytes(&serde_json::to_vec(&v).unwrap()).unwrap();
    let encoded = encode_ocdraw_document(loaded.document()).unwrap();
    let written: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    assert_eq!(written["preservation"], payload);
    assert_eq!(
        written["streams"]["opaqueEntityStream"]["appearance"][0]["color"]["value"],
        json!({"kind":"Foreground"})
    );
    assert!(load_ocdraw_bytes(encoded.bytes()).is_ok());
}

fn read(v: &Value) -> bool {
    load_ocdraw_bytes(&serde_json::to_vec(v).unwrap()).is_ok()
}

#[test]
fn foreground_survives_layer_entity_inline_and_background_readback() {
    let mut v = base();
    let fg = json!({"kind":"Foreground"});
    v["layers"][0]["color"] = fg.clone();
    let stream = &mut v["streams"]["mTextStream"];
    stream["colorMode"][0] = json!("Explicit");
    stream["color"][0] = fg.clone();
    stream["characterFormat"][0]["color"] = json!({"kind":"explicit","color":fg});
    stream["background"][0] = json!({"fill":{"kind":"color","color":fg}});
    stream["content"][0][0]["inlines"][0]["characterFormat"]["color"] =
        json!({"kind":"explicit","color":fg});
    let loaded = load_ocdraw_bytes(&serde_json::to_vec(&v).unwrap()).unwrap();
    let encoded = encode_ocdraw_document(loaded.document()).unwrap();
    let restored = load_ocdraw_bytes(encoded.bytes()).unwrap();
    assert_eq!(loaded.typed_layers(), restored.typed_layers());
    assert_eq!(loaded.mtext_entities(), restored.mtext_entities());
    let written: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    assert_eq!(written["layers"][0]["color"], fg);
    assert_eq!(written["streams"]["mTextStream"]["color"][0], fg);
    assert_eq!(
        written["streams"]["mTextStream"]["background"][0]["fill"]["color"],
        fg
    );
    assert_eq!(
        written["streams"]["mTextStream"]["content"][0][0]["inlines"][0]["characterFormat"]
            ["color"]["color"],
        fg
    );
}

#[test]
fn foreground_grammar_rejects_mixed_and_legacy_aci_seven_values() {
    for color in [
        json!({"kind":"foreground"}),
        json!({"kind":"Foreground","rgb":[0,0,0]}),
        json!({"kind":"Foreground","indexedColor":{"system":"ACI","index":7}}),
        json!({"kind":"Foreground","namedColor":{"catalog":"x","name":"y"}}),
        json!({"kind":"Foreground","unknown":true}),
        json!(null),
        json!({"rgb":[255,255,255],"indexedColor":{"system":"ACI","index":7}}),
        json!({"rgb":[0,0,0],"indexedColor":{"system":"aCi","index":7}}),
    ] {
        let mut v = base();
        v["layers"][0]["color"] = color.clone();
        assert!(!read(&v), "accepted {color}");
    }
}

#[test]
fn fixed_and_other_identity_colors_remain_ordinary_values() {
    for color in [
        json!({"rgb":[0,0,0]}),
        json!({"rgb":[255,255,255]}),
        json!({"rgb":[255,0,0],"indexedColor":{"system":"ACI","index":1}}),
        json!({"rgb":[1,2,3],"indexedColor":{"system":"custom","index":18446744073709551615u64},"namedColor":{"catalog":"x","name":"y"}}),
    ] {
        let mut v = base();
        v["layers"][0]["color"] = color.clone();
        let loaded = load_ocdraw_bytes(&serde_json::to_vec(&v).unwrap()).unwrap();
        let written: Value =
            serde_json::from_slice(encode_ocdraw_document(loaded.document()).unwrap().bytes())
                .unwrap();
        assert_eq!(written["layers"][0]["color"], color);
    }
}
