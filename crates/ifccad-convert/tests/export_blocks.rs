use cadcodec::entities::Block;
use cadcodec::{BlockRecord, CadDocument, EntityType, Vector3};
use ifccad::package::PackageOptions;
use ifccad::PackageId;
use ifccad_convert::{
    cad_document_to_package, ExportError, ExportLossPolicy, ExportOptions, SourceStructureProblem,
};

fn options() -> PackageOptions {
    PackageOptions {
        package_id: PackageId::new("blocks").unwrap(),
        data_version: "1".into(),
        author: "Block conversion tests".into(),
        timestamp: "2026-09-22T10:00:00Z".into(),
    }
}

#[test]
fn full_fold_collision_never_merges_or_retargets_definitions() {
    let mut document = CadDocument::new();
    // Distinct under cadcodec's uppercase lookup, equal under full default fold.
    for name in ["İ", "i\u{307}"] {
        let mut record = BlockRecord::new(name);
        record.handle = document.allocate_handle();
        document.block_records.add(record).unwrap();
        document
            .add_entity(EntityType::Insert(cadcodec::entities::Insert::new(
                name,
                Vector3::ZERO,
            )))
            .unwrap();
    }
    for loss_policy in [ExportLossPolicy::Allow, ExportLossPolicy::Reject] {
        assert!(cad_document_to_package(
            &document,
            options(),
            ExportOptions {
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
        cadcodec::objects::BlockVisibilityParameter {
            handle,
            ..Default::default()
        },
    );
    document.objects.insert(
        handle,
        cadcodec::objects::ObjectType::Unknown {
            type_name: "BLOCKVISIBILITYPARAMETER".into(),
            handle,
            owner: cadcodec::Handle::NULL,
            raw_dxf_codes: None,
            raw_dwg_data: None,
            raw_dwg_handle_bits: 0,
            raw_dwg_version: None,
        },
    );
    document
        .add_entity(EntityType::Insert(cadcodec::entities::Insert::new(
            "Door",
            Vector3::ZERO,
        )))
        .unwrap();
    let result = cad_document_to_package(&document, options(), ExportOptions::default()).unwrap();
    assert_eq!(result.diagnostics().iter().flat_map(|d| d.reasons()).filter(|r| matches!(r,ifccad_convert::ExportLossReason::UnsupportedCollection {kind,..} if kind=="objects")).count(), 1);
    assert!(matches!(
        cad_document_to_package(
            &document,
            options(),
            ExportOptions {
                loss_policy: ExportLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(ExportError::LossRejected { .. })
    ));
}

#[test]
fn nested_shared_definition_loss_reaches_every_affected_instance() {
    let mut document = definition(None);
    let owner = document.block_records.get("Door").unwrap().handle;
    let mut circle = cadcodec::Circle::new();
    circle.common.owner_handle = owner;
    circle.thickness = 1.0;
    document.add_entity(EntityType::Circle(circle)).unwrap();
    let mut outer = BlockRecord::new("Outer");
    outer.handle = document.allocate_handle();
    let outer_owner = outer.handle;
    document.block_records.add(outer).unwrap();
    let mut nested = cadcodec::entities::Insert::new("Door", Vector3::ZERO);
    nested.common.owner_handle = outer_owner;
    let nested = document.add_entity(EntityType::Insert(nested)).unwrap();
    let mut expected = vec![nested];
    for name in ["Outer", "Outer", "Door"] {
        expected.push(
            document
                .add_entity(EntityType::Insert(cadcodec::entities::Insert::new(
                    name,
                    Vector3::ZERO,
                )))
                .unwrap(),
        );
    }
    expected.sort();
    let result = cad_document_to_package(&document, options(), ExportOptions::default()).unwrap();
    assert!(result.diagnostics().iter().flat_map(|d| d.reasons()).any(|r| matches!(r,ifccad_convert::ExportLossReason::BlockContentLoss {definition,affected_instances} if *definition==owner && *affected_instances==expected)));
}

#[test]
fn mixed_definition_order_and_negative_uniformity_survive_strict_readback() {
    use ifccad::ifcdr::{IfcdrEntityRef, ScopeRef};
    let mut document = definition(None);
    let owner = document.block_records.get("Door").unwrap().handle;
    let mut leaf = BlockRecord::new("Leaf");
    leaf.handle = document.allocate_handle();
    leaf.scale_uniformly = true;
    document.block_records.add(leaf).unwrap();
    let mut line = cadcodec::Line::from_coords(2., 0., 0., 3., 0., 0.);
    line.common.owner_handle = owner;
    document.add_entity(EntityType::Line(line)).unwrap();
    let mut insert = cadcodec::entities::Insert::new("Leaf", Vector3::ZERO);
    insert.common.owner_handle = owner;
    insert.set_x_scale(-2.);
    insert.set_y_scale(-2.);
    insert.set_z_scale(-2.);
    document.add_entity(EntityType::Insert(insert)).unwrap();
    let mut poly = cadcodec::LwPolyline::from_points(vec![
        cadcodec::Vector2::new(0., 0.),
        cadcodec::Vector2::new(1., 0.),
    ]);
    poly.common.owner_handle = owner;
    document.add_entity(EntityType::LwPolyline(poly)).unwrap();
    document
        .block_records
        .get_mut("Door")
        .unwrap()
        .entity_handles
        .reverse();
    let result = cad_document_to_package(
        &document,
        options(),
        ExportOptions {
            loss_policy: ExportLossPolicy::Reject,
            ..Default::default()
        },
    )
    .unwrap();
    let tick = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("ifccad-block-order-{}-{tick}", std::process::id()));
    result.package().write_directory(&path).unwrap();
    let loaded = ifccad::package::load_directory_package(&path).unwrap();
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    let resource = drawing.representation().resource();
    let scope = resource
        .scopes()
        .find_map(|scope| match scope {
            ScopeRef::BlockDefinition(d) if d.name() == "Door" => Some(d.scope_id()),
            _ => None,
        })
        .unwrap();
    let kinds: Vec<_> = resource
        .entities(scope)
        .map(|entity| match entity {
            IfcdrEntityRef::Point(_) => "point",
            IfcdrEntityRef::Circle(_) => "circle",
            IfcdrEntityRef::Arc(_) => "arc",
            IfcdrEntityRef::Ellipse(_) => "ellipse",
            IfcdrEntityRef::EllipseArc(_) => "ellipseArc",
            IfcdrEntityRef::PlanarPolyline(_) => "polyline",
            IfcdrEntityRef::SpatialPolyline(_) => "spatialPolyline",
            IfcdrEntityRef::BlockInstance(i) => {
                assert_eq!(i.transform().scale().x(), -2.);
                "insert"
            }
            IfcdrEntityRef::Line(_) => "line",
            IfcdrEntityRef::Viewport(_) => "viewport",
        })
        .collect();
    assert_eq!(kinds, ["polyline", "insert", "line"]);
    let imported = ifccad_convert::drawing_to_cad_document(drawing).unwrap();
    let kinds: Vec<_> = imported
        .document()
        .entities_in_block("Door")
        .filter_map(|e| match e {
            EntityType::LwPolyline(_) => Some("polyline"),
            EntityType::Insert(_) => Some("insert"),
            EntityType::Line(_) => Some("line"),
            _ => None,
        })
        .collect();
    assert_eq!(kinds, ["polyline", "insert", "line"]);
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn array_and_view_specific_inserts_are_not_exported_as_ordinary_instances() {
    for variant in 0..6 {
        let mut document = definition(None);
        let mut insert = cadcodec::entities::Insert::new("Door", Vector3::ZERO);
        match variant {
            0 => insert.column_count = 2,
            1 => insert.column_spacing = 12., // even a 1x1 array has metadata
            2 => insert.row_count = 0,
            3 => insert.view_rep_handle = Some(document.allocate_handle()),
            4 => insert
                .attributes
                .push(cadcodec::entities::AttributeEntity::new(
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
            cad_document_to_package(&document, options(), ExportOptions::default()).unwrap();
        let body: serde_json::Value = serde_json::from_slice(
            result
                .package()
                .file("resources/drawing.ifcdr.json")
                .unwrap(),
        )
        .unwrap();
        assert!(
            body["streams"]
                .get("blockInstanceStream")
                .is_none_or(|s| s["count"] == 0),
            "variant {variant}"
        );
        assert!(!result.diagnostics().is_empty());
        assert!(matches!(
            cad_document_to_package(
                &document,
                options(),
                ExportOptions {
                    loss_policy: ExportLossPolicy::Reject,
                    ..Default::default()
                }
            ),
            Err(ExportError::LossRejected { .. })
        ));
    }
}

#[test]
fn duplicate_record_handles_never_disguise_a_definition_as_model_space() {
    let mut document = definition(None);
    document.block_records.get_mut("Door").unwrap().handle =
        document.header.model_space_block_handle;
    for loss_policy in [ExportLossPolicy::Allow, ExportLossPolicy::Reject] {
        assert!(matches!(
            cad_document_to_package(
                &document,
                options(),
                ExportOptions {
                    loss_policy,
                    ..Default::default()
                }
            ),
            Err(ExportError::InvalidSourceStructure { .. })
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
    let mut insert = cadcodec::entities::Insert::new("Door", Vector3::ZERO);
    insert.normal = Vector3::new(0., 0., 2.);
    document.add_entity(EntityType::Insert(insert)).unwrap();
    let result = cad_document_to_package(&document, options(), ExportOptions::default()).unwrap();
    assert!(result.diagnostics().iter().any(|d| d
        .reasons()
        .iter()
        .any(|r| matches!(r, ifccad_convert::ExportLossReason::SourceNormalNormalized))));
    assert!(matches!(
        cad_document_to_package(
            &document,
            options(),
            ExportOptions {
                loss_policy: ExportLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(ExportError::LossRejected { .. })
    ));
}

#[test]
fn definition_entity_order_follows_the_owned_handle_list() {
    let mut document = definition(None);
    let owner = document.block_records.get("Door").unwrap().handle;
    for x in [2., 3.] {
        let mut line = cadcodec::Line::from_coords(x, 0., 0., x + 1., 0., 0.);
        line.common.owner_handle = owner;
        document.add_entity(EntityType::Line(line)).unwrap();
    }
    document
        .block_records
        .get_mut("Door")
        .unwrap()
        .entity_handles
        .reverse();
    let result = cad_document_to_package(&document, options(), ExportOptions::default()).unwrap();
    let body: serde_json::Value = serde_json::from_slice(
        result
            .package()
            .file("resources/drawing.ifcdr.json")
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        body["streams"]["lineStream"]["x1"],
        serde_json::json!([3., 2.])
    );
    document
        .block_records
        .get_mut("Door")
        .unwrap()
        .entity_handles
        .clear();
    assert!(matches!(
        cad_document_to_package(&document, options(), ExportOptions::default()),
        Err(ExportError::InvalidSourceStructure { .. })
    ));
}

#[test]
fn outer_instance_scaling_can_turn_acceptable_rounding_into_failure() {
    let mut document = definition(None);
    document.header.insertion_units = 6;
    let owner = document.block_records.get("Door").unwrap().handle;
    let mut line = cadcodec::Line::from_coords(2., 0., 0., 3., 0., 0.);
    line.common.owner_handle = owner;
    document.add_entity(EntityType::Line(line)).unwrap();
    let mut outer = BlockRecord::new("Outer");
    outer.handle = document.allocate_handle();
    let outer_owner = outer.handle;
    document.block_records.add(outer).unwrap();
    let mut inner = cadcodec::entities::Insert::new("Door", Vector3::new(3000., 4000., 5000.));
    inner.normal = Vector3::new(1., 2., 3.);
    inner.common.owner_handle = outer_owner;
    document.add_entity(EntityType::Insert(inner)).unwrap();
    cad_document_to_package(&document, options(), ExportOptions::default())
        .expect("local rounding fits the default metre tolerance");
    let mut insert = cadcodec::entities::Insert::new("Outer", Vector3::ZERO);
    insert.set_x_scale(1e9);
    insert.set_y_scale(1e9);
    insert.set_z_scale(1e9);
    document.add_entity(EntityType::Insert(insert)).unwrap();
    for loss_policy in [ExportLossPolicy::Allow, ExportLossPolicy::Reject] {
        let error = cad_document_to_package(
            &document,
            options(),
            ExportOptions {
                loss_policy,
                ..Default::default()
            },
        )
        .err()
        .expect("outer scale exceeds tolerance");
        let failure = match error {
            ExportError::GeometryToleranceExceeded { failure }
            | ExportError::GeometryAccuracyNotEstablished { failure } => failure,
            other => panic!("unexpected: {other:?}"),
        };
        assert!(
            matches!(failure.source, ifccad_convert::ConversionEntitySource::BlockOccurrence { path, .. } if path.len() == 2)
        );
    }
}

#[test]
fn partial_definition_loss_identifies_affected_instances() {
    let mut document = definition(None);
    let owner = document.block_records.get("Door").unwrap().handle;
    let mut circle = cadcodec::Circle::new();
    circle.common.owner_handle = owner;
    circle.thickness = 1.0;
    document.add_entity(EntityType::Circle(circle)).unwrap();
    let instance = document
        .add_entity(EntityType::Insert(cadcodec::entities::Insert::new(
            "Door",
            Vector3::ZERO,
        )))
        .unwrap();
    let result = cad_document_to_package(&document, options(), ExportOptions::default()).unwrap();
    assert!(result.diagnostics().iter().flat_map(|d| d.reasons()).any(|reason| matches!(reason, ifccad_convert::ExportLossReason::BlockContentLoss { affected_instances, .. } if affected_instances.contains(&instance))));
    assert!(matches!(
        cad_document_to_package(
            &document,
            options(),
            ExportOptions {
                loss_policy: ExportLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(ExportError::LossRejected { .. })
    ));
}

#[test]
fn two_instances_share_one_definition_without_exploding() {
    let mut document = definition(None);
    let owner = document.block_records.get("Door").unwrap().handle;
    let mut line = cadcodec::Line::from_coords(2., 0., 0., 3., 0., 0.);
    line.common.owner_handle = owner;
    document.add_entity(EntityType::Line(line)).unwrap();
    for x in [10., 20.] {
        let mut insert = cadcodec::entities::Insert::new("Door", Vector3::new(x, 0., 0.));
        insert.common.invisible = true;
        document.add_entity(EntityType::Insert(insert)).unwrap();
    }
    let result = cad_document_to_package(&document, options(), ExportOptions::default()).unwrap();
    let body: serde_json::Value = serde_json::from_slice(
        result
            .package()
            .file("resources/drawing.ifcdr.json")
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["streams"]["blockInstanceStream"]["count"], 2);
    assert_eq!(body["streams"]["lineStream"]["count"], 1);
    assert_eq!(body["blockDefinitionTable"].as_array().unwrap().len(), 1);
    assert!(
        result.diagnostics().is_empty(),
        "{:?}",
        result.diagnostics()
    );
}

#[test]
fn missing_definition_and_unused_cycle_are_structural_errors() {
    for cycle in [false, true] {
        let mut document = definition(None);
        let mut insert =
            cadcodec::entities::Insert::new(if cycle { "Door" } else { "Missing" }, Vector3::ZERO);
        if cycle {
            insert.common.owner_handle = document.block_records.get("Door").unwrap().handle;
        }
        document.add_entity(EntityType::Insert(insert)).unwrap();
        for loss_policy in [ExportLossPolicy::Allow, ExportLossPolicy::Reject] {
            assert!(matches!(
                cad_document_to_package(
                    &document,
                    options(),
                    ExportOptions {
                        loss_policy,
                        ..Default::default()
                    }
                ),
                Err(ExportError::InvalidSourceStructure { .. })
            ));
        }
    }
}

#[test]
fn definition_contents_keep_their_local_coordinates_and_owner() {
    let mut document = definition(None);
    let owner = document.block_records.get("Door").unwrap().handle;
    let mut line = cadcodec::Line::from_coords(2., 0., 0., 3., 0., 0.);
    line.common.owner_handle = owner;
    document.add_entity(EntityType::Line(line)).unwrap();
    let result = cad_document_to_package(&document, options(), ExportOptions::default()).unwrap();
    let body: serde_json::Value = serde_json::from_slice(
        result
            .package()
            .file("resources/drawing.ifcdr.json")
            .unwrap(),
    )
    .unwrap();
    let scope = &body["blockDefinitionTable"][0]["scopeId"];
    assert_eq!(body["streams"]["lineStream"]["count"], 1);
    assert_eq!(&body["streams"]["lineStream"]["scopeId"][0], scope);
    assert_eq!(body["streams"]["lineStream"]["x1"][0], 2.);
    assert_eq!(body["streams"]["lineStream"]["x2"][0], 3.);
    assert!(
        result.diagnostics().is_empty(),
        "{:?}",
        result.diagnostics()
    );
}

#[test]
fn unused_definition_metadata_is_preserved_without_unit_rescaling() {
    let mut document = definition(None);
    document.header.insertion_units = 4;
    let record = document.block_records.get_mut("Door").unwrap();
    record.units = 1;
    record.description = "Entrance door".into();
    record.flags.anonymous = true;
    record.explodable = false;
    record.scale_uniformly = true;
    let result = cad_document_to_package(&document, options(), ExportOptions::default()).unwrap();
    let body: serde_json::Value = serde_json::from_slice(
        result
            .package()
            .file("resources/drawing.ifcdr.json")
            .unwrap(),
    )
    .unwrap();
    let definitions = body["blockDefinitionTable"]
        .as_array()
        .expect("definition table");
    assert_eq!(definitions.len(), 1);
    assert_eq!(definitions[0]["name"], "Door");
    assert_eq!(
        definitions[0]["basePoint"],
        serde_json::json!({"x":2.,"y":0.,"z":0.})
    );
    assert_eq!(definitions[0]["description"], "Entrance door");
    assert_eq!(definitions[0]["insertionUnit"], "in");
    assert_eq!(definitions[0]["anonymous"], true);
    assert_eq!(definitions[0]["explodable"], false);
    assert_eq!(definitions[0]["scaling"], 1);
    assert!(
        result.diagnostics().is_empty(),
        "{:?}",
        result.diagnostics()
    );
}

#[test]
fn conflicting_marker_is_structural_failure_under_both_loss_policies() {
    for loss_policy in [ExportLossPolicy::Allow, ExportLossPolicy::Reject] {
        let error = cad_document_to_package(
            &definition(Some(Vector3::ZERO)),
            options(),
            ExportOptions {
                loss_policy,
                ..Default::default()
            },
        )
        .err()
        .expect("conflicting block data must fail");
        let ExportError::InvalidSourceStructure { problems } = error else {
            panic!("expected structural failure, got {error:?}")
        };
        assert!(problems.iter().any(|problem| matches!(problem,
            SourceStructureProblem::InconsistentRelationship { description }
            if description.contains("Door") && description.contains("base point") && description.contains("cadcodec/issues/52")
        )));
    }
}

#[test]
fn absent_or_consistent_marker_is_not_a_structural_error() {
    for marker in [None, Some(Vector3::new(2., 0., 0.))] {
        cad_document_to_package(&definition(marker), options(), ExportOptions::default()).unwrap();
    }
}
