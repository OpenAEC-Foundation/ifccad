use ocdraw::dimension::*;
use std::collections::BTreeMap;

fn aligned(first: [f64; 2], second: [f64; 2]) -> DimensionDefinition {
    DimensionDefinition::Aligned {
        first_point: first,
        second_point: second,
        dimension_line_point: [0.0, 300.0],
        extension_direction_angle: None,
    }
}

#[test]
fn linear_projects_and_aligned_measures_distance() {
    let linear = DimensionDefinition::Linear {
        first_point: [0.0, 0.0],
        second_point: [3.0, 4.0],
        direction_angle: 0.0,
        dimension_line_point: [0.0, 300.0],
        extension_direction_angle: None,
    };
    assert_eq!(
        evaluate_dimension(&linear).unwrap(),
        DimensionMeasurement::Distance(3.0)
    );
    assert_eq!(
        evaluate_dimension(&aligned([0.0, 0.0], [3.0, 4.0])).unwrap(),
        DimensionMeasurement::Distance(5.0)
    );
    assert_eq!(
        evaluate_dimension(&aligned([2.0, 2.0], [2.0, 2.0])).unwrap(),
        DimensionMeasurement::Distance(0.0)
    );
}

#[test]
fn same_as_base_override_keeps_presence_and_leaf_resets() {
    let mut base: DimensionStyleSettings<u32, u32, [u8; 3], u32> = DimensionStyleSettings::new(7);
    assert_eq!(base.text.height, 2.5);
    let mut local = DimensionStyleOverrides::default();
    local.text.height = Some(2.5);
    local.arrows.flip_first = Some(false);
    local.arrows.tick_size = Some(0.0);
    base.text.height = 3.5;
    base.arrows.flip_first = true;
    base.arrows.tick_size = 1.0;
    let effective =
        resolve_dimension_settings(&base, DimensionKind::Linear, &BTreeMap::new(), &local);
    assert_eq!(effective.text.height, 2.5);
    assert!(!effective.arrows.flip_first);
    assert_eq!(effective.arrows.tick_size, 0.0);
    assert_eq!(local.text.height, Some(2.5));
    local.text.height = None;
    assert_eq!(
        resolve_dimension_settings(&base, DimensionKind::Linear, &BTreeMap::new(), &local)
            .text
            .height,
        3.5
    );
}

#[test]
fn type_settings_precede_entity_overrides_without_changing_other_leaves() {
    let base: DimensionStyleSettings<u32, u32, [u8; 3], u32> = DimensionStyleSettings::new(7);
    let mut radial = DimensionStyleOverrides::default();
    radial.text.height = Some(8.0);
    let mut linear = DimensionStyleOverrides::default();
    linear.text.height = Some(4.0);
    let types = BTreeMap::from([
        (DimensionKind::Radius, radial),
        (DimensionKind::Linear, linear),
    ]);
    let mut local = DimensionStyleOverrides::default();
    local.text.gap = Some(0.0);
    let effective = resolve_dimension_settings(&base, DimensionKind::Linear, &types, &local);
    assert_eq!(effective.text.height, 4.0);
    assert_eq!(effective.text.gap, 0.0);
    assert_eq!(effective.arrows.size, 2.5);
}

#[test]
fn nonfinite_input_and_unrepresentable_measurement_are_errors() {
    let error = validate_dimension_definition(&aligned([f64::NAN, 0.0], [1.0, 0.0])).unwrap_err();
    assert!(error.field.contains("firstPoint"));
    assert!(evaluate_dimension(&aligned([-f64::MAX, 0.0], [f64::MAX, 0.0])).is_err());
    let mut base: DimensionStyleSettings<u32, u32, [u8; 3], u32> = DimensionStyleSettings::new(7);
    base.text.height = 0.0;
    assert!(validate_dimension_settings(&base).is_err());
    base.text.height = 2.5;
    base.presentation_scale = f64::INFINITY;
    assert!(validate_dimension_settings(&base).is_err());
}

