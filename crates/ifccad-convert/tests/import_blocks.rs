use cadcodec::EntityType;
use ifccad::ifcdr::{BlockTransform, IfcdrLengthUnit, Point3, Scale3, Vector3};
use ifccad::package::*;
use ifccad::{PackageId, ResourceId};
use ifccad_convert::{drawing_to_cad_document, ImportError};
use std::sync::atomic::{AtomicU64, Ordering};

struct Temp(std::path::PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let tick = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "ifccad-import-blocks-{}-{tick}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn native(transform: BlockTransform, empty: bool) -> EncodedPackage {
    native_with_outer(transform, empty, false)
}

fn native_with_outer(transform: BlockTransform, empty: bool, nested: bool) -> EncodedPackage {
    let mut builder = PackageBuilder::new(PackageOptions {
        package_id: PackageId::new("blocks").unwrap(),
        data_version: "1".into(),
        author: "Import tests".into(),
        timestamp: "2026-09-22T10:00:00Z".into(),
    })
    .unwrap();
    let mut drawing = builder
        .add_drawing(DrawingOptions {
            model_layout_name: "Model".into(),
            representation_resource_id: ResourceId::new("drawing").unwrap(),
            length_unit: IfcdrLengthUnit::Metre,
        })
        .unwrap();
    let appearance = drawing
        .appearances()
        .add(AppearanceDefinition {
            name: "Default".into(),
            color: AppearanceColor::rgb(0, 0, 0),
            opacity: 1.,
            line_pattern: LinePatternDefinition::named("Continuous"),
            line_weight: 0.25,
        })
        .unwrap();
    let layer = drawing
        .layers()
        .add(LayerDefinition {
            name: "0".into(),
            visible: true,
            appearance,
        })
        .unwrap();
    let mut options = BlockDefinitionOptions::named("Door".into());
    options.base_point = Point3::new(2., 0., 0.);
    options.description = "Entrance door".into();
    options.insertion_unit = IfcdrLengthUnit::Inch;
    let definition = drawing.add_block_definition(options).unwrap();
    drawing
        .add_block_definition(BlockDefinitionOptions::named("Unused".into()))
        .unwrap();
    if !empty {
        drawing
            .block_definition(definition)
            .unwrap()
            .add_line(LineDefinition {
                start: Point3::new(2., 0., 0.),
                end: Point3::new(3., 0., 0.),
                layer,
                appearance: EntityAppearance::by_block(),
                visible: true,
            })
            .unwrap();
    }
    let parent = if nested {
        Some(
            drawing
                .add_block_definition(BlockDefinitionOptions::named("Outer".into()))
                .unwrap(),
        )
    } else {
        None
    };
    let mut scope = if let Some(parent) = parent {
        drawing.block_definition(parent).unwrap()
    } else {
        drawing.model_space()
    };
    scope
        .add_block_instance(BlockInstanceDefinition {
            definition,
            transform,
            layer,
            appearance: EntityAppearance::by_layer(),
            visible: false,
        })
        .unwrap();
    if let Some(parent) = parent {
        drawing
            .model_space()
            .add_block_instance(BlockInstanceDefinition {
                definition: parent,
                transform: BlockTransform::try_new(
                    Default::default(),
                    0.,
                    Scale3::new(1e9, 1e9, 1e9),
                )
                .unwrap(),
                layer,
                appearance: EntityAppearance::by_layer(),
                visible: true,
            })
            .unwrap();
    }
    builder.finish().unwrap()
}

#[test]
fn nearly_orthonormal_frame_error_is_checked_after_outer_scaling() {
    let placement = ifccad::ifcdr::PlanePlacement::try_new(
        Point3::new(0., 0., 0.),
        Vector3::new(1. + 2e-13, 0., 0.),
        Vector3::new(0., 1., 0.),
    )
    .unwrap();
    let transform = BlockTransform::try_new(placement, 0., Scale3::new(1., 1., 1.)).unwrap();
    let root = Temp::new();
    for nested in [false, true] {
        let path = root.0.join(if nested { "nested" } else { "direct" });
        native_with_outer(transform, false, nested)
            .write_directory(&path)
            .unwrap();
        let loaded = load_directory_package(&path).unwrap();
        let drawing = loaded
            .validated_package()
            .unwrap()
            .drawings()
            .next()
            .unwrap();
        if !nested {
            assert!(drawing_to_cad_document(drawing).is_ok());
            continue;
        }
        for loss_policy in [
            ifccad_convert::ConversionLossPolicy::Allow,
            ifccad_convert::ConversionLossPolicy::Reject,
        ] {
            let error = ifccad_convert::drawing_to_cad_document_with_options(
                drawing,
                ifccad_convert::ImportOptions {
                    loss_policy,
                    ..Default::default()
                },
            )
            .err()
            .expect("amplified error must block");
            let failure = match error {
                ImportError::GeometryToleranceExceeded { failure }
                | ImportError::GeometryAccuracyNotEstablished { failure } => failure,
                other => panic!("{other:?}"),
            };
            assert!(
                matches!(failure.source,ifccad_convert::ConversionEntitySource::BlockOccurrence {path,..} if path.len()==2)
            );
        }
    }
}

