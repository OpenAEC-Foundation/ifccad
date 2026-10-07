use ocdraw::geometry_kernel::{CoordinateFrame3, Point3, Vector3};
use ocdraw::text::*;

fn text<'a>(content: &'a [TextRun]) -> ResolvedTextExtentInput<'a> {
    ResolvedTextExtentInput {
        content,
        layout: TextLayout::Anchored {
            horizontal: TextHorizontalAlignment::Left,
            vertical: TextVerticalAlignment::Baseline,
            height: 2.5,
            width_factor: 1.0,
        },
        placement: CoordinateFrame3::default(),
        rotation: 0.,
        backward: false,
        upside_down: false,
        oblique_angle: 0.,
        thickness: 0.,
        vertical: false,
    }
}

fn mtext<'a>(
    content: &'a [MTextParagraph<u32>],
    style: &'a TextStyleProperties,
    character: &'a CharacterFormat<u32>,
    paragraph: &'a ParagraphFormat,
) -> ResolvedMTextExtentInput<'a, u32> {
    ResolvedMTextExtentInput {
        content,
        style,
        character_format: character,
        paragraph_format: paragraph,
        height: 2.5,
        attachment: MTextAttachment::TopLeft,
        flow: MTextFlow::Horizontal,
        wrap_width: None,
        columns: None,
        background: None,
        frame_stroke_width: None,
        placement: CoordinateFrame3::default(),
        rotation: 0.,
        backward: false,
        upside_down: false,
    }
}

#[test]
fn fontless_estimate_never_certifies_glyph_enclosure() {
    for literal in ["AB", "  ", "e\u{301}", "漢字", "אב", "😀"] {
        let content = [TextRun {
            text: literal.into(),
            ..Default::default()
        }];
        let estimate = estimate_text_extent(&text(&content)).unwrap();
        assert_eq!(estimate.status, TextExtentStatus::Estimated);
        assert!(estimate
            .reasons
            .contains(&TextExtentReason::FontMetricsUnavailable));
        let bounds = estimate.local_bounds.unwrap();
        assert_eq!(bounds.max().x(), 5. * literal.chars().count() as f64);
        assert_eq!(bounds.min().y(), -5.);
        assert_eq!(bounds.max().y(), 5.);
    }
    let empty = estimate_text_extent(&text(&[])).unwrap();
    assert_eq!(empty.status, TextExtentStatus::Empty);
    assert!(empty.bounds.is_none());
}

#[test]
fn alignment_mirror_rotation_and_thickness_keep_the_active_anchor() {
    let content = [TextRun {
        text: "AB".into(),
        ..Default::default()
    }];
    let mut input = text(&content);
    input.layout = TextLayout::Anchored {
        horizontal: TextHorizontalAlignment::Right,
        vertical: TextVerticalAlignment::Baseline,
        height: 2.5,
        width_factor: 1.,
    };
    input.backward = true;
    input.thickness = -3.;
    input.placement = CoordinateFrame3::try_new(
        Point3::new(100., 50., 7.),
        Vector3::new(1., 0., 0.),
        Vector3::new(0., 1., 0.),
    )
    .unwrap();
    let estimate = estimate_text_extent(&input).unwrap();
    let local = estimate.local_bounds.unwrap();
    assert_eq!(local.min().components(), [-10., -5., -3.]);
    assert_eq!(local.max().components(), [0., 5., 0.]);
    let placed = estimate.bounds.unwrap();
    for (actual, expected) in placed.min().components().into_iter().zip([100., 45., 4.]) {
        assert!(actual <= expected && expected - actual < 1e-12);
    }
    for (actual, expected) in placed.max().components().into_iter().zip([110., 55., 7.]) {
        assert!(actual >= expected && actual - expected < 1e-12);
    }
    input.rotation = std::f64::consts::FRAC_PI_2;
    let rotated = estimate_text_extent(&input).unwrap().bounds.unwrap();
    assert!(rotated.min().x() <= 95. && rotated.max().x() >= 105.);
    assert!(rotated.min().y() <= 50. && rotated.max().y() >= 60.);
}

#[test]
fn aligned_and_fit_estimates_do_not_store_redundant_parameters_or_divide_by_zero() {
    let content = [TextRun {
        text: "AB".into(),
        ..Default::default()
    }];
    let mut input = text(&content);
    input.layout = TextLayout::Aligned {
        length: 8.,
        width_factor: 1.,
    };
    let aligned = estimate_text_extent(&input).unwrap().local_bounds.unwrap();
    assert_eq!(aligned.max().x(), 8.);
    assert_eq!(aligned.min().y(), -4.);
    input.layout = TextLayout::Fit {
        length: 8.,
        height: 2.5,
    };
    let fit = estimate_text_extent(&input).unwrap().local_bounds.unwrap();
    assert_eq!(fit.max().x(), 8.);
    assert_eq!(fit.min().y(), -5.);
    input.content = &[];
    let empty = estimate_text_extent(&input).unwrap();
    assert_eq!(empty.status, TextExtentStatus::Empty);
    assert!(empty.reasons.contains(&TextExtentReason::ZeroAdvance));
}

