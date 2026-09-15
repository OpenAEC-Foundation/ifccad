use cadcodec::{
    CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType, Line, LwPolyline, Vector2,
};
use ifccad::ifcdr::{IfcdrEntityRef, IfcdrLengthUnit, PlanePlacement, Point2, Point3, Vector3};
use ifccad::package::*;
use ifccad::{PackageId, ResourceId};
use ifccad_convert::*;
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "ifccad-spatial-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn options() -> PackageOptions {
    PackageOptions {
        package_id: PackageId::new("spatial").unwrap(),
        data_version: "1".into(),
        author: "Spatial exchange tests".into(),
        timestamp: "2026-09-15T00:00:00Z".into(),
    }
}
fn native(plane: PlanePlacement, points: Vec<Point2>) -> EncodedPackage {
    let mut b = PackageBuilder::new(options()).unwrap();
    let mut d = b
        .add_drawing(DrawingOptions {
            model_layout_name: "Model".into(),
            representation_resource_id: ResourceId::new("drawing").unwrap(),
            length_unit: IfcdrLengthUnit::Metre,
        })
        .unwrap();
    let a = d
        .appearances()
        .add(AppearanceDefinition {
            name: "Default".into(),
            color: AppearanceColor::rgb(0, 0, 0),
            opacity: 1.0,
            line_pattern: LinePatternDefinition::named("Continuous"),
            line_weight: 0.25,
        })
        .unwrap();
    let layer = d
        .layers()
        .add(LayerDefinition {
            name: "0".into(),
            visible: true,
            appearance: a,
        })
        .unwrap();
    d.model_space()
        .add_line(LineDefinition {
            start: Point3::new(0., 1., 2.),
            end: Point3::new(3., 4., 5.),
            layer,
            appearance: EntityAppearance::by_layer(),
            visible: false,
        })
        .unwrap();
    d.model_space()
        .add_polyline(PolylineDefinition {
            points,
            placement: plane,
            closed: false,
            layer,
            appearance: EntityAppearance::by_layer(),
            visible: true,
        })
        .unwrap();
    b.finish().unwrap()
}
#[test]
fn native_shift_and_rotation_preserve_geometry_but_reject_parameter_loss() {
    let root = Temp::new();
    let plane = PlanePlacement::try_new(
        Point3::new(10., 20., 5.),
        Vector3::new(0., 1., 0.),
        Vector3::new(-1., 0., 0.),
    )
    .unwrap();
    native(plane, vec![Point2::new(2., 3.), Point2::new(4., 5.)])
        .write_directory(root.0.join("package"))
        .unwrap();
    let loaded = load_directory_package(root.0.join("package")).unwrap();
    assert!(loaded.report().is_empty(), "{:?}", loaded.report());
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    let scope = drawing.layouts().next().unwrap().scope();
    let bound = scope.bounds().unwrap();
    assert_eq!(bound.min().z(), 2.);
    assert_eq!(bound.max().z(), 5.);
    let imported = drawing_to_cad_document(drawing).unwrap();
    assert_eq!(
        imported.geometry_assessment().status(),
        ConversionGeometryStatus::Exact
    );
    assert_eq!(imported.geometry_assessment().assessed_vertices(), 4);
    assert!(imported
        .diagnostics()
        .iter()
        .any(|d| matches!(d, ImportDiagnostic::PlaneParameterizationChanged { .. })));
    assert!(matches!(
        drawing_to_cad_document_with_options(
            drawing,
            ImportOptions {
                loss_policy: ConversionLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(ImportError::LossRejected { .. })
    ));
    let target = imported
        .document()
        .entities()
        .find_map(|e| {
            if let EntityType::LwPolyline(p) = e {
                Some(p)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(target.vertices[0].location, Vector2::new(7., 22.));
    assert_eq!(target.elevation, 5.);
    let parts = imported.into_all_parts();
    assert_eq!(parts.geometry_assessment.assessed_entities(), 2);
    assert_eq!(
        parts.transfer_assessment.conclusion(),
        TransferConclusion::LossDetected
    );
}
#[test]
fn geometric_accuracy_is_a_hard_gate_for_both_policies() {
    let mut document = CadDocument::new();
    document.header.insertion_units = 6;
    let mut poly = LwPolyline::from_points(vec![Vector2::new(0., 0.), Vector2::new(1., 1.)]);
    poly.normal = cadcodec::Vector3::new(1., 1., 1.);
    poly.elevation = 1.1;
    document.add_entity(EntityType::LwPolyline(poly)).unwrap();
    for loss_policy in [ConversionLossPolicy::Allow, ConversionLossPolicy::Reject] {
        let result = cad_document_to_package(
            &document,
            options(),
            ExportOptions {
                loss_policy,
                geometry_tolerance: ConversionGeometryTolerance::exact(),
            },
        );
        assert!(matches!(
            result,
            Err(ExportError::GeometryToleranceExceeded { .. })
        ));
    }
    let accepted = cad_document_to_package(&document, options(), ExportOptions::default()).unwrap();
    assert_eq!(
        accepted.geometry_assessment().status(),
        ConversionGeometryStatus::RoundedWithinTolerance
    );
    assert!(accepted.geometry_assessment().max_deviation_upper_bound() > 0.);
    assert!(accepted.geometry_assessment().max_deviation_upper_bound() < 1e-6);
}
#[test]
fn vertex_ids_and_line_normal_are_partial_losses_with_geometry_retained() {
    let mut document = CadDocument::new();
    let mut line = Line::from_coords(0., 1., 2., 3., 4., 5.);
    line.normal = cadcodec::Vector3::UNIT_X;
    document.add_entity(EntityType::Line(line)).unwrap();
    let mut poly = LwPolyline::from_points(vec![Vector2::new(0., 0.), Vector2::new(1., 2.)]);
    poly.vertices[0].vertex_id = -7;
    document.add_entity(EntityType::LwPolyline(poly)).unwrap();
    let out = cad_document_to_package(&document, options(), ExportOptions::default()).unwrap();
    assert_eq!(out.entity_mapping().len(), 2);
    assert_eq!(out.geometry_assessment().assessed_vertices(), 4);
    assert!(out
        .diagnostics()
        .iter()
        .all(|d| d.action() == ExportAction::PartiallyExported));
    assert!(out
        .diagnostics()
        .iter()
        .flat_map(|d| d.reasons())
        .any(|r| matches!(r, ExportLossReason::PolylineVertexIdentifiers { count: 1 })));
    assert!(matches!(
        cad_document_to_package(
            &document,
            options(),
            ExportOptions {
                loss_policy: ConversionLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(ExportError::LossRejected { .. })
    ));
}
#[test]
fn spatial_geometry_crosses_dxf_and_dwg_and_strict_package_readers() {
    let root = Temp::new();
    let mut document = CadDocument::new();
    document
        .add_entity(EntityType::Line(Line::from_coords(1., 2., 3., 4., 5., 6.)))
        .unwrap();
    let mut poly = LwPolyline::from_points(vec![Vector2::new(2., 3.), Vector2::new(4., 5.)]);
    poly.normal = cadcodec::Vector3::UNIT_X;
    poly.elevation = 7.;
    document.add_entity(EntityType::LwPolyline(poly)).unwrap();
    for format in ["dxf", "dwg"] {
        let path = root.0.join(format!("source.{format}"));
        if format == "dxf" {
            DxfWriter::new(&document).write_to_file(&path).unwrap();
        } else {
            fs::write(&path, DwgWriter::write_to_vec(&document).unwrap()).unwrap();
        }
        let cad = if format == "dxf" {
            DxfReader::from_file(&path).unwrap().read().unwrap()
        } else {
            DwgReader::from_file(&path).unwrap().read().unwrap()
        };
        let package = cad_document_to_package(&cad, options(), ExportOptions::default()).unwrap();
        assert_eq!(
            package.geometry_assessment().status(),
            ConversionGeometryStatus::Exact
        );
        let dir = root.0.join(format);
        package.package().write_directory(&dir).unwrap();
        let loaded = load_directory_package(&dir).unwrap();
        assert!(loaded.report().is_empty(), "{:?}", loaded.report());
        let drawing = loaded
            .validated_package()
            .unwrap()
            .drawings()
            .next()
            .unwrap();
        let layout = drawing.layouts().next().unwrap();
        let points = layout
            .representation()
            .resource()
            .entities(layout.scope().id())
            .flat_map(|e| match e {
                IfcdrEntityRef::Line(l) => vec![l.start(), l.end()],
                IfcdrEntityRef::Polyline(p) => {
                    p.scope_points().collect::<Result<Vec<_>, _>>().unwrap()
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(
            points,
            vec![
                Point3::new(1., 2., 3.),
                Point3::new(4., 5., 6.),
                Point3::new(7., 2., 3.),
                Point3::new(7., 4., 5.)
            ]
        );
        let back = drawing_to_cad_document(drawing).unwrap();
        assert_eq!(
            back.geometry_assessment().status(),
            ConversionGeometryStatus::Exact
        );
        let back_poly = back
            .document()
            .entities()
            .find_map(|e| {
                if let EntityType::LwPolyline(p) = e {
                    Some(p)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(back_poly.normal, cadcodec::Vector3::UNIT_X);
        assert_eq!(back_poly.elevation, 7.);
        let final_path = root.0.join(format!("returned.{format}"));
        if format == "dxf" {
            DxfWriter::new(back.document())
                .write_to_file(&final_path)
                .unwrap();
        } else {
            fs::write(
                &final_path,
                DwgWriter::write_to_vec(back.document()).unwrap(),
            )
            .unwrap();
        }
        let returned = if format == "dxf" {
            DxfReader::from_file(&final_path).unwrap().read().unwrap()
        } else {
            DwgReader::from_file(&final_path).unwrap().read().unwrap()
        };
        let returned_poly = returned
            .entities()
            .find_map(|e| {
                if let EntityType::LwPolyline(p) = e {
                    Some(p)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(returned_poly.normal, back_poly.normal);
        assert_eq!(returned_poly.elevation, back_poly.elevation);
        assert_eq!(
            returned_poly
                .vertices
                .iter()
                .map(|v| v.location)
                .collect::<Vec<_>>(),
            back_poly
                .vertices
                .iter()
                .map(|v| v.location)
                .collect::<Vec<_>>()
        );
        let returned_line = returned
            .entities()
            .find_map(|e| {
                if let EntityType::Line(l) = e {
                    Some(l)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(returned_line.start, cadcodec::Vector3::new(1., 2., 3.));
        assert_eq!(returned_line.end, cadcodec::Vector3::new(4., 5., 6.));
    }
}

#[test]
fn unrepresentable_placed_geometry_is_fatal_under_both_policies() {
    let mut document = CadDocument::new();
    document.header.insertion_units = 6;
    let mut poly = LwPolyline::from_points(vec![Vector2::new(0., f64::MAX), Vector2::new(0., 0.)]);
    poly.normal = cadcodec::Vector3::new(1., 0., 1.);
    poly.elevation = f64::MAX;
    document.add_entity(EntityType::LwPolyline(poly)).unwrap();
    for loss_policy in [ConversionLossPolicy::Allow, ConversionLossPolicy::Reject] {
        match cad_document_to_package(
            &document,
            options(),
            ExportOptions {
                loss_policy,
                ..Default::default()
            },
        ) {
            Err(ExportError::GeometryAccuracyNotEstablished { failure }) => {
                assert_eq!(
                    failure.reason,
                    ConversionGeometryFailureReason::TargetCoordinateOutOfRange
                );
                assert_eq!(failure.vertex_index, Some(0));
                assert!(failure.deviation.is_none());
            }
            _ => panic!("out-of-range geometry must be a fatal numerical failure"),
        }
    }
}

#[test]
fn reject_accepts_proven_numeric_rounding_and_retains_loss_evidence() {
    let mut document = CadDocument::new();
    document.header.insertion_units = 6;
    let mut poly = LwPolyline::from_points(vec![Vector2::new(0., 0.), Vector2::new(1., 1.)]);
    let n = 1.0 / 2.0_f64.sqrt();
    poly.normal = cadcodec::Vector3::new(0., n, n);
    poly.elevation = 1.1;
    document.add_entity(EntityType::LwPolyline(poly)).unwrap();
    let outcome = cad_document_to_package(
        &document,
        options(),
        ExportOptions {
            loss_policy: ConversionLossPolicy::Reject,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        outcome.geometry_assessment().status(),
        ConversionGeometryStatus::RoundedWithinTolerance
    );
    assert_eq!(
        outcome.transfer_assessment().conclusion(),
        TransferConclusion::LossDetected
    );
    assert!(outcome
        .diagnostics()
        .iter()
        .flat_map(|d| d.reasons())
        .all(|r| matches!(r, ExportLossReason::GeometryRoundedWithinTolerance { .. })));
    let parts = outcome.into_all_parts();
    assert_eq!(parts.geometry_assessment.assessed_vertices(), 2);
    let root = Temp::new();
    parts
        .package
        .write_directory(root.0.join("package"))
        .unwrap();
    assert!(load_directory_package(root.0.join("package"))
        .unwrap()
        .validated_package()
        .is_some());
}

#[test]
fn import_accuracy_failure_precedes_loss_policy_for_native_oblique_frame() {
    let h = 0.5_f64.sqrt();
    let plane = PlanePlacement::try_new(
        Point3::new(0.1, 0.2, 0.3),
        Vector3::new(h, h, 0.),
        Vector3::new(0., 0., 1.),
    )
    .unwrap();
    let root = Temp::new();
    native(plane, vec![Point2::new(0., 0.), Point2::new(1., 1.)])
        .write_directory(root.0.join("package"))
        .unwrap();
    let loaded = load_directory_package(root.0.join("package")).unwrap();
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    for loss_policy in [ConversionLossPolicy::Allow, ConversionLossPolicy::Reject] {
        assert!(matches!(
            drawing_to_cad_document_with_options(
                drawing,
                ImportOptions {
                    loss_policy,
                    geometry_tolerance: ConversionGeometryTolerance::exact()
                }
            ),
            Err(ImportError::GeometryToleranceExceeded { .. })
        ));
    }
}

#[test]
fn native_shifted_plane_crosses_both_file_codecs_with_explicit_parameter_loss() {
    let root = Temp::new();
    let plane = PlanePlacement::try_new(
        Point3::new(7., 20., 30.),
        Vector3::new(0., 1., 0.),
        Vector3::new(0., 0., 1.),
    )
    .unwrap();
    native(plane, vec![Point2::new(2., 3.), Point2::new(4., 5.)])
        .write_directory(root.0.join("source"))
        .unwrap();
    let loaded = load_directory_package(root.0.join("source")).unwrap();
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    let cad = drawing_to_cad_document(drawing).unwrap();
    assert!(cad
        .diagnostics()
        .iter()
        .any(|d| matches!(d, ImportDiagnostic::PlaneParameterizationChanged { .. })));
    for format in ["dxf", "dwg"] {
        let path = root.0.join(format!("native.{format}"));
        if format == "dxf" {
            DxfWriter::new(cad.document()).write_to_file(&path).unwrap();
        } else {
            fs::write(&path, DwgWriter::write_to_vec(cad.document()).unwrap()).unwrap();
        }
        let returned = if format == "dxf" {
            DxfReader::from_file(&path).unwrap().read().unwrap()
        } else {
            DwgReader::from_file(&path).unwrap().read().unwrap()
        };
        let package =
            cad_document_to_package(&returned, options(), ExportOptions::default()).unwrap();
        assert_eq!(
            package.geometry_assessment().status(),
            ConversionGeometryStatus::Exact
        );
        package
            .package()
            .write_directory(root.0.join(format))
            .unwrap();
        let loaded = load_directory_package(root.0.join(format)).unwrap();
        let drawing = loaded
            .validated_package()
            .unwrap()
            .drawings()
            .next()
            .unwrap();
        let layout = drawing.layouts().next().unwrap();
        let resource = layout.representation().resource();
        let poly = resource
            .entities(layout.scope().id())
            .find_map(|e| {
                if let IfcdrEntityRef::Polyline(p) = e {
                    Some(p)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(
            poly.scope_points().collect::<Result<Vec<_>, _>>().unwrap(),
            vec![Point3::new(7., 22., 33.), Point3::new(7., 24., 35.)]
        );
        assert_eq!(poly.placement().origin(), Point3::new(7., 0., 0.));
        assert_ne!(poly.placement(), plane);
    }
}
