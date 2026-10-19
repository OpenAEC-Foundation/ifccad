use ifccad::{line_pattern::*, text::*, *};
fn drawing(text: bool) -> IfccadDocument {
    let mut doc = load_ifccad_bytes(
        include_bytes!("../tests/fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    doc.blocks.clear();
    doc.paper_layouts.clear();
    let marker = if text {
        doc.text_styles.push(IfccadTextStyle {
            id: IfccadTextStyleId(0),
            name: "Text".into(),
            properties: TextStyleProperties::new(FontRequest::family("Unknown")),
        });
        IfccadLinePatternMarker::Text {
            text: "GAS".into(),
            text_style_id: IfccadTextStyleId(0),
            transform: Default::default(),
        }
    } else {
        doc.symbol_sources.push(IfccadSymbolSource {
            id: IfccadSymbolSourceId(0),
            source: SymbolSource::Shx {
                file_name: "missing.shx".into(),
            },
        });
        IfccadLinePatternMarker::Symbol {
            symbol_source_id: IfccadSymbolSourceId(0),
            shape_number: 42,
            transform: Default::default(),
        }
    };
    doc.line_patterns.push(IfccadLinePattern {
        id: IfccadLinePatternId(1),
        name: "Complex".into(),
        description: None,
        pattern: vec![LinePatternElement {
            length: 1.,
            marker: Some(marker),
        }],
    });
    doc.id_counters.next_line_pattern_id = 2;
    doc.model.entities = vec![IfccadEntity::Native(IfccadNativeEntity {
        id: 1,
        layer_id: doc.layers[0].id,
        visible: true,
        appearance: IfccadEntityAppearance {
            color: IfccadMode::ByLayer,
            opacity: IfccadMode::ByLayer,
            line_pattern: IfccadMode::ByLayer,
            line_weight: IfccadMode::ByLayer,
        },
        line_pattern_scale: 1.,
        kind: IfccadEntityKind::LineSegment {
            start: [0.; 3],
            end: [10., 0., 0.],
        },
    })];
    doc.layers[0].appearance.line_pattern = IfccadLinePatternId(1);
    doc.model.bounds = Some(IfccadBounds3d {
        min: [0.; 3],
        max: [10., 0., 0.],
    });
    doc.model.bounds_quality = Some(IfccadBoundsQuality::Enclosing);
    doc
}
#[test]
fn producer_enclosing_box_survives_unknown_symbol() {
    let doc = drawing(false);
    validate_ifccad_document(&doc).unwrap();
    let assessment = assess_ifccad_document_bounds(&doc).unwrap();
    let s = &assessment.scopes[&IfccadScopeId::Layout(doc.model.id)];
    assert_eq!(s.quality, Some(IfccadBoundsQuality::Partial));
    assert!(!s.enclosure_verified);
    let encoded = encode_ifccad_document(&doc).unwrap();
    let reopened = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    assert_eq!(doc.model.bounds, reopened.document().model.bounds);
    assert_eq!(
        doc.model.bounds_quality,
        reopened.document().model.bounds_quality
    );
}
#[test]
fn known_curve_outside_box_still_fails() {
    let mut doc = drawing(false);
    doc.model.bounds.as_mut().unwrap().max[0] = 9.;
    assert!(validate_ifccad_document(&doc).is_err());
}
#[test]
fn text_estimate_is_not_a_containment_requirement() {
    let doc = drawing(true);
    validate_ifccad_document(&doc).unwrap();
    assert_eq!(
        assess_ifccad_document_bounds(&doc).unwrap().scopes[&IfccadScopeId::Layout(doc.model.id)]
            .quality,
        Some(IfccadBoundsQuality::Estimated)
    );
}
#[test]
fn explicit_recompute_estimates_text_and_clears_partial_box() {
    let mut doc = drawing(true);
    recompute_ifccad_document_bounds(&mut doc).unwrap();
    assert_eq!(
        doc.model.bounds_quality,
        Some(IfccadBoundsQuality::Estimated)
    );
    assert!(doc.model.bounds.unwrap().max[0] > 10.);
    let mut doc = drawing(false);
    recompute_ifccad_document_bounds(&mut doc).unwrap();
    assert!(doc.model.bounds.is_none());
}
#[test]
fn unused_complex_definitions_do_not_change_bounds() {
    let mut doc = drawing(false);
    doc.layers[0].appearance.line_pattern = IfccadLinePatternId(0);
    assert_eq!(
        assess_ifccad_document_bounds(&doc).unwrap().scopes[&IfccadScopeId::Layout(doc.model.id)]
            .quality,
        Some(IfccadBoundsQuality::Enclosing)
    );
}

fn occurrences(byblock: bool) -> IfccadDocument {
    let mut doc = drawing(false);
    doc.model.bounds = None;
    doc.model.bounds_quality = None;
    let mut line = doc.model.entities.remove(0);
    line.as_native_mut().unwrap().layer_id = 0;
    if byblock {
        line.as_native_mut().unwrap().appearance.line_pattern = IfccadMode::ByBlock;
    }
    doc.layers
        .iter_mut()
        .find(|l| l.id == 0)
        .unwrap()
        .appearance
        .line_pattern = IfccadLinePatternId(0);
    let block = doc.id_counters.allocate_block_id().unwrap();
    doc.blocks.push(IfccadBlockDefinition {
        id: block,
        name: "Leaf".into(),
        description: String::new(),
        anonymous: false,
        explodable: true,
        uniform_scaling: false,
        bounds_quality: None,
        base_point: [0.; 3],
        insertion_unit: "mm".into(),
        bounds: None,
        entities: vec![line],
    });
    let layer = doc.layers.iter().find(|l| l.id != 0).unwrap().id;
    doc.layers
        .iter_mut()
        .find(|l| l.id == layer)
        .unwrap()
        .appearance
        .line_pattern = IfccadLinePatternId(1);
    let instance = IfccadNativeEntity {
        id: doc.id_counters.allocate_entity_id().unwrap(),
        layer_id: layer,
        visible: true,
        line_pattern_scale: 1.,
        appearance: IfccadEntityAppearance {
            color: IfccadMode::ByLayer,
            opacity: IfccadMode::ByLayer,
            line_pattern: IfccadMode::ByLayer,
            line_weight: IfccadMode::ByLayer,
        },
        kind: IfccadEntityKind::BlockInstance {
            definition_id: block,
            transform: IfccadBlockTransform {
                placement: IfccadPlacement {
                    origin: [0.; 3],
                    x_axis: [1., 0., 0.],
                    y_axis: [0., 1., 0.],
                },
                rotation: 0.,
                scale: [1.; 3],
            },
        },
    };
    doc.model.entities.push(IfccadEntity::Native(instance));
    doc
}

#[test]
fn byblock_and_layer_zero_depend_on_occurrence() {
    for byblock in [false, true] {
        let mut doc = occurrences(byblock);
        let owner = IfccadScopeId::Layout(doc.model.id);
        assert_eq!(
            assess_ifccad_document_bounds(&doc).unwrap().scopes[&owner].quality,
            Some(IfccadBoundsQuality::Partial)
        );
        doc.model.entities[0]
            .as_native_mut()
            .unwrap()
            .appearance
            .line_pattern = IfccadMode::Explicit(IfccadLinePatternId(0));
        // Layer zero still resolves its actual insert layer; switch that layer too.
        let layer = doc.model.entities[0].as_native().unwrap().layer_id;
        doc.layers
            .iter_mut()
            .find(|l| l.id == layer)
            .unwrap()
            .appearance
            .line_pattern = IfccadLinePatternId(0);
        assert_eq!(
            assess_ifccad_document_bounds(&doc).unwrap().scopes[&owner].quality,
            Some(IfccadBoundsQuality::Enclosing)
        );
    }
}

#[test]
fn nested_known_subset_is_still_checked() {
    let mut doc = occurrences(true);
    doc.model.bounds = Some(IfccadBounds3d {
        min: [-1.; 3],
        max: [11., 1., 1.],
    });
    validate_ifccad_document(&doc).unwrap();
    doc.model.bounds.as_mut().unwrap().max[0] = 9.;
    assert!(validate_ifccad_document(&doc)
        .unwrap_err()
        .errors
        .iter()
        .any(|e| e.contains("conservative geometry envelope")));
}

#[test]
fn recompute_marker_overflow_is_atomic() {
    let mut doc = drawing(true);
    doc.line_pattern_scale = f64::MAX;
    let before = doc.clone();
    assert!(recompute_ifccad_document_bounds(&mut doc).is_err());
    assert_eq!(doc, before);
}

#[path = "support/ifccad_preservation.rs"]
mod preservation_support;
#[test]
fn opaque_policy_is_not_relaxed() {
    let mut doc = preservation_support::with_opaque();
    let markers = drawing(false);
    doc.line_patterns = markers.line_patterns;
    doc.id_counters.next_line_pattern_id = 2;
    doc.symbol_sources = markers.symbol_sources;
    doc.layers[0].appearance.line_pattern = IfccadLinePatternId(1);
    doc.model.bounds = Some(IfccadBounds3d {
        min: [-1e9; 3],
        max: [1e9; 3],
    });
    assert!(validate_ifccad_document(&doc).is_err());
}
