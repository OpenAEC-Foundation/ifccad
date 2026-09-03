use ifccad::builder::{
    AppearanceColor, AppearanceDefinition, BuildError, EntityAppearance, IfccadPackageBuilder,
    LayerDefinition, LineDefinition, LinePatternDefinition, PackageOptions, PolylineDefinition,
};
use ifccad::ifcdr::{IfccadLengthUnit, Point2};
use ifccad::{PackageId, ResourceId};

fn options(timestamp: &str) -> PackageOptions {
    PackageOptions {
        package_id: PackageId::new("building-a").unwrap(),
        data_version: "1".to_owned(),
        author: "Example application".to_owned(),
        timestamp: timestamp.to_owned(),
        model_layout_name: "Model".to_owned(),
        representation_resource_id: ResourceId::new("geometry-modelspace-main").unwrap(),
        length_unit: IfccadLengthUnit::Millimetre,
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

fn builder_with_layer() -> (
    IfccadPackageBuilder,
    ifccad::builder::LayerKey,
    ifccad::builder::AppearanceKey,
) {
    let mut builder = IfccadPackageBuilder::new(options("2026-09-02T10:00:00Z")).unwrap();
    let style = builder.appearances().add(appearance("Default")).unwrap();
    let layer = builder
        .layers()
        .add(LayerDefinition {
            name: "0".to_owned(),
            visible: true,
            appearance: style,
        })
        .unwrap();
    (builder, layer, style)
}

#[test]
fn metadata_accepts_supported_values_without_raw_json() {
    for timestamp in [
        "2026-09-02T10:00:00Z",
        "2026-09-02T10:00:00+00:00",
        "2026-09-02T10:00:00.125+00:00",
    ] {
        assert!(IfccadPackageBuilder::new(options(timestamp)).is_ok());
    }

    let _point = Point2::new(1.0, 2.0);
    let _appearance = AppearanceDefinition {
        name: "Wall style".to_owned(),
        color: AppearanceColor::rgb(255, 0, 0),
        opacity: 1.0,
        line_pattern: LinePatternDefinition::named("continuous"),
        line_weight: 0.25,
    };
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
        ("model_layout_name", |options: &mut PackageOptions| {
            options.model_layout_name.clear()
        }),
    ] {
        let mut invalid = options("2026-09-02T10:00:00Z");
        mutate(&mut invalid);
        assert!(matches!(
            IfccadPackageBuilder::new(invalid),
            Err(BuildError::EmptyValue { field: actual }) if actual == field
        ));
    }
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
            IfccadPackageBuilder::new(options(timestamp)),
            Err(BuildError::InvalidTimestamp)
        ));
    }
}

#[test]
fn registries_add_and_find_builder_scoped_definitions() {
    let mut builder = IfccadPackageBuilder::new(options("2026-09-02T10:00:00Z")).unwrap();
    let walls_style = builder.appearances().add(appearance("Walls")).unwrap();
    let walls = builder
        .layers()
        .add(LayerDefinition {
            name: "A-WALL".to_owned(),
            visible: true,
            appearance: walls_style,
        })
        .unwrap();

    assert_eq!(builder.layers().by_name("a-wall"), Some(walls));
    assert_eq!(builder.layers().by_name("A-WaLl"), Some(walls));
    assert_eq!(builder.layers().by_name("missing"), None);
}

#[test]
fn registries_reject_duplicates_and_foreign_appearance_keys_without_mutation() {
    let mut first = IfccadPackageBuilder::new(options("2026-09-02T10:00:00Z")).unwrap();
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
        Err(BuildError::DuplicateLayerName { name }) if name == "a-wall"
    ));
    assert_eq!(first.layers().by_name("A-WALL").is_some(), true);

    let mut second = IfccadPackageBuilder::new(options("2026-09-02T10:00:00Z")).unwrap();
    assert!(matches!(
        second.layers().add(LayerDefinition {
            name: "foreign".to_owned(),
            visible: true,
            appearance: first_style,
        }),
        Err(BuildError::ForeignAppearanceKey)
    ));
    assert_eq!(second.layers().by_name("foreign"), None);
}

#[test]
fn registries_reject_invalid_appearance_and_layer_values() {
    let mut builder = IfccadPackageBuilder::new(options("2026-09-02T10:00:00Z")).unwrap();

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
        assert!(builder.appearances().add(invalid).is_err());
    }

    let style = builder.appearances().add(appearance("valid")).unwrap();
    assert!(matches!(
        builder.layers().add(LayerDefinition {
            name: String::new(),
            visible: true,
            appearance: style,
        }),
        Err(BuildError::EmptyValue {
            field: "layer_name"
        })
    ));
}

#[test]
fn entities_receive_global_ids_in_mixed_insertion_order() {
    let (mut builder, layer, style) = builder_with_layer();

    let first = builder
        .model_space()
        .add_line(LineDefinition {
            start: Point2::new(0.0, 0.0),
            end: Point2::new(1.0, 1.0),
            layer,
            appearance: EntityAppearance::ByLayer,
            visible: true,
        })
        .unwrap();
    let second = builder
        .model_space()
        .add_polyline(PolylineDefinition {
            points: vec![Point2::new(-2.0, 3.0), Point2::new(4.0, -5.0)],
            closed: false,
            layer,
            appearance: EntityAppearance::Explicit(style),
            visible: false,
        })
        .unwrap();
    let third = builder
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
    let (mut first, first_layer, first_style) = builder_with_layer();
    let (mut second, second_layer, second_style) = builder_with_layer();

    assert!(matches!(
        second.model_space().add_line(LineDefinition {
            start: Point2::new(0.0, 0.0),
            end: Point2::new(1.0, 1.0),
            layer: first_layer,
            appearance: EntityAppearance::ByLayer,
            visible: true,
        }),
        Err(BuildError::ForeignLayerKey)
    ));
    assert!(matches!(
        second.model_space().add_line(LineDefinition {
            start: Point2::new(0.0, 0.0),
            end: Point2::new(1.0, 1.0),
            layer: second_layer,
            appearance: EntityAppearance::Explicit(first_style),
            visible: true,
        }),
        Err(BuildError::ForeignAppearanceKey)
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

    let _ = first.model_space();
}

#[test]
fn entities_reject_invalid_geometry_without_advancing_ids() {
    let (mut builder, layer, _) = builder_with_layer();

    assert!(matches!(
        builder.model_space().add_line(LineDefinition {
            start: Point2::new(f64::NAN, 0.0),
            end: Point2::new(1.0, 1.0),
            layer,
            appearance: EntityAppearance::ByLayer,
            visible: true,
        }),
        Err(BuildError::NonFiniteCoordinate)
    ));
    for points in [Vec::new(), vec![Point2::new(0.0, 0.0)]] {
        assert!(matches!(
            builder.model_space().add_polyline(PolylineDefinition {
                points,
                closed: false,
                layer,
                appearance: EntityAppearance::ByLayer,
                visible: true,
            }),
            Err(BuildError::PolylineTooShort)
        ));
    }
    assert!(matches!(
        builder.model_space().add_polyline(PolylineDefinition {
            points: vec![Point2::new(0.0, 0.0), Point2::new(f64::INFINITY, 1.0)],
            closed: false,
            layer,
            appearance: EntityAppearance::ByLayer,
            visible: true,
        }),
        Err(BuildError::NonFiniteCoordinate)
    ));

    let first_valid = builder
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
    let (builder, _, _) = builder_with_layer();
    let encoded = builder.finish().unwrap();
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
