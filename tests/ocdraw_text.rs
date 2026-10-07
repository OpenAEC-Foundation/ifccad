use ocdraw::ocdraw::*;
use ocdraw::text::*;

fn drawing() -> OcdrawDocument {
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("text", "mm")).unwrap();
    let pattern = builder
        .add_line_pattern(LinePatternDefinition {
            name: "Continuous".into(),
            description: None,
            pattern: vec![],
        })
        .unwrap();
    builder
        .add_layer(LayerDefinition::new(
            "Text",
            RgbColor::new(255, 255, 255),
            pattern,
        ))
        .unwrap();
    builder.build_document().unwrap()
}

fn style(id: u32, name: &str) -> DrawingTextStyle {
    DrawingTextStyle {
        id: OcdrawTextStyleId(id),
        name: name.into(),
        properties: TextStyleProperties::new(FontRequest::family("Requested face")),
    }
}

fn has_error(doc: &OcdrawDocument, code: &str) -> bool {
    validate_ocdraw_document(doc)
        .unwrap_err()
        .diagnostics()
        .iter()
        .any(|d| d.code == code)
}

#[test]
fn unused_styles_have_independent_zero_valid_ids_and_monotonic_watermarks() {
    let mut doc = drawing();
    assert!(doc.text_styles.is_empty());
    assert_eq!(doc.next_text_style_id, 1);
    doc.text_styles.push(style(0, "Zero"));
    doc.text_styles.push(style(19, "Unused"));
    doc.next_text_style_id = 20;
    validate_ocdraw_document(&doc).unwrap();
    doc.next_text_style_id = 19;
    assert!(has_error(&doc, "TEXT_STYLE_WATERMARK"));
    doc.text_styles.clear();
    doc.next_text_style_id = 20;
    validate_ocdraw_document(&doc).unwrap();
}

#[test]
fn style_lookup_is_casefolded_without_trimming_or_normalizing_names() {
    let mut doc = drawing();
    doc.next_text_style_id = 4;
    doc.text_styles = vec![style(0, "Straße"), style(1, "STRASSE")];
    assert!(has_error(&doc, "TEXT_STYLE_NAME"));
    doc.text_styles[1].name = " Straße".into();
    validate_ocdraw_document(&doc).unwrap();
    doc.text_styles[1].id = OcdrawTextStyleId(0);
    assert!(has_error(&doc, "TEXT_STYLE_ID"));
    doc.text_styles[1].id = OcdrawTextStyleId(1);
    doc.text_styles[1].properties.font = FontRequest::default();
    assert!(has_error(&doc, "TEXT_VALUE"));
}

#[test]
fn text_shares_entity_identity_and_ownership_but_requires_its_own_style() {
    let mut doc = drawing();
    doc.text_entities.push(DrawingTextEntity {
        id: 9007199254740993,
        layer_id: doc.layers[0].id,
        visible: true,
        appearance: EntityAppearance::default(),
        style_id: OcdrawTextStyleId(7),
        placement: CoordinateFrame3::default(),
        rotation: 0.0,
        backward: false,
        upside_down: false,
        layout: TextLayout::Anchored {
            horizontal: TextHorizontalAlignment::Left,
            vertical: TextVerticalAlignment::Baseline,
            height: 2.0,
            width_factor: 1.0,
        },
        oblique_angle: 0.0,
        thickness: 0.0,
        content: vec![],
    });
    doc.next_entity_id = 9007199254740994;
    doc.scopes[0].entities.push(9007199254740993);
    assert!(has_error(&doc, "TEXT_STYLE_REFERENCE"));
    doc.text_styles.push(style(7, "Text"));
    doc.next_text_style_id = 8;
    doc.text_entities.push(doc.text_entities[0].clone());
    assert!(has_error(&doc, "ENTITY_ID"));
    doc.text_entities.pop();
    doc.scopes[0].entities.clear();
    assert!(has_error(&doc, "ENTITY_OWNERSHIP"));
}