#[test]
fn empty_mtext_and_background_have_layout_extents() {
    let content = [MTextParagraph::<u32>::default()];
    let style = TextStyleProperties::new(FontRequest::family("Arial"));
    let c = CharacterFormat::default();
    let p = ParagraphFormat::default();
    let mut input = mtext(&content, &style, &c, &p);
    input.wrap_width = Some(80.);
    let bg = MTextBackground {
        fill: MTextFill::Canvas,
        padding: TextPadding::Relative { factor: 0.5 },
        ..Default::default()
    };
    input.background = Some(&bg);
    let result = estimate_mtext_extent(&input).unwrap();
    assert_eq!(result.status, TextExtentStatus::Estimated);
    let bounds = result.local_bounds.unwrap();
    assert_eq!(bounds.min().x(), -1.25);
    assert_eq!(bounds.max().x(), 81.25);
    assert!(bounds.min().y() < 0. && bounds.max().y() > 0.);
    let frame = MTextBackground {
        fill: MTextFill::None,
        frame: true,
        ..Default::default()
    };
    input.background = Some(&frame);
    assert_eq!(
        estimate_mtext_extent(&input).unwrap().status,
        TextExtentStatus::Unavailable
    );
    input.frame_stroke_width = Some(0.5);
    assert_eq!(
        estimate_mtext_extent(&input).unwrap().status,
        TextExtentStatus::Estimated
    );
}

