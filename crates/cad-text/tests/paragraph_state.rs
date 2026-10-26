use cad_text::*;
use ifccad::text::*;
use opencadcodec::MText;

fn resolved(source: &str) -> Vec<ResolvedParagraphFormat> {
    let parsed = parse_mtext(source, Default::default()).unwrap();
    parsed
        .content
        .iter()
        .map(|p| resolve_paragraph_format(&parsed.paragraph_format, &p.paragraph_format).unwrap())
        .collect()
}

#[test]
fn paragraph_fields_carry_independently_until_explicitly_changed() {
    let paragraphs = resolved(r"\pqc,i-1,l2,r3,t4,c8,r12;A\P\pl5;B\PC");
    assert_eq!(paragraphs.len(), 3);
    for (p, left) in paragraphs.iter().zip([2.0, 5.0, 5.0]) {
        assert_eq!(p.alignment, TextParagraphAlignment::Center);
        assert_eq!(p.first_line_indent_factor, -1.0);
        assert_eq!(p.left_indent_factor, left);
        assert_eq!(p.right_indent_factor, 3.0);
        assert_eq!(
            p.tab_stops,
            vec![
                TextTabStop {
                    position_factor: 4.0,
                    alignment: TextTabAlignment::Left
                },
                TextTabStop {
                    position_factor: 8.0,
                    alignment: TextTabAlignment::Center
                },
                TextTabStop {
                    position_factor: 12.0,
                    alignment: TextTabAlignment::Right
                },
            ]
        );
    }
}

#[test]
fn explicit_zero_left_alignment_and_empty_tabs_reset_carried_state() {
    let paragraphs = resolved(r"\pqc,i-1,l2,r3,t4;A\P\pi0,l0,r0,ql,t;B\PC");
    assert_eq!(paragraphs.len(), 3);
    assert_eq!(paragraphs[0].left_indent_factor, 2.0);
    for p in &paragraphs[1..] {
        assert_eq!(p.alignment, TextParagraphAlignment::Left);
        assert_eq!(p.first_line_indent_factor, 0.0);
        assert_eq!(p.left_indent_factor, 0.0);
        assert_eq!(p.right_indent_factor, 0.0);
        assert!(p.tab_stops.is_empty());
    }
    let parsed = parse_mtext(r"\pxt4;A\P\pxt;B\PC", Default::default()).unwrap();
    assert_eq!(parsed.content[2].paragraph_format.tab_stops, Some(vec![]));
}

#[test]
fn nested_and_cross_paragraph_scopes_restore_the_source_context() {
    for (source, expected) in [
        (
            r"\pl1;A\P{\pl2;B\P{\pl3;C}\PD}\PE",
            vec![1.0, 2.0, 3.0, 2.0, 1.0],
        ),
        (r"\pl1;A\P{\pl2;B\P}C\PD", vec![1.0, 2.0, 1.0, 1.0]),
        (r"{\pl2;A}\PB", vec![2.0, 0.0]),
    ] {
        assert_eq!(
            resolved(source)
                .iter()
                .map(|p| p.left_indent_factor)
                .collect::<Vec<_>>(),
            expected,
            "{source}"
        );
    }
}

#[test]
fn blank_trailing_and_raw_line_break_paragraphs_retain_formatting() {
    for source in [r"\pl1;A\P\PB\P", "\\pl1;A\n\rB\r\n"] {
        let parsed = parse_mtext(source, Default::default()).unwrap();
        assert_eq!(parsed.content.len(), 4);
        assert!(parsed.content[1].inlines.is_empty());
        assert!(parsed.content[3].inlines.is_empty());
        for p in resolved(source) {
            assert_eq!(p.left_indent_factor, 1.0);
        }
    }
}

#[test]
fn closing_a_scope_restores_alignment_and_tab_state_together() {
    for (p, (alignment, left, tab)) in resolved(r"\pqc,l1,t4;A\P{\pqr,l2,t8;B\PC}\PD").iter().zip([
        (TextParagraphAlignment::Center, 1.0, 4.0),
        (TextParagraphAlignment::Right, 2.0, 8.0),
        (TextParagraphAlignment::Right, 2.0, 8.0),
        (TextParagraphAlignment::Center, 1.0, 4.0),
    ]) {
        assert_eq!(p.alignment, alignment);
        assert_eq!(p.left_indent_factor, left);
        assert_eq!(
            p.tab_stops,
            vec![TextTabStop {
                position_factor: tab,
                alignment: TextTabAlignment::Left
            }]
        );
    }
}

