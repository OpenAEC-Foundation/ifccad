use ocdraw::{ifccad::*, text::*};

fn drawing() -> IfccadDocument {
    load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document()
}

#[test]
fn text_styles_keep_independent_large_ids_and_allocation_history() {
    let mut d = drawing();
    assert!(d.text_styles.is_empty());
    assert_eq!(d.id_counters.next_text_style_id, 1);
    d.id_counters.next_text_style_id = 9007199254740993;
    let id = d.id_counters.allocate_text_style_id().unwrap();
    assert_eq!(id, IfccadTextStyleId(9007199254740993));
    d.text_styles.push(IfccadTextStyle {
        id,
        name: "Unused".into(),
        properties: TextStyleProperties::new(FontRequest::family("Requested")),
    });
    d.text_styles.push(IfccadTextStyle {
        id: IfccadTextStyleId(0),
        name: "Zero".into(),
        properties: TextStyleProperties::new(FontRequest::family("Other")),
    });
    validate_ifccad_document(&d).unwrap();
    let output = encode_ifccad_document(&d).unwrap();
    let read = load_ifccad_bytes(output.bytes(), Default::default()).unwrap();
    let mut expected = d.text_styles.clone();
    expected.sort_by_key(|style| style.id);
    assert_eq!(read.document().text_styles, expected);
    assert_eq!(read.document().id_counters, d.id_counters);
    d.text_styles.clear();
    let output = encode_ifccad_document(&d).unwrap();
    assert_eq!(
        load_ifccad_bytes(output.bytes(), Default::default())
            .unwrap()
            .document()
            .id_counters
            .next_text_style_id,
        9007199254740994
    );
}

#[test]
fn text_style_exhaustion_is_atomic_and_native_lookup_does_not_trim() {
    let mut ids = IfccadIdCounters {
        next_text_style_id: u64::MAX,
        ..Default::default()
    };
    let before = ids;
    assert_eq!(
        ids.allocate_text_style_id().unwrap_err().domain,
        IfccadIdDomain::TextStyle
    );
    assert_eq!(ids, before);
    let mut d = drawing();
    d.text_styles = vec![
        IfccadTextStyle {
            id: IfccadTextStyleId(0),
            name: "Straße".into(),
            properties: TextStyleProperties::new(FontRequest::family("Requested")),
        },
        IfccadTextStyle {
            id: IfccadTextStyleId(1),
            name: "STRASSE".into(),
            properties: TextStyleProperties::new(FontRequest::family("Requested")),
        },
    ];
    d.id_counters.next_text_style_id = 2;
    assert!(validate_ifccad_document(&d).is_err());
    d.text_styles[1].name = " Straße".into();
    validate_ifccad_document(&d).unwrap();
    d.id_counters.next_text_style_id = 1;
    assert!(validate_ifccad_document(&d).is_err());
}

#[test]
fn text_and_mtext_keep_authored_content_and_share_only_pure_values() {
    let mut d = drawing();
    let style_id = IfccadTextStyleId(0);
    d.text_styles.push(IfccadTextStyle {
        id: style_id,
        name: "Labels".into(),
        properties: TextStyleProperties::new(FontRequest::family("Requested")),
    });
    let mut entity = d.model.entities[0].as_native().unwrap().clone();
    let placement = IfccadPlacement {
        origin: [5., 6., 7.],
        x_axis: [1., 0., 0.],
        y_axis: [0., 1., 0.],
    };
    entity.kind = IfccadEntityKind::Text(IfccadText {
        style_id,
        placement: placement.clone(),
        rotation: 0.25,
        backward: true,
        upside_down: false,
        layout: TextLayout::WholeTextMiddle {
            height: 2.,
            width_factor: 0.8,
        },
        oblique_angle: 0.1,
        thickness: -0.2,
        content: vec![TextRun {
            text: "Straße \\P %%d 世界".into(),
            underline: true,
            ..Default::default()
        }],
    });
    d.model.entities = vec![IfccadEntity::Native(entity.clone())];
    entity.id = d.id_counters.allocate_entity_id().unwrap();
    entity.kind = IfccadEntityKind::MText(Box::new(IfccadMText {
        style_id,
        placement,
        rotation: -0.5,
        backward: false,
        upside_down: true,
        height: 3.,
        attachment: MTextAttachment::BottomRight,
        flow: MTextFlow::ByStyle,
        wrap_width: Some(25.),
        columns: None,
        background: None,
        character_format: CharacterFormat::default(),
        paragraph_format: ParagraphFormat::default(),
        content: vec![
            MTextParagraph {
                inlines: vec![MTextInline::Run {
                    text: "literal \\P".into(),
                    character_format: CharacterFormat {
                        color: Some(TextColor::Explicit("#11AAEE".into())),
                        underline: Some(false),
                        ..Default::default()
                    },
                }],
                ..Default::default()
            },
            MTextParagraph::default(),
        ],
    }));
    d.model.entities.push(IfccadEntity::Native(entity));
    d.model.bounds = None;
    let expected = d.model.entities.clone();
    let bytes = encode_ifccad_document(&d).unwrap();
    let loaded = load_ifccad_bytes(bytes.bytes(), Default::default()).unwrap();
    assert_eq!(loaded.document().model.entities, expected);
    assert_eq!(loaded.document().text_styles, d.text_styles);
}
