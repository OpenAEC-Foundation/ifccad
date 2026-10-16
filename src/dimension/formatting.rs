use super::*;
use crate::{geometry_kernel::numeric::exact, text::TextRun};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, Zero};

fn ten(power: u32) -> BigInt {
    BigInt::from(10_u8).pow(power)
}
fn round_integer(value: &BigRational) -> BigInt {
    let n = value.numer().abs();
    let d = value.denom();
    let mut whole = &n / d;
    if (&n % d) * 2 >= *d {
        whole += 1;
    }
    if value.is_negative() {
        -whole
    } else {
        whole
    }
}
fn checked(value: BigRational) -> Result<BigRational, DimensionValueError> {
    if value.abs() > exact(f64::MAX) {
        Err(failure(
            "formattedMeasurement",
            DimensionValueReason::NumericUnavailable,
        ))
    } else {
        Ok(value)
    }
}
fn quantize(value: &BigRational, increment: f64) -> Result<BigRational, DimensionValueError> {
    if increment == 0.0 {
        return Ok(value.clone());
    }
    let step = exact(increment);
    checked(BigRational::from_integer(round_integer(&(value / &step))) * step)
}
fn decimal(
    value: &BigRational,
    precision: u8,
    separator: char,
    zero: DimensionZeroSuppression,
) -> String {
    let digits = round_integer(&(value * BigRational::from_integer(ten(precision as u32))));
    let sign = if digits.is_negative() { "-" } else { "" };
    let mut body = digits.abs().to_string();
    if precision > 0 {
        let width = precision as usize + 1;
        if body.len() < width {
            body = format!("{}{}", "0".repeat(width - body.len()), body);
        }
        body.insert(body.len() - precision as usize, '.');
        if zero.trailing {
            while body.ends_with('0') {
                body.pop();
            }
            if body.ends_with('.') {
                body.pop();
            }
        }
        if zero.leading && body.starts_with("0.") {
            body.remove(0);
        }
    }
    format!("{sign}{}", body.replace('.', &separator.to_string()))
}
fn run(text: String) -> DimensionFormattedFragment {
    DimensionFormattedFragment::Run(TextRun {
        text,
        ..Default::default()
    })
}
fn push_text(parts: &mut Vec<DimensionFormattedFragment>, text: String) {
    if !text.is_empty() {
        parts.push(run(text));
    }
}
fn plain_fragments(parts: &[DimensionFormattedFragment]) -> String {
    parts
        .iter()
        .map(|part| match part {
            DimensionFormattedFragment::Run(r) => r.text.clone(),
            DimensionFormattedFragment::Fraction {
                numerator,
                denominator,
                ..
            } => format!("{numerator}/{denominator}"),
        })
        .collect()
}
fn fractional_fragments(
    value: &BigRational,
    precision: u8,
    layout: DimensionFractionLayout,
    scale: f64,
) -> Vec<DimensionFormattedFragment> {
    let denominator = BigInt::one() << precision as usize;
    let count = round_integer(&(value * BigRational::from_integer(denominator.clone())));
    let negative = count.is_negative();
    let count = count.abs();
    let whole = &count / &denominator;
    let fraction = BigRational::new(&count % &denominator, denominator);
    let mut prefix = if negative {
        "-".to_owned()
    } else {
        String::new()
    };
    if fraction.is_zero() {
        return vec![run(format!("{prefix}{whole}"))];
    }
    if !whole.is_zero() {
        prefix.push_str(&format!("{whole} "));
    }
    if layout == DimensionFractionLayout::Inline {
        return vec![run(format!(
            "{prefix}{}/{}",
            fraction.numer(),
            fraction.denom()
        ))];
    }
    let mut parts = Vec::new();
    push_text(&mut parts, prefix);
    parts.push(DimensionFormattedFragment::Fraction {
        numerator: fraction.numer().to_string(),
        denominator: fraction.denom().to_string(),
        layout,
        text_scale: scale,
    });
    parts
}
fn fractional(value: &BigRational, precision: u8) -> String {
    plain_fragments(&fractional_fragments(
        value,
        precision,
        DimensionFractionLayout::Inline,
        1.0,
    ))
}
fn scientific(
    value: &BigRational,
    precision: u8,
    separator: char,
    zero: DimensionZeroSuppression,
) -> String {
    if value.is_zero() {
        return format!("{}E+00", decimal(value, precision, separator, zero));
    }
    let magnitude = value.abs();
    let mut exponent =
        magnitude.numer().to_string().len() as i32 - magnitude.denom().to_string().len() as i32;
    let power = |e: i32| {
        if e >= 0 {
            BigRational::from_integer(ten(e as u32))
        } else {
            BigRational::new(1.into(), ten((-e) as u32))
        }
    };
    if magnitude < power(exponent) {
        exponent -= 1;
    }
    let mut mantissa = value / power(exponent);
    if round_integer(&(mantissa.abs() * BigRational::from_integer(ten(precision as u32))))
        >= ten(precision as u32 + 1)
    {
        exponent += 1;
        mantissa = value / power(exponent);
    }
    format!(
        "{}E{}{e:02}",
        decimal(&mantissa, precision, separator, zero),
        if exponent < 0 { "-" } else { "+" },
        e = exponent.abs()
    )
}
fn feet_inches_fragments(
    value: &BigRational,
    format: DimensionDistanceFormat,
    precision: u8,
    separator: char,
    zero: DimensionZeroSuppression,
    layout: DimensionFractionLayout,
    scale: f64,
) -> Vec<DimensionFormattedFragment> {
    let step = match format {
        DimensionDistanceFormat::Architectural => {
            BigRational::new(1.into(), BigInt::one() << precision as usize)
        }
        _ => BigRational::new(1.into(), ten(precision as u32)),
    };
    let signed_count = round_integer(&(value / &step));
    let negative = signed_count.is_negative();
    let inches = BigRational::from_integer(signed_count.abs()) * step;
    let feet = (&inches / BigRational::from_integer(12.into())).to_integer();
    let remainder = inches - BigRational::from_integer(&feet * 12);
    let foot_text = if zero.zero_feet && feet.is_zero() {
        String::new()
    } else {
        format!("{feet}'")
    };
    let mut parts = Vec::new();
    push_text(
        &mut parts,
        format!("{}{foot_text}", if negative { "-" } else { "" }),
    );
    if zero.zero_inches && remainder.is_zero() && !foot_text.is_empty() {
        return parts;
    }
    if !foot_text.is_empty() {
        push_text(&mut parts, "-".into());
    }
    if format == DimensionDistanceFormat::Architectural {
        if !foot_text.is_empty() && !remainder.is_zero() && remainder < BigRational::one() {
            push_text(&mut parts, "0 ".into());
        }
        parts.extend(fractional_fragments(&remainder, precision, layout, scale));
    } else {
        push_text(&mut parts, decimal(&remainder, precision, separator, zero));
    }
    push_text(&mut parts, "\"".into());
    parts
}
fn feet_inches(
    value: &BigRational,
    format: DimensionDistanceFormat,
    precision: u8,
    separator: char,
    zero: DimensionZeroSuppression,
) -> String {
    plain_fragments(&feet_inches_fragments(
        value,
        format,
        precision,
        separator,
        zero,
        DimensionFractionLayout::Inline,
        1.0,
    ))
}
fn number_fragments(
    value: &BigRational,
    format: DimensionDistanceFormat,
    precision: u8,
    separator: char,
    zero: DimensionZeroSuppression,
    layout: DimensionFractionLayout,
    scale: f64,
) -> Vec<DimensionFormattedFragment> {
    match format {
        DimensionDistanceFormat::Fractional => {
            fractional_fragments(value, precision, layout, scale)
        }
        DimensionDistanceFormat::Architectural => {
            feet_inches_fragments(value, format, precision, separator, zero, layout, scale)
        }
        _ => vec![run(number(value, format, precision, separator, zero))],
    }
}
fn number(
    value: &BigRational,
    format: DimensionDistanceFormat,
    precision: u8,
    separator: char,
    zero: DimensionZeroSuppression,
) -> String {
    match format {
        DimensionDistanceFormat::Decimal => decimal(value, precision, separator, zero),
        DimensionDistanceFormat::Fractional => fractional(value, precision),
        DimensionDistanceFormat::Scientific => scientific(value, precision, separator, zero),
        DimensionDistanceFormat::Engineering | DimensionDistanceFormat::Architectural => {
            feet_inches(value, format, precision, separator, zero)
        }
    }
}
fn tolerance(
    value: &BigRational,
    factor: &BigRational,
    t: &DimensionToleranceSettings,
    separator: char,
) -> Result<String, DimensionValueError> {
    match t {
        DimensionToleranceSettings::None => Ok(String::new()),
        DimensionToleranceSettings::Deviations {
            upper,
            lower,
            precision,
        } => Ok(format!(
            " +{} -{}",
            decimal(
                &checked(exact(*upper) * factor)?,
                *precision,
                separator,
                Default::default()
            ),
            decimal(
                &checked(exact(*lower) * factor)?,
                *precision,
                separator,
                Default::default()
            )
        )),
        DimensionToleranceSettings::Limits {
            upper,
            lower,
            precision,
        } => Ok(format!(
            " {} / {}",
            decimal(
                &checked(value + exact(*upper) * factor)?,
                *precision,
                separator,
                Default::default()
            ),
            decimal(
                &checked(value - exact(*lower) * factor)?,
                *precision,
                separator,
                Default::default()
            )
        )),
    }
}

