use super::*;

fn style() -> TextStyleProperties {
    TextStyleProperties::new(FontRequest::family("Arial"))
}

fn paragraph(inlines: Vec<MTextInline<u32>>) -> MTextParagraph<u32> {
    MTextParagraph {
        inlines,
        ..Default::default()
    }
}

fn context<'a>(columns: Option<&'a MTextColumns>) -> MTextValueContext<'a> {
    MTextValueContext {
        nominal_height: 2.5,
        wrap_width: None,
        columns,
    }
}

#[test]
fn authored_inheritance_is_property_replacement() {
    let mut base = style();
    base.width_factor = 1.2;
    base.creation_height = Some(99.0);
    base.creation_backward = true;
    let entity = CharacterFormat::<u32> {
        width_factor: Some(0.8),
        underline: Some(true),
        height: Some(TextHeight::Relative { factor: 1.5 }),
        color: Some(TextColor::ByLayer),
        ..Default::default()
    };
    let p = CharacterFormat {
        underline: Some(false),
        oblique_angle: Some(0.0),
        ..Default::default()
    };
    let local = CharacterFormat {
        color: Some(TextColor::Entity),
        ..Default::default()
    };
    let resolved = resolve_character_format(2.5, &base, &entity, &p, &local).unwrap();
    assert_eq!(resolved.width_factor, 0.8);
    assert_eq!(resolved.height, 3.75);
    assert!(!resolved.underline);
    assert_eq!(resolved.color, TextColor::Entity);
    assert_eq!(resolved.oblique_angle, 0.0);
    assert_eq!(
        resolve_character_format(5.0, &base, &entity, &p, &local)
            .unwrap()
            .height,
        7.5
    );
    let absolute = CharacterFormat {
        height: Some(TextHeight::Absolute { distance: 4.0 }),
        ..local
    };
    assert_eq!(
        resolve_character_format(5.0, &base, &entity, &p, &absolute)
            .unwrap()
            .height,
        4.0
    );
    assert_eq!(
        entity.underline,
        Some(true),
        "resolving must not rewrite authored values"
    );
}

#[test]
fn font_overrides_replace_the_complete_symbolic_request() {
    let mut base = style();
    base.font = FontRequest {
        cad_font_name: Some("simplex.shx".into()),
        big_font_name: Some("big.shx".into()),
        ..Default::default()
    };
    let entity = CharacterFormat::<u32> {
        font: Some(FontRequest::family("Noto Sans")),
        ..Default::default()
    };
    let result = resolve_character_format(
        2.5,
        &base,
        &entity,
        &Default::default(),
        &Default::default(),
    )
    .unwrap();
    assert_eq!(result.font.family.as_deref(), Some("Noto Sans"));
    assert_eq!(result.font.cad_font_name, None);
    assert_eq!(result.font.big_font_name, None);
    assert_eq!(result.font.bold, None);
}

#[test]
fn paragraph_overrides_have_no_previous_paragraph_dependency() {
    let entity = ParagraphFormat {
        left_indent_factor: Some(2.0),
        space_after: Some(1.0),
        ..Default::default()
    };
    let first = ParagraphFormat {
        left_indent_factor: Some(0.0),
        space_after: Some(0.0),
        ..Default::default()
    };
    let second = ParagraphFormat::default();
    let before = resolve_paragraph_format(&entity, &second).unwrap();
    assert_eq!(
        resolve_paragraph_format(&entity, &first)
            .unwrap()
            .left_indent_factor,
        0.0
    );
    assert_eq!(before.left_indent_factor, 2.0);
    assert_eq!(before.space_after, 1.0);
    assert_eq!(resolve_paragraph_format(&entity, &second).unwrap(), before);
}

#[test]
fn literal_content_preserves_unicode_and_significant_whitespace() {
    let literal = "  e\u{301} 😀 \u{a0}\u{200d}\u{202e}\\P %<literal>% ";
    let runs = vec![TextRun {
        text: literal.into(),
        ..Default::default()
    }];
    assert!(validate_text_runs(&runs).is_ok());
    assert_eq!(runs[0].text, literal);
    assert!(validate_text_runs(&[]).is_ok());
    assert!(validate_text_runs(&[TextRun::default()]).is_err());
    for forbidden in [
        '\0', '\t', '\r', '\n', '\u{7f}', '\u{85}', '\u{2028}', '\u{2029}',
    ] {
        let result = validate_text_runs(&[TextRun {
            text: format!("a{forbidden}b"),
            ..Default::default()
        }]);
        assert!(result.is_err(), "forbidden literal {forbidden:?}");
        assert_eq!(result.unwrap_err().path, "/content/0/text");
    }
}