#[test]
fn unused_styles_and_deleted_style_history_survive_strict_native_readback() {
    let mut doc = drawing();
    doc.next_text_style_id = 42;
    let mut authored = style(0, "Unused");
    authored.properties.font.bold = Some(false);
    authored.properties.last_used_height = Some(0.0);
    authored.properties.creation_upside_down = true;
    doc.text_styles.push(authored);
    let bytes = encode_ocdraw_document(&doc).unwrap();
    let loaded = load_ocdraw_bytes(bytes.bytes()).unwrap();
    assert_eq!(loaded.document().text_styles, doc.text_styles);
    assert_eq!(loaded.document().next_text_style_id, 42);
    doc.text_styles.clear();
    let bytes = encode_ocdraw_document(&doc).unwrap();
    assert_eq!(
        load_ocdraw_bytes(bytes.bytes())
            .unwrap()
            .document()
            .next_text_style_id,
        42
    );
}

#[test]
fn text_bounds_are_estimates_while_empty_text_has_no_geometric_extent() {
    let mut doc = drawing();
    doc.text_styles.push(style(0, "Text"));
    doc.text_entities.push(DrawingTextEntity {
        id: 1,
        layer_id: doc.layers[0].id,
        visible: true,
        appearance: EntityAppearance::default(),
        style_id: OcdrawTextStyleId(0),
        placement: CoordinateFrame3::default(),
        rotation: 0.0,
        backward: false,
        upside_down: false,
        layout: TextLayout::Anchored {
            horizontal: TextHorizontalAlignment::Left,
            vertical: TextVerticalAlignment::Baseline,
            height: 2.0,
            width_factor: 1.0,
        },
        oblique_angle: 0.0,
        thickness: 0.0,
        content: vec![TextRun {
            text: "Literal".into(),
            ..Default::default()
        }],
    });
    doc.next_entity_id = 2;
    doc.scopes[0].entities.push(1);
    recompute_ocdraw_document_bounds(&mut doc).unwrap();
    assert!(doc.scopes[0].bounds.is_some());
    assert_eq!(
        doc.scopes[0].bounds_quality,
        Some(OcdrawBoundsQuality::Estimated)
    );
    let assessed = assess_ocdraw_document_bounds(&doc).unwrap();
    assert!(!assessed[0].enclosure_verified);
    validate_ocdraw_document(&doc).unwrap();
    let encoded = encode_ocdraw_document(&doc).unwrap();
    let read = load_ocdraw_bytes(encoded.bytes()).unwrap();
    assert_eq!(read.text_entities(), doc.text_entities);
    assert_eq!(
        read.scopes()[0].bounds_quality,
        Some(OcdrawBoundsQuality::Estimated)
    );
    doc.text_entities[0].content.clear();
    recompute_ocdraw_document_bounds(&mut doc).unwrap();
    assert!(doc.scopes[0].bounds.is_none());
    assert!(doc.scopes[0].bounds_quality.is_none());
    validate_ocdraw_document(&doc).unwrap();
}

#[test]
fn builder_allocates_text_atomically_in_the_existing_draw_order() {
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("build-text", "mm")).unwrap();
    let p = b.ensure_continuous_line_pattern().unwrap();
    let layer = b
        .add_layer(LayerDefinition::new(
            "Text",
            RgbColor::new(255, 255, 255),
            p,
        ))
        .unwrap();
    let style = b
        .add_text_style(TextStyleDefinition {
            name: "Text".into(),
            properties: TextStyleProperties::new(FontRequest::family("Requested")),
        })
        .unwrap();
    assert!(b
        .add_text_style(TextStyleDefinition {
            name: "TEXT".into(),
            properties: TextStyleProperties::new(FontRequest::family("Other"))
        })
        .is_err());
    let first = b
        .add_line(LineDefinition::new(layer, [0.; 3], [1., 1., 0.]))
        .unwrap();
    let mut text = TextEntityDefinition::new(
        layer,
        style,
        2.,
        vec![TextRun {
            text: "First".into(),
            ..Default::default()
        }],
    );
    text.rotation = f64::NAN;
    assert!(b.add_text(text.clone()).is_err());
    text.rotation = 0.;
    let second = b.add_text(text).unwrap();
    let third = b
        .add_mtext(MTextEntityDefinition::new(
            layer,
            style,
            2.,
            vec![MTextParagraph::default()],
        ))
        .unwrap();
    assert_eq!((first, second, third), (1, 2, 3));
    let output = b.finish().unwrap();
    let read = load_ocdraw_bytes(output.bytes()).unwrap();
    assert_eq!(read.scopes()[0].entities, vec![1, 2, 3]);
    assert_eq!(read.text_entities().len(), 1);
    assert_eq!(read.mtext_entities().len(), 1);
    assert_eq!(read.document().next_text_style_id, style.0 + 1);
}
