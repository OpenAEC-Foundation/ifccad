use ocdraw::ifccad::*;
#[path = "support/ifccad_preservation.rs"]
mod support;
#[test]
fn opaque_geometry_is_unavailable_and_supplied_bounds_are_rejected() {
    let mut d = support::with_opaque();
    assert_eq!(
        derive_ifccad_geometry_completeness(&d).unwrap()[&IfccadScopeId::Layout(d.model.id)],
        IfccadGeometryCompleteness::Unavailable
    );
    recompute_ifccad_document_bounds(&mut d).unwrap();
    assert!(d.model.bounds.is_none());
    d.model.bounds = Some(IfccadBounds3d {
        min: [-1e9; 3],
        max: [1e9; 3],
    });
    assert!(validate_ifccad_document(&d).is_err());
}
#[test]
fn opaque_only_unused_and_nested_definitions_never_use_empty_origin_fallback() {
    let mut d = support::with_opaque();
    let opaque = d.model.entities.pop().unwrap();
    let leaf_id = d.id_counters.allocate_block_id().unwrap();
    d.blocks.push(IfccadBlockDefinition {
        id: leaf_id,
        name: "Opaque leaf".into(),
        base_point: [0.; 3],
        insertion_unit: "mm".into(),
        bounds: None,
        entities: vec![opaque],
    });
    let mut instance = d.model.entities[0].as_native().unwrap().clone();
    instance.id = d.id_counters.allocate_entity_id().unwrap();
    instance.kind = IfccadEntityKind::BlockInstance {
        definition_id: leaf_id,
        transform: IfccadBlockTransform {
            placement: IfccadPlacement {
                origin: [10.; 3],
                x_axis: [1., 0., 0.],
                y_axis: [0., 1., 0.],
            },
            rotation: 0.,
            scale: [1.; 3],
        },
    };
    d.model.entities.push(IfccadEntity::Native(instance));
    let empty_id = d.id_counters.allocate_block_id().unwrap();
    d.blocks.push(IfccadBlockDefinition {
        id: empty_id,
        name: "Empty".into(),
        base_point: [0.; 3],
        insertion_unit: "mm".into(),
        bounds: None,
        entities: vec![],
    });
    recompute_ifccad_document_bounds(&mut d).unwrap();
    assert!(d.model.bounds.is_none());
    let states = derive_ifccad_geometry_completeness(&d).unwrap();
    assert_eq!(
        states[&IfccadScopeId::BlockDefinition(leaf_id)],
        IfccadGeometryCompleteness::Unavailable
    );
    assert_eq!(
        states[&IfccadScopeId::BlockDefinition(empty_id)],
        IfccadGeometryCompleteness::Empty
    );
    let e = d
        .model
        .entities
        .last_mut()
        .unwrap()
        .as_native_mut()
        .unwrap();
    if let IfccadEntityKind::BlockInstance { transform, .. } = &mut e.kind {
        transform.scale = [f64::MAX; 3];
        transform.placement.x_axis = [1.0_f64.next_up(), 0., 0.];
    }
    let before = d.clone();
    assert!(recompute_ifccad_document_bounds(&mut d).is_err());
    assert_eq!(d, before);
}

#[test]
fn opaque_clip_is_valid_only_when_dormant_same_owner_and_exclusive() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-viewports.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let boundary = d
        .paper_layouts
        .iter_mut()
        .flat_map(|p| &mut p.entities)
        .find_map(|e| match &mut e.as_native_mut().unwrap().kind {
            IfccadEntityKind::Viewport(v) => {
                v.paper_clip.enabled = false;
                v.paper_clip.boundary_entity_id
            }
            _ => None,
        })
        .unwrap();
    let mut p = support::with_opaque().preservation.unwrap();
    let rid = d.id_counters.allocate_preservation_record_id().unwrap();
    p.records[0].id = rid;
    p.records[0].subject = Some(IfccadPreservationTarget::Entity(boundary));
    d.preservation = Some(p);
    let entity = d
        .paper_layouts
        .iter_mut()
        .flat_map(|p| &mut p.entities)
        .find(|e| e.id() == boundary)
        .unwrap();
    *entity = IfccadEntity::Opaque(IfccadOpaqueEntity {
        id: boundary,
        preservation_record_id: rid,
        layer_id: None,
        appearance: None,
        visible: true,
    });
    for p in &mut d.paper_layouts {
        p.bounds = None;
    }
    validate_ifccad_document(&d).unwrap();
    encode_ifccad_document(&d).unwrap();
    let original = d.clone();
    for mutation in 0..3 {
        let mut d = original.clone();
        match mutation {
            0 => {
                let viewport = d
                    .paper_layouts
                    .iter_mut()
                    .flat_map(|p| &mut p.entities)
                    .filter_map(IfccadEntity::as_native_mut)
                    .find_map(|e| {
                        if let IfccadEntityKind::Viewport(v) = &mut e.kind {
                            Some(v)
                        } else {
                            None
                        }
                    })
                    .unwrap();
                viewport.paper_clip.enabled = true;
            }
            1 => {
                let p = d
                    .paper_layouts
                    .iter_mut()
                    .find(|p| p.entities.iter().any(|e| e.id() == boundary))
                    .unwrap();
                let index = p.entities.iter().position(|e| e.id() == boundary).unwrap();
                let e = p.entities.remove(index);
                d.model.entities.push(e);
                d.model.bounds = None;
            }
            _ => {
                let p = d
                    .paper_layouts
                    .iter_mut()
                    .find(|p| {
                        p.entities.iter().any(|e| {
                            e.as_native()
                                .is_some_and(|e| matches!(e.kind, IfccadEntityKind::Viewport(_)))
                        })
                    })
                    .unwrap();
                let mut viewport = p
                    .entities
                    .iter()
                    .filter_map(IfccadEntity::as_native)
                    .find(|e| matches!(e.kind, IfccadEntityKind::Viewport(_)))
                    .unwrap()
                    .clone();
                viewport.id = d.id_counters.allocate_entity_id().unwrap();
                p.entities.push(IfccadEntity::Native(viewport));
            }
        }
        assert!(validate_ifccad_document(&d).is_err(), "mutation {mutation}");
    }
}