#[test]
fn empty_and_trailing_paragraphs_remain_authored_structure() {
    let content = vec![
        paragraph(vec![]),
        paragraph(vec![MTextInline::Run {
            text: "A".into(),
            character_format: Default::default(),
        }]),
        paragraph(vec![]),
    ];
    assert!(validate_mtext_content(&content, context(None), |_| true).is_ok());
    assert_eq!(content.len(), 3);
    assert!(validate_mtext_content(&[paragraph(vec![])], context(None), |_| true).is_ok());
    assert!(validate_mtext_content::<u32>(&[], context(None), |_| true).is_err());
    let empty_run = paragraph(vec![MTextInline::Run {
        text: String::new(),
        character_format: Default::default(),
    }]);
    assert!(validate_mtext_content(&[empty_run], context(None), |_| true).is_err());
}

#[test]
fn tab_reset_and_grid_do_not_restart_after_custom_stops() {
    let custom = TextTabStop {
        position_factor: 6.0,
        alignment: TextTabAlignment::Center,
    };
    let entity = ParagraphFormat {
        tab_stops: Some(vec![custom]),
        ..Default::default()
    };
    assert_eq!(
        resolve_paragraph_format(&entity, &Default::default())
            .unwrap()
            .tab_stops,
        vec![custom]
    );
    let reset = ParagraphFormat {
        tab_stops: Some(vec![]),
        ..Default::default()
    };
    assert!(resolve_paragraph_format(&entity, &reset)
        .unwrap()
        .tab_stops
        .is_empty());
    assert_eq!(next_tab_stop(&[custom], 0.0).unwrap().position_factor, 6.0);
    assert_eq!(next_tab_stop(&[custom], 6.0).unwrap().position_factor, 8.0);
    for (position, expected_absolute) in [(0.0, 13.0), (4.0, 23.0), (8.0, 33.0)] {
        assert_eq!(
            3.0 + 2.5 * next_tab_stop(&[], position).unwrap().position_factor,
            expected_absolute
        );
    }
    assert!(next_tab_stop(&[], f64::MAX).is_err());
}

#[test]
fn tab_stops_require_order_and_explicit_valid_decimal_separator() {
    for stops in [
        vec![TextTabStop {
            position_factor: 0.0,
            alignment: TextTabAlignment::Left,
        }],
        vec![
            TextTabStop {
                position_factor: 8.0,
                alignment: TextTabAlignment::Left,
            },
            TextTabStop {
                position_factor: 4.0,
                alignment: TextTabAlignment::Right,
            },
        ],
        vec![
            TextTabStop {
                position_factor: 4.0,
                alignment: TextTabAlignment::Left
            };
            2
        ],
        vec![TextTabStop {
            position_factor: 4.0,
            alignment: TextTabAlignment::Decimal {
                separator: '\u{202e}',
            },
        }],
    ] {
        assert!(validate_paragraph_format(&ParagraphFormat {
            tab_stops: Some(stops),
            ..Default::default()
        })
        .is_err());
    }
    for separator in [',', '.', ' ', '٫'] {
        assert!(validate_paragraph_format(&ParagraphFormat {
            tab_stops: Some(vec![TextTabStop {
                position_factor: 4.0,
                alignment: TextTabAlignment::Decimal { separator }
            }]),
            ..Default::default()
        })
        .is_ok());
    }
}