/// Forms text from stored measurement/style only, without geometry or font IO.
pub fn format_dimension_label(
    measurement: DimensionMeasurement,
    format: &DimensionNumberSettings,
    context: DimensionFormatContext,
    label: &DimensionLabel,
) -> Result<DimensionFormattedLabel, DimensionValueError> {
    validate_dimension_numbers(format)?;
    let primary = &format.primary;
    let (raw, angle) = match measurement {
        DimensionMeasurement::Distance(v) => (v, false),
        DimensionMeasurement::Angle(v) => (v, true),
    };
    finite("measurement", raw)?;
    if matches!(label, DimensionLabel::Suppressed) {
        return Ok(DimensionFormattedLabel {
            state: DimensionFormattedLabelState::Suppressed,
            fragments: Vec::new(),
            runs: Vec::new(),
            primary: String::new(),
            alternate: None,
        });
    }
    if let DimensionLabel::Template(parts) = label {
        if !parts
            .iter()
            .any(|p| matches!(p, DimensionLabelPart::Measurement))
        {
            return Ok(DimensionFormattedLabel {
                state: DimensionFormattedLabelState::Visible,
                fragments: parts
                    .iter()
                    .flat_map(|p| match p {
                        DimensionLabelPart::Literal(runs) => runs.clone(),
                        DimensionLabelPart::Measurement => unreachable!(),
                    })
                    .map(DimensionFormattedFragment::Run)
                    .collect(),
                runs: parts
                    .iter()
                    .flat_map(|p| match p {
                        DimensionLabelPart::Literal(runs) => runs.clone(),
                        DimensionLabelPart::Measurement => unreachable!(),
                    })
                    .collect(),
                primary: String::new(),
                alternate: None,
            });
        }
    }
    let factor = if angle
        || (primary.application == DimensionFactorApplication::PaperOnly && !context.is_paper)
    {
        1.0
    } else {
        primary.measurement_factor
    };
    let value = checked(exact(raw) * exact(factor))?;
    let value = if angle {
        match primary.angle_format {
            DimensionAngleFormat::Radians => value,
            DimensionAngleFormat::Grads => {
                value * BigRational::from_integer(200.into()) / exact(std::f64::consts::PI)
            }
            _ => value * BigRational::from_integer(180.into()) / exact(std::f64::consts::PI),
        }
    } else {
        value
    };
    let value = checked(value)?;
    let rounded = quantize(&value, primary.rounding_increment)?;
    let primary_text =
        if angle && primary.angle_format == DimensionAngleFormat::DegreesMinutesSeconds {
            let seconds = round_integer(
                &(rounded.abs()
                    * BigRational::from_integer(3600.into())
                    * BigRational::from_integer(ten(primary.precision as u32))),
            );
            let total = BigRational::new(seconds, ten(primary.precision as u32));
            let degrees = (&total / BigRational::from_integer(3600.into())).to_integer();
            let rest = total - BigRational::from_integer(&degrees * 3600);
            let minutes = (&rest / BigRational::from_integer(60.into())).to_integer();
            let seconds = rest - BigRational::from_integer(&minutes * 60);
            format!(
                "{}{degrees}°{minutes:02}'{}\"",
                if rounded.is_negative() { "-" } else { "" },
                decimal(
                    &seconds,
                    primary.precision,
                    primary.decimal_separator,
                    primary.zero_suppression
                )
            )
        } else {
            number(
                &rounded,
                if angle {
                    DimensionDistanceFormat::Decimal
                } else {
                    primary.format
                },
                primary.precision,
                primary.decimal_separator,
                primary.zero_suppression,
            )
        };
    let primary_tolerance = tolerance(
        &value,
        &BigRational::one(),
        &format.tolerance,
        primary.decimal_separator,
    )?;
    let mut generated = format!(
        "{}{}{}{}",
        primary.prefix, primary_text, primary.suffix, primary_tolerance
    );
    let mut generated_fragments = Vec::new();
    push_text(&mut generated_fragments, primary.prefix.clone());
    if angle {
        push_text(&mut generated_fragments, primary_text.clone());
    } else {
        generated_fragments.extend(number_fragments(
            &rounded,
            primary.format,
            primary.precision,
            primary.decimal_separator,
            primary.zero_suppression,
            primary.fraction_layout,
            primary.fraction_scale,
        ));
    }
    push_text(
        &mut generated_fragments,
        format!("{}{}", primary.suffix, primary_tolerance),
    );
    let alternate = if let Some(a) = &format.alternate {
        let a_value = checked(&value * exact(a.factor))?;
        let a_rounded = quantize(&a_value, a.rounding_increment)?;
        let text = number(
            &a_rounded,
            a.format,
            a.precision,
            a.decimal_separator,
            a.zero_suppression,
        );
        let alternate_tolerance = tolerance(
            &a_value,
            &exact(a.factor),
            &format.tolerance,
            a.decimal_separator,
        )?;
        generated.push_str(&format!(
            " [{}{}{}{}]",
            a.prefix, text, a.suffix, alternate_tolerance
        ));
        push_text(&mut generated_fragments, format!(" [{}", a.prefix));
        generated_fragments.extend(number_fragments(
            &a_rounded,
            a.format,
            a.precision,
            a.decimal_separator,
            a.zero_suppression,
            a.fraction_layout,
            a.fraction_scale,
        ));
        push_text(
            &mut generated_fragments,
            format!("{}{}]", a.suffix, alternate_tolerance),
        );
        Some(text)
    } else {
        None
    };
    let plain = || TextRun {
        text: generated.clone(),
        ..Default::default()
    };
    let runs = match label {
        DimensionLabel::Generated => vec![plain()],
        DimensionLabel::Suppressed => Vec::new(),
        DimensionLabel::Template(parts) => parts
            .iter()
            .flat_map(|part| match part {
                DimensionLabelPart::Literal(runs) => runs.clone(),
                DimensionLabelPart::Measurement => vec![plain()],
            })
            .collect(),
    };
    let fragments = match label {
        DimensionLabel::Generated => generated_fragments,
        DimensionLabel::Suppressed => Vec::new(),
        DimensionLabel::Template(parts) => parts
            .iter()
            .flat_map(|part| match part {
                DimensionLabelPart::Literal(runs) => runs
                    .iter()
                    .cloned()
                    .map(DimensionFormattedFragment::Run)
                    .collect(),
                DimensionLabelPart::Measurement => generated_fragments.clone(),
            })
            .collect(),
    };
    Ok(DimensionFormattedLabel {
        fragments,
        state: if matches!(label, DimensionLabel::Suppressed) {
            DimensionFormattedLabelState::Suppressed
        } else {
            DimensionFormattedLabelState::Visible
        },
        runs,
        primary: primary_text,
        alternate,
    })
}
