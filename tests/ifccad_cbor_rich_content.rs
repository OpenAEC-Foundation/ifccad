use ifccad::*;
#[path = "support/ifccad_dimension.rs"]
mod dimensions;
#[path = "support/ifccad_hatch.rs"]
mod hatches;

#[test]
fn inactive_gradient_and_font_false_remain_authored() {
    use ifccad::geometry_kernel::hatch::*;
    let mut d = hatches::drawing();
    let IfccadEntityKind::Hatch(h) = &mut d.model.entities[0].as_native_mut().unwrap().kind else {
        panic!()
    };
    h.gradient = Some(HatchGradient {
        gradient_type: None,
        colors: None,
        rotation: -0.0,
        shift: 0.0,
        color_mode: HatchGradientColorMode::TwoColor,
        tint: 0.5,
    });
    let bytes = encode_ifccad_cbor_document(&d).unwrap();
    let loaded = load_ifccad_cbor_bytes(bytes.bytes(), Default::default())
        .unwrap()
        .into_document();
    let IfccadEntityKind::Hatch(h) = &loaded.model.entities[0].as_native().unwrap().kind else {
        panic!()
    };
    assert_eq!(
        h.gradient.as_ref().unwrap().rotation.to_bits(),
        (-0.0_f64).to_bits()
    );
    assert_eq!(loaded, d);
    let mut d = dimensions::drawing();
    d.text_styles[0].properties.font.bold = Some(false);
    let bytes = encode_ifccad_cbor_document(&d).unwrap();
    let loaded = load_ifccad_cbor_bytes(bytes.bytes(), Default::default())
        .unwrap()
        .into_document();
    assert_eq!(loaded.text_styles[0].properties.font.bold, Some(false));
}

#[test]
fn dimensions_retain_all_style_selectors_and_authored_presence() {
    use ifccad::dimension::*;
    let mut d = dimensions::drawing();
    for kind in [
        DimensionKind::Linear,
        DimensionKind::Aligned,
        DimensionKind::AngularTwoLines,
        DimensionKind::AngularThreePoints,
        DimensionKind::Radius,
        DimensionKind::Diameter,
        DimensionKind::Ordinate,
        DimensionKind::ArcLength,
        DimensionKind::JoggedRadius,
    ] {
        let mut overrides = IfccadDimensionOverrides::default();
        overrides.numbers.primary.precision = Some(0);
        overrides.text.background = Some(DimensionBackground::None);
        d.dimension_styles[0].type_overrides.insert(kind, overrides);
    }
    let e = d.model.entities[0].as_native_mut().unwrap();
    if let IfccadEntityKind::Dimension(dim) = &mut e.kind {
        dim.overrides.numbers.primary.rounding_increment = Some(-0.0);
        dim.overrides.numbers.primary.prefix = Some(String::new());
        dim.overrides.numbers.primary.decimal_separator = Some('٫');
        dim.label = DimensionLabel::Suppressed;
    }
    let output = encode_ifccad_cbor_document(&d).unwrap();
    let loaded = load_ifccad_cbor_bytes(output.bytes(), Default::default())
        .unwrap()
        .into_document();
    assert_eq!(loaded, d);
    let IfccadEntityKind::Dimension(dim) = &loaded.model.entities[0].as_native().unwrap().kind
    else {
        panic!()
    };
    assert_eq!(
        dim.overrides
            .numbers
            .primary
            .rounding_increment
            .unwrap()
            .to_bits(),
        (-0.0_f64).to_bits()
    );
    assert_eq!(dim.overrides.numbers.primary.prefix, Some(String::new()));
    assert_eq!(loaded.dimension_styles[0].type_overrides.len(), 9);
}

