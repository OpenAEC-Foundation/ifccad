mod common;
use common::metadata;
use ifccad_convert::*;
use ocdraw::ifccad::*;
use opencadcodec::{CadDocument, EntityType, Text, Vector3};
use serde_json::{json, Value};
fn source() -> Value {
    let mut cad = CadDocument::new();
    cad.add_entity(EntityType::Text(Text::with_value("Literal", Vector3::ZERO)))
        .unwrap();
    let d = cad_document_to_ifccad_document(&cad, metadata(), Default::default()).unwrap();
    serde_json::from_slice(encode_ifccad_document(d.document()).unwrap().bytes()).unwrap()
}
fn text(v: &mut Value) -> &mut Value {
    &mut v["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"].get("ifccad::text").is_some())
        .unwrap()["attributes"]["ifccad::text"]
}
#[test]
fn source_text_defaults_are_semantic_and_original_bytes_stay_immutable() {
    let mut v = source();
    let t = text(&mut v).as_object_mut().unwrap();
    for key in [
        "rotation",
        "backward",
        "upsideDown",
        "thickness",
        "obliqueAngle",
    ] {
        t.remove(key);
    }
    let bytes = serde_json::to_vec(&v).unwrap();
    let loaded = load_ifccad_bytes(&bytes, Default::default()).unwrap();
    let out = ifccad_source_to_cad_document(&loaded, Default::default()).unwrap();
    assert!(!out.diagnostics().iter().any(|d| d.code == "foreign-ifcx"));
    assert_eq!(loaded.graph().source_bytes(), bytes);
}
#[test]
fn exact_source_text_numbers_cannot_be_rounded_under_either_loss_policy() {
    let mut v = source();
    text(&mut v)["layout"]["widthFactor"] = json!(9007199254740993u64);
    let bytes = serde_json::to_vec(&v).unwrap();
    let loaded = load_ifccad_bytes(&bytes, Default::default()).unwrap();
    for loss_policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        assert!(
            matches!(ifccad_source_to_cad_document(&loaded,IfccadToCadOptions{loss_policy,..Default::default()}),Err(IfccadConversionError::Unsupported(d)) if d.iter().any(|d|d.code=="precision"))
        );
    }
    assert_eq!(loaded.graph().source_bytes(), bytes);
}

#[test]
fn absent_style_creation_history_gets_located_cad_default_loss() {
    let bytes = serde_json::to_vec(&source()).unwrap();
    let mut d = load_ifccad_bytes(&bytes, Default::default())
        .unwrap()
        .into_document();
    d.text_styles[0].properties.last_used_height = None;
    let out = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "text-style-default"));
    assert!(matches!(
        ifccad_document_to_cad_document(
            &d,
            IfccadToCadOptions {
                loss_policy: IfccadLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(IfccadConversionError::Unsupported(_))
    ));
}