#[test]
fn columns_validate_state_without_discarding_overflow_content() {
    let columns = MTextColumns::DynamicManualHeight {
        column_width: 40.0,
        gutter: 5.0,
        column_heights: vec![
            MTextColumnHeight::Fixed { distance: 60.0 },
            MTextColumnHeight::Auto,
        ],
        flow_reversed: true,
    };
    let content = vec![paragraph(vec![
        MTextInline::ColumnBreak,
        MTextInline::ColumnBreak,
        MTextInline::ColumnBreak,
    ])];
    assert!(validate_mtext_content(&content, context(Some(&columns)), |_| true).is_ok());
    assert!(validate_mtext_content(&content, context(None), |_| true).is_err());
    let conflict = MTextValueContext {
        wrap_width: Some(80.0),
        ..context(Some(&columns))
    };
    assert!(validate_mtext_content(&content, conflict, |_| true).is_err());
    let bad = MTextColumns::DynamicManualHeight {
        column_width: 40.0,
        gutter: 5.0,
        column_heights: vec![
            MTextColumnHeight::Auto,
            MTextColumnHeight::Fixed { distance: 60.0 },
        ],
        flow_reversed: false,
    };
    assert!(validate_mtext_content(&content, context(Some(&bad)), |_| true).is_err());
    let fixed = MTextColumns::DynamicManualHeight {
        column_width: 40.0,
        gutter: 0.0,
        column_heights: vec![MTextColumnHeight::Fixed { distance: 1.0 }],
        flow_reversed: false,
    };
    assert!(validate_mtext_content(&content, context(Some(&fixed)), |_| true).is_ok());
}

#[test]
fn usable_width_and_derived_products_are_checked_without_font_metrics() {
    let mut p = paragraph(vec![MTextInline::Run {
        text: "long text may overflow the font-dependent layout".into(),
        character_format: Default::default(),
    }]);
    p.paragraph_format = ParagraphFormat {
        left_indent_factor: Some(2.0),
        first_line_indent_factor: Some(-1.0),
        ..Default::default()
    };
    let ctx = MTextValueContext {
        nominal_height: 2.5,
        wrap_width: Some(10.0),
        columns: None,
    };
    assert!(validate_mtext_content(&[p.clone()], ctx, |_| true).is_ok());
    p.paragraph_format.right_indent_factor = Some(2.0);
    assert!(validate_mtext_content(&[p], ctx, |_| true).is_err());
    let entity = CharacterFormat::<u32> {
        height: Some(TextHeight::Relative { factor: f64::MAX }),
        ..Default::default()
    };
    assert!(resolve_character_format(
        2.5,
        &style(),
        &entity,
        &Default::default(),
        &Default::default()
    )
    .is_err());
}

#[test]
fn stack_parts_and_effective_height_have_explicit_semantics() {
    let mut stack = TextStack::<u32> {
        upper: "1".into(),
        lower: "2".into(),
        ..Default::default()
    };
    stack.character_format.height = Some(TextHeight::Absolute { distance: 2.5 });
    assert!(validate_mtext_content(
        &[paragraph(vec![MTextInline::Stack(stack.clone())])],
        context(None),
        |_| true
    )
    .is_ok());
    let effective = resolve_character_format(
        9.0,
        &style(),
        &Default::default(),
        &Default::default(),
        &stack.character_format,
    )
    .unwrap();
    assert_eq!(effective.height * stack.text_scale, 1.75);
    stack.upper.clear();
    stack.lower.clear();
    assert!(validate_mtext_content(
        &[paragraph(vec![MTextInline::Stack(stack.clone())])],
        context(None),
        |_| true
    )
    .is_err());
    stack.upper = "+0.1".into();
    stack.stack_kind = TextStackKind::DecimalTolerance;
    assert!(validate_mtext_content(
        &[paragraph(vec![MTextInline::Stack(stack.clone())])],
        context(None),
        |_| true
    )
    .is_err());
    stack.separator = Some('.');
    assert!(validate_mtext_content(
        &[paragraph(vec![MTextInline::Stack(stack)])],
        context(None),
        |_| true
    )
    .is_ok());
}

#[test]
fn layout_and_style_scalars_reject_invalid_values() {
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(validate_text_layout(&TextLayout::Fit {
            length: 10.0,
            height: bad
        })
        .is_err());
        assert!(validate_text_layout(&TextLayout::Aligned {
            length: bad,
            width_factor: 1.0
        })
        .is_err());
        let mut invalid = style();
        invalid.width_factor = bad;
        assert!(validate_text_style(&invalid).is_err());
    }
    let mut base = style();
    base.last_used_height = Some(0.0);
    assert!(validate_text_style(&base).is_ok());
    base.creation_height = Some(0.0);
    assert!(validate_text_style(&base).is_err());
    base.creation_height = None;
    base.oblique_angle = std::f64::consts::FRAC_PI_2;
    assert!(validate_text_style(&base).is_err());
    base.font = FontRequest::default();
    assert!(validate_text_style(&base).is_err());
}

