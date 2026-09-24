use cadcodec::{
    BlockRecord, CadDocument, Circle, Color, EntityType, Handle, Line, LineWeight, LwPolyline,
    Transparency, Vector2, Vector3,
};
use ifccad::ifcdr::{AppearanceId, IfcdrEntityRef, Point2, Point3};
use ifccad::package::{load_directory_package, AppearanceProperty, PackageOptions};
use ifccad::PackageId;
use ifccad_convert::{
    cad_document_to_package, drawing_to_cad_document, ExportAction, ExportDiagnosticSource,
    ExportError, ExportLossPolicy, ExportLossReason, ExportOptions, SourceStructureProblem,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

struct TempRoot(PathBuf);

impl TempRoot {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "ifccad-export-entities-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn package_options(label: &str) -> PackageOptions {
    PackageOptions {
        package_id: PackageId::new(format!("entities-{label}")).unwrap(),
        data_version: "1".to_owned(),
        author: "Export entity test".to_owned(),
        timestamp: "2026-09-04T10:00:00Z".to_owned(),
    }
}

#[test]
fn cad_point_exports_as_distinct_ifcdr_point() {
    let mut document = CadDocument::new();
    document.header.point_display_mode = 34;
    document.header.point_display_size = -5.0;
    let handle = document
        .add_entity(EntityType::Point(cadcodec::Point::from_coords(
            4.0, 5.0, 6.0,
        )))
        .unwrap();
    let outcome = cad_document_to_package(
        &document,
        package_options("point"),
        ExportOptions::default(),
    )
    .unwrap();
    assert!(
        outcome.diagnostics().is_empty(),
        "{:#?}",
        outcome.diagnostics()
    );
    assert!(outcome.entity_mapping().target_entity_id(handle).is_some());
    let root = TempRoot::new();
    let target = root.0.join("point-package");
    outcome.package().write_directory(&target).unwrap();
    let loaded = load_directory_package(target).unwrap();
    assert!(loaded.report().is_empty(), "{:#?}", loaded.report());
    let package = loaded.validated_package().unwrap();
    let drawing = package.drawings().next().unwrap();
    assert_eq!(
        drawing.point_display().glyph,
        ifccad::package::PointGlyph::Plus
    );
    assert!(drawing.point_display().circle);
    assert_eq!(
        drawing.point_display().size,
        ifccad::package::PointSize::ViewportPercent(5.0)
    );
    let layout = drawing.layouts().next().unwrap();
    let resource = drawing.representation().resource();
    let entity = resource.entities(layout.scope().id()).next().unwrap();
    assert!(
        matches!(entity, IfcdrEntityRef::Point(point) if point.position() == ifccad::ifcdr::Point3::new(4.0, 5.0, 6.0))
    );
    let imported = ifccad_convert::drawing_to_cad_document(drawing).unwrap();
    assert_eq!(imported.document().header.point_display_mode, 34);
    assert_eq!(imported.document().header.point_display_size, -5.0);
    assert!(imported.document().entities().any(|entity| {
        matches!(entity, EntityType::Point(point) if point.location == Vector3::new(4.0, 5.0, 6.0))
    }));
}

#[test]
fn cad_circle_and_arc_keep_entity_kind_and_oblique_plane() {
    let mut document = CadDocument::new();
    let normal = Vector3::new(0.0, 1.0, 0.0);
    let mut circle = Circle::from_center_radius(Vector3::new(2.0, 3.0, 4.0), 5.0);
    circle.normal = normal;
    let mut arc =
        cadcodec::Arc::from_center_radius_angles(Vector3::new(1.0, 2.0, 3.0), 2.0, 0.25, 2.25);
    arc.normal = normal;
    document.add_entity(EntityType::Circle(circle)).unwrap();
    document.add_entity(EntityType::Arc(arc)).unwrap();
    let outcome = cad_document_to_package(
        &document,
        package_options("circle-arc"),
        ExportOptions::default(),
    )
    .unwrap();
    assert!(
        outcome.diagnostics().is_empty(),
        "{:#?}",
        outcome.diagnostics()
    );
    let root = TempRoot::new();
    let target = root.0.join("circle-arc-package");
    outcome.package().write_directory(&target).unwrap();
    let loaded = load_directory_package(&target).unwrap();
    assert!(loaded.report().is_empty(), "{:#?}", loaded.report());
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    let scope = drawing.layouts().next().unwrap().scope().id();
    let resource = drawing.representation().resource();
    let entities: Vec<_> = resource.entities(scope).collect();
    assert!(matches!(entities[0], IfcdrEntityRef::Circle(circle) if circle.radius() == 5.0));
    assert!(matches!(entities[1], IfcdrEntityRef::Arc(arc) if arc.sweep_parameter() == 2.0));
    let imported = ifccad_convert::drawing_to_cad_document(drawing).unwrap();
    assert!(imported
        .document()
        .entities()
        .any(|entity| matches!(entity, EntityType::Circle(_))));
    assert!(imported
        .document()
        .entities()
        .any(|entity| matches!(entity, EntityType::Arc(_))));
}

#[test]
fn cad_ellipse_full_and_nearly_full_keep_distinct_ifcdr_kinds() {
    let mut document = CadDocument::new();
    let full = cadcodec::Ellipse::from_center_axes(
        Vector3::new(1.0, 2.0, 3.0),
        Vector3::new(0.0, 4.0, 0.0),
        0.5,
    );
    let mut partial = full.clone();
    partial.center = Vector3::new(5.0, 6.0, 7.0);
    partial.end_parameter = std::f64::consts::TAU.next_down();
    document.add_entity(EntityType::Ellipse(full)).unwrap();
    document.add_entity(EntityType::Ellipse(partial)).unwrap();
    let exported = cad_document_to_package(
        &document,
        package_options("ellipses"),
        ExportOptions::default(),
    )
    .unwrap();
    let root = TempRoot::new();
    let target = root.0.join("ellipses");
    exported.package().write_directory(&target).unwrap();
    let loaded = load_directory_package(&target).unwrap();
    assert!(loaded.report().is_empty(), "{:#?}", loaded.report());
    let package = loaded.validated_package().unwrap();
    let drawing = package.drawings().next().unwrap();
    let scope = drawing.layouts().next().unwrap().scope().id();
    let resource = drawing.representation().resource();
    let entities: Vec<_> = resource.entities(scope).collect();
    assert!(matches!(entities[0], IfcdrEntityRef::Ellipse(_)));
    assert!(matches!(entities[1], IfcdrEntityRef::EllipseArc(_)));
    let imported = ifccad_convert::drawing_to_cad_document(drawing).unwrap();
    assert_eq!(
        imported
            .document()
            .entities()
            .filter(|e| matches!(e, EntityType::Ellipse(_)))
            .count(),
        2
    );
}

#[test]
fn exact_model_space_lines_and_straight_lwpolylines_are_emitted_and_mapped() {
    let mut document = CadDocument::new();
    let line_handle = document
        .add_entity(EntityType::Line(Line::from_coords(
            1.0, 2.0, 0.0, 3.0, 4.0, 0.0,
        )))
        .unwrap();
    let mut polyline = LwPolyline::from_points(vec![
        Vector2::new(-2.0, 3.0),
        Vector2::new(4.0, -5.0),
        Vector2::new(8.0, 1.0),
    ]);
    polyline.is_closed = true;
    polyline.vertices[0].bulge = 0.5;
    polyline.vertices[2].bulge = -0.25;
    polyline.common.color = Color::Index(5);
    polyline.common.transparency = Transparency::ByLayer;
    polyline.common.linetype = "ByBlock".to_owned();
    polyline.common.line_weight = LineWeight::W0_18;
    let polyline_handle = document
        .add_entity(EntityType::LwPolyline(polyline))
        .unwrap();

    let outcome = cad_document_to_package(
        &document,
        package_options("exact"),
        ExportOptions::default(),
    )
    .unwrap_or_else(|error| panic!("export failed: {error}"));
    assert!(outcome.diagnostics().is_empty());
    assert_eq!(
        outcome.transfer_assessment().conclusion(),
        ifccad_convert::TransferConclusion::NoLossDetected
    );
    assert_eq!(
        outcome.transfer_assessment().scope(),
        ifccad_convert::TransferScope::PublicCadDocumentToPackage
    );
    assert_eq!(outcome.entity_mapping().len(), 2);
    assert_eq!(
        outcome
            .entity_mapping()
            .target_entity_id(line_handle)
            .unwrap()
            .get(),
        1
    );
    assert_eq!(
        outcome
            .entity_mapping()
            .target_entity_id(polyline_handle)
            .unwrap()
            .get(),
        2
    );

    let root = TempRoot::new();
    let package_root = root.0.join("package");
    outcome.package().write_directory(&package_root).unwrap();
    let loaded = load_directory_package(package_root).unwrap();
    assert!(loaded.report().is_empty(), "{:#?}", loaded.report());
    let package = loaded.validated_package().unwrap();
    let drawing = package.drawings().next().unwrap();
    let layout = drawing.layouts().next().unwrap();
    let representation = drawing.representation();
    let resource = representation.resource();
    let entities = resource.entities(layout.scope().id()).collect::<Vec<_>>();
    let IfcdrEntityRef::Line(line) = entities[0] else {
        panic!("first entity must be LINE");
    };
    assert_eq!(line.start(), ifccad::ifcdr::Point3::new(1.0, 2.0, 0.0));
    assert_eq!(line.end(), ifccad::ifcdr::Point3::new(3.0, 4.0, 0.0));
    let IfcdrEntityRef::PlanarPolyline(polyline) = entities[1] else {
        panic!("second entity must be LWPOLYLINE");
    };
    assert_eq!(
        polyline.local_points().collect::<Vec<_>>(),
        [
            Point2::new(-2.0, 3.0),
            Point2::new(4.0, -5.0),
            Point2::new(8.0, 1.0),
        ]
    );
    assert!(polyline.closed());
    assert_eq!(polyline.bulges().collect::<Vec<_>>(), [0.5, 0.0, -0.25]);
    let imported = drawing_to_cad_document(drawing).unwrap();
    let imported_polyline = imported
        .document()
        .entities()
        .find_map(|entity| match entity {
            EntityType::LwPolyline(polyline) => Some(polyline),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        imported_polyline
            .vertices
            .iter()
            .map(|vertex| vertex.bulge)
            .collect::<Vec<_>>(),
        [0.5, 0.0, -0.25]
    );
    let appearance = representation
        .appearance(AppearanceId::from(polyline.appearance_id().get()))
        .unwrap();
    assert!(matches!(
        appearance.color(),
        AppearanceProperty::Explicit(color) if color.indexed().unwrap().index() == 5
    ));
    assert!(matches!(appearance.opacity(), AppearanceProperty::ByLayer));
    assert!(matches!(
        appearance.line_pattern(),
        AppearanceProperty::ByBlock
    ));
    assert!(matches!(
        appearance.line_weight(),
        AppearanceProperty::Explicit(0.18)
    ));
}

#[test]
fn ordinary_3d_polylines_export_as_spatial_and_import_as_polyline3d() {
    let mut document = CadDocument::new();
    let points = vec![Vector3::new(1.0, 2.0, 3.0), Vector3::new(4.0, -5.0, 6.0)];
    let mut generic = cadcodec::entities::Polyline::from_points(points.clone());
    generic.close();
    document.add_entity(EntityType::Polyline(generic)).unwrap();
    document
        .add_entity(EntityType::Polyline3D(
            cadcodec::entities::Polyline3D::from_points(points.clone()),
        ))
        .unwrap();
    let outcome = cad_document_to_package(
        &document,
        package_options("spatial"),
        ExportOptions::default(),
    )
    .unwrap();
    assert!(
        outcome.diagnostics().is_empty(),
        "{:?}",
        outcome.diagnostics()
    );
    let root = TempRoot::new();
    let package_root = root.0.join("package");
    outcome.package().write_directory(&package_root).unwrap();
    let loaded = load_directory_package(package_root).unwrap();
    assert!(loaded.report().is_empty(), "{:#?}", loaded.report());
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    let scope = drawing.layouts().next().unwrap().scope().id();
    let resource = drawing.representation().resource();
    let spatial = resource
        .entities(scope)
        .map(|entity| {
            let IfcdrEntityRef::SpatialPolyline(polyline) = entity else {
                panic!("spatial polyline")
            };
            (polyline.points().to_vec(), polyline.closed())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        spatial[0].0,
        vec![Point3::new(1.0, 2.0, 3.0), Point3::new(4.0, -5.0, 6.0)]
    );
    assert_eq!([spatial[0].1, spatial[1].1], [true, false]);
    let imported = drawing_to_cad_document(drawing).unwrap();
    let entities = imported.document().entities().collect::<Vec<_>>();
    assert_eq!(entities.len(), 2);
    assert!(matches!(entities[0], EntityType::Polyline3D(polyline) if polyline.flags.closed));
    assert!(matches!(entities[1], EntityType::Polyline3D(polyline) if !polyline.flags.closed));
}

#[test]
fn fitted_and_mesh_polylines_are_skipped_or_rejected_without_straight_substitutes() {
    let points = vec![Vector3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 2.0, 3.0)];
    let mut document = CadDocument::new();
    let mut fitted_2d = cadcodec::entities::Polyline2D::new();
    fitted_2d.flags = cadcodec::entities::PolylineFlags::CURVE_FIT;
    fitted_2d.add_vertex(cadcodec::entities::Vertex2D::from_point(Vector2::new(
        0.0, 0.0,
    )));
    fitted_2d.add_vertex(cadcodec::entities::Vertex2D::from_point(Vector2::new(
        1.0, 2.0,
    )));
    document
        .add_entity(EntityType::Polyline2D(fitted_2d))
        .unwrap();

    let mut fitted_generic = cadcodec::entities::Polyline::from_points(points.clone());
    fitted_generic.flags = cadcodec::entities::PolylineFlags::SPLINE_FIT;
    document
        .add_entity(EntityType::Polyline(fitted_generic))
        .unwrap();

    let mut fitted_3d = cadcodec::entities::Polyline3D::from_points(points.clone());
    fitted_3d.flags.spline_fit = true;
    document
        .add_entity(EntityType::Polyline3D(fitted_3d))
        .unwrap();

    let mut mesh_3d = cadcodec::entities::Polyline3D::from_points(points.clone());
    mesh_3d.flags.is_3d_mesh = true;
    document
        .add_entity(EntityType::Polyline3D(mesh_3d))
        .unwrap();

    let mut inconsistent_3d = cadcodec::entities::Polyline3D::from_points(points);
    inconsistent_3d.flags.is_3d = false;
    document
        .add_entity(EntityType::Polyline3D(inconsistent_3d))
        .unwrap();

    let allowed = cad_document_to_package(
        &document,
        package_options("fitted-polyline-allow"),
        ExportOptions::default(),
    )
    .unwrap();
    assert!(allowed.entity_mapping().is_empty());
    assert_eq!(allowed.diagnostics().len(), 5);
    assert!(allowed
        .diagnostics()
        .iter()
        .all(|diagnostic| diagnostic.action() == ExportAction::Skipped));
    for diagnostic in &allowed.diagnostics()[..3] {
        assert!(diagnostic
            .reasons()
            .contains(&ExportLossReason::UnsupportedSemantic {
                name: "polyline fit curve".into(),
            }));
    }
    assert!(allowed.diagnostics()[3]
        .reasons()
        .contains(&ExportLossReason::UnsupportedSemantic {
            name: "polyline mesh".into(),
        }));
    assert!(allowed.diagnostics()[4]
        .reasons()
        .contains(&ExportLossReason::UnsupportedSemantic {
            name: "3D polyline source properties".into(),
        }));

    let rejected = cad_document_to_package(
        &document,
        package_options("fitted-polyline-reject"),
        ExportOptions {
            loss_policy: ExportLossPolicy::Reject,
            ..ExportOptions::default()
        },
    )
    .err()
    .expect("reject refuses unsupported polylines");
    assert!(
        matches!(rejected, ExportError::LossRejected { diagnostics } if diagnostics == allowed.diagnostics())
    );
}

#[test]
fn two_dimensional_polyline_keeps_bulge_and_reports_width_loss() {
    let mut document = CadDocument::new();
    let mut polyline = cadcodec::entities::Polyline2D::new();
    polyline.add_vertex(
        cadcodec::entities::Vertex2D::from_point(Vector2::new(0.0, 0.0))
            .with_bulge(0.5)
            .with_width(1.0, 2.0),
    );
    polyline.add_vertex(
        cadcodec::entities::Vertex2D::from_point(Vector2::new(4.0, 0.0)).with_bulge(-0.25),
    );
    document
        .add_entity(EntityType::Polyline2D(polyline))
        .unwrap();
    let outcome = cad_document_to_package(
        &document,
        package_options("polyline-2d"),
        ExportOptions::default(),
    )
    .unwrap();
    assert!(outcome.diagnostics().iter().any(|diagnostic| diagnostic
        .reasons()
        .contains(&ExportLossReason::PolylineWidth)));
    let root = TempRoot::new();
    let package_root = root.0.join("package");
    outcome.package().write_directory(&package_root).unwrap();
    let loaded = load_directory_package(package_root).unwrap();
    assert!(loaded.report().is_empty(), "{:#?}", loaded.report());
    let drawing = loaded
        .validated_package()
        .unwrap()
        .drawings()
        .next()
        .unwrap();
    let scope = drawing.layouts().next().unwrap().scope().id();
    let resource = drawing.representation().resource();
    let IfcdrEntityRef::PlanarPolyline(polyline) = resource.entities(scope).next().unwrap() else {
        panic!("planar polyline")
    };
    assert_eq!(polyline.bulges().collect::<Vec<_>>(), [0.5, -0.25]);
}

#[test]
fn two_dimensional_polyline_rejects_unsupported_surface_and_vertex_flags() {
    let mut document = CadDocument::new();
    let mut polyline = cadcodec::entities::Polyline2D::new();
    polyline.smooth_surface = cadcodec::entities::SmoothSurfaceType::CubicBSpline;
    polyline.flags = cadcodec::entities::PolylineFlags::from_bits(0x100);
    polyline.add_vertex(cadcodec::entities::Vertex2D::from_point(Vector2::new(
        0.0, 0.0,
    )));
    let mut vertex = cadcodec::entities::Vertex2D::from_point(Vector2::new(4.0, 0.0));
    vertex.flags = cadcodec::entities::VertexFlags::POLYGON_MESH;
    polyline.add_vertex(vertex);
    document
        .add_entity(EntityType::Polyline2D(polyline))
        .unwrap();

    let outcome = cad_document_to_package(
        &document,
        package_options("polyline-2d-unsupported"),
        ExportOptions::default(),
    )
    .unwrap();
    assert!(outcome.entity_mapping().is_empty());
    assert_eq!(outcome.diagnostics().len(), 1);
    let diagnostic = &outcome.diagnostics()[0];
    assert_eq!(diagnostic.action(), ExportAction::Skipped);
    assert!(diagnostic
        .reasons()
        .contains(&ExportLossReason::UnsupportedSemantic {
            name: "polyline fit curve".into(),
        }));
    assert!(diagnostic
        .reasons()
        .contains(&ExportLossReason::UnsupportedSemantic {
            name: "2D polyline vertex flags".into(),
        }));
    assert!(diagnostic
        .reasons()
        .contains(&ExportLossReason::UnsupportedSemantic {
            name: "polyline flags".into(),
        }));
}

#[test]
fn inexact_geometry_is_skipped_once_with_all_reasons() {
    let mut document = CadDocument::new();
    let mut line = Line::from_coords(0.0, 0.0, 1.0, f64::INFINITY, 2.0, 0.0);
    line.thickness = 2.0;
    line.normal = Vector3::UNIT_X;
    let line_handle = document.add_entity(EntityType::Line(line)).unwrap();

    let mut polyline = LwPolyline::from_points(vec![Vector2::new(f64::NAN, 0.0)]);
    polyline.vertices[0].bulge = 0.5;
    polyline.vertices[0].start_width = 1.0;
    polyline.vertices[0].end_width = 2.0;
    polyline.plinegen = true;
    polyline.constant_width = 3.0;
    polyline.elevation = 4.0;
    polyline.thickness = 5.0;
    polyline.normal = Vector3::UNIT_X;
    let polyline_handle = document
        .add_entity(EntityType::LwPolyline(polyline))
        .unwrap();

    let outcome = cad_document_to_package(
        &document,
        package_options("inexact"),
        ExportOptions::default(),
    )
    .unwrap_or_else(|error| panic!("export failed: {error}"));
    assert!(outcome.entity_mapping().is_empty());
    assert_eq!(outcome.diagnostics().len(), 2);

    let line = &outcome.diagnostics()[0];
    assert_eq!(line.action(), ExportAction::Skipped);
    assert_eq!(
        line.source(),
        &ExportDiagnosticSource::Entity {
            handle: line_handle,
            kind: "LINE".to_owned(),
        }
    );
    assert_eq!(
        line.reasons(),
        [
            ExportLossReason::NonFiniteCoordinate,
            ExportLossReason::NonZeroThickness,
        ]
    );

    let polyline = &outcome.diagnostics()[1];
    assert_eq!(
        polyline.source(),
        &ExportDiagnosticSource::Entity {
            handle: polyline_handle,
            kind: "LWPOLYLINE".to_owned(),
        }
    );
    assert_eq!(
        polyline.reasons(),
        [
            ExportLossReason::NonFiniteCoordinate,
            ExportLossReason::PolylineTooFewVertices { count: 1 },
            ExportLossReason::NonZeroThickness,
            ExportLossReason::PolylinePlinegen,
            ExportLossReason::PolylineWidth,
        ]
    );
}

#[test]
fn paper_ownership_is_retained_while_unsupported_layers_and_types_are_skipped() {
    let mut document = CadDocument::new();

    let mut missing_layer = Line::from_coords(0.0, 0.0, 0.0, 1.0, 1.0, 0.0);
    missing_layer.common.layer = "DOES-NOT-EXIST".to_owned();
    let missing_layer_handle = document
        .add_entity(EntityType::Line(missing_layer))
        .unwrap();

    let mut paper = Line::from_coords(0.0, 0.0, 0.0, 1.0, 1.0, 0.0);
    paper.common.owner_handle = document.header.paper_space_block_handle;
    let paper_handle = document.add_entity(EntityType::Line(paper)).unwrap();

    let block_handle = Handle::new(0xF000);
    let mut block = BlockRecord::new("Detail block");
    block.handle = block_handle;
    // Local definitions are supported; external-reference contents still skip.
    block.flags.is_xref = true;
    document.block_records.add(block).unwrap();
    let mut block_line = Line::from_coords(0.0, 0.0, 0.0, 1.0, 1.0, 0.0);
    block_line.common.owner_handle = block_handle;
    let block_line_handle = document.add_entity(EntityType::Line(block_line)).unwrap();

    let circle_handle = document
        .add_entity(EntityType::Circle(Circle::new()))
        .unwrap();

    let outcome = cad_document_to_package(
        &document,
        package_options("coherent-skips"),
        ExportOptions::default(),
    )
    .unwrap_or_else(|error| panic!("export failed: {error}"));
    assert!(outcome
        .entity_mapping()
        .target_entity_id(paper_handle)
        .is_some());
    assert_eq!(outcome.diagnostics().len(), 3);
    assert!(outcome
        .entity_mapping()
        .target_entity_id(circle_handle)
        .is_some());
    assert_eq!(
        outcome.diagnostics()[0].source(),
        &ExportDiagnosticSource::Entity {
            handle: missing_layer_handle,
            kind: "LINE".to_owned(),
        }
    );
    assert_eq!(
        outcome.diagnostics()[0].reasons(),
        [ExportLossReason::MissingEntityLayer {
            name: "DOES-NOT-EXIST".to_owned(),
        }]
    );
    assert_eq!(
        outcome.diagnostics()[1].source(),
        &ExportDiagnosticSource::Entity {
            handle: block_line_handle,
            kind: "LINE".to_owned(),
        }
    );
    assert_eq!(
        outcome.diagnostics()[1].reasons(),
        [ExportLossReason::BlockOwnedEntity {
            owner: block_handle,
        }]
    );
    assert_eq!(
        outcome.diagnostics()[2].source(),
        &ExportDiagnosticSource::Table {
            kind: "block_records".to_owned(),
        }
    );
    assert_eq!(
        outcome.diagnostics()[2].reasons(),
        [ExportLossReason::UnsupportedTableRecords {
            kind: "block_records".to_owned(),
            count: 1,
        }]
    );
}

#[test]
fn inconsistent_entity_owners_are_aggregated_as_fatal_structure_problems() {
    let mut document = CadDocument::new();
    let missing_handle = document.add_entity(EntityType::Line(Line::new())).unwrap();
    let unknown_owner = Handle::new(0xDEAD);
    let mut unknown = Line::new();
    unknown.common.owner_handle = unknown_owner;
    let unknown_handle = document.add_entity(EntityType::Line(unknown)).unwrap();
    document
        .entities_mut()
        .find(|entity| entity.common().handle == missing_handle)
        .unwrap()
        .common_mut()
        .owner_handle = Handle::NULL;
    document
        .entities_mut()
        .find(|entity| entity.common().handle == unknown_handle)
        .unwrap()
        .common_mut()
        .owner_handle = unknown_owner;

    let error = cad_document_to_package(
        &document,
        package_options("fatal-owners"),
        ExportOptions::default(),
    )
    .err()
    .expect("inconsistent ownership must fail export");
    let ExportError::InvalidSourceStructure { problems } = error else {
        panic!("unexpected export error: {error}");
    };
    assert_eq!(
        problems,
        [
            SourceStructureProblem::EntityOwnerMissing {
                entity: missing_handle,
            },
            SourceStructureProblem::EntityOwnerUnknown {
                entity: unknown_handle,
                owner: unknown_owner,
            },
        ]
    );
}