#[test]
fn signed_zero_subnormals_and_high_watermarks_remain_exact() {
    let mut d = load_ifccad_bytes(
        include_bytes!("fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    d.model.bounds = None;
    d.model.bounds_quality = None;
    let id = d.id_counters.allocate_entity_id().unwrap();
    d.model
        .entities
        .push(IfccadEntity::Native(IfccadNativeEntity {
            id,
            visible: true,
            layer_id: d.layers[0].id,
            appearance: IfccadEntityAppearance {
                color: IfccadMode::ByLayer,
                opacity: IfccadMode::ByLayer,
                line_pattern: IfccadMode::ByLayer,
                line_weight: IfccadMode::ByLayer,
            },
            line_pattern_scale: 1.0,
            kind: IfccadEntityKind::LineSegment {
                start: [-0.0, f64::from_bits(1), 0.0],
                end: [1.0, 2.0, -0.0],
            },
        }));
    d.id_counters.next_entity_id = u64::MAX;
    d.layers.reverse();
    d.blocks.reverse();
    let bytes = encode_ifccad_cbor_document(&d).unwrap();
    let loaded = load_ifccad_cbor_bytes(bytes.bytes(), Default::default())
        .unwrap()
        .into_document();
    assert_eq!(loaded, d);
    let IfccadEntityKind::LineSegment { start, end } = &loaded
        .model
        .entities
        .last()
        .unwrap()
        .as_native()
        .unwrap()
        .kind
    else {
        panic!()
    };
    assert_eq!(start[0].to_bits(), (-0.0_f64).to_bits());
    assert_eq!(start[1].to_bits(), 1);
    assert_eq!(end[2].to_bits(), (-0.0_f64).to_bits());
    assert_eq!(loaded.id_counters.next_entity_id, u64::MAX);
}

#[test]
fn current_fraction_settings_and_explicit_resets_survive() {
    use ifccad::dimension::*;
    for layout in [
        DimensionFractionLayout::Inline,
        DimensionFractionLayout::Horizontal,
        DimensionFractionLayout::Diagonal,
    ] {
        let mut d = dimensions::drawing();
        let numbers = &mut d.dimension_styles[0].settings.numbers;
        numbers.primary.fraction_layout = layout;
        numbers.primary.fraction_scale = 0.625;
        numbers.alternate = Some(DimensionAlternateNumberSettings {
            fraction_layout: layout,
            fraction_scale: 0.75,
            ..Default::default()
        });
        let IfccadEntityKind::Dimension(dim) =
            &mut d.model.entities[0].as_native_mut().unwrap().kind
        else {
            panic!()
        };
        dim.overrides.numbers.primary.fraction_layout = Some(DimensionFractionLayout::Inline);
        dim.overrides.numbers.primary.fraction_scale = Some(1.0);
        let out = encode_ifccad_cbor_document(&d).unwrap();
        let loaded = load_ifccad_cbor_bytes(out.bytes(), Default::default())
            .unwrap()
            .into_document();
        assert_eq!(loaded, d);
        let IfccadEntityKind::Dimension(dim) = &loaded.model.entities[0].as_native().unwrap().kind
        else {
            panic!()
        };
        assert_eq!(
            dim.overrides.numbers.primary.fraction_layout,
            Some(DimensionFractionLayout::Inline)
        );
        assert_eq!(
            dim.overrides
                .numbers
                .primary
                .fraction_scale
                .unwrap()
                .to_bits(),
            1.0_f64.to_bits()
        );
    }
}

#[test]
fn ordered_hatch_sources_provenance_and_text_islands_survive() {
    use ifccad::geometry_kernel::hatch::HatchLoopProvenance;
    for enabled in [false, true] {
        let mut d = hatches::drawing();
        let IfccadEntityKind::Hatch(h) = &mut d.model.entities[0].as_native_mut().unwrap().kind
        else {
            panic!()
        };
        h.association_enabled = enabled;
        h.loops[1].source_entity_ids = vec![hatches::SOURCE; 3];
        h.loops[1].text_island = true;
        h.loops[1].provenance = Some(HatchLoopProvenance {
            from_source_objects: true,
            derived_from_pick_point: true,
            outermost_hint: true,
            text_box: true,
        });
        let out = encode_ifccad_cbor_document(&d).unwrap();
        let loaded = load_ifccad_cbor_bytes(out.bytes(), Default::default())
            .unwrap()
            .into_document();
        assert_eq!(loaded, d);
    }
}
