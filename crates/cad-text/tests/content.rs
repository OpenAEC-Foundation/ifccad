use cad_text::*;
use ocdraw::text::*;

#[test]
fn text_parses_only_literal_unicode_and_supported_run_decorations() {
    let content = parse_text(r"Deur %%uD-01%%u \U+00BD", Default::default()).unwrap();
    assert_eq!(content.len(), 3);
    assert_eq!(content[0].text, "Deur ");
    assert_eq!(content[1].text, "D-01");
    assert!(content[1].underline);
    assert!(!content[2].underline);
    assert_eq!(content[2].text, " ½");
    assert_eq!(
        parse_text(r"{\fArial;text}", Default::default()).unwrap()[0].text,
        r"{\fArial;text}"
    );
    assert!(parse_text(r"A\PB", Default::default()).is_err());
    assert!(parse_text("", Default::default()).unwrap().is_empty());
}

#[test]
fn typed_text_emission_preserves_literals_and_decorations() {
    let content = [TextRun {
        text: r"\P {literal} %<field>%".into(),
        underline: true,
        overline: true,
        strike_through: false,
    }];
    let restored = parse_text(&emit_text(&content).unwrap(), Default::default()).unwrap();
    assert_eq!(restored, content);
    assert!(emit_text(&[TextRun {
        text: "strike".into(),
        strike_through: true,
        ..Default::default()
    }])
    .is_err());
}

#[test]
fn mtext_scopes_compound_relative_heights_and_restore_inheritance() {
    let parsed = parse_mtext(r"{\L\H0.2x;small\H1.25x;quarter}plain", Default::default()).unwrap();
    assert_eq!(parsed.content.len(), 1);
    let inline = &parsed.content[0].inlines;
    assert_eq!(inline.len(), 3);
    let format = |index| match &inline[index] {
        MTextInline::Run {
            character_format, ..
        } => character_format,
        _ => panic!(),
    };
    assert_eq!(format(0).height, Some(TextHeight::Relative { factor: 0.2 }));
    assert_eq!(
        format(1).height,
        Some(TextHeight::Relative { factor: 0.25 })
    );
    assert_eq!(format(1).underline, Some(true));
    assert_eq!(format(2).height, None);
    assert_eq!(format(2).underline, None);
}

#[test]
fn global_paragraph_and_inline_scopes_keep_authored_false_overrides() {
    let parsed = parse_mtext(r"\Lfirst\P\lsecond{\l again}", Default::default()).unwrap();
    assert_eq!(parsed.character_format.underline, Some(true));
    assert_eq!(parsed.content[1].character_format.underline, Some(false));
    let MTextInline::Run {
        character_format, ..
    } = &parsed.content[1].inlines[1]
    else {
        panic!()
    };
    assert_eq!(
        character_format.underline,
        Some(false),
        "explicit local false remains explicit even when paragraph false matches"
    );
}

