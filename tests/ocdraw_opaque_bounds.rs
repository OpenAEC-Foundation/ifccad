use ocdraw::ocdraw::*;

fn preservation(entity_id: u64) -> OcdrawPreservation {
    OcdrawPreservation {
        version: 1,
        next_record_id: 2,
        sources: vec![OcdrawPreservationSource {
            id: "source".into(),
            provider: "generic".into(),
            provider_revision: "rev".into(),
            origin: OcdrawPreservationOrigin::CadDocument,
            source_version: None,
        }],
        records: vec![OcdrawPreservationRecord {
            id: OcdrawPreservationRecordId(1),
            source_id: "source".into(),
            source_key: "1".into(),
            category: OcdrawPreservationCategory::Entity,
            role: OcdrawPreservationRole::Complete,
            representation: OcdrawPreservationRepresentation::CodecOpaque,
            subject: Some(OcdrawPreservationTarget::Entity(entity_id)),
            dependency_coverage: OcdrawPreservationDependencyCoverage::Unknown,
            bindings: vec![],
            conditions: vec![],
            payload: OcdrawPreservationPayload {
                schema: "generic".into(),
                version: 1,
                kind: OcdrawPreservationPayloadKind::AdapterSnapshot,
                bytes: vec![0xff],
            },
        }],
    }
}

#[test]
fn opaque_builder_finalizes_live_links_and_unknown_bounds_without_defaults() {
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("opaque", "mm")).unwrap();
    let id = builder
        .add_opaque_entity(OpaqueEntityDefinition {
            scope_id: 0,
            preservation_record_id: OcdrawPreservationRecordId(1),
            layer_id: None,
            appearance: None,
            visible: false,
        })
        .unwrap();
    assert_eq!(id, 1);
    builder.set_preservation(preservation(id));
    let doc = builder.build_document().unwrap();
    assert_eq!(doc.scopes[0].entities, vec![1]);
    assert!(doc.scopes[0].bounds.is_none());
    assert_eq!(doc.next_entity_id, 2);
    assert_eq!(doc.opaque_entities[0].layer_id, None);
    assert_eq!(doc.opaque_entities[0].appearance, None);
    assert!(!doc.opaque_entities[0].visible);
}

fn direct() -> OcdrawDocument {
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("opaque", "mm")).unwrap();
    let id = builder
        .add_opaque_entity(OpaqueEntityDefinition {
            scope_id: 0,
            preservation_record_id: OcdrawPreservationRecordId(1),
            layer_id: None,
            appearance: None,
            visible: true,
        })
        .unwrap();
    builder.set_preservation(preservation(id));
    builder.build_document().unwrap()
}

#[test]
fn unknown_bounds_require_null_even_when_supplied_box_is_large() {
    let mut doc = direct();
    doc.scopes[0].bounds = Some(Bounds3d::new(
        Point3::new(-1e9, -1e9, -1e9),
        Point3::new(1e9, 1e9, 1e9),
    ));
    assert!(validate_ocdraw_document(&doc).is_err());
    recompute_ocdraw_document_bounds(&mut doc).unwrap();
    assert!(doc.scopes[0].bounds.is_none());
    validate_ocdraw_document(&doc).unwrap();
}

#[test]
fn removing_last_opaque_keeps_archive_and_returns_empty_bounds_rules() {
    let mut doc = direct();
    doc.opaque_entities.clear();
    doc.scopes[0].entities.clear();
    recompute_ocdraw_document_bounds(&mut doc).unwrap();
    validate_ocdraw_document(&doc).unwrap();
    assert!(doc.scopes[0].bounds.is_none());
    assert_eq!(doc.preservation.as_ref().unwrap().next_record_id, 2);
    assert_eq!(doc.preservation.as_ref().unwrap().records.len(), 1);
}

fn nested() -> OcdrawDocument {
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("nested", "mm")).unwrap();
    let pattern = builder
        .add_line_pattern(LinePatternDefinition {
            name: "Continuous".into(),
            description: None,
            pattern: vec![],
        })
        .unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new(
            "0",
            DrawingColor::rgb(0, 0, 0),
            pattern,
        ))
        .unwrap();
    let paper = builder.add_paper_layout("Sheet").unwrap();
    let leaf = builder
        .add_block_definition(BlockDefinition::new("Leaf"))
        .unwrap();
    let parent = builder
        .add_block_definition(BlockDefinition::new("Parent"))
        .unwrap();
    let unused = builder
        .add_block_definition(BlockDefinition::new("UnusedNative"))
        .unwrap();
    let id = builder
        .add_opaque_entity(OpaqueEntityDefinition {
            scope_id: leaf,
            preservation_record_id: OcdrawPreservationRecordId(1),
            layer_id: None,
            appearance: None,
            visible: true,
        })
        .unwrap();
    builder.set_preservation(preservation(id));
    builder
        .add_line(LineDefinition::new(layer, [0., 0., 0.], [10., 1., 0.]).in_scope(leaf))
        .unwrap();
    builder
        .add_line(LineDefinition::new(layer, [0., 0., 0.], [1., 1., 0.]).in_scope(unused))
        .unwrap();
    builder
        .add_block_instance(
            BlockInstanceDefinition::new(layer, leaf, [5., 0., 0.]).in_scope(parent),
        )
        .unwrap();
    builder
        .add_block_instance(BlockInstanceDefinition::new(layer, parent, [0., 0., 0.]))
        .unwrap();
    builder
        .add_block_instance(BlockInstanceDefinition::new(layer, parent, [20., 0., 0.]))
        .unwrap();
    builder
        .add_block_instance(BlockInstanceDefinition::new(layer, leaf, [0., 0., 0.]).in_scope(paper))
        .unwrap();
    builder.build_document().unwrap()
}

