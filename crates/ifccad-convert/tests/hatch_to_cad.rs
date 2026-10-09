use ifccad_convert::{ifccad_document_to_cad_document, IfccadToCadOptions};
#[path = "../../../tests/support/ifccad_hatch.rs"]
mod fixture;
#[test]
fn native_ifccad_hatch_materializes_contours_and_association() {
    let d = fixture::drawing();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ifccad_document_to_cad_document(&d, IfccadToCadOptions::default())
    }))
    .expect("recognized native Hatch must not panic in conversion");
    let out = outcome.unwrap();
    assert_eq!(
        out.document()
            .entities()
            .filter(|e| matches!(e, opencadcodec::EntityType::Hatch(_)))
            .count(),
        1
    );
    assert!(!out.geometry_assessment().is_complete());
}

#[test]
fn pattern_with_large_ids_keeps_phase_and_stored_contours_after_source_edits() {
    use ocdraw::ifccad::*;
    use opencadcodec::{entities::hatch::BoundaryEdge, EntityType, Vector2};
    let mut d = fixture::drawing();
    let pattern = load_ifccad_bytes(
        include_bytes!("../../../conformance/next/ifccad/valid/hatch-pattern.ifcx"),
        Default::default(),
    )
    .unwrap();
    let fill = pattern
        .document()
        .model
        .entities
        .iter()
        .filter_map(IfccadEntity::as_native)
        .find_map(|e| {
            if let IfccadEntityKind::Hatch(h) = &e.kind {
                Some(h.fill.clone())
            } else {
                None
            }
        })
        .unwrap();
    for e in d
        .model
        .entities
        .iter_mut()
        .filter_map(IfccadEntity::as_native_mut)
    {
        match &mut e.kind {
            IfccadEntityKind::Hatch(h) => h.fill = fill.clone(),
            IfccadEntityKind::Circle { radius, .. } => *radius = 7.,
            _ => {}
        }
    }
    let encoded = encode_ifccad_document(&d).unwrap();
    let loaded = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    let out = ifccad_document_to_cad_document(loaded.document(), Default::default()).unwrap();
    let hh = out.mappings().entities.cad_handle(fixture::HATCH).unwrap();
    let ch = out.mappings().entities.cad_handle(fixture::SOURCE).unwrap();
    let EntityType::Hatch(h) = out.document().get_entity(hh).unwrap() else {
        panic!()
    };
    assert_eq!(h.pattern_origin(), Vector2::new(0.5, -0.5));
    assert_eq!(h.paths[1].boundary_handles, vec![ch]);
    let BoundaryEdge::CircularArc(a) = &h.paths[1].edges[0] else {
        panic!()
    };
    assert_eq!(a.radius, 1.);
    let EntityType::Circle(c) = out.document().get_entity(ch).unwrap() else {
        panic!()
    };
    assert_eq!(c.radius, 7.);
}
