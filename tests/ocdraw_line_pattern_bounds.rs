use ocdraw::{line_pattern::*, ocdraw::*, text::*};

fn drawing(text: bool) -> OcdrawDocument {
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("pattern-bounds", "mm")).unwrap();
    let continuous = b.ensure_continuous_line_pattern().unwrap();
    let layer = b
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(0, 0, 0),
            continuous,
        ))
        .unwrap();
    b.add_line(LineDefinition::new(layer, [0.; 3], [10., 0., 0.]))
        .unwrap();
    let mut doc = b.build_document().unwrap();
    let marker = if text {
        doc.text_styles.push(DrawingTextStyle {
            id: OcdrawTextStyleId(0),
            name: "Text".into(),
            properties: TextStyleProperties::new(FontRequest::family("Unknown")),
        });
        DrawingLinePatternMarker::Text {
            text: "GAS".into(),
            text_style_id: OcdrawTextStyleId(0),
            transform: Default::default(),
        }
    } else {
        doc.symbol_sources.push(DrawingSymbolSource {
            id: SymbolSourceId(0),
            source: SymbolSource::Shx {
                file_name: "missing.shx".into(),
            },
        });
        doc.next_symbol_source_id = 1;
        DrawingLinePatternMarker::Symbol {
            symbol_source_id: SymbolSourceId(0),
            shape_number: 42,
            transform: Default::default(),
        }
    };
    doc.line_patterns.push(DrawingLinePattern {
        id: LinePatternId(1),
        name: "Complex".into(),
        description: None,
        pattern: vec![LinePatternElement {
            length: 1.,
            marker: Some(marker),
        }],
    });
    doc.next_line_pattern_id = 2;
    doc.layers[0].line_pattern_id = LinePatternId(1);
    doc.scopes[0].bounds = Some(Bounds3d::new(
        Point3::new(0., 0., 0.),
        Point3::new(10., 0., 0.),
    ));
    doc.scopes[0].bounds_quality = Some(OcdrawBoundsQuality::Enclosing);
    doc
}

#[test]
fn producer_enclosing_box_survives_unknown_symbol() {
    let doc = drawing(false);
    validate_ocdraw_document(&doc).unwrap();
    let assessment = assess_ocdraw_document_bounds(&doc).unwrap();
    assert_eq!(assessment[0].quality, Some(OcdrawBoundsQuality::Partial));
    assert!(!assessment[0].enclosure_verified);
    let encoded = encode_ocdraw_document(&doc).unwrap();
    let reopened = load_ocdraw_bytes(encoded.bytes()).unwrap();
    assert_eq!(doc.scopes[0].bounds, reopened.document().scopes[0].bounds);
    assert_eq!(
        reopened.document().scopes[0].bounds_quality,
        Some(OcdrawBoundsQuality::Enclosing)
    );
}

#[test]
fn known_curve_outside_box_still_fails() {
    let mut doc = drawing(false);
    doc.scopes[0].bounds = Some(Bounds3d::new(
        Point3::new(0., 0., 0.),
        Point3::new(9., 0., 0.),
    ));
    assert!(validate_ocdraw_document(&doc).is_err());
}

#[test]
fn text_estimate_is_not_a_containment_requirement() {
    let doc = drawing(true);
    validate_ocdraw_document(&doc).unwrap();
    assert_eq!(
        assess_ocdraw_document_bounds(&doc).unwrap()[0].quality,
        Some(OcdrawBoundsQuality::Estimated)
    );
}

#[test]
fn explicit_recompute_estimates_text_and_clears_partial_box() {
    let mut text = drawing(true);
    recompute_ocdraw_document_bounds(&mut text).unwrap();
    assert_eq!(
        text.scopes[0].bounds_quality,
        Some(OcdrawBoundsQuality::Estimated)
    );
    assert!(text.scopes[0].bounds.unwrap().max().components()[0] > 10.);
    let mut symbols = drawing(false);
    recompute_ocdraw_document_bounds(&mut symbols).unwrap();
    assert!(symbols.scopes[0].bounds.is_none());
}

