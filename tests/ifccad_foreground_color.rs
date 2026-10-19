use ifccad::{encode_ifccad_document, load_ifccad_bytes};
use serde_json::{json, Value};
#[path = "support/ifccad_preservation.rs"]
mod preservation_support;

#[test]
fn editable_opaque_foreground_preserves_source_payload_bytes() {
    let original = preservation_support::with_opaque();
    let mut v: Value =
        serde_json::from_slice(encode_ifccad_document(&original).unwrap().bytes()).unwrap();
    // Opaque appearance stays explicit even when native ByLayer fields are omitted.
    let appearance = json!({"color":{"mode":"ByLayer"},"opacity":{"mode":"ByLayer"},
        "linePattern":{"mode":"ByLayer"},"lineWeight":{"mode":"ByLayer"}});
    let opaque = role(&mut v, "ifccad::opaqueEntity");
    opaque["nativeLayer"] = json!("/cad/d1/layer/0");
    opaque["nativeAppearance"] = json!({"appearance":appearance,"linePatternScale":1.0});
    opaque["nativeAppearance"]["appearance"]["color"] =
        json!({"mode":"Explicit","value":{"kind":"Foreground"}});
    let read = load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).unwrap();
    assert_eq!(
        read.document().preservation.as_ref().unwrap().records[0].payload,
        original.preservation.as_ref().unwrap().records[0].payload
    );
    let encoded = encode_ifccad_document(read.document()).unwrap();
    let mut written: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    assert_eq!(
        role(&mut written, "ifccad::opaqueEntity")["nativeAppearance"]["appearance"]["color"]
            ["value"],
        json!({"kind":"Foreground"})
    );
    assert!(load_ifccad_bytes(encoded.bytes(), Default::default()).is_ok());
}

fn base() -> Value {
    serde_json::from_slice(include_bytes!("../examples/ifccad/text.ifcx")).unwrap()
}
fn role<'a>(v: &'a mut Value, name: &str) -> &'a mut Value {
    &mut v["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"].get(name).is_some())
        .unwrap()["attributes"][name]
}

#[test]
fn foreground_survives_layer_entity_inline_and_background_readback() {
    let mut v = base();
    let fg = json!({"kind":"Foreground"});
    role(&mut v, "ifccad::layer")["appearance"]["color"] = fg.clone();
    role(&mut v, "ifccad::entity")["appearance"]["color"] = json!({"mode":"Explicit","value":fg});
    let text = role(&mut v, "ifccad::mText");
    text["characterFormat"]["color"] = json!({"kind":"explicit","color":fg});
    text["background"] = json!({"fill":{"kind":"color","color":fg}});
    text["content"][0]["inlines"][0]["characterFormat"]["color"] =
        json!({"kind":"explicit","color":fg});
    let loaded = load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).unwrap();
    let encoded = encode_ifccad_document(loaded.document()).unwrap();
    let restored = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    assert_eq!(loaded.document(), restored.document());
    let mut written: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    assert_eq!(
        role(&mut written, "ifccad::layer")["appearance"]["color"],
        fg
    );
    assert_eq!(
        role(&mut written, "ifccad::entity")["appearance"]["color"]["value"],
        fg
    );
    assert_eq!(
        role(&mut written, "ifccad::mText")["background"]["fill"]["color"],
        fg
    );
    assert_eq!(
        role(&mut written, "ifccad::mText")["content"][0]["inlines"][0]["characterFormat"]["color"]
            ["color"],
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
        role(&mut v, "ifccad::layer")["appearance"]["color"] = color.clone();
        assert!(
            load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).is_err(),
            "accepted {color}"
        );
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
        role(&mut v, "ifccad::layer")["appearance"]["color"] = color.clone();
        let loaded =
            load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).unwrap();
        let mut written: Value =
            serde_json::from_slice(encode_ifccad_document(loaded.document()).unwrap().bytes())
                .unwrap();
        assert_eq!(
            role(&mut written, "ifccad::layer")["appearance"]["color"],
            color
        );
    }
}