#[test]
fn paragraph_tab_materialization_obeys_the_resource_budget() {
    for suffix in [r"\P".repeat(20), "{}".repeat(20)] {
        let source = format!("\\pxt1,2,3,4;{suffix}");
        let error = parse_mtext(
            &source,
            cad_text::markup::MarkupLimits {
                max_bytes: 256,
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(
            error.code,
            cad_text::markup::TextMarkupErrorCode::ResourceLimit
        );
    }
}

#[test]
fn source_paragraph_carry_reports_authored_dependency_normalization() {
    for (source, normalized) in [
        (r"\pl1;A\PB\PC", true),
        (r"\pl1;A\P\pl1;B\P\pl1;C", false),
        (r"A\PB", false),
    ] {
        let mut text = MText::new();
        text.value = source.into();
        let prepared = prepare_mtext_from_cad(&text).unwrap();
        assert_eq!(
            prepared
                .issues
                .contains(&CadTextIssue::AuthoredFormattingNormalized),
            normalized,
            "{source}"
        );
    }
}

#[test]
fn independent_native_paragraphs_restore_defaults_in_cad_output() {
    let paragraphs = vec![
        MTextParagraph {
            paragraph_format: ParagraphFormat {
                alignment: Some(TextParagraphAlignment::Right),
                first_line_indent_factor: Some(-1.0),
                left_indent_factor: Some(2.0),
                right_indent_factor: Some(3.0),
                tab_stops: Some(vec![TextTabStop {
                    position_factor: 4.0,
                    alignment: TextTabAlignment::Center,
                }]),
                ..Default::default()
            },
            inlines: vec![MTextInline::Run {
                text: "A".into(),
                character_format: Default::default(),
            }],
            ..Default::default()
        },
        MTextParagraph::default(),
        MTextParagraph {
            inlines: vec![MTextInline::Run {
                text: "B".into(),
                character_format: Default::default(),
            }],
            ..Default::default()
        },
    ];
    let character = CharacterFormat::default();
    for (basis, expected_left) in [
        (ParagraphFormat::default(), 0.0),
        (
            ParagraphFormat {
                left_indent_factor: Some(0.5),
                ..Default::default()
            },
            0.5,
        ),
    ] {
        let markup = emit_mtext(
            MTextMarkupInput {
                content: &paragraphs,
                character_format: &character,
                paragraph_format: &basis,
            },
            Default::default(),
        )
        .unwrap();
        let read = resolved(&markup);
        assert_eq!(read.len(), 3);
        assert_eq!(read[0].alignment, TextParagraphAlignment::Right);
        assert_eq!(read[0].left_indent_factor, 2.0);
        for p in &read[1..] {
            assert_eq!(p.alignment, TextParagraphAlignment::Left);
            assert_eq!(p.first_line_indent_factor, 0.0);
            assert_eq!(p.left_indent_factor, expected_left);
            assert_eq!(p.right_indent_factor, 0.0);
            assert!(p.tab_stops.is_empty());
        }
    }
}

#[test]
fn scoped_mid_paragraph_layout_changes_remain_explicitly_unsupported() {
    for source in [
        r"{\pl2;A}B",
        r"{\pxt4;A}^IB",
        r"{\pl2;A}\S1/2;",
        r"{\pl2;A}\N",
    ] {
        assert!(parse_mtext(source, Default::default()).is_err(), "{source}");
    }
    // A restored authored presence with the same current value is normalization,
    // rather than a layout change or a reason to discard the text.
    let mut text = MText::new();
    text.value = r"{\pl0;A}B".into();
    let prepared = prepare_mtext_from_cad(&text).unwrap();
    assert!(prepared
        .issues
        .contains(&CadTextIssue::AuthoredFormattingNormalized));
}