#[test]
fn unused_complex_definitions_do_not_change_bounds() {
    let mut doc = drawing(false);
    doc.layers[0].line_pattern_id = LinePatternId(0);
    assert_eq!(
        assess_ocdraw_document_bounds(&doc).unwrap()[0].quality,
        Some(OcdrawBoundsQuality::Enclosing)
    );
}

fn occurrences(byblock: bool) -> OcdrawDocument {
    let source = drawing(false);
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("occurrences", "mm")).unwrap();
    let continuous = b.ensure_continuous_line_pattern().unwrap();
    b.add_symbol_source(source.symbol_sources[0].source.clone())
        .unwrap();
    let pattern = b
        .add_line_pattern(LinePatternDefinition {
            name: "Complex".into(),
            description: None,
            pattern: source.line_patterns[1].pattern.clone(),
        })
        .unwrap();
    let zero = b
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(0, 0, 0),
            continuous,
        ))
        .unwrap();
    let complex = b
        .add_layer(LayerDefinition::new(
            "Complex",
            RgbColor::new(0, 0, 0),
            pattern,
        ))
        .unwrap();
    let simple = b
        .add_layer(LayerDefinition::new(
            "Simple",
            RgbColor::new(0, 0, 0),
            continuous,
        ))
        .unwrap();
    let paper = b.add_paper_layout("Sheet").unwrap();
    let block = b
        .add_block_definition(BlockDefinition::new("Leaf"))
        .unwrap();
    let mut line = LineDefinition::new(zero, [0.; 3], [10., 0., 0.]).in_scope(block);
    if byblock {
        line.appearance.line_pattern = AppearanceSelection::ByBlock;
    }
    b.add_line(line).unwrap();
    b.add_block_instance(BlockInstanceDefinition::new(complex, block, [0.; 3]))
        .unwrap();
    b.add_block_instance(BlockInstanceDefinition::new(simple, block, [0.; 3]).in_scope(paper))
        .unwrap();
    b.build_document().unwrap()
}

#[test]
fn byblock_and_layer_zero_depend_on_occurrence() {
    for byblock in [false, true] {
        let mut doc = occurrences(byblock);
        let evidence = assess_ocdraw_document_bounds(&doc).unwrap();
        assert_eq!(
            evidence.iter().find(|s| s.scope_id == 0).unwrap().quality,
            Some(OcdrawBoundsQuality::Partial)
        );
        let paper = doc
            .layouts
            .iter()
            .find(|l| l.kind == DrawingLayoutKind::Paper)
            .unwrap()
            .scope_id;
        assert_eq!(
            evidence
                .iter()
                .find(|s| s.scope_id == paper)
                .unwrap()
                .quality,
            Some(OcdrawBoundsQuality::Enclosing)
        );
        doc.layers
            .iter_mut()
            .find(|l| l.name == "Complex")
            .unwrap()
            .line_pattern_id = LinePatternId(0);
        assert_eq!(
            assess_ocdraw_document_bounds(&doc)
                .unwrap()
                .iter()
                .find(|s| s.scope_id == 0)
                .unwrap()
                .quality,
            Some(OcdrawBoundsQuality::Enclosing)
        );
    }
}

#[test]
fn nested_known_subset_is_still_checked() {
    let mut doc = occurrences(true);
    doc.scopes.iter_mut().find(|s| s.id == 0).unwrap().bounds = Some(Bounds3d::new(
        Point3::new(-1., -1., -1.),
        Point3::new(11., 1., 1.),
    ));
    validate_ocdraw_document(&doc).unwrap();
    doc.scopes.iter_mut().find(|s| s.id == 0).unwrap().bounds = Some(Bounds3d::new(
        Point3::new(-1., -1., -1.),
        Point3::new(9., 1., 1.),
    ));
    assert!(validate_ocdraw_document(&doc)
        .unwrap_err()
        .diagnostics()
        .iter()
        .any(|d| d.code == "BLOCK_BOUNDS"));
}

#[test]
fn recompute_marker_overflow_is_atomic() {
    let mut doc = drawing(true);
    doc.line_pattern_scale = f64::MAX;
    let old = doc.scopes.clone();
    assert!(recompute_ocdraw_document_bounds(&mut doc).is_err());
    assert_eq!(doc.scopes, old);
}
