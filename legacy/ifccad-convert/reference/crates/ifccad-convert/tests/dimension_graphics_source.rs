mod common;
#[path = "support/dimension_graphics.rs"]
mod support;
use ifccad::*;
use ifccad_convert::*;
use serde_json::{json, Value};
fn wire() -> Value {
    let n = cad_document_to_encoded_ifccad(
        &support::picture(false),
        common::metadata(),
        Default::default(),
    )
    .unwrap();
    serde_json::from_slice(n.encoded().bytes()).unwrap()
}
fn part(v: &mut Value) -> &mut Value {
    &mut v["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| {
            n["path"]
                .as_str()
                .is_some_and(|p| p.ends_with("/graphics/p0"))
        })
        .unwrap()["attributes"]
}
fn loaded(v: &Value) -> ValidatedIfccad {
    load_ifccad_bytes(&serde_json::to_vec(v).unwrap(), Default::default()).unwrap()
}
#[test]
fn graphics_defaults_are_known_profile_fields_in_the_source_audit() {
    let mut v = wire();
    part(&mut v)["ifccad::dimensionGraphicsPart"] = json!({"visible":true,"purpose":"display","linePatternScale":1.0,"appearance":{"color":{"mode":"ByLayer"},"opacity":{"mode":"ByLayer"},"linePattern":{"mode":"ByLayer"},"lineWeight":{"mode":"ByLayer"}}});
    let out = ifccad_source_to_cad_document(
        &loaded(&v),
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(out.diagnostics().is_empty());
}
#[test]
fn graphics_foreign_state_remains_visible_and_scalar_precision_remains_hard() {
    let mut v = wire();
    part(&mut v)["vendor::note"] = json!("authored");
    let source = loaded(&v);
    let out = ifccad_source_to_cad_document(&source, Default::default()).unwrap();
    assert!(out.diagnostics().iter().any(|d| d.code == "foreign-ifcx"));
    assert!(ifccad_source_to_cad_document(
        &source,
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        }
    )
    .is_err());
    let mut v = wire();
    part(&mut v)["ifccad::dimensionGraphicsPart"]["linePatternScale"] = json!(9007199254740993_u64);
    let source = loaded(&v);
    let Err(IfccadConversionError::Unsupported(v)) =
        ifccad_source_to_cad_document(&source, Default::default())
    else {
        panic!("part scalar projection must not be accepted")
    };
    assert!(v
        .iter()
        .any(|d| d.code == "precision" && d.location.contains("/graphics/p0")));
}
