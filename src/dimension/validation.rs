use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DimensionValueReason {
    NonFinite,
    OutOfRange,
    InvalidDefinition,
    NumericUnavailable,
}
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("dimension {field}: {reason:?}")]
pub struct DimensionValueError {
    pub field: String,
    pub reason: DimensionValueReason,
}
pub(crate) fn failure(field: &str, reason: DimensionValueReason) -> DimensionValueError {
    DimensionValueError {
        field: field.into(),
        reason,
    }
}
pub(crate) fn finite(field: &str, value: f64) -> Result<(), DimensionValueError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(failure(field, DimensionValueReason::NonFinite))
    }
}
pub(crate) fn nonnegative(field: &str, value: f64) -> Result<(), DimensionValueError> {
    finite(field, value)?;
    if value >= 0.0 {
        Ok(())
    } else {
        Err(failure(field, DimensionValueReason::OutOfRange))
    }
}
pub(crate) fn positive(field: &str, value: f64) -> Result<(), DimensionValueError> {
    finite(field, value)?;
    if value > 0.0 {
        Ok(())
    } else {
        Err(failure(field, DimensionValueReason::OutOfRange))
    }
}
pub(crate) fn point(field: &str, value: [f64; 2]) -> Result<(), DimensionValueError> {
    finite(&format!("{field}/0"), value[0])?;
    finite(&format!("{field}/1"), value[1])
}