#[test]
fn nonneutral_frame_imports_with_explicit_parameterization_evidence() {
    let root = Temp::new();
    let path = root.0.join("package");
    let placement = ifccad::ifcdr::PlanePlacement::try_new(
        Point3::new(10., 20., 30.),
        Vector3::new(0., 1., 0.),
        Vector3::new(-1., 0., 0.),
    )
    .unwrap();
    native(
        BlockTransform::try_new(placement, 0.3, Scale3::new(2., 3., -4.)).unwrap(),
        false,
    )
    .write_directory(&path)
    .unwrap();
    let loaded = load_directory_package(&path).unwrap();
    let result = drawing_to_cad_document(
        loaded
            .validated_package()
            .unwrap()
            .drawings()
            .next()
            .unwrap(),
    )
    .unwrap();
    assert!(result.diagnostics().iter().any(|d| matches!(
        d,
        ifccad_convert::ImportDiagnostic::BlockParameterizationChanged { .. }
    )));
    let insert = result
        .document()
        .entities()
        .find_map(|e| {
            if let EntityType::Insert(i) = e {
                Some(i)
            } else {
                None
            }
        })
        .unwrap();
    assert!((insert.rotation - (std::f64::consts::FRAC_PI_2 + 0.3)).abs() < 1e-14);
    assert_eq!(insert.insert_point, cadcodec::Vector3::new(10., 20., 30.));
    assert!(result.geometry_assessment().max_deviation_upper_bound() < 1e-6);
}

#[test]
fn native_blocks_import_without_explosion_and_keep_consistent_markers() {
    let root = Temp::new();
    let path = root.0.join("package");
    native(
        BlockTransform::from_normal(
            Point3::new(10., 0., 0.),
            Vector3::new(0., 0., 1.),
            0.,
            Scale3::new(-2., -2., -2.),
        )
        .unwrap(),
        false,
    )
    .write_directory(&path)
    .unwrap();
    let loaded = load_directory_package(&path).unwrap();
    let result = drawing_to_cad_document(
        loaded
            .validated_package()
            .unwrap()
            .drawings()
            .next()
            .unwrap(),
    )
    .unwrap();
    let doc = result.document();
    let record = doc.block_records.get("Door").unwrap();
    assert_eq!(record.base_point, cadcodec::Vector3::new(2., 0., 0.));
    assert_eq!(record.units, 1);
    assert_eq!(doc.header.insertion_units, 6);
    assert_eq!(record.description, "Entrance door");
    assert!(doc.block_records.get("Unused").is_some());
    let EntityType::Block(marker) = doc.get_entity(record.block_entity_handle).unwrap() else {
        panic!("block marker")
    };
    assert_eq!(marker.base_point, record.base_point);
    let lines: Vec<_> = doc
        .entities_in_block("Door")
        .filter_map(|e| {
            if let EntityType::Line(line) = e {
                Some(line)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].start, cadcodec::Vector3::new(2., 0., 0.));
    let inserts: Vec<_> = doc
        .entities()
        .filter_map(|e| {
            if let EntityType::Insert(insert) = e {
                Some(insert)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(inserts.len(), 1);
    assert_eq!(inserts[0].block_name, "Door");
    assert_eq!(inserts[0].insert_point, cadcodec::Vector3::new(10., 0., 0.));
    assert_eq!(inserts[0].x_scale(), -2.);
    assert!(inserts[0].common.invisible);
}

#[test]
fn empty_definition_does_not_hide_target_scale_clamping() {
    let root = Temp::new();
    let path = root.0.join("package");
    native(
        BlockTransform::try_new(Default::default(), 0., Scale3::new(-1e-13, 1., 1.)).unwrap(),
        true,
    )
    .write_directory(&path)
    .unwrap();
    let loaded = load_directory_package(&path).unwrap();
    let error = drawing_to_cad_document(
        loaded
            .validated_package()
            .unwrap()
            .drawings()
            .next()
            .unwrap(),
    )
    .err()
    .expect("must not silently clamp");
    assert!(matches!(error, ImportError::BlockTargetLimitation { .. }));
}

#[test]
fn unbound_paper_scope_is_not_silently_discarded() {
    let root = Temp::new();
    let path = root.0.join("package");
    native(BlockTransform::default(), true)
        .write_directory(&path)
        .unwrap();
    let mut resource: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path.join("resources/drawing.ifcdr.json")).unwrap())
            .unwrap();
    resource["scopeTable"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"id":99,"kind":1,"bounds":null}));
    let order = &mut resource["streams"]["entityOrderStream"];
    order["scopeId"].as_array_mut().unwrap().push(99.into());
    order["entryOffset"].as_array_mut().unwrap().push(1.into());
    order["entryCount"].as_array_mut().unwrap().push(0.into());
    order["count"] = order["scopeId"].as_array().unwrap().len().into();
    let count = order["count"].clone();
    for stream in resource["streamDirectory"]["streams"]
        .as_array_mut()
        .unwrap()
    {
        if stream["name"] == "entityOrder" {
            stream["count"] = count.clone();
        }
    }
    let mut package: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path.join("package.ifcx.json")).unwrap()).unwrap();
    for node in package["data"].as_array_mut().unwrap() {
        if node["type"] == "openaec:DrawingRepresentation" {
            let descriptor = node["attributes"]["resource"].as_object_mut().unwrap();
            descriptor.remove("uri");
            descriptor.remove("checksum");
            descriptor.insert("content".into(), resource.clone());
        }
    }
    std::fs::write(
        path.join("package.ifcx.json"),
        serde_json::to_vec(&package).unwrap(),
    )
    .unwrap();
    let loaded = load_directory_package(&path).unwrap();
    let drawing = loaded
        .validated_package()
        .expect("unbound paper scope is valid native content")
        .drawings()
        .next()
        .unwrap();
    assert!(
        drawing_to_cad_document(drawing).is_err(),
        "paper scope must not disappear merely because no layout selected it"
    );
}
