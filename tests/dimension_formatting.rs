use ocdraw::dimension::*;
use ocdraw::text::TextRun;

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
