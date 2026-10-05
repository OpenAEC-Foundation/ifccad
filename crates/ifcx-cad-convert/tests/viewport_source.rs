mod common;
use ifcx_cad_convert::*;
use ocdraw::ifcx_cad::*;
use serde_json::{json, Value};

fn wire() -> Value {
    serde_json::from_slice(
        encode_ifcx_cad_document(&common::viewport_drawing())
            .unwrap()
            .bytes(),
    )
    .unwrap()
}

fn payload(wire: &mut Value) -> &mut Value {
    &mut wire["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|node| node["attributes"]["ifccad::viewport"].is_object())
        .unwrap()["attributes"]["ifccad::viewport"]
}

#[test]
fn source_viewport_numeric_rounding_is_fatal_under_both_policies() {
    for pointer in [
        "/frame/width",
        "/view/height",
        "/view/target/0",
        "/view/direction/2",
        "/view/twist",
        "/view/lensLengthMm",
        "/view/frontClip/distance",
        "/view/backClip/distance",
    ] {
        let mut raw = wire();
        *payload(&mut raw).pointer_mut(pointer).unwrap() = json!(9_007_199_254_740_993_u64);
        let source =
            load_ifcx_cad_bytes(&serde_json::to_vec(&raw).unwrap(), Default::default()).unwrap();
        for loss_policy in [IfcxCadLossPolicy::Allow, IfcxCadLossPolicy::Reject] {
            let result =
                ifcx_cad_source_to_cad_document(&source, IfcxCadToCadOptions { loss_policy });
            assert!(
                matches!(result, Err(IfcxCadConversionError::Unsupported(ref d))
                    if d.iter().any(|d| d.code == "precision")),
                "{pointer}, {loss_policy:?}: {:?}",
                result.err()
            );
        }
    }
}

#[test]
fn source_frozen_layer_set_order_is_not_a_semantic_loss() {
    let mut raw = wire();
    payload(&mut raw)["frozenLayers"] = json!(["/cad/d1/layer/4", "/cad/d1/layer/0"]);
    let source =
        load_ifcx_cad_bytes(&serde_json::to_vec(&raw).unwrap(), Default::default()).unwrap();
    let outcome = ifcx_cad_source_to_cad_document(
        &source,
        IfcxCadToCadOptions {
            loss_policy: IfcxCadLossPolicy::Reject,
        },
    )
    .unwrap();
    assert!(!outcome.diagnostics().iter().any(IfcxCadDiagnostic::is_loss));
}