#[test]
fn stored_defaults_are_coordinate_values_and_have_no_font_lookup() {
    let base: DimensionStyleSettings<u32, u32, [u8; 3], u32> = DimensionStyleSettings::new(7);
    assert_eq!(base.text.style, 7);
    assert_eq!(base.presentation_scale, 1.0);
    assert_eq!(base.text.gap, 0.625);
    assert_eq!(base.extension_lines.first.offset, 0.625);
    assert_eq!(base.extension_lines.first.extension, 1.25);
    validate_dimension_settings(&base).unwrap();
}

#[test]
fn core_definition_tokens_are_closed_and_null_is_not_an_omitted_choice() {
    let mut value = serde_json::to_value(aligned([0.0, 0.0], [3.0, 4.0])).unwrap();
    value["unknown"] = serde_json::json!(1);
    assert!(serde_json::from_value::<DimensionDefinition>(value).is_err());
    let mut value = serde_json::to_value(aligned([0.0, 0.0], [3.0, 4.0])).unwrap();
    value["extensionDirectionAngle"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<DimensionDefinition>(value).is_err());
}

#[test]
fn references_can_be_rebound_without_losing_settings_or_override_presence() {
    let mut base: DimensionStyleSettings<u32, u32, [u8; 3], u32> = DimensionStyleSettings::new(7);
    base.arrows.first = DimensionMarker::LocalBlock { block: 3 };
    base.text.height = 8.0;
    base.text.appearance.color = DimensionAppearanceChoice::Explicit([1, 2, 3]);
    let rebound = base.map_references(
        |s| s.to_string(),
        |b| b.to_string(),
        |c| format!("{:?}", c),
        |p| p.to_string(),
    );
    assert_eq!(rebound.text.style, "7");
    assert_eq!(rebound.text.height, 8.0);
    assert_eq!(
        rebound.arrows.first,
        DimensionMarker::LocalBlock { block: "3".into() }
    );
    assert_eq!(
        rebound.text.appearance.color,
        DimensionAppearanceChoice::Explicit("[1, 2, 3]".into())
    );
    let mut local: DimensionStyleOverrides<u32, u32, [u8; 3], u32> = Default::default();
    local.text.height = Some(2.5);
    local.arrows.flip_first = Some(false);
    local.text.style = Some(7);
    let rebound = local.map_references(
        |s| s.to_string(),
        |b| b.to_string(),
        |c| format!("{:?}", c),
        |p| p.to_string(),
    );
    assert_eq!(rebound.text.height, Some(2.5));
    assert_eq!(rebound.arrows.flip_first, Some(false));
    assert_eq!(rebound.text.style.as_deref(), Some("7"));
}

#[test]
fn component_appearance_override_is_sparse_at_the_property_level() {
    let mut base: DimensionStyleSettings<u32, u32, [u8; 3], u32> = DimensionStyleSettings::new(7);
    base.text.appearance.opacity = DimensionAppearanceChoice::Explicit(0.5);
    let local: DimensionStyleOverrides<u32, u32, [u8; 3], u32> =
        serde_json::from_value(serde_json::json!({
            "text": {"appearance": {"color": {"mode": "explicit", "value": [1, 2, 3]}}}
        }))
        .unwrap();
    let resolved =
        resolve_dimension_settings(&base, DimensionKind::Linear, &BTreeMap::new(), &local);
    assert_eq!(
        resolved.text.appearance.opacity,
        DimensionAppearanceChoice::Explicit(0.5)
    );
    assert_eq!(
        resolved.text.appearance.color,
        DimensionAppearanceChoice::Explicit([1, 2, 3])
    );
}

#[test]
fn literal_label_runs_use_existing_false_decoration_defaults() {
    let label:DimensionLabel=serde_json::from_value(serde_json::json!({"kind":"template","parts":[{"kind":"literal","runs":[{"text":"<> literal"}]}]})).unwrap();
    let DimensionLabel::Template(parts) = label else {
        panic!("template")
    };
    let DimensionLabelPart::Literal(runs) = &parts[0] else {
        panic!("literal")
    };
    assert_eq!(runs[0].text, "<> literal");
    assert!(!runs[0].underline);
    assert!(!runs[0].overline);
    assert!(!runs[0].strike_through);
}
