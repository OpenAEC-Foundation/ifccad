//! Rotation-independent fontless navigation estimates for line pattern text.
//! The radius contains the reused layout heuristic, not unknown font contours.
use crate::{
    geometry_kernel::{numeric::Interval, CoordinateFrame3},
    text::*,
};

pub struct ResolvedLinePatternTextExtentInput<'a> {
    pub text: &'a str,
    pub fixed_style_height: Option<f64>,
    pub marker_scale: f64,
    pub pattern_scale: f64,
    pub offset: [f64; 2],
}
#[derive(Clone, Debug, PartialEq)]
pub struct LinePatternTextExtentEstimate {
    pub radius: Option<f64>,
    pub reasons: Vec<TextExtentReason>,
}

pub fn estimate_line_pattern_text_extent(
    input: &ResolvedLinePatternTextExtentInput<'_>,
) -> Result<LinePatternTextExtentEstimate, TextExtentError> {
    for (path, value, positive) in [
        ("/markerScale", input.marker_scale, true),
        ("/patternScale", input.pattern_scale, true),
        ("/offset/0", input.offset[0], false),
        ("/offset/1", input.offset[1], false),
    ] {
        if !value.is_finite() || (positive && value <= 0.) {
            return Err(TextExtentError::Values(TextValueError {
                code: TextValueErrorCode::InvalidNumber,
                path: path.into(),
                message: "marker scales must be positive and all values finite",
            }));
        }
    }
    let fixed = input.fixed_style_height.unwrap_or(0.);
    if !fixed.is_finite() || fixed < 0. {
        return Err(TextExtentError::Values(TextValueError {
            code: TextValueErrorCode::InvalidNumber,
            path: "/fixedStyleHeight".into(),
            message: "fixed height must be finite and nonnegative",
        }));
    }
    if input.text.is_empty() {
        return Ok(LinePatternTextExtentEstimate {
            radius: None,
            reasons: vec![],
        });
    }
    let height = if fixed > 0. {
        Interval::point(fixed)
            .mul(Interval::point(input.marker_scale))
            .ok_or(TextExtentError::OutOfRange)?
            .upper
    } else {
        input.marker_scale
    };
    let content = [TextRun {
        text: input.text.into(),
        ..Default::default()
    }];
    let estimate = estimate_text_extent(&ResolvedTextExtentInput {
        content: &content,
        layout: TextLayout::Anchored {
            horizontal: TextHorizontalAlignment::Left,
            vertical: TextVerticalAlignment::Baseline,
            height,
            width_factor: 1.,
        },
        placement: CoordinateFrame3::default(),
        rotation: 0.,
        backward: false,
        upside_down: false,
        oblique_angle: 0.,
        thickness: 0.,
        vertical: false,
    })?;
    let Some(bounds) = estimate.local_bounds else {
        return Ok(LinePatternTextExtentEstimate {
            radius: None,
            reasons: estimate.reasons,
        });
    };
    let min = bounds.min().components();
    let max = bounds.max().components();
    let mut radius = Interval::point(0.);
    for value in (0..3)
        .map(|i| min[i].abs().max(max[i].abs()))
        .chain(input.offset.into_iter().map(f64::abs))
    {
        radius = radius
            .add(Interval::point(value))
            .ok_or(TextExtentError::OutOfRange)?;
    }
    radius = radius
        .mul(Interval::point(input.pattern_scale))
        .ok_or(TextExtentError::OutOfRange)?;
    Ok(LinePatternTextExtentEstimate {
        radius: Some(radius.upper),
        reasons: estimate.reasons,
    })
}
