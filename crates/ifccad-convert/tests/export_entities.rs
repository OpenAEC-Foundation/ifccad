use cadcodec::{
    BlockRecord, CadDocument, Circle, Color, EntityType, Handle, Line, LineWeight, LwPolyline,
    Transparency, Vector2, Vector3,
};
use ifccad::ifcdr::{AppearanceId, IfcdrEntityRef, Point2};
use ifccad::package::{load_directory_package, AppearanceProperty, PackageOptions};
use ifccad::PackageId;
use ifccad_convert::{
    cad_document_to_package, ExportAction, ExportDiagnosticSource, ExportError, ExportLossReason,
    ExportOptions, SourceStructureProblem,
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
    assert_eq!(line.start(), Point2::new(1.0, 2.0));
    assert_eq!(line.end(), Point2::new(3.0, 4.0));
    let IfcdrEntityRef::Polyline(polyline) = entities[1] else {
        panic!("second entity must be LWPOLYLINE");
    };
    assert_eq!(
        polyline.points().collect::<Vec<_>>(),
        [
            Point2::new(-2.0, 3.0),
            Point2::new(4.0, -5.0),
            Point2::new(8.0, 1.0),
        ]
    );
    assert!(polyline.closed());
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
            ExportLossReason::NonPlanarZ,
            ExportLossReason::NonZeroThickness,
            ExportLossReason::UnsupportedNormal,
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
            ExportLossReason::NonZeroElevation,
            ExportLossReason::NonZeroThickness,
            ExportLossReason::UnsupportedNormal,
            ExportLossReason::PolylineBulge,
            ExportLossReason::PolylineWidth,
            ExportLossReason::PolylinePlinegen,
        ]
    );
}

#[test]
fn coherent_unsupported_ownership_layers_and_entity_types_are_diagnosed_skips() {
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
    assert!(outcome.entity_mapping().is_empty());
    assert_eq!(outcome.diagnostics().len(), 4);
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
            handle: paper_handle,
            kind: "LINE".to_owned(),
        }
    );
    assert_eq!(
        outcome.diagnostics()[1].reasons(),
        [ExportLossReason::PaperSpaceEntity]
    );
    assert_eq!(
        outcome.diagnostics()[2].source(),
        &ExportDiagnosticSource::Entity {
            handle: block_line_handle,
            kind: "LINE".to_owned(),
        }
    );
    assert_eq!(
        outcome.diagnostics()[2].reasons(),
        [ExportLossReason::BlockOwnedEntity {
            owner: block_handle,
        }]
    );
    assert_eq!(
        outcome.diagnostics()[3].source(),
        &ExportDiagnosticSource::Entity {
            handle: circle_handle,
            kind: "CIRCLE".to_owned(),
        }
    );
    assert_eq!(
        outcome.diagnostics()[3].reasons(),
        [ExportLossReason::UnsupportedEntityType {
            kind: "CIRCLE".to_owned(),
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
