use ifccad::ifcdr::{IfcdrLengthUnit, Point2};
use ifccad::package::{
    AppearanceColor, AppearanceDefinition, AppearanceKey, DrawingBuilder, DrawingOptions,
    EntityAppearance, LayerDefinition, LayerKey, LineDefinition, LinePatternDefinition,
    PackageBuildError, PackageBuilder, PackageOptions, PolylineDefinition,
};
use ifccad::{PackageId, ResourceId};

fn package_options(timestamp: &str) -> PackageOptions {
    PackageOptions {
        package_id: PackageId::new("building-a").unwrap(),
        data_version: "1".to_owned(),
        author: "Example application".to_owned(),
        timestamp: timestamp.to_owned(),
    }
}

fn drawing_options() -> DrawingOptions {
    DrawingOptions {
        model_layout_name: "Model".to_owned(),
        representation_resource_id: ResourceId::new("geometry-modelspace-main").unwrap(),
        length_unit: IfcdrLengthUnit::Millimetre,
    }
}

fn appearance(name: &str) -> AppearanceDefinition {
    AppearanceDefinition {
        name: name.to_owned(),
        color: AppearanceColor::rgb(255, 0, 0),
        opacity: 1.0,
        line_pattern: LinePatternDefinition::named("continuous"),
        line_weight: 0.25,
    }
}

fn add_default_layer(drawing: &mut DrawingBuilder<'_>) -> (LayerKey, AppearanceKey) {
    let style = drawing.appearances().add(appearance("Default")).unwrap();
    let layer = drawing
        .layers()
        .add(LayerDefinition {
            name: "0".to_owned(),
            visible: true,
            appearance: style,
        })
        .unwrap();
    (layer, style)
}

#[test]
fn package_requires_exactly_one_drawing() {
    let empty = PackageBuilder::new(package_options("2026-09-03T10:00:00Z")).unwrap();
    assert!(matches!(
        empty.finish(),
        Err(PackageBuildError::DrawingMissing)
    ));

    let mut package = PackageBuilder::new(package_options("2026-09-03T10:00:00Z")).unwrap();
    package.add_drawing(drawing_options()).unwrap();
    assert!(matches!(
        package.add_drawing(drawing_options()),
        Err(PackageBuildError::DrawingAlreadyDefined)
    ));
}

