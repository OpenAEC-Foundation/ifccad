use ocdraw::ifccad::*;

fn base() -> IfccadDocument {
    load_ifccad_bytes(
        include_bytes!("fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document()
}

#[test]
fn binary_roundtrip_preserves_complete_document() {
    let document = base();
    let encoded = encode_ifccad_cbor_document(&document).unwrap();
    let loaded = load_ifccad_cbor_bytes(encoded.bytes(), Default::default()).unwrap();
    assert_eq!(loaded.document(), &document);
    assert_eq!(
        encoded.bytes(),
        encode_ifccad_cbor_document(&document).unwrap().bytes()
    );
}

#[test]
fn compact_and_expanded_current_conformance_survives_cbor() {
    let mut seen = std::collections::BTreeSet::new();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("conformance/next/ifccad");
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../conformance/next/ifccad/cases.json")).unwrap();
    for case in cases.iter().filter(|c| c["valid"] == true) {
        let name = case["file"].as_str().unwrap();
        let source = std::fs::read(root.join(name)).unwrap();
        let document = load_ifccad_bytes(&source, Default::default())
            .unwrap()
            .into_document();
        for e in document
            .model
            .entities
            .iter()
            .chain(document.paper_layouts.iter().flat_map(|p| &p.entities))
            .chain(document.blocks.iter().flat_map(|b| &b.entities))
        {
            if let Some(e) = e.as_native() {
                seen.insert(match e.kind {
                    IfccadEntityKind::LineSegment { .. } => 0,
                    IfccadEntityKind::Point { .. } => 1,
                    IfccadEntityKind::Circle { .. } => 2,
                    IfccadEntityKind::Arc { .. } => 3,
                    IfccadEntityKind::Ellipse { .. } => 4,
                    IfccadEntityKind::EllipseArc { .. } => 5,
                    IfccadEntityKind::PlanarPolyline { .. } => 6,
                    IfccadEntityKind::SpatialPolyline { .. } => 7,
                    IfccadEntityKind::BlockInstance { .. } => 8,
                    IfccadEntityKind::Viewport(_) => 9,
                    IfccadEntityKind::Text(_) => 10,
                    IfccadEntityKind::MText(_) => 11,
                    IfccadEntityKind::Hatch(_) => 12,
                    IfccadEntityKind::Dimension(_) => 13,
                });
            }
        }
        let encoded =
            encode_ifccad_cbor_document(&document).unwrap_or_else(|e| panic!("{name}: {e}"));
        let loaded = load_ifccad_cbor_bytes(encoded.bytes(), Default::default()).unwrap();
        assert_eq!(loaded.document(), &document, "{name}");
    }
    assert_eq!(
        seen,
        (0..14).collect(),
        "all current native families must be exercised"
    );
}

#[test]
fn unknown_revision_is_not_best_effort() {
    for revision in [1_u8, 3] {
        // {0: "IFCCAD-CBOR", 1: revision, 2: {}}; do not map the document.
        let mut bytes = b"\xa3\x00\x6bIFCCAD-CBOR\x01\x00\x02\xa0".to_vec();
        bytes[15] = revision;
        assert!(matches!(load_ifccad_cbor_bytes(&bytes, Default::default()),
            Err(IfccadCborReadError::UnsupportedRevision {found, supported: 2}) if found == u32::from(revision)));
    }
}
