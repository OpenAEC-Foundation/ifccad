//! Qualified scalar layout mapping. No font engine or coordinate-unit inference.
use crate::placement::{finite, positive};
use crate::MTextCadInput;
use crate::{CadTextError, CadTextIssue, MarkupColor};
use ocdraw::text::*;
use opencadcodec::{
    entities::{AttachmentPoint, DrawingDirection, LineSpacingStyle},
    Color, MText,
};

pub(crate) fn attachment(value: AttachmentPoint) -> MTextAttachment {
    match value {
        AttachmentPoint::TopLeft => MTextAttachment::TopLeft,
        AttachmentPoint::TopCenter => MTextAttachment::TopCenter,
        AttachmentPoint::TopRight => MTextAttachment::TopRight,
        AttachmentPoint::MiddleLeft => MTextAttachment::MiddleLeft,
        AttachmentPoint::MiddleCenter => MTextAttachment::MiddleCenter,
        AttachmentPoint::MiddleRight => MTextAttachment::MiddleRight,
        AttachmentPoint::BottomLeft => MTextAttachment::BottomLeft,
        AttachmentPoint::BottomCenter => MTextAttachment::BottomCenter,
        AttachmentPoint::BottomRight => MTextAttachment::BottomRight,
    }
}

pub(crate) fn cad_attachment(value: MTextAttachment) -> AttachmentPoint {
    match value {
        MTextAttachment::TopLeft => AttachmentPoint::TopLeft,
        MTextAttachment::TopCenter => AttachmentPoint::TopCenter,
        MTextAttachment::TopRight => AttachmentPoint::TopRight,
        MTextAttachment::MiddleLeft => AttachmentPoint::MiddleLeft,
        MTextAttachment::MiddleCenter => AttachmentPoint::MiddleCenter,
        MTextAttachment::MiddleRight => AttachmentPoint::MiddleRight,
        MTextAttachment::BottomLeft => AttachmentPoint::BottomLeft,
        MTextAttachment::BottomCenter => AttachmentPoint::BottomCenter,
        MTextAttachment::BottomRight => AttachmentPoint::BottomRight,
    }
}
pub(crate) fn cad_flow(value: MTextFlow) -> DrawingDirection {
    match value {
        MTextFlow::Horizontal => DrawingDirection::LeftToRight,
        MTextFlow::Vertical => DrawingDirection::TopToBottom,
        MTextFlow::ByStyle => DrawingDirection::ByStyle,
    }
}

pub(crate) fn columns_to_cad(
    input: &MTextCadInput<'_>,
    target: &mut MText,
) -> Result<(), CadTextError> {
    let Some(columns) = input.columns else {
        target.rectangle_width = input.wrap_width.unwrap_or(0.);
        return Ok(());
    };
    validate_mtext_columns(columns)?;
    let c = &mut target.column_data;
    let (count, width, gutter, reverse) = match columns {
        MTextColumns::Static {
            count,
            column_width,
            gutter,
            column_height,
            flow_reversed,
        } => {
            c.column_type = 1;
            c.auto_height = false;
            target.rectangle_height = Some(*column_height);
            (*count, *column_width, *gutter, *flow_reversed)
        }
        MTextColumns::DynamicAutoHeight {
            current_column_count,
            column_width,
            gutter,
            column_height,
            flow_reversed,
        } => {
            c.column_type = 2;
            c.auto_height = true;
            target.rectangle_height = Some(*column_height);
            (
                *current_column_count,
                *column_width,
                *gutter,
                *flow_reversed,
            )
        }
        MTextColumns::DynamicManualHeight {
            column_width,
            gutter,
            column_heights,
            flow_reversed,
        } => {
            c.column_type = 2;
            c.auto_height = false;
            for height in column_heights {
                match height {
                    MTextColumnHeight::Fixed { distance } => c.heights.push(*distance),
                    MTextColumnHeight::Auto => {
                        return Err(CadTextError::Unsupported(
                            "manual auto-tail sentinel export is not qualified",
                        ))
                    }
                }
            }
            (
                u32::try_from(column_heights.len())
                    .map_err(|_| CadTextError::Unsupported("column count exceeds target"))?,
                *column_width,
                *gutter,
                *flow_reversed,
            )
        }
    };
    c.column_count = i32::try_from(count)
        .map_err(|_| CadTextError::Unsupported("column count exceeds target int32"))?;
    c.width = width;
    c.gutter = gutter;
    c.flow_reversed = reverse;
    target.rectangle_width = f64::from(count) * width + f64::from(count - 1) * gutter;
    finite(target.rectangle_width)?;
    Ok(())
}

