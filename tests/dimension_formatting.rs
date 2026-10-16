use ocdraw::dimension::*;
use ocdraw::text::TextRun;

fn fragment_text(label: &DimensionFormattedLabel) -> String {
    label
        .fragments
        .iter()
        .map(|part| match part {
            DimensionFormattedFragment::Run(run) => run.text.clone(),
            DimensionFormattedFragment::Fraction {
                numerator,
                denominator,
                ..
            } => format!("{numerator}/{denominator}"),
        })
        .collect()
}

#[test]
fn architectural_fragments_preserve_layout_and_scale() {
    for layout in [
        DimensionFractionLayout::Inline,
        DimensionFractionLayout::Horizontal,
        DimensionFractionLayout::Diagonal,
    ] {
        let mut settings = DimensionNumberSettings::default();
        settings.primary.format = DimensionDistanceFormat::Architectural;
        settings.primary.precision = 2;
        settings.primary.fraction_layout = layout;
        settings.primary.fraction_scale = 0.75;
        let label = formatted(48.25, &settings, false, &DimensionLabel::Generated);
        assert_eq!(label.primary, "4'-0 1/4\"");
        assert_eq!(
            fragment_text(&label),
            label
                .runs
                .iter()
                .map(|r| r.text.as_str())
                .collect::<String>()
        );
        let stacks: Vec<_> = label
            .fragments
            .iter()
            .filter(|part| matches!(part, DimensionFormattedFragment::Fraction { .. }))
            .collect();
        if layout == DimensionFractionLayout::Inline {
            assert!(stacks.is_empty());
        } else {
            assert!(
                matches!(stacks.as_slice(), [DimensionFormattedFragment::Fraction { numerator, denominator, layout: actual, text_scale }] if numerator == "1" && denominator == "4" && *actual == layout && *text_scale == 0.75)
            );
        }
    }
}

#[test]
fn fraction_fragments_round_before_splitting() {
    let mut settings = DimensionNumberSettings::default();
    settings.primary.format = DimensionDistanceFormat::Architectural;
    settings.primary.precision = 2;
    settings.primary.fraction_layout = DimensionFractionLayout::Horizontal;
    for value in [11.99, 0.0, -0.01] {
        let label = formatted(value, &settings, false, &DimensionLabel::Generated);
        assert!(!label
            .fragments
            .iter()
            .any(|f| matches!(f, DimensionFormattedFragment::Fraction { .. })));
        assert_eq!(fragment_text(&label), label.primary);
    }
    assert_eq!(
        formatted(11.99, &settings, false, &DimensionLabel::Generated).primary,
        "1'-0\""
    );
    settings.primary.format = DimensionDistanceFormat::Fractional;
    let label = formatted(-0.25, &settings, false, &DimensionLabel::Generated);
    assert_eq!(fragment_text(&label), "-1/4");
}

#[test]
fn literal_fraction_text_is_not_reinterpreted() {
    let mut settings = DimensionNumberSettings::default();
    settings.primary.format = DimensionDistanceFormat::Fractional;
    settings.primary.fraction_layout = DimensionFractionLayout::Diagonal;
    let literal = TextRun {
        text: "1/4 <script> \\S1/2;".into(),
        ..Default::default()
    };
    let definition = DimensionLabel::Template(vec![
        DimensionLabelPart::Literal(vec![literal.clone()]),
        DimensionLabelPart::Measurement,
    ]);
    let label = formatted(0.25, &settings, false, &definition);
    assert_eq!(
        label.fragments.first(),
        Some(&DimensionFormattedFragment::Run(literal))
    );
    assert_eq!(
        fragment_text(&label),
        label
            .runs
            .iter()
            .map(|r| r.text.as_str())
            .collect::<String>()
    );
    assert!(
        formatted(0.25, &settings, false, &DimensionLabel::Suppressed)
            .fragments
            .is_empty()
    );
}

#[test]
fn alternate_fraction_fragments_keep_independent_settings() {
    let mut settings = DimensionNumberSettings::default();
    settings.primary.format = DimensionDistanceFormat::Fractional;
    settings.primary.fraction_layout = DimensionFractionLayout::Diagonal;
    settings.primary.fraction_scale = 0.75;
    settings.alternate = Some(DimensionAlternateNumberSettings {
        factor: 2.0,
        format: DimensionDistanceFormat::Fractional,
        fraction_layout: DimensionFractionLayout::Horizontal,
        fraction_scale: 0.8,
        ..Default::default()
    });
    let label = formatted(0.25, &settings, false, &DimensionLabel::Generated);
    assert_eq!(fragment_text(&label), "1/4 [1/2]");
    assert_eq!(
        label
            .fragments
            .iter()
            .filter(|f| matches!(f, DimensionFormattedFragment::Fraction { .. }))
            .count(),
        2
    );
    assert!(label.fragments.iter().any(|f| matches!(f, DimensionFormattedFragment::Fraction {layout: DimensionFractionLayout::Horizontal, text_scale, ..} if *text_scale == 0.8)));
}

fn formatted(
    value: f64,
    format: &DimensionNumberSettings,
    paper: bool,
    label: &DimensionLabel,
) -> DimensionFormattedLabel {
    format_dimension_label(
        DimensionMeasurement::Distance(value),
        format,
        DimensionFormatContext { is_paper: paper },
        label,
    )
    .unwrap()
}

