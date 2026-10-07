use ocdraw::{ocdraw::*, text::*};

fn rich() -> OcdrawDocument {
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("rich-text", "mm")).unwrap();
    let p = b.ensure_continuous_line_pattern().unwrap();
    let l = b
        .add_layer(LayerDefinition::new(
            "Text",
            RgbColor::new(255, 255, 255),
            p,
        ))
        .unwrap();
    let mut d = b.build_document().unwrap();
    d.text_styles.push(DrawingTextStyle {
        id: OcdrawTextStyleId(0),
        name: "Unused requested face".into(),
        properties: TextStyleProperties::new(FontRequest::family("Requested face")),
    });
    let character = CharacterFormat {
        underline: Some(false),
        height: Some(TextHeight::Relative { factor: 1.2 }),
        color: Some(TextColor::Explicit(
            DrawingColor::rgb(1, 2, 3).with_indexed("ACI", 7),
        )),
        ..Default::default()
    };
    d.mtext_entities.push(DrawingMTextEntity {
        id: 1,
        layer_id: l,
        visible: false,
        appearance: EntityAppearance::default(),
        style_id: OcdrawTextStyleId(0),
        placement: CoordinateFrame3::default(),
        rotation: 0.2,
        backward: true,
        upside_down: true,
        height: 2.5,
        attachment: MTextAttachment::BottomRight,
        flow: MTextFlow::ByStyle,
        wrap_width: None,
        columns: Some(MTextColumns::DynamicAutoHeight {
            column_width: 40.,
            gutter: 1.,
            column_height: 20.,
            current_column_count: 2,
            flow_reversed: true,
        }),
        background: Some(MTextBackground {
            fill: MTextFill::Canvas,
            padding: TextPadding::Absolute { distance: 1. },
            opacity: 0.5,
            frame: false,
        }),
        character_format: CharacterFormat {
            font: Some(FontRequest {
                bold: Some(false),
                ..FontRequest::family("Requested alternative")
            }),
            tracking: Some(1.1),
            ..Default::default()
        },
        paragraph_format: ParagraphFormat {
            alignment: Some(TextParagraphAlignment::Justified),
            line_spacing: Some(TextLineSpacing::Multiple { factor: 1. }),
            tab_stops: Some(vec![TextTabStop {
                position_factor: 12.,
                alignment: TextTabAlignment::Decimal { separator: ',' },
            }]),
            ..Default::default()
        },
        content: vec![
            MTextParagraph {
                paragraph_format: ParagraphFormat {
                    tab_stops: Some(vec![]),
                    space_after: Some(0.),
                    ..Default::default()
                },
                character_format: CharacterFormat {
                    underline: Some(true),
                    ..Default::default()
                },
                inlines: vec![
                    MTextInline::Run {
                        text: "Literal \\P %<field> 漢e\u{301}".into(),
                        character_format: character,
                    },
                    MTextInline::Tab,
                    MTextInline::Stack(TextStack {
                        upper: "1".into(),
                        lower: "2".into(),
                        ..Default::default()
                    }),
                    MTextInline::LineBreak,
                    MTextInline::ColumnBreak,
                ],
            },
            MTextParagraph::default(),
        ],
    });
    d.next_entity_id = 2;
    d.scopes[0].entities.push(1);
    recompute_ocdraw_document_bounds(&mut d).unwrap();
    d
}

#[test]
fn mtext_preserves_rich_authored_values_and_trailing_empty_paragraphs() {
    let d = rich();
    let output = encode_ocdraw_document(&d).unwrap();
    let read = load_ocdraw_bytes(output.bytes()).unwrap();
    assert_eq!(read.mtext_entities(), d.mtext_entities);
    assert_eq!(read.text_styles(), d.text_styles);
    assert_eq!(read.scopes(), d.scopes);
}

#[test]
fn nested_rows_reject_unknown_members_nulls_and_ambiguous_variants() {
    let output = encode_ocdraw_document(&rich()).unwrap();
    let base: serde_json::Value = serde_json::from_slice(output.bytes()).unwrap();
    for mutation in 0..8 {
        let mut bad = base.clone();
        let stream = &mut bad["streams"]["mTextStream"];
        match mutation {
            0 => stream["content"][0][0]["format"] = serde_json::json!({}),
            1 => stream["content"][0][0]["inlines"][0]["format"] = serde_json::json!({}),
            2 => stream["characterFormat"][0]["underline"] = serde_json::Value::Null,
            3 => stream["columns"][0]["columnHeights"] = serde_json::json!([]),
            4 => stream["paragraphFormat"][0]["tabStops"][0]["separator"] = serde_json::json!(",,"),
            5 => stream["content"][0][0]["inlines"][1]["kind"] = serde_json::json!("unknown"),
            6 => stream["height"] = serde_json::json!([]),
            _ => stream["styleId"][0] = serde_json::json!(0.0),
        }
        assert!(
            load_ocdraw_bytes(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "mutation {mutation}"
        );
    }
}

#[test]
fn all_text_layouts_and_mtext_column_states_use_production_readback() {
    for layout in [
        TextLayout::Anchored {
            horizontal: TextHorizontalAlignment::Right,
            vertical: TextVerticalAlignment::Top,
            height: 2.,
            width_factor: 0.8,
        },
        TextLayout::WholeTextMiddle {
            height: 2.,
            width_factor: 1.2,
        },
        TextLayout::Aligned {
            length: 10.,
            width_factor: 1.2,
        },
        TextLayout::Fit {
            length: 10.,
            height: 2.,
        },
    ] {
        let mut d = rich();
        d.text_entities.push(DrawingTextEntity {
            id: 2,
            layer_id: d.layers[0].id,
            visible: true,
            appearance: EntityAppearance::default(),
            style_id: OcdrawTextStyleId(0),
            placement: CoordinateFrame3::try_from_normal_arbitrary_axis(
                Point3::new(4., 5., 6.),
                Vector3::new(0., 1., 0.),
            )
            .unwrap(),
            rotation: 0.2,
            backward: true,
            upside_down: true,
            layout,
            oblique_angle: 0.1,
            thickness: -0.5,
            content: vec![
                TextRun {
                    text: "é".into(),
                    strike_through: true,
                    ..Default::default()
                },
                TextRun {
                    text: "🙂".into(),
                    underline: true,
                    ..Default::default()
                },
            ],
        });
        d.next_entity_id = 3;
        d.scopes[0].entities.push(2);
        recompute_ocdraw_document_bounds(&mut d).unwrap();
        let output = encode_ocdraw_document(&d).unwrap();
        assert_eq!(
            load_ocdraw_bytes(output.bytes()).unwrap().text_entities(),
            d.text_entities
        );
    }
    for columns in [
        MTextColumns::Static {
            count: 2,
            column_width: 40.,
            gutter: 1.,
            column_height: 20.,
            flow_reversed: false,
        },
        MTextColumns::DynamicManualHeight {
            column_width: 40.,
            gutter: 1.,
            column_heights: vec![
                MTextColumnHeight::Fixed { distance: 20. },
                MTextColumnHeight::Auto,
            ],
            flow_reversed: true,
        },
    ] {
        let mut d = rich();
        d.mtext_entities[0].columns = Some(columns);
        recompute_ocdraw_document_bounds(&mut d).unwrap();
        let output = encode_ocdraw_document(&d).unwrap();
        assert_eq!(
            load_ocdraw_bytes(output.bytes()).unwrap().mtext_entities(),
            d.mtext_entities
        );
    }
}