#[test]
fn drawing_facades_have_contextual_public_types() {
    fn appearances(_: ifccad::package::DrawingAppearances<'_>) {}
    fn layers(_: ifccad::package::DrawingLayers<'_>) {}
    fn model_space(_: ifccad::package::ModelSpaceBuilder<'_>) {}
    let _ = (appearances, layers, model_space);
}

#[test]
fn metadata_accepts_supported_values_without_raw_json() {
    for timestamp in [
        "2026-09-02T10:00:00Z",
        "2026-09-02T10:00:00+00:00",
        "2026-09-02T10:00:00.125+00:00",
    ] {
        let mut package = PackageBuilder::new(package_options(timestamp)).unwrap();
        assert!(package.add_drawing(drawing_options()).is_ok());
    }

    let _point = Point2::new(1.0, 2.0);
    let _appearance = appearance("Wall style");
    let _entity_appearance = EntityAppearance::ByLayer;
    let _line_type: Option<LineDefinition> = None;
    let _polyline_type: Option<PolylineDefinition> = None;
    let _layer_type: Option<LayerDefinition> = None;
}

#[test]
fn metadata_rejects_empty_required_strings() {
    for (field, mutate) in [
        (
            "data_version",
            (|options: &mut PackageOptions| options.data_version.clear())
                as fn(&mut PackageOptions),
        ),
        ("author", |options: &mut PackageOptions| {
            options.author.clear()
        }),
    ] {
        let mut invalid = package_options("2026-09-02T10:00:00Z");
        mutate(&mut invalid);
        assert!(matches!(
            PackageBuilder::new(invalid),
            Err(PackageBuildError::EmptyValue { field: actual }) if actual == field
        ));
    }

    let mut package = PackageBuilder::new(package_options("2026-09-02T10:00:00Z")).unwrap();
    let mut invalid = drawing_options();
    invalid.model_layout_name.clear();
    assert!(matches!(
        package.add_drawing(invalid),
        Err(PackageBuildError::EmptyValue {
            field: "model_layout_name"
        })
    ));
}

#[test]
fn metadata_rejects_non_utc_and_malformed_timestamps() {
    for timestamp in [
        "2026-09-02T10:00:00-00:00",
        "2026-09-02T12:00:00+02:00",
        "2026-02-30T10:00:00Z",
        "not-a-timestamp",
    ] {
        assert!(matches!(
            PackageBuilder::new(package_options(timestamp)),
            Err(PackageBuildError::InvalidTimestamp)
        ));
    }
}

#[test]
fn registries_add_and_find_drawing_scoped_definitions() {
    let mut package = PackageBuilder::new(package_options("2026-09-02T10:00:00Z")).unwrap();
    let mut drawing = package.add_drawing(drawing_options()).unwrap();
    let walls_style = drawing.appearances().add(appearance("Walls")).unwrap();
    let walls = drawing
        .layers()
        .add(LayerDefinition {
            name: "A-WALL".to_owned(),
            visible: true,
            appearance: walls_style,
        })
        .unwrap();

    assert_eq!(drawing.layers().by_name("a-wall"), Some(walls));
    assert_eq!(drawing.layers().by_name("A-WaLl"), Some(walls));
    assert_eq!(drawing.layers().by_name("missing"), None);
}

#[test]
fn registries_reject_duplicates_and_foreign_appearance_keys_without_mutation() {
    let mut first_package = PackageBuilder::new(package_options("2026-09-02T10:00:00Z")).unwrap();
    let mut first = first_package.add_drawing(drawing_options()).unwrap();
    let first_style = first.appearances().add(appearance("First")).unwrap();
    first
        .layers()
        .add(LayerDefinition {
            name: "A-WALL".to_owned(),
            visible: true,
            appearance: first_style,
        })
        .unwrap();

    assert!(matches!(
        first.layers().add(LayerDefinition {
            name: "a-wall".to_owned(),
            visible: false,
            appearance: first_style,
        }),
        Err(PackageBuildError::DuplicateLayerName { name }) if name == "a-wall"
    ));
    assert!(first.layers().by_name("A-WALL").is_some());

    let mut second_package = PackageBuilder::new(package_options("2026-09-02T10:00:00Z")).unwrap();
    let mut second = second_package.add_drawing(drawing_options()).unwrap();
    assert!(matches!(
        second.layers().add(LayerDefinition {
            name: "foreign".to_owned(),
            visible: true,
            appearance: first_style,
        }),
        Err(PackageBuildError::ForeignAppearanceKey)
    ));
    assert_eq!(second.layers().by_name("foreign"), None);
}

#[test]
fn registries_reject_invalid_appearance_and_layer_values() {
    let mut package = PackageBuilder::new(package_options("2026-09-02T10:00:00Z")).unwrap();
    let mut drawing = package.add_drawing(drawing_options()).unwrap();

    for invalid in [
        AppearanceDefinition {
            name: String::new(),
            ..appearance("valid")
        },
        AppearanceDefinition {
            opacity: f64::NAN,
            ..appearance("valid")
        },
        AppearanceDefinition {
            opacity: 1.01,
            ..appearance("valid")
        },
        AppearanceDefinition {
            line_weight: -0.01,
            ..appearance("valid")
        },
        AppearanceDefinition {
            line_weight: f64::INFINITY,
            ..appearance("valid")
        },
        AppearanceDefinition {
            line_pattern: LinePatternDefinition::named(""),
            ..appearance("valid")
        },
        AppearanceDefinition {
            color: AppearanceColor::rgb(1, 2, 3).with_indexed("", 7),
            ..appearance("valid")
        },
        AppearanceDefinition {
            color: AppearanceColor::rgb(1, 2, 3).with_named("RAL", ""),
            ..appearance("valid")
        },
    ] {
        assert!(drawing.appearances().add(invalid).is_err());
    }

    let style = drawing.appearances().add(appearance("valid")).unwrap();
    assert!(matches!(
        drawing.layers().add(LayerDefinition {
            name: String::new(),
            visible: true,
            appearance: style,
        }),
        Err(PackageBuildError::EmptyValue {
            field: "layer_name"
        })
    ));
}

#[test]
fn entities_receive_global_ids_in_mixed_insertion_order() {
    let mut package = PackageBuilder::new(package_options("2026-09-02T10:00:00Z")).unwrap();
    let mut drawing = package.add_drawing(drawing_options()).unwrap();
    let (layer, style) = add_default_layer(&mut drawing);

    let first = drawing
        .model_space()
        .add_line(LineDefinition {
            start: Point2::new(0.0, 0.0),
            end: Point2::new(1.0, 1.0),
            layer,
            appearance: EntityAppearance::ByLayer,
            visible: true,
        })
        .unwrap();
    let second = drawing
        .model_space()
        .add_polyline(PolylineDefinition {
            points: vec![Point2::new(-2.0, 3.0), Point2::new(4.0, -5.0)],
            closed: false,
            layer,
            appearance: EntityAppearance::Explicit(style),
            visible: false,
        })
        .unwrap();
    let third = drawing
        .model_space()
        .add_line(LineDefinition {
            start: Point2::new(2.0, 2.0),
            end: Point2::new(3.0, 3.0),
            layer,
            appearance: EntityAppearance::ByBlock,
            visible: true,
        })
        .unwrap();

    assert_eq!([first.get(), second.get(), third.get()], [1, 2, 3]);
}

#[test]
fn entities_reject_foreign_keys_without_advancing_ids() {
    let mut first_package = PackageBuilder::new(package_options("2026-09-02T10:00:00Z")).unwrap();
    let mut first = first_package.add_drawing(drawing_options()).unwrap();
    let (first_layer, first_style) = add_default_layer(&mut first);

    let mut second_package = PackageBuilder::new(package_options("2026-09-02T10:00:00Z")).unwrap();
    let mut second = second_package.add_drawing(drawing_options()).unwrap();
    let (second_layer, second_style) = add_default_layer(&mut second);

    assert!(matches!(
        second.model_space().add_line(LineDefinition {
            start: Point2::new(0.0, 0.0),
            end: Point2::new(1.0, 1.0),
            layer: first_layer,
            appearance: EntityAppearance::ByLayer,
            visible: true,
        }),
        Err(PackageBuildError::ForeignLayerKey)
    ));
    assert!(matches!(
        second.model_space().add_line(LineDefinition {
            start: Point2::new(0.0, 0.0),
            end: Point2::new(1.0, 1.0),
            layer: second_layer,
            appearance: EntityAppearance::Explicit(first_style),
            visible: true,
        }),
        Err(PackageBuildError::ForeignAppearanceKey)
    ));

    let first_valid = second
        .model_space()
        .add_line(LineDefinition {
            start: Point2::new(0.0, 0.0),
            end: Point2::new(1.0, 1.0),
            layer: second_layer,
            appearance: EntityAppearance::Explicit(second_style),
            visible: true,
        })
        .unwrap();
    assert_eq!(first_valid.get(), 1);
}

#[test]
fn entities_reject_invalid_geometry_without_advancing_ids() {
    let mut package = PackageBuilder::new(package_options("2026-09-02T10:00:00Z")).unwrap();
    let mut drawing = package.add_drawing(drawing_options()).unwrap();
    let (layer, _) = add_default_layer(&mut drawing);

    assert!(matches!(
        drawing.model_space().add_line(LineDefinition {
            start: Point2::new(f64::NAN, 0.0),
            end: Point2::new(1.0, 1.0),
            layer,
            appearance: EntityAppearance::ByLayer,
            visible: true,
        }),
        Err(PackageBuildError::NonFiniteCoordinate)
    ));
    for points in [Vec::new(), vec![Point2::new(0.0, 0.0)]] {
        assert!(matches!(
            drawing.model_space().add_polyline(PolylineDefinition {
                points,
                closed: false,
                layer,
                appearance: EntityAppearance::ByLayer,
                visible: true,
            }),
            Err(PackageBuildError::PolylineTooShort)
        ));
    }
    assert!(matches!(
        drawing.model_space().add_polyline(PolylineDefinition {
            points: vec![Point2::new(0.0, 0.0), Point2::new(f64::INFINITY, 1.0)],
            closed: false,
            layer,
            appearance: EntityAppearance::ByLayer,
            visible: true,
        }),
        Err(PackageBuildError::NonFiniteCoordinate)
    ));

    let first_valid = drawing
        .model_space()
        .add_line(LineDefinition {
            start: Point2::new(0.0, 0.0),
            end: Point2::new(1.0, 1.0),
            layer,
            appearance: EntityAppearance::ByLayer,
            visible: true,
        })
        .unwrap();
    assert_eq!(first_valid.get(), 1);
}

#[test]
fn finish_produces_the_two_logical_package_files() {
    let mut package = PackageBuilder::new(package_options("2026-09-02T10:00:00Z")).unwrap();
    let mut drawing = package.add_drawing(drawing_options()).unwrap();
    add_default_layer(&mut drawing);
    let encoded = package.finish().unwrap();
    let paths = encoded.files().map(|(path, _)| path).collect::<Vec<_>>();

    assert_eq!(
        paths,
        ["package.ifcx.json", "resources/model-space.ifcdr.json"]
    );
    let entrypoint: serde_json::Value =
        serde_json::from_slice(encoded.file("package.ifcx.json").unwrap()).unwrap();
    let resource: serde_json::Value =
        serde_json::from_slice(encoded.file("resources/model-space.ifcdr.json").unwrap()).unwrap();
    assert_eq!(entrypoint["header"]["id"], "building-a");
    assert_eq!(entrypoint["header"]["timestamp"], "2026-09-02T10:00:00Z");
    assert_eq!(resource["header"]["resourceId"], "geometry-modelspace-main");
}