#[test]
fn paragraphs_tabs_column_breaks_and_trailing_empty_content_remain_typed() {
    let parsed = parse_mtext("\\PHeader\\P\\PValue:\t\\S1/2;\\N\\P", Default::default()).unwrap();
    assert_eq!(parsed.content.len(), 5);
    assert!(parsed.content[0].inlines.is_empty());
    assert!(parsed.content[4].inlines.is_empty());
    assert!(parsed.content[3]
        .inlines
        .iter()
        .any(|inline| matches!(inline, MTextInline::Tab)));
    assert!(parsed.content[3]
        .inlines
        .iter()
        .any(|inline| matches!(inline, MTextInline::ColumnBreak)));
    let stack = parsed.content[3]
        .inlines
        .iter()
        .find_map(|inline| {
            if let MTextInline::Stack(stack) = inline {
                Some(stack)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(stack.stack_kind, TextStackKind::Fraction);
    assert_eq!(stack.upper, "1");
    assert_eq!(stack.lower, "2");
}

#[test]
fn colors_fonts_and_paragraph_codes_are_typed_without_runtime_resolution() {
    let parsed = parse_mtext(
        r"\fArial|b1|i0|c1|p2;\c255;\pqc,i-1,l2,r0,t4,c8,r12;red",
        Default::default(),
    )
    .unwrap();
    let font = parsed.character_format.font.as_ref().unwrap();
    assert_eq!(font.family.as_deref(), Some("Arial"));
    assert_eq!(font.bold, Some(true));
    assert_eq!(font.italic, Some(false));
    assert_eq!(font.charset, Some(1));
    assert_eq!(
        parsed.character_format.color,
        Some(TextColor::Explicit(MarkupColor::Rgb {
            red: 255,
            green: 0,
            blue: 0
        }))
    );
    let p = &parsed.content[0].paragraph_format;
    assert_eq!(p.alignment, Some(TextParagraphAlignment::Center));
    assert_eq!(p.left_indent_factor, Some(2.));
    assert_eq!(p.first_line_indent_factor, Some(-1.));
    assert_eq!(
        p.tab_stops.as_ref().unwrap()[1].alignment,
        TextTabAlignment::Center
    );
}

#[test]
fn unqualified_units_decimal_markers_and_fields_are_explicit_restrictions() {
    for source in [
        r"\pb2,a3,se4;text",
        r"\pxtD8;^I12,5",
        "%<field>%",
        r"\fArial|unknown;word",
        "A^JB",
    ] {
        assert!(parse_mtext(source, Default::default()).is_err(), "{source}");
    }
}

#[test]
fn mtext_emission_preserves_semantics_and_literal_field_like_content() {
    let character = CharacterFormat::<MarkupColor> {
        underline: Some(true),
        ..Default::default()
    };
    let paragraph = ParagraphFormat::default();
    let content = [
        MTextParagraph {
            inlines: vec![
                MTextInline::Run {
                    text: r"\P %<literal>%".into(),
                    character_format: CharacterFormat {
                        underline: Some(false),
                        height: Some(TextHeight::Relative { factor: 1.5 }),
                        ..Default::default()
                    },
                },
                MTextInline::Tab,
                MTextInline::Stack(TextStack {
                    upper: "1".into(),
                    lower: "2".into(),
                    ..Default::default()
                }),
            ],
            ..Default::default()
        },
        MTextParagraph::default(),
    ];
    let source = emit_mtext(
        MTextMarkupInput {
            content: &content,
            character_format: &character,
            paragraph_format: &paragraph,
        },
        CadTextProfile::default(),
    )
    .unwrap();
    let parsed = parse_mtext(&source, Default::default()).unwrap();
    assert_eq!(parsed.character_format.underline, Some(true));
    assert_eq!(parsed.content.len(), 2);
    let MTextInline::Run {
        text,
        character_format,
    } = &parsed.content[0].inlines[0]
    else {
        panic!()
    };
    assert_eq!(text, r"\P %<literal>%");
    assert_eq!(character_format.underline, Some(false));
    assert_eq!(
        character_format.height,
        Some(TextHeight::Relative { factor: 1.5 })
    );
}

#[test]
fn stack_unicode_is_decoded_and_unqualified_font_escapes_are_rejected() {
    let parsed = parse_mtext(r"\S\U+00BD/2;", Default::default()).unwrap();
    let MTextInline::Stack(stack) = &parsed.content[0].inlines[0] else {
        panic!()
    };
    assert_eq!(stack.upper, "½");
    assert!(parse_mtext(r"\fA\;B;word", Default::default()).is_err());
}

#[test]
fn paragraph_character_basis_and_materialization_budget_remain_explicit() {
    let c = CharacterFormat::<MarkupColor>::default();
    let p = ParagraphFormat::default();
    let content = [MTextParagraph {
        character_format: CharacterFormat {
            underline: Some(true),
            ..Default::default()
        },
        inlines: vec![MTextInline::Run {
            text: "paragraph".into(),
            character_format: Default::default(),
        }],
        ..Default::default()
    }];
    let source = emit_mtext(
        MTextMarkupInput {
            content: &content,
            character_format: &c,
            paragraph_format: &p,
        },
        Default::default(),
    )
    .unwrap();
    let parsed = parse_mtext(&source, Default::default()).unwrap();
    assert_eq!(parsed.content[0].character_format.underline, Some(true));
    assert_eq!(parsed.character_format.underline, None);
    let source = "{\\fLongFontName;A\\C1;B\\C2;C}";
    let limits = cad_text::markup::MarkupLimits {
        max_bytes: source.len(),
        ..Default::default()
    };
    assert!(
        parse_mtext(source, limits).is_err(),
        "bound repeatedly materialized font strings, not just source bytes"
    );
}

#[test]
fn closing_a_scope_after_a_paragraph_break_restores_source_inheritance() {
    let parsed = parse_mtext(r"{\Lfirst\Psecond}plain", Default::default()).unwrap();
    let inlines = &parsed.content[1].inlines;
    let style = TextStyleProperties::new(FontRequest::family("Arial"));
    let formats = inlines
        .iter()
        .filter_map(|inline| match inline {
            MTextInline::Run {
                character_format, ..
            } => Some(character_format),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(formats.len(), 2);
    let first = resolve_character_format(
        2.5,
        &style,
        &parsed.character_format,
        &parsed.content[1].character_format,
        formats[0],
    )
    .unwrap();
    let second = resolve_character_format(
        2.5,
        &style,
        &parsed.character_format,
        &parsed.content[1].character_format,
        formats[1],
    )
    .unwrap();
    assert!(first.underline);
    assert!(!second.underline);
}
