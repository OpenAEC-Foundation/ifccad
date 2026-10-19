mod common;
#[path = "../../../tests/support/cad_hatch_associations.rs"]
mod hatch_sources;
#[path = "../../../tests/support/cad_dimension_repair.rs"]
mod repair;
mod spline_support;
use ifccad::*;
use ifccad_convert::*;
#[path = "../../../tests/support/ifccad_dimension.rs"]
mod dimensions;

#[test]
fn cbor_documents_use_the_existing_logical_converter_without_graph_audit() {
    let hello = load_ifccad_bytes(
        include_bytes!("../../../tests/fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let repaired =
        cad_document_to_ifccad_document(&repair::drawing(), common::metadata(), Default::default())
            .unwrap()
            .into_document();
    assert_eq!(
        repaired
            .model
            .entities
            .iter()
            .filter_map(IfccadEntity::as_native)
            .filter(|e| matches!(e.kind, IfccadEntityKind::Dimension(_)))
            .count(),
        2
    );
    let (hatch_cad, _, _) = hatch_sources::source(false, false);
    let hatch_cad = hatch_sources::exchange(&hatch_cad, false);
    let associated =
        cad_document_to_ifccad_document(&hatch_cad, common::metadata(), Default::default())
            .unwrap()
            .into_document();
    assert!(associated.model.entities.iter().filter_map(IfccadEntity::as_native).any(|e| matches!(&e.kind, IfccadEntityKind::Hatch(h) if h.loops.iter().any(|l| l.source_entity_ids.len() > 1))));
    for document in [
        hello,
        dimensions::drawing(),
        spline_support::captured(),
        repaired,
        associated,
    ] {
        let bytes = encode_ifccad_cbor_document(&document).unwrap();
        let loaded = load_ifccad_cbor_bytes(bytes.bytes(), Default::default()).unwrap();
        let before = ifccad_document_to_cad_document(&document, Default::default()).unwrap();
        let after = ifccad_document_to_cad_document(loaded.document(), Default::default()).unwrap();
        assert_eq!(before.diagnostics(), after.diagnostics());
        let options = CadToIfccadOptions {
            preservation: IfccadPreservationCapture::SupportedTyped,
            ..Default::default()
        };
        let a = cad_document_to_ifccad_document(before.document(), common::metadata(), options)
            .unwrap();
        let b =
            cad_document_to_ifccad_document(after.document(), common::metadata(), options).unwrap();
        assert_eq!(a.document(), b.document());
        if document.preservation.is_some() {
            assert!(after
                .document()
                .entities()
                .any(|e| matches!(e, opencadcodec::EntityType::Spline(_))));
        }
    }
}