#[test]
fn tab_fields_cross_runs_and_preserve_decimal_alignment() {
    let content = [MTextParagraph {
        inlines: vec![
            MTextInline::Tab,
            MTextInline::Run {
                text: "12".into(),
                character_format: Default::default(),
            },
            MTextInline::Run {
                text: ",5".into(),
                character_format: CharacterFormat {
                    underline: Some(true),
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }];
    let style = TextStyleProperties::new(FontRequest::family("Arial"));
    let c = CharacterFormat::default();
    let p = ParagraphFormat {
        tab_stops: Some(vec![TextTabStop {
            position_factor: 8.,
            alignment: TextTabAlignment::Decimal { separator: ',' },
        }]),
        ..Default::default()
    };
    let input = mtext(&content, &style, &c, &p);
    let result = estimate_mtext_extent(&input).unwrap();
    assert_eq!(
        result.local_bounds.unwrap().max().x(),
        30.,
        "decimal at 20, suffix advances another 10"
    );
}

#[test]
fn wrapping_and_column_overflow_do_not_clip_or_rewrite_content() {
    let content = [MTextParagraph {
        inlines: vec![MTextInline::Run {
            text: "unbreakable".into(),
            character_format: Default::default(),
        }],
        ..Default::default()
    }];
    let saved = content.clone();
    let style = TextStyleProperties::new(FontRequest::family("Arial"));
    let c = CharacterFormat::default();
    let p = ParagraphFormat::default();
    let mut input = mtext(&content, &style, &c, &p);
    input.wrap_width = Some(10.);
    let result = estimate_mtext_extent(&input).unwrap();
    assert!(result.local_bounds.unwrap().max().x() >= 55.);
    assert!(result.reasons.contains(&TextExtentReason::Overflow));
    let columns = MTextColumns::Static {
        count: 1,
        column_width: 10.,
        gutter: 0.,
        column_height: 1.,
        flow_reversed: false,
    };
    input.wrap_width = None;
    input.columns = Some(&columns);
    let result = estimate_mtext_extent(&input).unwrap();
    assert!(result.reasons.contains(&TextExtentReason::Overflow));
    assert!(result.local_bounds.unwrap().min().y() < -1.);
    assert_eq!(content, saved);
}

#[test]
fn extent_numeric_failure_is_not_misreported_as_unknown_font_geometry() {
    let content = [TextRun {
        text: "AB".into(),
        ..Default::default()
    }];
    let mut input = text(&content);
    input.layout = TextLayout::Fit {
        length: 1.,
        height: f64::MAX,
    };
    assert!(estimate_text_extent(&input).is_err());
    input = text(&content);
    input.rotation = f64::NAN;
    assert!(estimate_text_extent(&input).is_err());
}

#[test]
fn moving_a_tab_field_to_the_next_column_keeps_its_stop_alignment() {
    let content = [MTextParagraph {
        inlines: vec![
            MTextInline::Run {
                text: "A".into(),
                character_format: Default::default(),
            },
            MTextInline::LineBreak,
            MTextInline::Tab,
            MTextInline::Run {
                text: "12,5".into(),
                character_format: Default::default(),
            },
        ],
        ..Default::default()
    }];
    let style = TextStyleProperties::new(FontRequest::family("Arial"));
    let c = CharacterFormat::default();
    let p = ParagraphFormat {
        tab_stops: Some(vec![TextTabStop {
            position_factor: 20.,
            alignment: TextTabAlignment::Decimal { separator: ',' },
        }]),
        ..Default::default()
    };
    let columns = MTextColumns::Static {
        count: 2,
        column_width: 40.,
        gutter: 5.,
        column_height: 7.5,
        flow_reversed: false,
    };
    let mut input = mtext(&content, &style, &c, &p);
    input.columns = Some(&columns);
    let estimate = estimate_mtext_extent(&input).unwrap();
    assert_eq!(
        estimate.local_bounds.unwrap().max().x(),
        105.,
        "column origin45 + decimal stop50 + suffix10"
    );
}

#[test]
fn paragraph_alignment_keeps_sheared_ink_beyond_the_layout_width() {
    let content = [MTextParagraph {
        inlines: vec![MTextInline::Run {
            text: "A".into(),
            character_format: CharacterFormat {
                oblique_angle: Some(std::f64::consts::FRAC_PI_4),
                ..Default::default()
            },
        }],
        ..Default::default()
    }];
    let style = TextStyleProperties::new(FontRequest::family("Arial"));
    let c = CharacterFormat::default();
    let p = ParagraphFormat {
        alignment: Some(TextParagraphAlignment::Right),
        ..Default::default()
    };
    let mut input = mtext(&content, &style, &c, &p);
    input.wrap_width = Some(100.);
    assert!(
        estimate_mtext_extent(&input)
            .unwrap()
            .local_bounds
            .unwrap()
            .max()
            .x()
            > 104.9
    );
}

#[test]
fn empty_line_column_overflow_and_large_counts_do_not_allocate_columns() {
    let content = [MTextParagraph::<u32>::default()];
    let style = TextStyleProperties::new(FontRequest::family("Arial"));
    let c = CharacterFormat::default();
    let p = ParagraphFormat::default();
    let columns = MTextColumns::Static {
        count: u32::MAX,
        column_width: 1.,
        gutter: 0.,
        column_height: 1.,
        flow_reversed: true,
    };
    let mut input = mtext(&content, &style, &c, &p);
    input.columns = Some(&columns);
    let estimate = estimate_mtext_extent(&input).unwrap();
    assert_eq!(
        estimate.local_bounds.unwrap().max().x(),
        f64::from(u32::MAX)
    );
    assert!(estimate.reasons.contains(&TextExtentReason::Overflow));
}

#[test]
fn dynamic_column_state_vertical_flow_and_attachment_are_retained() {
    let content = [MTextParagraph::<u32>::default()];
    let style = TextStyleProperties::new(FontRequest::family("Arial"));
    let c = CharacterFormat::default();
    let p = ParagraphFormat::default();
    let columns = [
        MTextColumns::DynamicAutoHeight {
            column_width: 40.,
            gutter: 5.,
            column_height: 60.,
            current_column_count: 3,
            flow_reversed: false,
        },
        MTextColumns::DynamicManualHeight {
            column_width: 40.,
            gutter: 5.,
            column_heights: vec![
                MTextColumnHeight::Fixed { distance: 60. },
                MTextColumnHeight::Fixed { distance: 45. },
                MTextColumnHeight::Auto,
            ],
            flow_reversed: true,
        },
    ];
    for config in &columns {
        let mut input = mtext(&content, &style, &c, &p);
        input.columns = Some(config);
        input.attachment = MTextAttachment::BottomRight;
        let saved = config.clone();
        let bounds = estimate_mtext_extent(&input).unwrap().local_bounds.unwrap();
        assert_eq!(bounds.min().x(), -130.);
        assert_eq!(bounds.max().x(), 0.);
        assert_eq!(bounds.min().y(), 0.);
        assert_eq!(bounds.max().y(), 60.);
        assert_eq!(*config, saved);
    }
    let mut input = mtext(&content, &style, &c, &p);
    input.flow = MTextFlow::Vertical;
    input.wrap_width = Some(80.);
    let estimate = estimate_mtext_extent(&input).unwrap();
    assert!(estimate
        .reasons
        .contains(&TextExtentReason::VerticalFlowEstimated));
    assert_eq!(estimate.local_bounds.unwrap().min().y(), -80.);
    assert_eq!(input.flow, MTextFlow::Vertical);
}

#[test]
fn aligned_derived_height_underflow_is_an_explicit_numeric_failure() {
    let content = [TextRun {
        text: "A".into(),
        ..Default::default()
    }];
    let mut input = text(&content);
    input.layout = TextLayout::Aligned {
        length: f64::from_bits(1),
        width_factor: 1.,
    };
    assert!(estimate_text_extent(&input).is_err());
}

#[test]
fn empty_paragraph_extent_uses_its_effective_character_height() {
    let content = [MTextParagraph::<u32> {
        character_format: CharacterFormat {
            height: Some(TextHeight::Absolute { distance: 4. }),
            ..Default::default()
        },
        ..Default::default()
    }];
    let style = TextStyleProperties::new(FontRequest::family("Arial"));
    let c = CharacterFormat::default();
    let p = ParagraphFormat::default();
    assert_eq!(
        estimate_mtext_extent(&mtext(&content, &style, &c, &p))
            .unwrap()
            .local_bounds
            .unwrap()
            .min()
            .y(),
        -12.
    );
}
