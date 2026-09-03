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
