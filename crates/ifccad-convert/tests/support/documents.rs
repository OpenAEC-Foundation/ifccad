#![allow(dead_code)]

use ifccad::conformance::bundled_conformance_root;
use ifccad::package::{load_directory_package, PackageOptions};
use ifccad::PackageId;
use ifccad_convert::cadcodec::{
    CadDocument, Circle, Color, DxfReader, DxfWriter, EntityType, Layer, Line, LineWeight,
    LwPolyline, Transparency, Vector2,
};
use ifccad_convert::{
    cad_document_to_package, drawing_to_cad_document, ExportAction, ExportDiagnostic, ExportError,
    ExportLossPolicy, ExportLossReason, ExportOptions,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

pub fn supported_model_space_document() -> CadDocument {
    let mut document = CadDocument::new();
    document.header.insertion_units = 4;
    document.header.measurement = 1;

    let layer_0 = document.layers.get_mut("0").expect("default layer");
    layer_0.color = Color::Index(7);
    layer_0.line_weight = LineWeight::W0_25;

    let mut geometry = Layer::with_color("Geometry", Color::Index(3));
    geometry.line_weight = LineWeight::W0_18;
    document.layers.add(geometry).expect("add Geometry layer");
    document
        .layers
        .add(Layer::with_color("Empty", Color::from_rgb(10, 20, 30)))
        .expect("add empty layer");

    document
        .add_entity(EntityType::Line(Line::from_coords(
            0.0, 0.0, 0.0, 20.0, 10.0, 0.0,
        )))
        .expect("add supported line");
    let mut polyline = LwPolyline::from_points(vec![
        Vector2::new(2.0, 3.0),
        Vector2::new(8.0, 3.0),
        Vector2::new(8.0, 9.0),
    ]);
    polyline.is_closed = true;
    polyline.common.layer = "Geometry".to_owned();
    polyline.common.color = Color::Index(1);
    polyline.common.line_weight = LineWeight::W0_18;
    polyline.common.transparency = Transparency::OPAQUE;
    document
        .add_entity(EntityType::LwPolyline(polyline))
        .expect("add supported polyline");
    document
}

pub fn loss_heavy_document() -> CadDocument {
    let mut document = supported_model_space_document();
    document
        .add_entity(EntityType::Circle(Circle::new()))
        .expect("add unsupported circle");
    let mut nonplanar = Line::from_coords(0.0, 0.0, 2.0, 1.0, 1.0, 3.0);
    nonplanar.thickness = 1.0;
    document
        .add_entity(EntityType::Line(nonplanar))
        .expect("add lossy line");
    document
}

pub fn assert_export_chain(document: CadDocument, expect_loss: bool) {
    let root = TempRoot::new("chain");
    let source_dxf = root.path().join("source.dxf");
    DxfWriter::new(&document)
        .write_to_file(&source_dxf)
        .expect("write source DXF");
    let source = read_dxf(&source_dxf);

    let allowed = cad_document_to_package(
        &source,
        package_options("chain"),
        ExportOptions {
            loss_policy: ExportLossPolicy::Allow,
        },
    )
    .expect("allow export");

    if expect_loss {
        assert_loss_heavy_diagnostics(allowed.diagnostics());
        let rejected = cad_document_to_package(
            &source,
            package_options("chain"),
            ExportOptions {
                loss_policy: ExportLossPolicy::Reject,
            },
        )
        .err()
        .expect("reject policy must withhold the package");
        assert!(matches!(
            rejected,
            ExportError::LossRejected { diagnostics }
                if diagnostics == allowed.diagnostics()
        ));
    } else {
        assert!(
            allowed.diagnostics().is_empty(),
            "unexpected diagnostics: {:#?}",
            allowed.diagnostics()
        );
    }

    let second =
        cad_document_to_package(&source, package_options("chain"), ExportOptions::default())
            .expect("repeat deterministic export");
    assert_eq!(second.diagnostics(), allowed.diagnostics());
    assert_eq!(second.entity_mapping(), allowed.entity_mapping());
    assert_eq!(
        second.package().files().collect::<Vec<_>>(),
        allowed.package().files().collect::<Vec<_>>()
    );

    let package_root = root.path().join("ifccad");
    allowed
        .package()
        .write_directory(&package_root)
        .expect("write IFCCAD package");
    let inspected = load_directory_package(&package_root).expect("strict-load exported package");
    assert!(inspected.report().is_empty(), "{:#?}", inspected.report());
    let package = inspected
        .validated_package()
        .expect("valid exported package");
    let drawing = package.drawings().next().expect("one exported drawing");
    let imported = drawing_to_cad_document(drawing).expect("import exported drawing");
    let roundtrip_dxf = root.path().join("roundtrip.dxf");
    DxfWriter::new(imported.document())
        .write_to_file(&roundtrip_dxf)
        .expect("write roundtrip DXF");
    let final_document = read_dxf(&roundtrip_dxf);

    let expected = if expect_loss {
        supported_projection(&source)
    } else {
        semantic_projection(&source)
    };
    assert_eq!(semantic_projection(&final_document), expected);
}

pub fn assert_minimal_ifccad_chain() {
    let initial_root = bundled_conformance_root()
        .join("packages")
        .join("valid")
        .join("minimal-no-preservation");
    let initial_inspected =
        load_directory_package(initial_root).expect("load bundled IFCCAD package");
    assert!(
        initial_inspected.report().is_empty(),
        "{:#?}",
        initial_inspected.report()
    );
    let initial_package = initial_inspected
        .validated_package()
        .expect("valid bundled package");
    let initial_drawing = initial_package.drawings().next().expect("one drawing");
    let initial_document = drawing_to_cad_document(initial_drawing)
        .expect("import bundled drawing")
        .into_document();
    let initial_projection = semantic_projection(&initial_document);

    let root = TempRoot::new("ifccad-origin");
    let dxf_path = root.path().join("from-ifccad.dxf");
    DxfWriter::new(&initial_document)
        .write_to_file(&dxf_path)
        .expect("write intermediate DXF");
    let reloaded = read_dxf(&dxf_path);
    let exported = cad_document_to_package(
        &reloaded,
        package_options("ifccad-origin"),
        ExportOptions::default(),
    )
    .expect("export reloaded DXF");
    assert!(
        exported.diagnostics().is_empty(),
        "unexpected diagnostics: {:#?}",
        exported.diagnostics()
    );
    let final_root = root.path().join("final-ifccad");
    exported
        .package()
        .write_directory(&final_root)
        .expect("write final IFCCAD package");
    let final_inspected =
        load_directory_package(final_root).expect("strict-load final IFCCAD package");
    assert!(
        final_inspected.report().is_empty(),
        "{:#?}",
        final_inspected.report()
    );
    let final_package = final_inspected
        .validated_package()
        .expect("valid final package");
    let final_drawing = final_package.drawings().next().expect("one final drawing");
    let final_document = drawing_to_cad_document(final_drawing)
        .expect("import final drawing")
        .into_document();
    assert_eq!(semantic_projection(&final_document), initial_projection);
}

fn package_options(label: &str) -> PackageOptions {
    PackageOptions {
        package_id: PackageId::new(format!("export-chain-{label}")).unwrap(),
        data_version: "1".to_owned(),
        author: "IFCCAD chain tests".to_owned(),
        timestamp: "2026-09-04T10:00:00Z".to_owned(),
    }
}

fn read_dxf(path: &Path) -> CadDocument {
    DxfReader::from_file(path)
        .expect("open DXF")
        .read()
        .expect("read DXF")
}

fn assert_loss_heavy_diagnostics(diagnostics: &[ExportDiagnostic]) {
    assert_eq!(diagnostics.len(), 2, "{diagnostics:#?}");
    assert_eq!(diagnostics[0].action(), ExportAction::Skipped);
    assert_eq!(
        diagnostics[0].reasons(),
        [ExportLossReason::UnsupportedEntityType {
            kind: "CIRCLE".to_owned(),
        }]
    );
    assert_eq!(diagnostics[1].action(), ExportAction::Skipped);
    assert_eq!(
        diagnostics[1].reasons(),
        [
            ExportLossReason::NonPlanarZ,
            ExportLossReason::NonZeroThickness,
        ]
    );
}

#[derive(Debug, PartialEq)]
struct DocumentProjection {
    units: i16,
    layers: Vec<LayerProjection>,
    entities: Vec<EntityProjection>,
}

#[derive(Debug, PartialEq)]
struct LayerProjection {
    name: String,
    visible: bool,
    color: Color,
    line_type: String,
    line_weight: i16,
    transparency: Transparency,
}

#[derive(Debug, PartialEq)]
struct EntityProjection {
    geometry: GeometryProjection,
    layer: String,
    invisible: bool,
    color: Color,
    linetype: String,
    line_weight: i16,
    transparency: Transparency,
}

#[derive(Debug, PartialEq)]
enum GeometryProjection {
    Line { start: [f64; 2], end: [f64; 2] },
    Polyline { points: Vec<[f64; 2]>, closed: bool },
}

fn semantic_projection(document: &CadDocument) -> DocumentProjection {
    DocumentProjection {
        units: document.header.insertion_units,
        layers: document
            .layers
            .iter()
            .map(|layer| LayerProjection {
                name: layer.name.clone(),
                visible: layer.is_visible(),
                color: layer.color,
                line_type: layer.line_type.clone(),
                line_weight: effective_line_weight(layer.line_weight),
                transparency: layer.transparency,
            })
            .collect(),
        entities: supported_entities(document),
    }
}

fn supported_projection(document: &CadDocument) -> DocumentProjection {
    semantic_projection(document)
}

fn supported_entities(document: &CadDocument) -> Vec<EntityProjection> {
    document
        .entities()
        .filter_map(|entity| {
            let common = entity.common();
            let geometry = match entity {
                EntityType::Line(line)
                    if line.start.z == 0.0 && line.end.z == 0.0 && line.thickness == 0.0 =>
                {
                    GeometryProjection::Line {
                        start: [line.start.x, line.start.y],
                        end: [line.end.x, line.end.y],
                    }
                }
                EntityType::LwPolyline(polyline)
                    if polyline.elevation == 0.0
                        && polyline.thickness == 0.0
                        && polyline.vertices.iter().all(|vertex| {
                            vertex.start_width == 0.0
                                && vertex.end_width == 0.0
                                && vertex.bulge == 0.0
                        }) =>
                {
                    GeometryProjection::Polyline {
                        points: polyline
                            .vertices
                            .iter()
                            .map(|vertex| [vertex.location.x, vertex.location.y])
                            .collect(),
                        closed: polyline.is_closed,
                    }
                }
                _ => return None,
            };
            Some(EntityProjection {
                geometry,
                layer: common.layer.clone(),
                invisible: common.invisible,
                color: common.color,
                linetype: normalize_linetype(&common.linetype),
                line_weight: effective_line_weight(common.line_weight),
                transparency: common.transparency,
            })
        })
        .collect()
}

fn normalize_linetype(value: &str) -> String {
    if value.is_empty() {
        "ByLayer".to_owned()
    } else {
        value.to_owned()
    }
}

fn effective_line_weight(value: LineWeight) -> i16 {
    match value {
        LineWeight::Default => 25,
        other => other.value(),
    }
}

struct TempRoot(PathBuf);

impl TempRoot {
    fn new(label: &str) -> Self {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("ifccad-export-{label}-{}-{id}", std::process::id()));
        fs::create_dir_all(&path).expect("create temporary test root");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
