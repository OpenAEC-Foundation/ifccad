use ocdraw::{ifccad::*, text::*};
#[path = "support/ifccad_preservation.rs"]
mod preservation;
fn drawing() -> IfccadDocument {
    let mut d = load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    d.text_styles.push(IfccadTextStyle {
        id: IfccadTextStyleId(0),
        name: "Labels".into(),
        properties: TextStyleProperties::new(FontRequest::family("Requested")),
    });
    let mut entity = d.model.entities[0].as_native().unwrap().clone();
    entity.kind = IfccadEntityKind::Text(IfccadText {
        style_id: IfccadTextStyleId(0),
        placement: IfccadPlacement {
            origin: [50., 60., 0.],
            x_axis: [1., 0., 0.],
            y_axis: [0., 1., 0.],
        },
        rotation: 0.,
        backward: false,
        upside_down: false,
        layout: TextLayout::Anchored {
            horizontal: TextHorizontalAlignment::Left,
            vertical: TextVerticalAlignment::Baseline,
            height: 2.,
            width_factor: 1.,
        },
        oblique_angle: 0.,
        thickness: 0.,
        content: vec![TextRun {
            text: "Estimated glyphs".into(),
            ..Default::default()
        }],
    });
    entity.id = d.id_counters.allocate_entity_id().unwrap();
    d.model.entities.push(IfccadEntity::Native(entity));
    d.model.bounds = None;
    d
}
#[test]
fn recompute_stores_estimates_with_separate_unverified_glyph_evidence() {
    let mut d = drawing();
    recompute_ifccad_document_bounds(&mut d).unwrap();
    assert_eq!(d.model.bounds_quality, Some(IfccadBoundsQuality::Estimated));
    assert!(d.model.bounds.is_some());
    let assessment = assess_ifccad_document_bounds(&d).unwrap();
    let scope = &assessment.scopes[&IfccadScopeId::Layout(d.model.id)];
    assert_eq!(scope.quality, Some(IfccadBoundsQuality::Estimated));
    assert!(!scope.enclosure_verified);
    let bytes = encode_ifccad_document(&d).unwrap();
    let loaded = load_ifccad_bytes(bytes.bytes(), Default::default()).unwrap();
    assert_eq!(loaded.document(), &d);
    d.model.entities.pop();
    recompute_ifccad_document_bounds(&mut d).unwrap();
    assert_eq!(d.model.bounds_quality, Some(IfccadBoundsQuality::Enclosing));
    assert!(
        assess_ifccad_document_bounds(&d).unwrap().scopes[&IfccadScopeId::Layout(d.model.id)]
            .enclosure_verified
    );
}
#[test]
fn supplied_glyph_claims_preserve_values_but_cannot_hide_proven_geometry() {
    let mut d = drawing();
    recompute_ifccad_document_bounds(&mut d).unwrap();
    d.model.bounds = Some(IfccadBounds3d {
        min: [-1000.; 3],
        max: [1000.; 3],
    });
    d.model.bounds_quality = Some(IfccadBoundsQuality::Enclosing);
    let bytes = encode_ifccad_document(&d).unwrap();
    assert_eq!(
        load_ifccad_bytes(bytes.bytes(), Default::default())
            .unwrap()
            .document(),
        &d
    );
    assert!(
        !assess_ifccad_document_bounds(&d).unwrap().scopes[&IfccadScopeId::Layout(d.model.id)]
            .enclosure_verified
    );
    d.model.bounds = Some(IfccadBounds3d {
        min: [0.; 3],
        max: [0.; 3],
    });
    d.model.bounds_quality = Some(IfccadBoundsQuality::Estimated);
    assert!(validate_ifccad_document(&d).is_err());
    d.model.bounds = None;
    assert!(validate_ifccad_document(&d).is_err());
    d.model.bounds_quality = None;
    assert!(validate_ifccad_document(&d).is_ok());
}

#[test]
fn frame_stroke_uses_known_physical_units_and_unknown_units_stay_partial() {
    let mut d = drawing();
    let entity = d
        .model
        .entities
        .last_mut()
        .unwrap()
        .as_native_mut()
        .unwrap();
    let IfccadEntityKind::Text(text) = &entity.kind else {
        unreachable!()
    };
    entity.kind = IfccadEntityKind::MText(Box::new(IfccadMText {
        style_id: text.style_id,
        placement: text.placement.clone(),
        rotation: 0.,
        backward: false,
        upside_down: false,
        height: 2.,
        attachment: MTextAttachment::TopLeft,
        flow: MTextFlow::Horizontal,
        wrap_width: None,
        columns: None,
        background: Some(MTextBackground {
            fill: MTextFill::None,
            padding: TextPadding::Absolute { distance: 0. },
            opacity: 1.,
            frame: true,
        }),
        character_format: CharacterFormat::default(),
        paragraph_format: ParagraphFormat::default(),
        content: vec![MTextParagraph {
            inlines: vec![MTextInline::Run {
                text: "Framed".into(),
                character_format: Default::default(),
            }],
            ..Default::default()
        }],
    }));
    entity.appearance.line_weight = IfccadMode::Explicit(0.25);
    d.length_unit = "unitless".into();
    recompute_ifccad_document_bounds(&mut d).unwrap();
    assert!(d.model.bounds.is_none());
    assert_eq!(
        assess_ifccad_document_bounds(&d).unwrap().scopes[&IfccadScopeId::Layout(d.model.id)]
            .quality,
        Some(IfccadBoundsQuality::Partial)
    );
    d.length_unit = "mm".into();
    recompute_ifccad_document_bounds(&mut d).unwrap();
    assert!(d.model.bounds.is_some());
    assert_eq!(d.model.bounds_quality, Some(IfccadBoundsQuality::Estimated));
}

