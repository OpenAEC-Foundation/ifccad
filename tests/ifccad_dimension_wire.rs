#[path = "support/ifccad_dimension.rs"]
mod support;
use ifccad::{dimension::*, *};

#[test]
fn spatial_reference_anchors_and_signed_zero_survive_both_strict_readers() {
    let mut d = support::drawing();
    let IfccadEntityKind::Dimension(dim) = &mut d.model.entities[0].as_native_mut().unwrap().kind
    else {
        panic!("dimension")
    };
    dim.placement.origin = [-17., 10., 30.];
    dim.placement.x_axis = [0., 1., 0.];
    dim.placement.y_axis = [0., 0., 1.];
    dim.definition = DimensionDefinition::Aligned {
        first_point: [0., 0., -0.],
        second_point: [3., 4., 1632.],
        dimension_line_point: [0., 8.],
        extension_direction_angle: None,
    };
    validate_ifccad_document(&d).unwrap();
    let frame = dim_frame(&d);
    assert_eq!(
        frame
            .try_to_scope_spatial_point(ifccad::geometry_kernel::Point3::new(3., 4., 1632.))
            .unwrap()
            .components(),
        [1615., 13., 34.]
    );
    let json = encode_ifccad_document(&d).unwrap();
    let cbor = encode_ifccad_cbor_document(&d).unwrap();
    let reads = [
        load_ifccad_bytes(json.bytes(), Default::default())
            .unwrap()
            .into_document(),
        load_ifccad_cbor_bytes(cbor.bytes(), Default::default())
            .unwrap()
            .into_document(),
    ];
    for read in reads {
        validate_ifccad_document(&read).unwrap();
        let IfccadEntityKind::Dimension(dim) = &read.model.entities[0].as_native().unwrap().kind
        else {
            panic!("dimension")
        };
        let (a, b) = dim.definition.reference_points();
        assert_eq!(a[2].to_bits(), (-0.0_f64).to_bits());
        assert_eq!(b, [3., 4., 1632.]);
        assert_eq!(
            evaluate_dimension(&dim.definition).unwrap(),
            DimensionMeasurement::Distance(5.)
        );
    }
}

fn dim_frame(d: &IfccadDocument) -> ifccad::geometry_kernel::CoordinateFrame3 {
    let IfccadEntityKind::Dimension(dim) = &d.model.entities[0].as_native().unwrap().kind else {
        panic!("dimension")
    };
    dim.placement.coordinate_frame().unwrap()
}

