use ifccad::builder::{
    AppearanceColor, AppearanceDefinition, EntityAppearance, IfccadPackageBuilder, LayerDefinition,
    LineDefinition, LinePatternDefinition, PackageOptions, PolylineDefinition,
};
use ifccad::ifcdr::{AppearanceId, IfcdrEntityRef, IfcdrLengthUnit, Point2};
use ifccad::package::{
    load_directory_package, AppearanceProperty, DrawingLayoutKind, LinePatternRef,
};
use ifccad::{PackageId, ResourceId};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

struct TempRoot(PathBuf);

impl TempRoot {
    fn new() -> Self {
        let nonce = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ifccad-writer-roundtrip-{}-{nonce}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn representative_builder() -> IfccadPackageBuilder {
    let mut builder = IfccadPackageBuilder::new(PackageOptions {
        package_id: PackageId::new("roundtrip-package").unwrap(),
        data_version: "42".to_owned(),
        author: "Writer integration test".to_owned(),
        timestamp: "2026-09-03T14:15:16.125+00:00".to_owned(),
        model_layout_name: "Model layout".to_owned(),
        representation_resource_id: ResourceId::new("geometry-main").unwrap(),
        length_unit: IfcdrLengthUnit::Millimetre,
    })
    .unwrap();
    let solid = builder
        .appearances()
        .add(AppearanceDefinition {
            name: "Solid black".to_owned(),
            color: AppearanceColor::rgb(0, 0, 0).with_indexed("ACI", 7),
            opacity: 1.0,
            line_pattern: LinePatternDefinition::named("continuous"),
            line_weight: 0.25,
        })
        .unwrap();
    let dashed = builder
        .appearances()
        .add(AppearanceDefinition {
            name: "Dashed red".to_owned(),
            color: AppearanceColor::rgb(255, 0, 0).with_named("RAL", "Traffic red"),
            opacity: 0.5,
            line_pattern: LinePatternDefinition::named("dashed"),
            line_weight: 0.18,
        })
        .unwrap();
    let layer_0 = builder
        .layers()
        .add(LayerDefinition {
            name: "0".to_owned(),
            visible: true,
            appearance: solid,
        })
        .unwrap();
    let walls = builder
        .layers()
        .add(LayerDefinition {
            name: "A-WALL".to_owned(),
            visible: false,
            appearance: dashed,
        })
        .unwrap();

    builder
        .model_space()
        .add_line(LineDefinition {
            start: Point2::new(0.0, 0.0),
            end: Point2::new(10.0, 5.0),
            layer: layer_0,
            appearance: EntityAppearance::ByLayer,
            visible: true,
        })
        .unwrap();
    builder
        .model_space()
        .add_polyline(PolylineDefinition {
            points: vec![Point2::new(-2.0, 3.0), Point2::new(4.0, -5.0)],
            closed: false,
            layer: walls,
            appearance: EntityAppearance::Explicit(solid),
            visible: false,
        })
        .unwrap();
    builder
        .model_space()
        .add_line(LineDefinition {
            start: Point2::new(1.0, 2.0),
            end: Point2::new(3.0, 4.0),
            layer: walls,
            appearance: EntityAppearance::ByBlock,
            visible: false,
        })
        .unwrap();
    builder
        .model_space()
        .add_polyline(PolylineDefinition {
            points: vec![
                Point2::new(2.0, 2.0),
                Point2::new(8.0, 8.0),
                Point2::new(5.0, -1.0),
            ],
            closed: true,
            layer: layer_0,
            appearance: EntityAppearance::Explicit(dashed),
            visible: true,
        })
        .unwrap();
    builder
}

#[test]
fn writer_output_reloads_without_diagnostics_and_preserves_semantics() {
    let root = TempRoot::new();
    let target = root.0.join("project");
    representative_builder()
        .finish()
        .unwrap()
        .write_directory(&target)
        .unwrap();

    let loaded = load_directory_package(&target).unwrap();
    assert!(loaded.report().is_empty(), "{:#?}", loaded.report());
    let package = loaded.validated_package().expect("strict writer output");
    assert_eq!(package.header().package_id().as_str(), "roundtrip-package");
    assert_eq!(package.header().data_version(), "42");
    assert_eq!(package.header().author(), "Writer integration test");
    assert_eq!(package.header().timestamp(), "2026-09-03T14:15:16.125Z");

    let drawing_set = package.drawing_sets().next().expect("drawing set");
    assert_eq!(package.drawing_sets().count(), 1);
    assert_eq!(drawing_set.path(), "drawing-set-0");
    let drawing = drawing_set.drawings().next().expect("drawing");
    assert_eq!(drawing.path(), "drawing-0");
    let layout = drawing.layouts().next().expect("layout");
    assert_eq!(drawing.layouts().count(), 1);
    assert_eq!(layout.path(), "layout-0");
    assert_eq!(layout.name(), "Model layout");
    assert_eq!(layout.kind(), DrawingLayoutKind::Model);
    assert_eq!(layout.scope().id().get(), 0);
    assert_eq!(layout.scope().name(), "ModelSpace");

    let representation = drawing.representation();
    assert_eq!(representation.path(), "representation-0");
    assert_eq!(representation.role(), "modelspace");
    assert_eq!(representation.resource_id().as_str(), "geometry-main");
    assert_eq!(
        representation.external_uri(),
        Some("resources/model-space.ifcdr.json")
    );
    assert_eq!(
        representation.resource().unit(),
        IfcdrLengthUnit::Millimetre
    );
    let bounds = representation.resource().bounds();
    assert_eq!(bounds.min(), Point2::new(-2.0, -5.0));
    assert_eq!(bounds.max(), Point2::new(10.0, 8.0));

    let layers = representation.layers().collect::<Vec<_>>();
    assert_eq!(layers.len(), 2);
    assert_eq!(
        (layers[0].id().get(), layers[0].name(), layers[0].visible()),
        (0, "0", true)
    );
    assert_eq!(
        (layers[1].id().get(), layers[1].name(), layers[1].visible()),
        (1, "A-WALL", false)
    );
    assert_eq!(layers[0].appearance().unwrap().name(), "Solid black");
    assert_eq!(layers[1].appearance().unwrap().name(), "Dashed red");

    let by_layer = representation.appearance(AppearanceId::from(0)).unwrap();
    assert!(matches!(by_layer.color(), AppearanceProperty::ByLayer));
    assert!(matches!(by_layer.opacity(), AppearanceProperty::ByLayer));
    let by_block = representation.appearance(AppearanceId::from(1)).unwrap();
    assert!(matches!(by_block.color(), AppearanceProperty::ByBlock));
    let solid = representation.appearance(AppearanceId::from(2)).unwrap();
    assert_eq!(solid.ifcx_definition().unwrap().name(), "Solid black");
    assert!(matches!(solid.opacity(), AppearanceProperty::Explicit(1.0)));
    assert!(matches!(
        solid.line_pattern(),
        AppearanceProperty::Explicit(LinePatternRef::Name("continuous"))
    ));
    let dashed = representation.appearance(AppearanceId::from(3)).unwrap();
    assert!(matches!(
        dashed.opacity(),
        AppearanceProperty::Explicit(0.5)
    ));

    let resource = representation.resource();
    let entities = resource.entities(layout.scope().id()).collect::<Vec<_>>();
    assert_eq!(entities.len(), 4);
    let IfcdrEntityRef::Line(first) = &entities[0] else {
        panic!("entity 1 must be a line");
    };
    assert_eq!(first.entity_id().get(), 1);
    assert_eq!(first.start(), Point2::new(0.0, 0.0));
    assert_eq!(first.end(), Point2::new(10.0, 5.0));
    assert_eq!(first.appearance_id().get(), 0);
    assert!(first.visible());
    let IfcdrEntityRef::Polyline(second) = &entities[1] else {
        panic!("entity 2 must be a polyline");
    };
    assert_eq!(second.entity_id().get(), 2);
    assert_eq!(
        second.points().collect::<Vec<_>>(),
        [Point2::new(-2.0, 3.0), Point2::new(4.0, -5.0)]
    );
    assert!(!second.closed());
    assert!(!second.visible());
    assert_eq!(second.appearance_id().get(), 2);
    let IfcdrEntityRef::Line(third) = &entities[2] else {
        panic!("entity 3 must be a line");
    };
    assert_eq!(third.entity_id().get(), 3);
    assert_eq!(third.appearance_id().get(), 1);
    assert!(!third.visible());
    let IfcdrEntityRef::Polyline(fourth) = &entities[3] else {
        panic!("entity 4 must be a polyline");
    };
    assert_eq!(fourth.entity_id().get(), 4);
    assert!(fourth.closed());
    assert!(fourth.visible());
    assert_eq!(fourth.appearance_id().get(), 3);
}

#[test]
fn identical_input_is_byte_deterministic_across_builder_tokens() {
    let first = representative_builder().finish().unwrap();
    let second = representative_builder().finish().unwrap();

    let first_files = first
        .files()
        .map(|(path, bytes)| (path.to_owned(), bytes.to_vec()))
        .collect::<Vec<_>>();
    let second_files = second
        .files()
        .map(|(path, bytes)| (path.to_owned(), bytes.to_vec()))
        .collect::<Vec<_>>();
    assert_eq!(first_files, second_files);
    assert_eq!(first_files.len(), 2);
}

#[test]
fn empty_model_space_reloads_with_zero_bounds_and_no_entities() {
    let root = TempRoot::new();
    let target = root.0.join("empty-project");
    IfccadPackageBuilder::new(PackageOptions {
        package_id: PackageId::new("empty-package").unwrap(),
        data_version: "1".to_owned(),
        author: "Writer integration test".to_owned(),
        timestamp: "2026-09-03T14:15:16Z".to_owned(),
        model_layout_name: "Model".to_owned(),
        representation_resource_id: ResourceId::new("empty-geometry").unwrap(),
        length_unit: IfcdrLengthUnit::Unitless,
    })
    .unwrap()
    .finish()
    .unwrap()
    .write_directory(&target)
    .unwrap();

    let loaded = load_directory_package(&target).unwrap();
    assert!(loaded.report().is_empty(), "{:#?}", loaded.report());
    let package = loaded.validated_package().expect("strict empty package");
    let drawing = package.drawings().next().unwrap();
    let layout = drawing.layouts().next().unwrap();
    let resource = drawing.representation().resource();
    assert_eq!(resource.bounds().min(), Point2::new(0.0, 0.0));
    assert_eq!(resource.bounds().max(), Point2::new(0.0, 0.0));
    assert_eq!(resource.entities(layout.scope().id()).count(), 0);
}