#[test]
fn measure_factor_paper_application_and_rounding_are_explicit() {
    let mut settings = DimensionNumberSettings::default();
    settings.primary.measurement_factor = 0.001;
    assert_eq!(
        formatted(4000.0, &settings, false, &DimensionLabel::Generated).primary,
        "4.00"
    );
    settings.primary.measurement_factor = 2.0;
    settings.primary.application = DimensionFactorApplication::PaperOnly;
    assert_eq!(
        formatted(4000.0, &settings, false, &DimensionLabel::Generated).primary,
        "4000.00"
    );
    assert_eq!(
        formatted(4000.0, &settings, true, &DimensionLabel::Generated).primary,
        "8000.00"
    );
    settings.primary.measurement_factor = 1.0;
    settings.primary.precision = 1;
    assert_eq!(
        formatted(1.25, &settings, false, &DimensionLabel::Generated).primary,
        "1.3"
    );
    assert_eq!(
        formatted(-1.25, &settings, false, &DimensionLabel::Generated).primary,
        "-1.3"
    );
    assert_eq!(
        formatted(-0.0, &settings, false, &DimensionLabel::Generated).primary,
        "0.0"
    );
}

#[test]
fn label_tokens_preserve_literal_text_and_suppression() {
    let settings = DimensionNumberSettings::default();
    let literal = |text: &str| {
        DimensionLabelPart::Literal(vec![TextRun {
            text: text.into(),
            ..Default::default()
        }])
    };
    let template = DimensionLabel::Template(vec![
        literal("ca. "),
        DimensionLabelPart::Measurement,
        literal(" mm <>"),
    ]);
    let output = formatted(4000.0, &settings, false, &template);
    assert_eq!(
        output
            .runs
            .iter()
            .map(|r| r.text.as_str())
            .collect::<String>(),
        "ca. 4000.00 mm <>"
    );
    let literal_only = formatted(
        4500.0,
        &settings,
        false,
        &DimensionLabel::Template(vec![literal("ca. 4000")]),
    );
    assert_eq!(literal_only.runs[0].text, "ca. 4000");
    let suppressed = formatted(4000.0, &settings, false, &DimensionLabel::Suppressed);
    assert_eq!(suppressed.state, DimensionFormattedLabelState::Suppressed);
    assert!(suppressed.runs.is_empty());
}

#[test]
fn invalid_formatting_and_factor_overflow_never_become_truncated_text() {
    let mut settings = DimensionNumberSettings::default();
    settings.primary.precision = 17;
    assert!(format_dimension_label(
        DimensionMeasurement::Distance(1.0),
        &settings,
        DimensionFormatContext { is_paper: false },
        &DimensionLabel::Generated
    )
    .is_err());
    settings.primary.precision = 2;
    settings.primary.measurement_factor = -1.0;
    assert!(format_dimension_label(
        DimensionMeasurement::Distance(1.0),
        &settings,
        DimensionFormatContext { is_paper: false },
        &DimensionLabel::Generated
    )
    .is_err());
    settings.primary.measurement_factor = 2.0;
    assert!(format_dimension_label(
        DimensionMeasurement::Distance(f64::MAX),
        &settings,
        DimensionFormatContext { is_paper: false },
        &DimensionLabel::Generated
    )
    .is_err());
}

#[test]
fn suppressed_and_literal_only_labels_do_not_require_unused_number_formatting() {
    let mut settings = DimensionNumberSettings::default();
    settings.primary.measurement_factor = 2.0;
    let suppressed = formatted(f64::MAX, &settings, false, &DimensionLabel::Suppressed);
    assert_eq!(suppressed.state, DimensionFormattedLabelState::Suppressed);
    assert!(suppressed.runs.is_empty());
    let runs = vec![TextRun {
        text: "Literal <> %<field>%".into(),
        ..Default::default()
    }];
    let label = formatted(
        f64::MAX,
        &settings,
        false,
        &DimensionLabel::Template(vec![DimensionLabelPart::Literal(runs.clone())]),
    );
    assert_eq!(label.runs, runs);
    assert_eq!(label.state, DimensionFormattedLabelState::Visible);
}

#[test]
fn fraction_and_feet_inch_carry_have_defined_output() {
    let mut settings = DimensionNumberSettings::default();
    settings.primary.format = DimensionDistanceFormat::Fractional;
    settings.primary.precision = 3;
    assert_eq!(
        formatted(0.625, &settings, false, &DimensionLabel::Generated).primary,
        "5/8"
    );
    settings.primary.format = DimensionDistanceFormat::Architectural;
    assert_eq!(
        formatted(11.99, &settings, false, &DimensionLabel::Generated).primary,
        "1'-0\""
    );
    settings.primary.format = DimensionDistanceFormat::Scientific;
    settings.primary.precision = 2;
    assert_eq!(
        formatted(12.5, &settings, false, &DimensionLabel::Generated).primary,
        "1.25E+01"
    );
}

#[test]
fn alternate_factor_and_tolerances_apply_to_the_displayed_primary_value() {
    let mut settings = DimensionNumberSettings::default();
    settings.primary.measurement_factor = 0.001;
    settings.alternate = Some(DimensionAlternateNumberSettings {
        factor: 1000.0,
        ..Default::default()
    });
    settings.tolerance = DimensionToleranceSettings::Deviations {
        upper: 0.05,
        lower: 0.02,
        precision: 2,
    };
    let result = formatted(4000.0, &settings, false, &DimensionLabel::Generated);
    assert_eq!(result.primary, "4.00");
    assert_eq!(result.alternate.as_deref(), Some("4000.00"));
    let text = result
        .runs
        .iter()
        .map(|r| r.text.as_str())
        .collect::<String>();
    assert!(text.contains("+0.05"));
    assert!(text.contains("-0.02"));
    assert!(text.contains("+50.00"));
    assert!(text.contains("-20.00"));
}