pub fn validate_dimension_definition(v: &DimensionDefinition) -> Result<(), DimensionValueError> {
    for (role, p) in v.measurement_anchors() {
        for (i, value) in p.into_iter().enumerate() {
            finite(&format!("definition/{}/{i}", role.field_name()), value)?;
        }
    }
    use DimensionDefinition as D;
    match v {
        D::Linear {
            direction_angle,
            dimension_line_point,
            extension_direction_angle,
            ..
        } => {
            finite("definition/directionAngle", *direction_angle)?;
            point("definition/dimensionLinePoint", *dimension_line_point)?;
            if let Some(a) = extension_direction_angle {
                finite("definition/extensionDirectionAngle", *a)?;
            }
        }
        D::Aligned {
            dimension_line_point,
            extension_direction_angle,
            ..
        } => {
            point("definition/dimensionLinePoint", *dimension_line_point)?;
            if let Some(a) = extension_direction_angle {
                finite("definition/extensionDirectionAngle", *a)?;
            }
        }
        D::Radius {
            center,
            curve_point,
            leader_length,
        } => {
            nonnegative("definition/leaderLength", *leader_length)?;
            distinct_xy(*center, *curve_point)?;
        }
        D::Diameter {
            first_diameter_point,
            second_diameter_point,
            leader_length,
        } => {
            nonnegative("definition/leaderLength", *leader_length)?;
            distinct_xy(*first_diameter_point, *second_diameter_point)?;
        }
        D::AngularTwoLines {
            line1_start,
            line1_end,
            line2_start,
            line2_end,
            dimension_arc_point,
            ..
        } => {
            distinct_xy(*line1_start, *line1_end)?;
            distinct_xy(*line2_start, *line2_end)?;
            let a = crate::geometry_kernel::numeric::exact;
            let det = (a(line1_end[0]) - a(line1_start[0])) * (a(line2_end[1]) - a(line2_start[1]))
                - (a(line1_end[1]) - a(line1_start[1])) * (a(line2_end[0]) - a(line2_start[0]));
            if det == num_rational::BigRational::from_integer(0.into()) {
                return Err(failure(
                    "definition/lines",
                    DimensionValueReason::InvalidDefinition,
                ));
            }
            point("definition/dimensionArcPoint", *dimension_arc_point)?;
            let d1x = a(line1_end[0]) - a(line1_start[0]);
            let d1y = a(line1_end[1]) - a(line1_start[1]);
            let d2x = a(line2_end[0]) - a(line2_start[0]);
            let d2y = a(line2_end[1]) - a(line2_start[1]);
            let t = ((a(line2_start[0]) - a(line1_start[0])) * d2y
                - (a(line2_start[1]) - a(line1_start[1])) * d2x)
                / det;
            let x = a(line1_start[0]) + &t * d1x;
            let y = a(line1_start[1]) + t * d1y;
            let bound = a(f64::MAX);
            if x < -&bound || x > bound || y < -&bound || y > bound {
                return Err(failure(
                    "definition/intersection",
                    DimensionValueReason::NumericUnavailable,
                ));
            }
            if x == a(dimension_arc_point[0]) && y == a(dimension_arc_point[1]) {
                return Err(failure(
                    "definition/dimensionArcPoint",
                    DimensionValueReason::InvalidDefinition,
                ));
            }
            point("definition/dimensionArcPoint", *dimension_arc_point)?;
        }
        D::AngularThreePoints {
            vertex,
            first_point,
            second_point,
            dimension_arc_point,
            ..
        } => {
            distinct_xy(*vertex, *first_point)?;
            distinct_xy(*vertex, *second_point)?;
            point("definition/dimensionArcPoint", *dimension_arc_point)?;
            if vertex[0] == dimension_arc_point[0] && vertex[1] == dimension_arc_point[1] {
                return Err(failure(
                    "definition/dimensionArcPoint",
                    DimensionValueReason::InvalidDefinition,
                ));
            }
        }
        D::Ordinate {
            axis_angle,
            leader_end,
            ..
        } => {
            finite("definition/axisAngle", *axis_angle)?;
            point("definition/leaderEnd", *leader_end)?;
        }
        D::ArcLength {
            center,
            radius,
            start_parameter,
            sweep,
            dimension_arc_point,
            leader,
        } => {
            positive("definition/radius", *radius)?;
            finite("definition/startParameter", *start_parameter)?;
            if let DimensionArcSpan::Partial { radians } = sweep {
                finite("definition/sweep/radians", *radians)?;
                if *radians == 0. || radians.abs() >= std::f64::consts::TAU {
                    return Err(failure(
                        "definition/sweep",
                        DimensionValueReason::OutOfRange,
                    ));
                }
            }
            point("definition/dimensionArcPoint", *dimension_arc_point)?;
            if center[0] == dimension_arc_point[0] && center[1] == dimension_arc_point[1] {
                return Err(failure(
                    "definition/dimensionArcPoint",
                    DimensionValueReason::InvalidDefinition,
                ));
            }
            if let Some(l) = leader {
                point("definition/leader/start", l.start)?;
                point("definition/leader/end", l.end)?;
            }
        }
        D::JoggedRadius {
            center,
            curve_point,
            display_center,
            jog_point,
            jog_angle,
        } => {
            distinct_xy(*center, *curve_point)?;
            point("definition/displayCenter", *display_center)?;
            point("definition/jogPoint", *jog_point)?;
            finite("definition/jogAngle", *jog_angle)?;
        }
    }
    Ok(())
}
fn distinct_xy(a: [f64; 3], b: [f64; 3]) -> Result<(), DimensionValueError> {
    if a[0] == b[0] && a[1] == b[1] {
        Err(failure(
            "definition/points",
            DimensionValueReason::InvalidDefinition,
        ))
    } else {
        Ok(())
    }
}
pub fn validate_dimension_text_placement(
    v: &DimensionTextPlacement,
) -> Result<(), DimensionValueError> {
    if let DimensionTextPlacement::Explicit {
        position, rotation, ..
    } = v
    {
        point("textPlacement/position", *position)?;
        finite("textPlacement/rotation", *rotation)?;
    }
    Ok(())
}
fn precision(field: &str, value: u8) -> Result<(), DimensionValueError> {
    if value <= 16 {
        Ok(())
    } else {
        Err(failure(field, DimensionValueReason::OutOfRange))
    }
}
pub fn validate_dimension_numbers(v: &DimensionNumberSettings) -> Result<(), DimensionValueError> {
    positive(
        "numbers/primary/measurementFactor",
        v.primary.measurement_factor,
    )?;
    positive("numbers/primary/fractionScale", v.primary.fraction_scale)?;
    precision("numbers/primary/precision", v.primary.precision)?;
    nonnegative(
        "numbers/primary/roundingIncrement",
        v.primary.rounding_increment,
    )?;
    if v.primary.decimal_separator.is_control() || v.primary.decimal_separator.is_ascii_digit() {
        return Err(failure(
            "numbers/primary/decimalSeparator",
            DimensionValueReason::OutOfRange,
        ));
    }
    if let Some(a) = &v.alternate {
        positive("numbers/alternate/factor", a.factor)?;
        positive("numbers/alternate/fractionScale", a.fraction_scale)?;
        precision("numbers/alternate/precision", a.precision)?;
        nonnegative("numbers/alternate/roundingIncrement", a.rounding_increment)?;
        if a.decimal_separator.is_control() || a.decimal_separator.is_ascii_digit() {
            return Err(failure(
                "numbers/alternate/decimalSeparator",
                DimensionValueReason::OutOfRange,
            ));
        }
    }
    if let DimensionToleranceSettings::Deviations {
        upper,
        lower,
        precision: p,
    }
    | DimensionToleranceSettings::Limits {
        upper,
        lower,
        precision: p,
    } = &v.tolerance
    {
        nonnegative("numbers/tolerance/upper", *upper)?;
        nonnegative("numbers/tolerance/lower", *lower)?;
        precision("numbers/tolerance/precision", *p)?;
    }
    Ok(())
}
fn appearance<C, P>(
    path: &str,
    v: &DimensionComponentAppearance<C, P>,
) -> Result<(), DimensionValueError> {
    if let DimensionAppearanceChoice::Explicit(o) = v.opacity {
        finite(&format!("{path}/opacity"), o)?;
        if !(0.0..=1.0).contains(&o) {
            return Err(failure(
                &format!("{path}/opacity"),
                DimensionValueReason::OutOfRange,
            ));
        }
    }
    if let DimensionAppearanceChoice::Explicit(w) = v.line_weight {
        nonnegative(&format!("{path}/lineWeight"), w)?;
    }
    Ok(())
}
pub fn validate_dimension_settings<S, B, C, P>(
    v: &DimensionStyleSettings<S, B, C, P>,
) -> Result<(), DimensionValueError> {
    positive("presentationScale", v.presentation_scale)?;
    positive("text/height", v.text.height)?;
    nonnegative("text/gap", v.text.gap)?;
    appearance("text/appearance", &v.text.appearance)?;
    let (DimensionLineSpacing::AtLeast { factor } | DimensionLineSpacing::Exact { factor }) =
        v.text.line_spacing;
    finite("text/lineSpacing/factor", factor)?;
    if !(0.25..=4.0).contains(&factor) {
        return Err(failure(
            "text/lineSpacing/factor",
            DimensionValueReason::OutOfRange,
        ));
    }
    if let DimensionBackground::Color {
        opacity, padding, ..
    }
    | DimensionBackground::Canvas { opacity, padding } = &v.text.background
    {
        finite("text/background/opacity", *opacity)?;
        if !(0.0..=1.0).contains(opacity) {
            return Err(failure(
                "text/background/opacity",
                DimensionValueReason::OutOfRange,
            ));
        }
        let (DimensionPadding::Absolute { distance: x } | DimensionPadding::Relative { factor: x }) =
            padding;
        nonnegative("text/background/padding", *x)?;
    }
    nonnegative("dimensionLine/extension", v.dimension_line.extension)?;
    appearance("dimensionLine/appearance", &v.dimension_line.appearance)?;
    for (key, e) in [
        ("first", &v.extension_lines.first),
        ("second", &v.extension_lines.second),
    ] {
        let path = format!("extensionLines/{key}");
        appearance(&format!("{path}/appearance"), &e.appearance)?;
        nonnegative(&format!("{path}/offset"), e.offset)?;
        nonnegative(&format!("{path}/extension"), e.extension)?;
        if let DimensionExtensionLength::Fixed { length } = e.length {
            nonnegative(&format!("{path}/length"), length)?;
        }
    }
    nonnegative("arrows/size", v.arrows.size)?;
    nonnegative("arrows/tickSize", v.arrows.tick_size)?;
    if let DimensionCenterMark::Mark { size } | DimensionCenterMark::Lines { size } =
        v.arrows.center_mark
    {
        nonnegative("arrows/centerMark/size", size)?;
    }
    validate_dimension_numbers(&v.numbers)
}
