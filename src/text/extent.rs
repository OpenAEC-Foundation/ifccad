//! Fontless navigation estimates. These boxes never certify glyph enclosure.
use super::*;
use crate::geometry_kernel::{
    numeric::Interval, BlockTransform, BlockTransformError, Bounds3d, CoordinateFrame3, Point3,
    PreparedBlockTransform, Scale3,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextExtentStatus {
    Empty,
    Estimated,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextExtentReason {
    FontMetricsUnavailable,
    LayoutEstimated,
    VerticalFlowEstimated,
    ZeroAdvance,
    Overflow,
    StrokeExtentUnavailable,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextExtentEstimate {
    /// Authored-plane box after estimated layout/alignment, before mirrors/rotation.
    pub local_bounds: Option<Bounds3d>,
    /// Owning-scope box, derived through the shared outward transform kernel.
    /// When status is Unavailable this is only the known subset, never a full box.
    pub bounds: Option<Bounds3d>,
    pub status: TextExtentStatus,
    pub reasons: Vec<TextExtentReason>,
}

#[derive(Debug, thiserror::Error)]
pub enum TextExtentError {
    #[error("invalid authored text values: {0}")]
    Values(#[from] TextValueError),
    #[error("invalid text placement: {0}")]
    Placement(#[from] BlockTransformError),
    #[error("text extent arithmetic exceeds the finite binary64 range")]
    OutOfRange,
}

/// Drawing-owned references have been resolved; no font is loaded or substituted.
pub struct ResolvedTextExtentInput<'a> {
    pub content: &'a [TextRun],
    pub layout: TextLayout,
    pub placement: CoordinateFrame3,
    pub rotation: f64,
    pub backward: bool,
    pub upside_down: bool,
    pub oblique_angle: f64,
    pub thickness: f64,
    pub vertical: bool,
}

pub struct ResolvedMTextExtentInput<'a, C> {
    pub content: &'a [MTextParagraph<C>],
    pub style: &'a TextStyleProperties,
    pub character_format: &'a CharacterFormat<C>,
    pub paragraph_format: &'a ParagraphFormat,
    pub height: f64,
    pub attachment: MTextAttachment,
    pub flow: MTextFlow,
    pub wrap_width: Option<f64>,
    pub columns: Option<&'a MTextColumns>,
    pub background: Option<&'a MTextBackground<C>>,
    /// Effective frame stroke width in scope coordinates; None means unknown.
    /// A layout medium alone is not a conversion from physical lineweight.
    pub frame_stroke_width: Option<f64>,
    pub placement: CoordinateFrame3,
    pub rotation: f64,
    pub backward: bool,
    pub upside_down: bool,
}

pub fn estimate_text_extent(
    input: &ResolvedTextExtentInput<'_>,
) -> Result<TextExtentEstimate, TextExtentError> {
    validate_text_runs(input.content)?;
    validate_text_layout(&input.layout)?;
    validation::oblique(input.oblique_angle, "/obliqueAngle")?;
    validation::finite(input.thickness, "/thickness")?;
    let transform = transform(
        input.placement,
        input.rotation,
        input.backward,
        input.upside_down,
    )?;
    let mut reasons = vec![TextExtentReason::FontMetricsUnavailable];
    if input.content.is_empty() {
        if matches!(
            input.layout,
            TextLayout::Aligned { .. } | TextLayout::Fit { .. }
        ) {
            reasons.push(TextExtentReason::ZeroAdvance);
        }
        return Ok(TextExtentEstimate {
            local_bounds: None,
            bounds: None,
            status: TextExtentStatus::Empty,
            reasons,
        });
    }
    let (mut height, width_factor) = match input.layout {
        TextLayout::Anchored {
            height,
            width_factor,
            ..
        }
        | TextLayout::WholeTextMiddle {
            height,
            width_factor,
        } => (height, width_factor),
        TextLayout::Aligned { width_factor, .. } => (1., width_factor),
        TextLayout::Fit { height, .. } => (height, 1.),
    };
    let count: usize = input
        .content
        .iter()
        .map(|run| run.text.chars().count())
        .sum();
    let mut advance = checked(checked(2. * height)? * width_factor * count as f64)?;
    match input.layout {
        TextLayout::Aligned { length, .. } => {
            height = checked(height * checked(length / advance)?)?;
            if height <= 0. {
                return Err(TextExtentError::OutOfRange);
            }
            advance = length;
        }
        TextLayout::Fit { length, .. } => advance = length,
        _ => {}
    }
    let mut band = checked(2. * height)?;
    if input
        .content
        .iter()
        .any(|run| run.underline || run.overline || run.strike_through)
    {
        band = checked(band + 0.5 * height)?;
    }
    let (mut min, mut max) = if input.vertical {
        reasons.push(TextExtentReason::VerticalFlowEstimated);
        (
            [-band, -advance, input.thickness.min(0.)],
            [band, 0., input.thickness.max(0.)],
        )
    } else {
        (
            [0., -band, input.thickness.min(0.)],
            [advance, band, input.thickness.max(0.)],
        )
    };
    // Shear affects ink estimates, not the baseline advance used for alignment.
    let shear = checked(input.oblique_angle.tan() * band)?;
    min[0] = checked(min[0] - shear.abs())?;
    max[0] = checked(max[0] + shear.abs())?;
    let (dx, dy) = match input.layout {
        TextLayout::Anchored {
            horizontal,
            vertical,
            ..
        } => (
            match horizontal {
                TextHorizontalAlignment::Left => 0.,
                TextHorizontalAlignment::Center => -advance / 2.,
                TextHorizontalAlignment::Right => -advance,
            },
            match vertical {
                TextVerticalAlignment::Baseline => 0.,
                TextVerticalAlignment::Bottom => -min[1],
                TextVerticalAlignment::Middle => -height / 2.,
                TextVerticalAlignment::Top => -max[1],
            },
        ),
        TextLayout::WholeTextMiddle { .. } => {
            (-min[0] / 2. - max[0] / 2., -min[1] / 2. - max[1] / 2.)
        }
        _ => (0., 0.),
    };
    min[0] = checked(min[0] + dx)?;
    max[0] = checked(max[0] + dx)?;
    min[1] = checked(min[1] + dy)?;
    max[1] = checked(max[1] + dy)?;
    finish(min, max, transform, TextExtentStatus::Estimated, reasons)
}

pub(super) fn checked(value: f64) -> Result<f64, TextExtentError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(TextExtentError::OutOfRange)
    }
}

pub(super) fn transform(
    placement: CoordinateFrame3,
    rotation: f64,
    backward: bool,
    upside_down: bool,
) -> Result<PreparedBlockTransform, TextExtentError> {
    let t = BlockTransform::try_new(
        placement,
        rotation,
        Scale3::new(
            if backward { -1. } else { 1. },
            if upside_down { -1. } else { 1. },
            1.,
        ),
    )?;
    PreparedBlockTransform::new(t, Point3::new(0., 0., 0.)).ok_or(TextExtentError::OutOfRange)
}

pub(super) fn finish(
    min: [f64; 3],
    max: [f64; 3],
    transform: PreparedBlockTransform,
    status: TextExtentStatus,
    reasons: Vec<TextExtentReason>,
) -> Result<TextExtentEstimate, TextExtentError> {
    if min.into_iter().chain(max).any(|value| !value.is_finite()) || (0..3).any(|i| min[i] > max[i])
    {
        return Err(TextExtentError::OutOfRange);
    }
    let placed = transform
        .apply_intervals(std::array::from_fn(|i| Interval {
            lower: min[i],
            upper: max[i],
        }))
        .ok_or(TextExtentError::OutOfRange)?;
    let point = |v: [f64; 3]| Point3::new(v[0], v[1], v[2]);
    Ok(TextExtentEstimate {
        local_bounds: Some(Bounds3d::new(point(min), point(max))),
        bounds: Some(Bounds3d::new(
            point(placed.map(|value| value.lower)),
            point(placed.map(|value| value.upper)),
        )),
        status,
        reasons,
    })
}