#[test]
fn opaque_bounds_propagate_through_nested_and_unused_definitions() {
    let mut doc = nested();
    for scope in &doc.scopes {
        assert_eq!(scope.bounds.is_some(), scope.id == 4);
    }
    validate_ocdraw_document(&doc).unwrap();
    // Removing occurrences does not make the unused opaque definitions empty.
    let ids = doc
        .geometric_entities
        .iter()
        .filter(|e| matches!(e.geometry, DrawingGeometry::BlockInstance { .. }))
        .map(|e| e.id)
        .collect::<Vec<_>>();
    doc.geometric_entities.retain(|e| !ids.contains(&e.id));
    for scope in &mut doc.scopes {
        scope.entities.retain(|id| !ids.contains(id));
    }
    recompute_ocdraw_document_bounds(&mut doc).unwrap();
    assert!(doc
        .scopes
        .iter()
        .find(|s| s.id == 2)
        .unwrap()
        .bounds
        .is_none());
    validate_ocdraw_document(&doc).unwrap();
}

#[test]
fn unavailable_bounds_do_not_mask_native_occurrence_overflow_or_partial_updates() {
    let mut doc = nested();
    for entity in &mut doc.geometric_entities {
        if let DrawingGeometry::BlockInstance { transform, .. } = &mut entity.geometry {
            *transform = BlockTransform::try_new(
                transform.placement(),
                0.,
                Scale3::new(f64::MAX, f64::MAX, f64::MAX),
            )
            .unwrap();
            break;
        }
    }
    let before = doc.scopes.clone();
    assert!(
        validate_ocdraw_document(&doc).is_err(),
        "opaque must not hide overflowing native occurrences"
    );
    assert!(recompute_ocdraw_document_bounds(&mut doc).is_err());
    assert_eq!(doc.scopes, before);
}

#[test]
fn opaque_clip_targets_are_dormant_same_scope_and_unique_only() {
    let mut doc = load_ocdraw_bytes(include_bytes!(
        "../conformance/next/ocdraw/valid/paper-viewport.ocdraw.json"
    ))
    .unwrap()
    .into_document();
    doc.preservation = Some(preservation(2));
    doc.opaque_entities.push(DrawingOpaqueEntity {
        id: 2,
        preservation_record_id: OcdrawPreservationRecordId(1),
        layer_id: None,
        appearance: None,
        visible: false,
    });
    doc.next_entity_id = 3;
    doc.scopes[1].entities.push(2);
    doc.viewports[0].paper_clip = DrawingPaperClip {
        enabled: false,
        boundary_entity_id: Some(2),
    };
    recompute_ocdraw_document_bounds(&mut doc).unwrap();
    validate_ocdraw_document(&doc).unwrap();
    doc.viewports[0].paper_clip.enabled = true;
    assert!(validate_ocdraw_document(&doc).is_err());
    doc.viewports[0].paper_clip.enabled = false;
    doc.scopes[1].entities.retain(|id| *id != 2);
    doc.scopes[0].entities.push(2);
    assert!(validate_ocdraw_document(&doc).is_err());
}

#[test]
fn unknown_child_does_not_bypass_native_transform_preparation_failure() {
    let mut doc = nested();
    // Remove the known leaf, leaving only the opaque child; keep unrelated native bounds.
    doc.geometric_entities.retain(|e| e.id != 2);
    for scope in &mut doc.scopes {
        scope.entities.retain(|id| *id != 2);
    }
    for e in &mut doc.geometric_entities {
        if let DrawingGeometry::BlockInstance {
            definition_scope_id: 2,
            transform,
        } = &mut e.geometry
        {
            // The frame tolerance admits this axis without normalization. Its native
            // matrix coefficient overflows at this scale, independently of any curve.
            let frame = CoordinateFrame3::try_new(
                transform.placement().origin(),
                Vector3::new(1.0_f64.next_up(), 0., 0.),
                Vector3::new(0., 1., 0.),
            )
            .unwrap();
            *transform =
                BlockTransform::try_new(frame, 0., Scale3::new(f64::MAX, f64::MAX, f64::MAX))
                    .unwrap();
        }
    }
    let before = doc.scopes.clone();
    assert!(recompute_ocdraw_document_bounds(&mut doc).is_err());
    assert_eq!(doc.scopes, before);
}