pub(crate) fn spacing_to_cad(
    spacing: TextLineSpacing,
    target: &mut MText,
    issues: &mut Vec<CadTextIssue>,
) -> Result<(), CadTextError> {
    let (distance, style) = match spacing {
        TextLineSpacing::Multiple { factor: 1. } => return Ok(()),
        TextLineSpacing::Multiple { .. } => {
            return Err(CadTextError::Unsupported(
                "multiple spacing has different larger-glyph semantics",
            ))
        }
        TextLineSpacing::Exact { distance } => (distance, LineSpacingStyle::Exactly),
        TextLineSpacing::AtLeast { distance } => (distance, LineSpacingStyle::AtLeast),
    };
    positive(distance)?;
    let ratio = distance / ((5. / 3.) * target.height);
    let ratio = [ratio, ratio.next_down(), ratio.next_up()]
        .into_iter()
        .find(|r| r.is_finite() && *r > 0. && *r * (5. / 3.) * target.height == distance)
        .ok_or(CadTextError::Unsupported(
            "line spacing cannot be represented without numeric parameter loss",
        ))?;
    target.line_spacing_style = style;
    target.line_spacing_factor = ratio;
    issues.push(CadTextIssue::LineSpacingReferenceChanged);
    Ok(())
}

pub(crate) fn background_to_cad(
    input: &MTextCadInput<'_>,
    target: &mut MText,
    issues: &mut Vec<CadTextIssue>,
) -> Result<(), CadTextError> {
    let Some(bg) = input.background else {
        return Ok(());
    };
    validate_background(bg, crate::content::valid_color)?;
    if bg.opacity != 1. {
        return Err(CadTextError::Unsupported(
            "fill opacity packing is not qualified",
        ));
    }
    let padding = match bg.padding {
        TextPadding::Relative { factor } => factor,
        TextPadding::Absolute { distance } => {
            issues.push(CadTextIssue::PaddingReferenceChanged);
            distance / input.height
        }
    };
    target.background_scale = 1. + padding;
    finite(target.background_scale)?;
    if target.background_scale - 1. != padding {
        return Err(CadTextError::Unsupported(
            "border factor would change the padding parameter",
        ));
    }
    target.background_fill_flags = if bg.frame { 0x10 } else { 0 };
    match bg.fill {
        MTextFill::None => {}
        MTextFill::Canvas => target.background_fill_flags |= 3,
        MTextFill::Color(ref color) => {
            target.background_fill_flags |= 1;
            target.background_color = match color {
                MarkupColor::Index(index) => Color::Index(
                    u8::try_from(*index)
                        .map_err(|_| CadTextError::Unsupported("fill ACI exceeds target range"))?,
                ),
                MarkupColor::Rgb { red, green, blue } => Color::from_rgb(*red, *green, *blue),
            };
        }
    }
    Ok(())
}
pub(crate) fn flow(value: DrawingDirection) -> MTextFlow {
    match value {
        DrawingDirection::LeftToRight => MTextFlow::Horizontal,
        DrawingDirection::TopToBottom => MTextFlow::Vertical,
        DrawingDirection::ByStyle => MTextFlow::ByStyle,
    }
}

