use ocdraw::ocdraw::*;
fn drawing() -> OcdrawDocument {
    load_ocdraw_bytes(include_bytes!(
        "../conformance/next/ocdraw/valid/placed-geometry.ocdraw.json"
    ))
    .ok()
    .unwrap()
    .into_document()
}
#[test]
fn recompute_uses_sparse_scope_ids_and_preserves_other_content() {
    let mut doc = drawing();
    for s in &mut doc.scopes {
        s.id += 10;
        s.bounds = None;
    }
    for l in &mut doc.layouts {
        l.scope_id += 10;
        l.id += 17;
    }
    doc.next_layout_id += 17;
    for b in &mut doc.block_definitions {
        b.scope_id += 10;
    }
    for e in &mut doc.geometric_entities {
        if let DrawingGeometry::BlockInstance {
            definition_scope_id,
            ..
        } = &mut e.geometry
        {
            *definition_scope_id += 10;
        }
    }
    doc.scopes.reverse();
    let ids = doc
        .scopes
        .iter()
        .map(|s| (s.id, s.entities.clone()))
        .collect::<Vec<_>>();
    recompute_ocdraw_document_bounds(&mut doc).unwrap();
    validate_ocdraw_document(&doc).unwrap();
    assert_eq!(
        ids,
        doc.scopes
            .iter()
            .map(|s| (s.id, s.entities.clone()))
            .collect::<Vec<_>>()
    );
}
#[test]
fn recompute_failure_does_not_partially_update_bounds() {
    let mut doc = drawing();
    let before = doc.scopes.iter().map(|s| s.bounds).collect::<Vec<_>>();
    doc.geometric_entities[0].geometry = DrawingGeometry::Line {
        start: [f64::NAN, 0., 0.],
        end: [0., 0., 0.],
    };
    assert!(recompute_ocdraw_document_bounds(&mut doc).is_err());
    assert_eq!(
        before,
        doc.scopes.iter().map(|s| s.bounds).collect::<Vec<_>>()
    );
}
#[test]
fn invalid_supplied_bounds_can_be_replaced_and_empty_scopes_remain_empty() {
    let mut doc = drawing();
    for s in &mut doc.scopes {
        s.bounds = Some(Bounds3d::new(
            Point3::new(f64::NAN, 0., 0.),
            Point3::new(-1., 0., 0.),
        ));
    }
    recompute_ocdraw_document_bounds(&mut doc).unwrap();
    validate_ocdraw_document(&doc).unwrap();
    let mut empty = load_ocdraw_bytes(include_bytes!(
        "../conformance/next/ocdraw/valid/empty.ocdraw.json"
    ))
    .ok()
    .unwrap()
    .into_document();
    recompute_ocdraw_document_bounds(&mut empty).unwrap();
    assert_eq!(empty.scopes[0].bounds, None);
}

#[test]
fn recompute_rejects_viewport_in_model_without_changing_bounds() {
    let mut d = load_ocdraw_bytes(include_bytes!(
        "../conformance/next/ocdraw/valid/paper-viewport.ocdraw.json"
    ))
    .ok()
    .unwrap()
    .into_document();
    let id = d.viewports[0].id;
    d.scopes[1].entities.clear();
    d.scopes[0].entities.push(id);
    let before = d.scopes.clone();
    assert!(recompute_ocdraw_document_bounds(&mut d).is_err());
    assert_eq!(before, d.scopes);
}
#[test]
fn late_block_overflow_leaves_previously_computed_scopes_unchanged() {
    let mut d = drawing();
    d.scopes.reverse();
    for e in &mut d.geometric_entities {
        if let DrawingGeometry::BlockInstance { transform, .. } = &mut e.geometry {
            *transform = BlockTransform::try_new(
                transform.placement(),
                0.,
                Scale3::new(f64::MAX, f64::MAX, f64::MAX),
            )
            .unwrap();
        }
    }
    let before = d.scopes.clone();
    assert!(recompute_ocdraw_document_bounds(&mut d).is_err());
    assert_eq!(before, d.scopes);
}
