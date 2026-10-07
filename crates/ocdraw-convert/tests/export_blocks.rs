use ocdraw_convert::{
    cad_document_to_encoded_ocdraw, CadSourceStructureProblem, CadToOcdrawError,
    CadToOcdrawOptions, OcdrawLossPolicy,
};
use opencadcodec::entities::Block;
use opencadcodec::{BlockRecord, CadDocument, EntityType, Vector3};

#[test]
fn full_fold_collision_never_merges_or_retargets_definitions() {
    let mut document = CadDocument::new();
    // Distinct under opencadcodec's uppercase lookup, equal under full default fold.
    for name in ["İ", "i\u{307}"] {
        let mut record = BlockRecord::new(name);
        record.handle = document.allocate_handle();
        document.block_records.add(record).unwrap();
        document
            .add_entity(EntityType::Insert(opencadcodec::entities::Insert::new(
                name,
                Vector3::ZERO,
            )))
            .unwrap();
    }
    for loss_policy in [OcdrawLossPolicy::Allow, OcdrawLossPolicy::Reject] {
        assert!(cad_document_to_encoded_ocdraw(
            &document,
            CadToOcdrawOptions {
                loss_policy,
                ..Default::default()
            }
        )
        .is_err());
    }
}

#[test]
fn dynamic_visibility_data_remains_explicit_loss_not_supported_behavior() {
    let mut document = definition(None);
    let handle = document.allocate_handle();
    document.block_visibility_params.insert(
        handle,
        opencadcodec::objects::BlockVisibilityParameter {
            handle,
            ..Default::default()
        },
    );
    document.objects.insert(
        handle,
        opencadcodec::objects::ObjectType::Unknown {
            type_name: "BLOCKVISIBILITYPARAMETER".into(),
            handle,
            owner: opencadcodec::Handle::NULL,
            raw_dxf_codes: None,
            raw_dwg_data: None,
            raw_dwg_handle_bits: 0,
            raw_dwg_version: None,
        },
    );
    document
        .add_entity(EntityType::Insert(opencadcodec::entities::Insert::new(
            "Door",
            Vector3::ZERO,
        )))
        .unwrap();
    let result = cad_document_to_encoded_ocdraw(&document, CadToOcdrawOptions::default()).unwrap();
    assert_eq!(result.diagnostics().iter().flat_map(|d| d.reasons()).filter(|r| matches!(r,ocdraw_convert::CadToOcdrawLossReason::UnsupportedCollection {kind,..} if kind=="objects")).count(), 1);
    assert!(matches!(
        cad_document_to_encoded_ocdraw(
            &document,
            CadToOcdrawOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(CadToOcdrawError::LossRejected { .. })
    ));
}

#[test]
fn nested_shared_definition_loss_reaches_every_affected_instance() {
    let mut document = definition(None);
    let owner = document.block_records.get("Door").unwrap().handle;
    let mut circle = opencadcodec::Circle::new();
    circle.common.owner_handle = owner;
    circle.thickness = 1.0;
    document.add_entity(EntityType::Circle(circle)).unwrap();
    let mut outer = BlockRecord::new("Outer");
    outer.handle = document.allocate_handle();
    let outer_owner = outer.handle;
    document.block_records.add(outer).unwrap();
    let mut nested = opencadcodec::entities::Insert::new("Door", Vector3::ZERO);
    nested.common.owner_handle = outer_owner;
    let nested = document.add_entity(EntityType::Insert(nested)).unwrap();
    let mut expected = vec![nested];
    for name in ["Outer", "Outer", "Door"] {
        expected.push(
            document
                .add_entity(EntityType::Insert(opencadcodec::entities::Insert::new(
                    name,
                    Vector3::ZERO,
                )))
                .unwrap(),
        );
    }
    expected.sort();
    let result = cad_document_to_encoded_ocdraw(&document, CadToOcdrawOptions::default()).unwrap();
    assert!(result.diagnostics().iter().flat_map(|d| d.reasons()).any(|r| matches!(r,ocdraw_convert::CadToOcdrawLossReason::BlockContentLoss {definition,affected_instances} if *definition==owner && *affected_instances==expected)));
}

#[test]
fn array_and_view_specific_inserts_are_not_exported_as_ordinary_instances() {
    for variant in 0..6 {
        let mut document = definition(None);
        let mut insert = opencadcodec::entities::Insert::new("Door", Vector3::ZERO);
        match variant {
            0 => insert.column_count = 2,
            1 => insert.column_spacing = 12., // even a 1x1 array has metadata
            2 => insert.row_count = 0,
            3 => insert.view_rep_handle = Some(document.allocate_handle()),
            4 => insert
                .attributes
                .push(opencadcodec::entities::AttributeEntity::new(
                    "TAG".into(),
                    "VALUE".into(),
                )),
            _ => {
                document
                    .block_records
                    .get_mut("Door")
                    .unwrap()
                    .flags
                    .is_xref = true
            }
        }
        document.add_entity(EntityType::Insert(insert)).unwrap();
        let result =
            cad_document_to_encoded_ocdraw(&document, CadToOcdrawOptions::default()).unwrap();
        let body: serde_json::Value = serde_json::from_slice(result.encoded().bytes()).unwrap();
        assert!(
            body["streams"]
                .get("blockInstanceStream")
                .is_none_or(|s| s["count"] == 0),
            "variant {variant}"
        );
        assert!(!result.diagnostics().is_empty());
        assert!(matches!(
            cad_document_to_encoded_ocdraw(
                &document,
                CadToOcdrawOptions {
                    loss_policy: OcdrawLossPolicy::Reject,
                    ..Default::default()
                }
            ),
            Err(CadToOcdrawError::LossRejected { .. })
        ));
    }
}

#[test]
fn duplicate_record_handles_never_disguise_a_definition_as_model_space() {
    let mut document = definition(None);
    document.block_records.get_mut("Door").unwrap().handle =
        document.header.model_space_block_handle;
    for loss_policy in [OcdrawLossPolicy::Allow, OcdrawLossPolicy::Reject] {
        assert!(matches!(
            cad_document_to_encoded_ocdraw(
                &document,
                CadToOcdrawOptions {
                    loss_policy,
                    ..Default::default()
                }
            ),
            Err(CadToOcdrawError::InvalidSourceStructure { .. })
        ));
    }
}

#[test]
fn anonymous_marker_name_collision_remains_structural_error() {
    let mut document = definition(Some(Vector3::new(2., 0., 0.)));
    let marker_handle = document
        .block_records
        .get("Door")
        .unwrap()
        .block_entity_handle;
    document.block_records.rename("Door", "*U24").unwrap();
    document
        .block_records
        .get_mut("*U24")
        .unwrap()
        .flags
        .anonymous = true;
    let Some(EntityType::Block(marker)) = document.get_entity_mut(marker_handle) else {
        panic!("expected BLOCK begin marker");
    };
    marker.name = "*U25".into();
    let mut other = BlockRecord::new("*U25");
    other.handle = document.allocate_handle();
    document.block_records.add(other).unwrap();
    assert!(matches!(
        cad_document_to_encoded_ocdraw(&document, CadToOcdrawOptions::default()),
        Err(CadToOcdrawError::InvalidSourceStructure { .. })
    ));
}

#[test]
fn anonymous_marker_name_conflict_without_collision_is_also_fatal() {
    let mut document = definition(Some(Vector3::new(2., 0., 0.)));
    let marker_handle = document
        .block_records
        .get("Door")
        .unwrap()
        .block_entity_handle;
    document.block_records.rename("Door", "*U24").unwrap();
    document
        .block_records
        .get_mut("*U24")
        .unwrap()
        .flags
        .anonymous = true;
    let Some(EntityType::Block(marker)) = document.get_entity_mut(marker_handle) else {
        panic!()
    };
    marker.name = "*U25".into();
    for loss_policy in [OcdrawLossPolicy::Allow, OcdrawLossPolicy::Reject] {
        assert!(matches!(
            cad_document_to_encoded_ocdraw(
                &document,
                CadToOcdrawOptions {
                    loss_policy,
                    ..Default::default()
                }
            ),
            Err(CadToOcdrawError::InvalidSourceStructure { .. })
        ));
    }
}

fn definition(marker_base: Option<Vector3>) -> CadDocument {
    let mut document = CadDocument::new();
    let mut record = BlockRecord::new("Door");
    record.handle = document.allocate_handle();
    record.block_entity_handle = document.allocate_handle();
    record.block_end_handle = document.allocate_handle();
    record.base_point = Vector3::new(2., 0., 0.);
    if let Some(base) = marker_base {
        let mut marker = Block::new("Door", base);
        marker.common.handle = record.block_entity_handle;
        marker.common.owner_handle = record.handle;
        document.add_entity(EntityType::Block(marker)).unwrap();
    }
    document.block_records.add(record).unwrap();
    document
}

#[test]
fn empty_instance_still_reports_changed_source_normal() {
    let mut document = definition(None);
    let mut insert = opencadcodec::entities::Insert::new("Door", Vector3::ZERO);
    insert.normal = Vector3::new(0., 0., 2.);
    document.add_entity(EntityType::Insert(insert)).unwrap();
    let result = cad_document_to_encoded_ocdraw(&document, CadToOcdrawOptions::default()).unwrap();
    assert!(result
        .diagnostics()
        .iter()
        .any(|d| d.reasons().iter().any(|r| matches!(
            r,
            ocdraw_convert::CadToOcdrawLossReason::SourceNormalNormalized
        ))));
    assert!(matches!(
        cad_document_to_encoded_ocdraw(
            &document,
            CadToOcdrawOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(CadToOcdrawError::LossRejected { .. })
    ));
}

#[test]
fn outer_instance_scaling_can_turn_acceptable_rounding_into_failure() {
    let mut document = definition(None);
    document.header.insertion_units = 6;
    let owner = document.block_records.get("Door").unwrap().handle;
    let mut line = opencadcodec::Line::from_coords(2., 0., 0., 3., 0., 0.);
    line.common.owner_handle = owner;
    document.add_entity(EntityType::Line(line)).unwrap();
    let mut outer = BlockRecord::new("Outer");
    outer.handle = document.allocate_handle();
    let outer_owner = outer.handle;
    document.block_records.add(outer).unwrap();
    let mut inner = opencadcodec::entities::Insert::new("Door", Vector3::new(3000., 4000., 5000.));
    inner.normal = Vector3::new(1., 2., 3.);
    inner.common.owner_handle = outer_owner;
    document.add_entity(EntityType::Insert(inner)).unwrap();
    cad_document_to_encoded_ocdraw(&document, CadToOcdrawOptions::default())
        .expect("local rounding fits the default metre tolerance");
    let mut insert = opencadcodec::entities::Insert::new("Outer", Vector3::ZERO);
    insert.set_x_scale(1e9);
    insert.set_y_scale(1e9);
    insert.set_z_scale(1e9);
    document.add_entity(EntityType::Insert(insert)).unwrap();
    for loss_policy in [OcdrawLossPolicy::Allow, OcdrawLossPolicy::Reject] {
        let error = cad_document_to_encoded_ocdraw(
            &document,
            CadToOcdrawOptions {
                loss_policy,
                ..Default::default()
            },
        )
        .err()
        .expect("outer scale exceeds tolerance");
        let failure = match error {
            CadToOcdrawError::Geometry(failure) => failure,
            other => panic!("unexpected: {other:?}"),
        };
        assert!(
            matches!(&failure.source, ocdraw_convert::OcdrawGeometryEntitySource::BlockOccurrence { path, .. } if path.len() == 2)
        );
    }
}

#[test]
fn partial_definition_loss_identifies_affected_instances() {
    let mut document = definition(None);
    let owner = document.block_records.get("Door").unwrap().handle;
    let mut circle = opencadcodec::Circle::new();
    circle.common.owner_handle = owner;
    circle.thickness = 1.0;
    document.add_entity(EntityType::Circle(circle)).unwrap();
    let instance = document
        .add_entity(EntityType::Insert(opencadcodec::entities::Insert::new(
            "Door",
            Vector3::ZERO,
        )))
        .unwrap();
    let result = cad_document_to_encoded_ocdraw(&document, CadToOcdrawOptions::default()).unwrap();
    assert!(result.diagnostics().iter().flat_map(|d| d.reasons()).any(|reason| matches!(reason, ocdraw_convert::CadToOcdrawLossReason::BlockContentLoss { affected_instances, .. } if affected_instances.contains(&instance))));
    assert!(matches!(
        cad_document_to_encoded_ocdraw(
            &document,
            CadToOcdrawOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(CadToOcdrawError::LossRejected { .. })
    ));
}

#[test]
fn missing_definition_and_unused_cycle_are_structural_errors() {
    for cycle in [false, true] {
        let mut document = definition(None);
        let mut insert = opencadcodec::entities::Insert::new(
            if cycle { "Door" } else { "Missing" },
            Vector3::ZERO,
        );
        if cycle {
            insert.common.owner_handle = document.block_records.get("Door").unwrap().handle;
        }
        document.add_entity(EntityType::Insert(insert)).unwrap();
        for loss_policy in [OcdrawLossPolicy::Allow, OcdrawLossPolicy::Reject] {
            assert!(matches!(
                cad_document_to_encoded_ocdraw(
                    &document,
                    CadToOcdrawOptions {
                        loss_policy,
                        ..Default::default()
                    }
                ),
                Err(CadToOcdrawError::InvalidSourceStructure { .. })
            ));
        }
    }
}

#[test]
fn conflicting_marker_is_structural_failure_under_both_loss_policies() {
    for loss_policy in [OcdrawLossPolicy::Allow, OcdrawLossPolicy::Reject] {
        let error = cad_document_to_encoded_ocdraw(
            &definition(Some(Vector3::ZERO)),
            CadToOcdrawOptions {
                loss_policy,
                ..Default::default()
            },
        )
        .err()
        .expect("conflicting block data must fail");
        let CadToOcdrawError::InvalidSourceStructure { problems } = error else {
            panic!("expected structural failure, got {error:?}")
        };
        assert!(problems.iter().any(|problem| matches!(problem,
            CadSourceStructureProblem::InconsistentRelationship { description }
            if description.contains("Door") && description.contains("base point")
        )));
    }
}

#[test]
fn absent_or_consistent_marker_is_not_a_structural_error() {
    for marker in [None, Some(Vector3::new(2., 0., 0.))] {
        cad_document_to_encoded_ocdraw(&definition(marker), CadToOcdrawOptions::default()).unwrap();
    }
}
