mod common;
use common::*;
use ifccad::*;
use ifccad_convert::*;

#[test]
fn compact_and_expanded_sources_have_equal_cad_semantics() {
    let mut outputs = Vec::new();
    for bytes in [
        include_bytes!("../../../conformance/next/ifccad/valid/defaults-expanded.ifcx").as_slice(),
        include_bytes!("../../../conformance/next/ifccad/valid/defaults-compact.ifcx").as_slice(),
    ] {
        let source = load_ifccad_bytes(bytes, Default::default()).unwrap();
        let options = IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Allow,
            ..Default::default()
        };
        let source_cad = ifccad_source_to_cad_document(&source, options).unwrap();
        let direct = ifccad_document_to_cad_document(source.document(), options).unwrap();
        assert_eq!(source_cad.diagnostics(), direct.diagnostics());
        let restored = cad_document_to_ifccad_document(
            source_cad.document(),
            metadata(),
            CadToIfccadOptions {
                loss_policy: IfccadLossPolicy::Allow,
                ..Default::default()
            },
        )
        .unwrap();
        outputs.push(restored.into_document());
    }
    assert_eq!(outputs[0], outputs[1]);
}

#[test]
fn compact_defaults_do_not_hide_foreign_content_or_precision_loss() {
    use serde_json::{json, Value};
    let base: Value = serde_json::from_slice(include_bytes!(
        "../../../conformance/next/ifccad/valid/defaults-compact.ifcx"
    ))
    .unwrap();
    let mut foreign = base.clone();
    foreign["data"]
        .as_array_mut()
        .unwrap()
        .push(json!({"path":"/foreign","attributes":{"example::note":"retain me"}}));
    let source =
        load_ifccad_bytes(&serde_json::to_vec(&foreign).unwrap(), Default::default()).unwrap();
    let result = ifccad_source_to_cad_document(
        &source,
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Allow,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(result
        .diagnostics()
        .iter()
        .any(|d| d.code == "foreign-ifcx"));
    let mut imprecise = base;
    let node = imprecise["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"].get("ifccad::geom::lineSegment").is_some())
        .unwrap();
    node["attributes"]["ifccad::geom::lineSegment"]["start"][0] = json!(9_007_199_254_740_993_u64);
    let source =
        load_ifccad_bytes(&serde_json::to_vec(&imprecise).unwrap(), Default::default()).unwrap();
    for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        assert!(
            matches!(ifccad_source_to_cad_document(&source,IfccadToCadOptions {loss_policy:policy,..Default::default()}),Err(IfccadConversionError::Unsupported(d)) if d.iter().any(|d|d.code=="precision"))
        );
    }
}