#[test]
fn strict_dimension_points_have_xyz_references_and_xy_line_only() {
    let bytes = encode_ifccad_document(&support::drawing()).unwrap();
    for (field, value) in [
        ("firstPoint", serde_json::json!([0., 0.])),
        ("secondPoint", serde_json::json!([1., 2., 3., 4.])),
        ("dimensionLinePoint", serde_json::json!([0., 8., 0.])),
    ] {
        let mut wire: serde_json::Value = serde_json::from_slice(bytes.bytes()).unwrap();
        let body = wire["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find_map(|n| n["attributes"].get_mut("ifccad::dimension"))
            .unwrap();
        body["definition"][field] = value;
        assert!(
            load_ifccad_bytes(&serde_json::to_vec(&wire).unwrap(), Default::default()).is_err(),
            "{field}"
        );
    }
}

#[test]
fn spatial_reference_placement_must_stay_in_finite_scope_range() {
    let mut d = support::drawing();
    let IfccadEntityKind::Dimension(dim) = &mut d.model.entities[0].as_native_mut().unwrap().kind
    else {
        panic!("dimension")
    };
    dim.placement.origin[2] = f64::MAX;
    if let DimensionDefinition::Aligned { first_point, .. } = &mut dim.definition {
        first_point[2] = f64::MAX;
    }
    assert!(validate_ifccad_document(&d).is_err());
    assert!(encode_ifccad_document(&d).is_err());
    assert!(encode_ifccad_cbor_document(&d).is_err());
}
#[test]
fn independent_style_and_entity_paths_roundtrip_with_unused_settings() {
    let mut d = support::drawing();
    let mut over = DimensionStyleOverrides::default();
    over.text.height = Some(8.0);
    d.dimension_styles[0]
        .type_overrides
        .insert(DimensionKind::Radius, over);
    if let IfccadEntityKind::Dimension(v) = &mut d.model.entities[0].as_native_mut().unwrap().kind {
        v.overrides.text.height = Some(2.5);
    }
    let bytes = encode_ifccad_document(&d).unwrap();
    let loaded = load_ifccad_bytes(bytes.bytes(), Default::default()).unwrap();
    assert_eq!(loaded.document(), &d);
    let value: serde_json::Value = serde_json::from_slice(bytes.bytes()).unwrap();
    let style = value["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["attributes"].get("ifccad::dimensionStyle").is_some())
        .unwrap();
    assert_eq!(
        style["path"],
        format!("/cad/d{}/dimensionStyle/0", d.drawing_id)
    );
}
#[test]
fn unknown_fields_null_overrides_and_foreign_style_paths_are_rejected() {
    let d = support::drawing();
    let b = encode_ifccad_document(&d).unwrap();
    for kind in ["unknown", "null", "foreign"] {
        let mut v: serde_json::Value = serde_json::from_slice(b.bytes()).unwrap();
        let node = v["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["attributes"].get("ifccad::dimension").is_some())
            .unwrap();
        let body = &mut node["attributes"]["ifccad::dimension"];
        match kind {
            "unknown" => body["unknown"] = serde_json::json!(1),
            "null" => body["overrides"]["text"]["height"] = serde_json::Value::Null,
            _ => body["style"] = serde_json::json!("/cad/d999/dimensionStyle/0"),
        };
        assert!(load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).is_err());
    }
}

#[test]
fn later_wins_replaces_whole_body_and_keeps_original_source_immutable() {
    let mut d = support::drawing();
    if let IfccadEntityKind::Dimension(v) = &mut d.model.entities[0].as_native_mut().unwrap().kind {
        v.overrides.text.height = Some(2.5);
    }
    let mut value: serde_json::Value =
        serde_json::from_slice(encode_ifccad_document(&d).unwrap().bytes()).unwrap();
    let original = value["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["attributes"].get("ifccad::dimension").is_some())
        .unwrap()
        .clone();
    let mut body = original["attributes"]["ifccad::dimension"].clone();
    body["overrides"] = serde_json::json!({});
    body["associations"] = serde_json::json!([]);
    value["data"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"path":original["path"],"attributes":{"ifccad::dimension":body}}));
    let bytes = serde_json::to_vec(&value).unwrap();
    let loaded = load_ifccad_bytes(&bytes, Default::default()).unwrap();
    assert_eq!(loaded.graph().source_bytes(), bytes);
    let IfccadEntityKind::Dimension(v) = &loaded.document().model.entities[0]
        .as_native()
        .unwrap()
        .kind
    else {
        panic!("dimension")
    };
    assert_eq!(v.overrides.text.height, None);
    assert!(load_ifccad_bytes(
        &bytes,
        IfccadReadOptions {
            composition_policy: IfccadCompositionPolicy::RejectConflicts
        }
    )
    .is_err());
}

#[test]
fn dimension_foreground_and_explicit_reset_preserve_native_defaults() {
    let mut d = support::drawing();
    if let IfccadEntityKind::Dimension(v) = &mut d.model.entities[0].as_native_mut().unwrap().kind {
        v.overrides.text.appearance.color =
            Some(DimensionAppearanceChoice::Explicit(IfccadColor::Foreground));
        v.overrides.extension_lines.first.suppressed = Some(false);
    }
    let encoded = encode_ifccad_document(&d).unwrap();
    assert_eq!(
        load_ifccad_bytes(encoded.bytes(), Default::default())
            .unwrap()
            .document(),
        &d
    );
    let value: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    let attrs = &value["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["attributes"].get("ifccad::dimension").is_some())
        .unwrap()["attributes"];
    assert!(attrs["ifccad::entity"].get("visible").is_none());
}

#[test]
fn fraction_styles_and_sparse_resets_survive_strict_native_reader() {
    let mut d = support::drawing();
    let primary = &mut d.dimension_styles[0].settings.numbers.primary;
    primary.fraction_layout = DimensionFractionLayout::Diagonal;
    primary.fraction_scale = 0.75;
    let mut over = DimensionStyleOverrides::default();
    over.numbers.primary.fraction_layout = Some(DimensionFractionLayout::Inline);
    over.numbers.primary.fraction_scale = Some(1.0);
    d.dimension_styles[0]
        .type_overrides
        .insert(DimensionKind::Aligned, over);
    d.dimension_styles[0].settings.numbers.alternate = Some(DimensionAlternateNumberSettings {
        fraction_layout: DimensionFractionLayout::Horizontal,
        fraction_scale: 0.8,
        ..Default::default()
    });
    let encoded = encode_ifccad_document(&d).unwrap();
    let loaded = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    assert_eq!(loaded.document(), &d);
}