#[test]
fn stored_bounds_quality_rejects_null_and_partial_boxes() {
    let mut d = drawing();
    recompute_ifccad_document_bounds(&mut d).unwrap();
    let bytes = encode_ifccad_document(&d).unwrap();
    let original: serde_json::Value = serde_json::from_slice(bytes.bytes()).unwrap();
    for value in [serde_json::Value::Null, serde_json::json!("partial")] {
        let mut v = original.clone();
        let node = v["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["attributes"].get("ifccad::layout").is_some())
            .unwrap();
        node["attributes"]["ifccad::layout"]["boundsQuality"] = value;
        assert!(load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).is_err());
    }
}

#[test]
fn missing_bounds_cannot_hide_numeric_extent_overflow() {
    let mut d = drawing();
    let mut e = d.model.entities[0].as_native().unwrap().clone();
    e.id = d.id_counters.allocate_entity_id().unwrap();
    e.kind = IfccadEntityKind::Circle {
        radius: 1e308,
        placement: IfccadPlacement {
            origin: [1.7e308, 0., 0.],
            x_axis: [1., 0., 0.],
            y_axis: [0., 1., 0.],
        },
    };
    d.model.entities.push(IfccadEntity::Native(e));
    d.model.bounds = None;
    assert!(validate_ifccad_document(&d).is_err());
    let before = d.clone();
    assert!(recompute_ifccad_document_bounds(&mut d).is_err());
    assert_eq!(d, before);
}

#[test]
fn text_numeric_extent_overflow_is_hard_without_stored_bounds() {
    let mut d = drawing();
    let e = d
        .model
        .entities
        .last_mut()
        .unwrap()
        .as_native_mut()
        .unwrap();
    let IfccadEntityKind::Text(t) = &mut e.kind else {
        unreachable!()
    };
    t.layout = TextLayout::WholeTextMiddle {
        height: 1e308,
        width_factor: 1.,
    };
    assert!(validate_ifccad_document(&d).is_err());
    let before = d.clone();
    assert!(recompute_ifccad_document_bounds(&mut d).is_err());
    assert_eq!(d, before);
}

#[test]
fn nested_estimates_and_partial_coverage_keep_projection_overflow_hard() {
    let mut d = preservation::with_opaque();
    let prototype = drawing();
    d.text_styles = prototype.text_styles.clone();
    let mut text = prototype
        .model
        .entities
        .last()
        .unwrap()
        .as_native()
        .unwrap()
        .clone();
    text.id = d.id_counters.allocate_entity_id().unwrap();
    d.model.entities.push(IfccadEntity::Native(text));
    let block_id = d.id_counters.allocate_block_id().unwrap();
    let contents = std::mem::take(&mut d.model.entities);
    d.blocks.push(IfccadBlockDefinition {
        id: block_id,
        name: "Mixed".into(),
        base_point: [0.; 3],
        insertion_unit: "unitless".into(),
        bounds: None,
        bounds_quality: None,
        entities: contents,
    });
    let mut instance = prototype.model.entities[0].as_native().unwrap().clone();
    instance.id = d.id_counters.allocate_entity_id().unwrap();
    instance.kind = IfccadEntityKind::BlockInstance {
        definition_id: block_id,
        transform: IfccadBlockTransform {
            placement: IfccadPlacement {
                origin: [0.; 3],
                x_axis: [1., 0., 0.],
                y_axis: [0., 1., 0.],
            },
            rotation: 0.,
            scale: [1.; 3],
        },
    };
    d.model.entities = vec![IfccadEntity::Native(instance)];
    recompute_ifccad_document_bounds(&mut d).unwrap();
    let a = assess_ifccad_document_bounds(&d).unwrap();
    assert_eq!(
        a.scopes[&IfccadScopeId::Layout(d.model.id)].quality,
        Some(IfccadBoundsQuality::Partial)
    );
    assert!(d.model.bounds.is_none());
    let e = d.model.entities[0].as_native_mut().unwrap();
    let IfccadEntityKind::BlockInstance { transform, .. } = &mut e.kind else {
        unreachable!()
    };
    transform.scale = [1e308, 1., 1.];
    for reverse in [false, true] {
        if reverse {
            d.blocks.last_mut().unwrap().entities.reverse();
        }
        let before = d.clone();
        assert!(recompute_ifccad_document_bounds(&mut d).is_err());
        assert_eq!(d, before);
        assert!(validate_ifccad_document(&d).is_err());
    }
}

#[test]
fn empty_glyphs_do_not_turn_nonempty_definitions_into_insertion_point_geometry() {
    let mut d = drawing();
    let mut text = d.model.entities.last().unwrap().clone();
    let IfccadEntityKind::Text(t) = &mut text.as_native_mut().unwrap().kind else {
        unreachable!()
    };
    t.content.clear();
    let id = d.id_counters.allocate_block_id().unwrap();
    d.blocks = vec![IfccadBlockDefinition {
        id,
        name: "Empty glyphs".into(),
        base_point: [0.; 3],
        insertion_unit: "unitless".into(),
        bounds: None,
        bounds_quality: None,
        entities: vec![text],
    }];
    let mut e = d.model.entities[0].as_native().unwrap().clone();
    e.kind = IfccadEntityKind::BlockInstance {
        definition_id: id,
        transform: IfccadBlockTransform {
            placement: IfccadPlacement {
                origin: [100., 200., 0.],
                x_axis: [1., 0., 0.],
                y_axis: [0., 1., 0.],
            },
            rotation: 0.,
            scale: [1.; 3],
        },
    };
    d.model.entities = vec![IfccadEntity::Native(e)];
    recompute_ifccad_document_bounds(&mut d).unwrap();
    assert!(d.model.bounds.is_none());
    assert!(d.blocks[0].bounds.is_none());
}