#[test]
fn color_validation_is_route_owned_and_background_padding_may_be_zero() {
    let c = CharacterFormat {
        color: Some(TextColor::Explicit(999u32)),
        ..Default::default()
    };
    assert!(validate_character_format(&c, |value| *value <= 255).is_err());
    assert!(validate_character_format(&c, |_| true).is_ok());
    let mut bg = MTextBackground::<u32> {
        fill: MTextFill::Canvas,
        padding: TextPadding::Absolute { distance: 0.0 },
        ..Default::default()
    };
    assert!(validate_background(&bg, |_| true).is_ok());
    bg.opacity = 1.001;
    assert!(validate_background(&bg, |_| true).is_err());
}

#[test]
fn entity_paragraph_basis_is_used_before_usable_width_validation() {
    let basis = ParagraphFormat {
        left_indent_factor: Some(-3.0),
        ..Default::default()
    };
    let mut p = paragraph(vec![]);
    p.paragraph_format.right_indent_factor = Some(4.0);
    let ctx = MTextValueContext {
        nominal_height: 2.5,
        wrap_width: Some(10.0),
        columns: None,
    };
    assert!(
        validate_mtext_content_with_paragraph_format(&[p.clone()], ctx, &basis, |_| true).is_ok()
    );
    assert!(
        validate_mtext_content_with_paragraph_format(&[p], ctx, &Default::default(), |_| true)
            .is_err()
    );
}

#[test]
fn effective_format_validation_checks_empty_paragraphs_and_stack_products() {
    let entity = CharacterFormat::<u32> {
        height: Some(TextHeight::Relative { factor: f64::MAX }),
        ..Default::default()
    };
    assert!(validate_mtext_effective_formats(
        &[paragraph(vec![])],
        context(None),
        &style(),
        &entity,
        &Default::default(),
        |_| true
    )
    .is_err());
    let safe = CharacterFormat {
        height: Some(TextHeight::Absolute { distance: 2.5 }),
        ..Default::default()
    };
    let mut stack = TextStack {
        upper: "1".into(),
        text_scale: f64::MAX,
        ..Default::default()
    };
    stack.character_format = safe.clone();
    assert!(validate_mtext_effective_formats(
        &[paragraph(vec![MTextInline::Stack(stack)])],
        context(None),
        &style(),
        &safe,
        &Default::default(),
        |_| true
    )
    .is_err());
}

#[test]
fn overridden_relative_height_is_not_evaluated_as_effective_geometry() {
    let entity = CharacterFormat::<u32> {
        height: Some(TextHeight::Relative { factor: f64::MAX }),
        ..Default::default()
    };
    let inline = CharacterFormat {
        height: Some(TextHeight::Absolute { distance: 4.0 }),
        ..Default::default()
    };
    assert_eq!(
        resolve_character_format(2.5, &style(), &entity, &Default::default(), &inline)
            .unwrap()
            .height,
        4.0
    );
}

#[test]
fn nonempty_paragraph_validates_final_inline_height_without_unused_base_extent() {
    let entity = CharacterFormat::<u32> {
        height: Some(TextHeight::Relative { factor: f64::MAX }),
        ..Default::default()
    };
    let inline = CharacterFormat {
        height: Some(TextHeight::Absolute { distance: 4.0 }),
        ..Default::default()
    };
    let content = vec![paragraph(vec![MTextInline::Run {
        text: "safe".into(),
        character_format: inline,
    }])];
    assert!(validate_mtext_effective_formats(
        &content,
        context(None),
        &style(),
        &entity,
        &Default::default(),
        |_| true
    )
    .is_ok());
    let mut with_empty_line = content;
    with_empty_line[0].inlines.insert(0, MTextInline::LineBreak);
    assert!(validate_mtext_effective_formats(
        &with_empty_line,
        context(None),
        &style(),
        &entity,
        &Default::default(),
        |_| true
    )
    .is_err());
}