pub(crate) fn columns_from_cad(
    source: &MText,
    issues: &mut Vec<CadTextIssue>,
) -> Result<Option<MTextColumns>, CadTextError> {
    let c = &source.column_data;
    if c.column_type == 0 {
        if c.column_count != 0
            || c.flow_reversed
            || c.auto_height
            || c.width != 0.
            || c.gutter != 0.
            || !c.heights.is_empty()
        {
            issues.push(CadTextIssue::InactiveColumnPropertiesOmitted);
        }
        if source.rectangle_height.is_some_and(|height| height != 0.) {
            return Err(CadTextError::Unsupported(
                "ordinary MText reference height has no native fixed-box meaning",
            ));
        }
        return Ok(None);
    }
    let count = u32::try_from(c.column_count)
        .ok()
        .filter(|count| *count > 0)
        .ok_or(CadTextError::Unsupported(
            "actual nonzero column count is unavailable",
        ))?;
    positive(c.width)?;
    finite(c.gutter)?;
    if c.gutter < 0. {
        return Err(CadTextError::InvalidSource("negative column gutter"));
    }
    let total = f64::from(count) * c.width + f64::from(count - 1) * c.gutter;
    finite(total)?;
    if source.rectangle_width != 0. && source.rectangle_width != total {
        return Err(CadTextError::Unsupported(
            "column reference width differs from native derived total width",
        ));
    }
    let columns = match c.column_type {
        1 => {
            if !c.heights.is_empty() || c.auto_height {
                return Err(CadTextError::Unsupported("static column auxiliary state"));
            }
            let column_height = source.rectangle_height.ok_or(CadTextError::Unsupported(
                "static common column height unavailable",
            ))?;
            positive(column_height)?;
            MTextColumns::Static {
                count,
                column_width: c.width,
                gutter: c.gutter,
                column_height,
                flow_reversed: c.flow_reversed,
            }
        }
        2 if c.auto_height => {
            if !c.heights.is_empty() {
                return Err(CadTextError::InvalidSource(
                    "auto-height columns have unexpected manual heights",
                ));
            }
            let column_height = source.rectangle_height.ok_or(CadTextError::Unsupported(
                "dynamic common column height unavailable",
            ))?;
            positive(column_height)?;
            MTextColumns::DynamicAutoHeight {
                column_width: c.width,
                gutter: c.gutter,
                column_height,
                current_column_count: count,
                flow_reversed: c.flow_reversed,
            }
        }
        2 => {
            if c.heights.len() != count as usize {
                return Err(CadTextError::InvalidSource(
                    "column count disagrees with height list",
                ));
            }
            let mut heights = Vec::with_capacity(c.heights.len());
            for height in &c.heights {
                if *height == 0. {
                    return Err(CadTextError::Unsupported(
                        "manual zero-height sentinel is not qualified",
                    ));
                }
                positive(*height)?;
                heights.push(MTextColumnHeight::Fixed { distance: *height });
            }
            MTextColumns::DynamicManualHeight {
                column_width: c.width,
                gutter: c.gutter,
                column_heights: heights,
                flow_reversed: c.flow_reversed,
            }
        }
        _ => return Err(CadTextError::Unsupported("unknown MText column type")),
    };
    validate_mtext_columns(&columns)?;
    Ok(Some(columns))
}

pub(crate) fn spacing_from_cad(
    source: &MText,
    format: &mut ParagraphFormat,
    issues: &mut Vec<CadTextIssue>,
) -> Result<(), CadTextError> {
    positive(source.line_spacing_factor)?;
    if source.line_spacing_style == LineSpacingStyle::AtLeast && source.line_spacing_factor == 1. {
        return Ok(());
    }
    let distance = source.line_spacing_factor * (5. / 3.) * source.height;
    positive(distance)?;
    format.line_spacing = Some(match source.line_spacing_style {
        LineSpacingStyle::AtLeast => TextLineSpacing::AtLeast { distance },
        LineSpacingStyle::Exactly => TextLineSpacing::Exact { distance },
    });
    // The CAD factor tracks nominal height, whereas native distance is absolute.
    // Current scalar spacing is represented; editing dependency is diagnosed.
    issues.push(CadTextIssue::LineSpacingReferenceChanged);
    Ok(())
}

pub(crate) fn background_from_cad(
    source: &MText,
    issues: &mut Vec<CadTextIssue>,
) -> Result<Option<MTextBackground<MarkupColor>>, CadTextError> {
    let flags = source.background_fill_flags;
    if flags & !0x13 != 0 {
        return Err(CadTextError::Unsupported("unknown background/frame flags"));
    }
    if flags == 0 {
        if source.background_scale != 1.5
            || source.background_color != Color::ByLayer
            || source.background_transparency != 0
        {
            issues.push(CadTextIssue::InactiveBackgroundPropertiesOmitted);
        }
        return Ok(None);
    }
    finite(source.background_scale)?;
    if source.background_scale < 1. {
        return Err(CadTextError::Unsupported(
            "background factor requires negative native padding",
        ));
    }
    if source.background_transparency != 0 {
        return Err(CadTextError::Unsupported(
            "background transparency packing is not qualified",
        ));
    }
    let fill = if flags & 2 != 0 {
        MTextFill::Canvas
    } else if flags & 1 != 0 {
        let color = match source.background_color {
            Color::ByLayer | Color::ByBlock => {
                return Err(CadTextError::Unsupported(
                    "inherited fill color has no explicit native fill mapping",
                ))
            }
            Color::Index(index) if (1..=255).contains(&index) => MarkupColor::Index(index as u16),
            Color::Rgb { r, g, b } => MarkupColor::Rgb {
                red: r,
                green: g,
                blue: b,
            },
            _ => {
                return Err(CadTextError::Unsupported(
                    "background color outside qualified palette",
                ))
            }
        };
        MTextFill::Color(color)
    } else {
        MTextFill::None
    };
    let background = MTextBackground {
        fill,
        padding: TextPadding::Relative {
            factor: source.background_scale - 1.,
        },
        opacity: 1.,
        frame: flags & 0x10 != 0,
    };
    validate_background(&background, crate::content::valid_color)?;
    Ok(Some(background))
}
