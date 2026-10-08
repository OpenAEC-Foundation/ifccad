use ocdraw::{ifccad::*, text::*};
fn rich() -> IfccadDocument {
    let mut d = load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    d.text_styles.push(IfccadTextStyle {
        id: IfccadTextStyleId(0),
        name: "Requested".into(),
        properties: TextStyleProperties::new(FontRequest::family("Requested face")),
    });
    let mut e = d.model.entities[0].as_native().unwrap().clone();
    let character = CharacterFormat {
        underline: Some(false),
        height: Some(TextHeight::Relative { factor: 1.2 }),
        color: Some(TextColor::Explicit("#01aB03".into())),
        ..Default::default()
    };
    let mut inlines = vec![
        MTextInline::Run {
            text: "Literal \\P %<field> 漢e\u{301}".into(),
            character_format: character,
        },
        MTextInline::Tab,
    ];
    for stack_kind in [
        TextStackKind::Fraction,
        TextStackKind::DiagonalFraction,
        TextStackKind::Tolerance,
        TextStackKind::DecimalTolerance,
    ] {
        inlines.push(MTextInline::Stack(TextStack {
            stack_kind,
            upper: "1,2".into(),
            lower: "3,4".into(),
            separator: (stack_kind == TextStackKind::DecimalTolerance).then_some(','),
            ..Default::default()
        }));
    }
    inlines.push(MTextInline::LineBreak);
    e.kind = IfccadEntityKind::MText(Box::new(IfccadMText {
        style_id: IfccadTextStyleId(0),
        placement: IfccadPlacement {
            origin: [20., 30., 40.],
            x_axis: [1., 0., 0.],
            y_axis: [0., 1., 0.],
        },
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
            fill: MTextFill::Color("#12aBcD".into()),
            padding: TextPadding::Absolute { distance: 1. },
            opacity: 0.5,
            frame: false,
        }),
        character_format: CharacterFormat {
            font: Some(FontRequest {
                bold: Some(false),
                italic: Some(true),
                charset: Some(123),
                pitch: Some(7),
                ..FontRequest::family("Alternative")
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
                inlines,
            },
            MTextParagraph::default(),
        ],
    }));
    d.model.entities = vec![IfccadEntity::Native(e)];
    d.model.bounds = None;
    recompute_ifccad_document_bounds(&mut d).unwrap();
    d
}
#[test]
fn native_rich_values_preserve_overrides_unicode_colour_case_and_all_stack_kinds() {
    let d = rich();
    let encoded = encode_ifccad_document(&d).unwrap();
    let loaded = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    assert_eq!(loaded.document(), &d);
}
#[test]
fn native_text_layouts_and_column_states_are_independently_qualified() {
    for columns in [
        None,
        Some(MTextColumns::Static {
            count: 2,
            column_width: 40.,
            gutter: 0.,
            column_height: 30.,
            flow_reversed: false,
        }),
        Some(MTextColumns::DynamicManualHeight {
            column_width: 40.,
            gutter: 1.,
            column_heights: vec![
                MTextColumnHeight::Fixed { distance: 30. },
                MTextColumnHeight::Auto,
            ],
            flow_reversed: true,
        }),
    ] {
        let mut d = rich();
        let IfccadEntityKind::MText(t) = &mut d.model.entities[0].as_native_mut().unwrap().kind
        else {
            unreachable!()
        };
        t.columns = columns;
        t.wrap_width = t.columns.is_none().then_some(40.);
        recompute_ifccad_document_bounds(&mut d).unwrap();
        let encoded = encode_ifccad_document(&d).unwrap();
        assert_eq!(
            load_ifccad_bytes(encoded.bytes(), Default::default())
                .unwrap()
                .document(),
            &d
        );
    }
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
        let e = d.model.entities[0].as_native_mut().unwrap();
        e.kind = IfccadEntityKind::Text(IfccadText {
            style_id: IfccadTextStyleId(0),
            placement: IfccadPlacement {
                origin: [5., 6., 7.],
                x_axis: [1., 0., 0.],
                y_axis: [0., 1., 0.],
            },
            rotation: 0.3,
            backward: true,
            upside_down: true,
            layout,
            oblique_angle: 0.2,
            thickness: -2.,
            content: vec![TextRun {
                text: "All layouts".into(),
                underline: true,
                overline: true,
                strike_through: true,
            }],
        });
        recompute_ifccad_document_bounds(&mut d).unwrap();
        let encoded = encode_ifccad_document(&d).unwrap();
        assert_eq!(
            load_ifccad_bytes(encoded.bytes(), Default::default())
                .unwrap()
                .document(),
            &d
        );
    }
}
#[test]
fn closed_nested_payloads_reject_unknown_fields_nulls_and_ambiguous_variants() {
    let encoded = encode_ifccad_document(&rich()).unwrap();
    let original: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    for mutation in 0..8 {
        let mut v = original.clone();
        let t = &mut v["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["attributes"].get("ifccad::mText").is_some())
            .unwrap()["attributes"]["ifccad::mText"];
        match mutation {
            0 => t["content"][0]["format"] = serde_json::json!({}),
            1 => t["content"][0]["inlines"][0]["format"] = serde_json::json!({}),
            2 => t["characterFormat"]["underline"] = serde_json::Value::Null,
            3 => t["columns"]["columnHeights"] = serde_json::json!([]),
            4 => t["paragraphFormat"]["tabStops"][0]["separator"] = serde_json::json!(",,"),
            5 => t["content"][0]["inlines"][1]["kind"] = serde_json::json!("unknown"),
            6 => t["height"] = serde_json::json!([]),
            _ => t["style"] = serde_json::json!(0),
        };
        assert!(
            load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).is_err(),
            "mutation {mutation}"
        );
    }
}

#[test]
fn every_attachment_and_flow_has_native_strict_readback() {
    for attachment in [
        MTextAttachment::TopLeft,
        MTextAttachment::TopCenter,
        MTextAttachment::TopRight,
        MTextAttachment::MiddleLeft,
        MTextAttachment::MiddleCenter,
        MTextAttachment::MiddleRight,
        MTextAttachment::BottomLeft,
        MTextAttachment::BottomCenter,
        MTextAttachment::BottomRight,
    ] {
        for flow in [
            MTextFlow::Horizontal,
            MTextFlow::Vertical,
            MTextFlow::ByStyle,
        ] {
            let mut d = rich();
            let IfccadEntityKind::MText(t) = &mut d.model.entities[0].as_native_mut().unwrap().kind
            else {
                unreachable!()
            };
            t.attachment = attachment;
            t.flow = flow;
            recompute_ifccad_document_bounds(&mut d).unwrap();
            let encoded = encode_ifccad_document(&d).unwrap();
            assert_eq!(
                load_ifccad_bytes(encoded.bytes(), Default::default())
                    .unwrap()
                    .document(),
                &d
            );
        }
    }
}
