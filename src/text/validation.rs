//! Intrinsic checks shared by authored values and each format's reader backing.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextValueErrorCode {
    InvalidNumber,
    InvalidFont,
    InvalidContent,
    InvalidTabs,
    InvalidColumns,
    InvalidBackground,
    InvalidColor,
    DerivedOutOfRange,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextValueError {
    pub code: TextValueErrorCode,
    pub path: String,
    pub message: &'static str,
}

impl TextValueError {
    pub(super) fn new(
        code: TextValueErrorCode,
        path: impl Into<String>,
        message: &'static str,
    ) -> Self {
        Self {
            code,
            path: path.into(),
            message,
        }
    }
    pub(super) fn prefixed(mut self, prefix: &str) -> Self {
        self.path.insert_str(0, prefix);
        self
    }
}

impl std::fmt::Display for TextValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} at {}: {}", self.code, self.path, self.message)
    }
}
impl std::error::Error for TextValueError {}

pub(super) fn finite(value: f64, path: &str) -> Result<(), TextValueError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(TextValueError::new(
            TextValueErrorCode::InvalidNumber,
            path,
            "value must be finite",
        ))
    }
}
pub(super) fn positive(value: f64, path: &str) -> Result<(), TextValueError> {
    finite(value, path)?;
    if value > 0.0 {
        Ok(())
    } else {
        Err(TextValueError::new(
            TextValueErrorCode::InvalidNumber,
            path,
            "value must be positive",
        ))
    }
}
fn nonnegative(value: f64, path: &str) -> Result<(), TextValueError> {
    finite(value, path)?;
    if value >= 0.0 {
        Ok(())
    } else {
        Err(TextValueError::new(
            TextValueErrorCode::InvalidNumber,
            path,
            "value must be nonnegative",
        ))
    }
}
pub(super) fn product(left: f64, right: f64, path: &str) -> Result<f64, TextValueError> {
    let value = left * right;
    if value.is_finite() && (value != 0.0 || left == 0.0 || right == 0.0) {
        Ok(value)
    } else {
        Err(TextValueError::new(
            TextValueErrorCode::DerivedOutOfRange,
            path,
            "derived product is outside the finite nonzero range",
        ))
    }
}
pub(super) fn oblique(value: f64, path: &str) -> Result<(), TextValueError> {
    finite(value, path)?;
    if value.abs() < std::f64::consts::FRAC_PI_2 && value.tan().is_finite() {
        Ok(())
    } else {
        Err(TextValueError::new(
            TextValueErrorCode::InvalidNumber,
            path,
            "oblique angle must be strictly between -pi/2 and pi/2",
        ))
    }
}
fn literal(text: &str, path: &str, allow_empty: bool) -> Result<(), TextValueError> {
    if (!allow_empty && text.is_empty())
        || text
            .chars()
            .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
    {
        Err(TextValueError::new(
            TextValueErrorCode::InvalidContent,
            path,
            "literal text cannot contain structural controls or be an empty run",
        ))
    } else {
        Ok(())
    }
}
fn separator(value: char, path: &str) -> Result<(), TextValueError> {
    if value.is_control()
        || matches!(value, '\u{2028}' | '\u{2029}' | '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
    {
        Err(TextValueError::new(
            TextValueErrorCode::InvalidContent,
            path,
            "separator must be a non-control Unicode scalar",
        ))
    } else {
        Ok(())
    }
}

pub fn validate_font_request(font: &FontRequest) -> Result<(), TextValueError> {
    if font.family.is_none() && font.cad_font_name.is_none() {
        return Err(TextValueError::new(
            TextValueErrorCode::InvalidFont,
            "/font",
            "font requires a family or CAD font name",
        ));
    }
    for (field, name) in [
        ("family", &font.family),
        ("cadFontName", &font.cad_font_name),
        ("bigFontName", &font.big_font_name),
    ] {
        if let Some(name) = name {
            if name.is_empty() || name.contains('\0') {
                return Err(TextValueError::new(
                    TextValueErrorCode::InvalidFont,
                    format!("/font/{field}"),
                    "font name must be nonempty and NUL-free",
                ));
            }
        }
    }
    if font.big_font_name.is_some() && font.cad_font_name.is_none() {
        return Err(TextValueError::new(
            TextValueErrorCode::InvalidFont,
            "/font/bigFontName",
            "big font requires a CAD font name",
        ));
    }
    Ok(())
}

pub fn validate_text_style(style: &TextStyleProperties) -> Result<(), TextValueError> {
    validate_font_request(&style.font)?;
    positive(style.width_factor, "/widthFactor")?;
    oblique(style.oblique_angle, "/obliqueAngle")?;
    if let Some(height) = style.creation_height {
        positive(height, "/creationHeight")?;
    }
    if let Some(height) = style.last_used_height {
        nonnegative(height, "/lastUsedHeight")?;
    }
    Ok(())
}

pub fn validate_text_layout(layout: &TextLayout) -> Result<(), TextValueError> {
    match *layout {
        TextLayout::Anchored {
            height,
            width_factor,
            ..
        }
        | TextLayout::WholeTextMiddle {
            height,
            width_factor,
        } => {
            positive(height, "/layout/height")?;
            positive(width_factor, "/layout/widthFactor")?;
            product(height, width_factor, "/layout/effectiveAdvance")?;
        }
        TextLayout::Aligned {
            length,
            width_factor,
        } => {
            positive(length, "/layout/length")?;
            positive(width_factor, "/layout/widthFactor")?;
        }
        TextLayout::Fit { length, height } => {
            positive(length, "/layout/length")?;
            positive(height, "/layout/height")?;
        }
    }
    Ok(())
}

pub fn validate_text_runs(runs: &[TextRun]) -> Result<(), TextValueError> {
    for (i, run) in runs.iter().enumerate() {
        literal(&run.text, &format!("/content/{i}/text"), false)?;
    }
    Ok(())
}

pub fn validate_character_format<C>(
    format: &CharacterFormat<C>,
    validate_color: impl Fn(&C) -> bool,
) -> Result<(), TextValueError> {
    if let Some(font) = &format.font {
        validate_font_request(font)?;
    }
    if let Some(TextColor::Explicit(color)) = &format.color {
        if !validate_color(color) {
            return Err(TextValueError::new(
                TextValueErrorCode::InvalidColor,
                "/color",
                "invalid explicit color",
            ));
        }
    }
    if let Some(height) = format.height {
        match height {
            TextHeight::Relative { factor } => positive(factor, "/height/factor")?,
            TextHeight::Absolute { distance } => positive(distance, "/height/distance")?,
        }
    }
    if let Some(value) = format.width_factor {
        positive(value, "/widthFactor")?;
    }
    if let Some(value) = format.tracking {
        positive(value, "/tracking")?;
    }
    if let Some(value) = format.oblique_angle {
        oblique(value, "/obliqueAngle")?;
    }
    Ok(())
}

pub(super) fn validate_tab_stops(stops: &[TextTabStop]) -> Result<(), TextValueError> {
    let mut previous = 0.0;
    for (index, stop) in stops.iter().enumerate() {
        let path = format!("/tabStops/{index}");
        positive(stop.position_factor, &format!("{path}/positionFactor"))?;
        if stop.position_factor <= previous {
            return Err(TextValueError::new(
                TextValueErrorCode::InvalidTabs,
                path,
                "tab stops must be strictly increasing",
            ));
        }
        if let TextTabAlignment::Decimal { separator: value } = stop.alignment {
            separator(value, &format!("{path}/separator"))?;
        }
        previous = stop.position_factor;
    }
    Ok(())
}

pub fn validate_paragraph_format(format: &ParagraphFormat) -> Result<(), TextValueError> {
    for (field, value) in [
        ("leftIndentFactor", format.left_indent_factor),
        ("rightIndentFactor", format.right_indent_factor),
        ("firstLineIndentFactor", format.first_line_indent_factor),
    ] {
        if let Some(value) = value {
            finite(value, &format!("/{field}"))?;
        }
    }
    for (field, value) in [
        ("spaceBefore", format.space_before),
        ("spaceAfter", format.space_after),
    ] {
        if let Some(value) = value {
            nonnegative(value, &format!("/{field}"))?;
        }
    }
    if let Some(spacing) = format.line_spacing {
        match spacing {
            TextLineSpacing::Exact { distance } | TextLineSpacing::AtLeast { distance } => {
                positive(distance, "/lineSpacing/distance")?
            }
            TextLineSpacing::Multiple { factor } => positive(factor, "/lineSpacing/factor")?,
        }
    }
    if let Some(stops) = &format.tab_stops {
        validate_tab_stops(stops)?;
    }
    Ok(())
}

pub fn validate_mtext_columns(columns: &MTextColumns) -> Result<(), TextValueError> {
    let (width, gutter, count) = match columns {
        MTextColumns::Static {
            count,
            column_width,
            gutter,
            column_height,
            ..
        } => {
            positive(*column_height, "/columns/columnHeight")?;
            (*column_width, *gutter, u64::from(*count))
        }
        MTextColumns::DynamicAutoHeight {
            current_column_count,
            column_width,
            gutter,
            column_height,
            ..
        } => {
            positive(*column_height, "/columns/columnHeight")?;
            (*column_width, *gutter, u64::from(*current_column_count))
        }
        MTextColumns::DynamicManualHeight {
            column_width,
            gutter,
            column_heights,
            ..
        } => {
            for (i, height) in column_heights.iter().enumerate() {
                match height {
                    MTextColumnHeight::Fixed { distance } => {
                        positive(*distance, &format!("/columns/columnHeights/{i}/distance"))?
                    }
                    MTextColumnHeight::Auto if i + 1 == column_heights.len() => {}
                    MTextColumnHeight::Auto => {
                        return Err(TextValueError::new(
                            TextValueErrorCode::InvalidColumns,
                            format!("/columns/columnHeights/{i}"),
                            "only the final column may have auto height",
                        ))
                    }
                }
            }
            (*column_width, *gutter, column_heights.len() as u64)
        }
    };
    positive(width, "/columns/columnWidth")?;
    nonnegative(gutter, "/columns/gutter")?;
    if count == 0 {
        return Err(TextValueError::new(
            TextValueErrorCode::InvalidColumns,
            "/columns",
            "columns require a nonzero count",
        ));
    }
    let total = product(width, count as f64, "/columns/totalWidth")?
        + product(gutter, (count - 1) as f64, "/columns/totalWidth")?;
    if !total.is_finite() {
        return Err(TextValueError::new(
            TextValueErrorCode::DerivedOutOfRange,
            "/columns/totalWidth",
            "total column width is not finite",
        ));
    }
    Ok(())
}

pub fn validate_resolved_paragraph_width(
    format: &ResolvedParagraphFormat,
    nominal_height: f64,
    width: Option<f64>,
) -> Result<(), TextValueError> {
    positive(nominal_height, "/height")?;
    let left = product(
        format.left_indent_factor,
        nominal_height,
        "/paragraphFormat/leftIndentFactor",
    )?;
    let right = product(
        format.right_indent_factor,
        nominal_height,
        "/paragraphFormat/rightIndentFactor",
    )?;
    let first = product(
        format.first_line_indent_factor,
        nominal_height,
        "/paragraphFormat/firstLineIndentFactor",
    )?;
    if let Some(width) = width {
        positive(width, "/wrapWidth")?;
        let usable = width - left - right;
        positive(usable, "/paragraphFormat/usableWidth")?;
        positive(usable - first, "/paragraphFormat/firstLineUsableWidth")?;
    }
    Ok(())
}

pub fn validate_mtext_content<C>(
    content: &[MTextParagraph<C>],
    context: MTextValueContext<'_>,
    validate_color: impl Fn(&C) -> bool,
) -> Result<(), TextValueError> {
    validate_mtext_content_with_paragraph_format(
        content,
        context,
        &ParagraphFormat::default(),
        validate_color,
    )
}

/// Validates content against the authored entity-wide paragraph basis.
pub fn validate_mtext_content_with_paragraph_format<C>(
    content: &[MTextParagraph<C>],
    context: MTextValueContext<'_>,
    paragraph_format: &ParagraphFormat,
    validate_color: impl Fn(&C) -> bool,
) -> Result<(), TextValueError> {
    positive(context.nominal_height, "/height")?;
    if content.is_empty() {
        return Err(TextValueError::new(
            TextValueErrorCode::InvalidContent,
            "/content",
            "MText requires at least one paragraph",
        ));
    }
    if context.columns.is_some() && context.wrap_width.is_some() {
        return Err(TextValueError::new(
            TextValueErrorCode::InvalidColumns,
            "/wrapWidth",
            "columns forbid ordinary wrap width",
        ));
    }
    let width = if let Some(columns) = context.columns {
        validate_mtext_columns(columns)?;
        Some(match columns {
            MTextColumns::Static { column_width, .. }
            | MTextColumns::DynamicAutoHeight { column_width, .. }
            | MTextColumns::DynamicManualHeight { column_width, .. } => *column_width,
        })
    } else {
        context.wrap_width
    };
    if let Some(width) = width {
        positive(width, "/wrapWidth")?;
    }
    for (i, paragraph) in content.iter().enumerate() {
        let path = format!("/content/{i}");
        validate_paragraph_format(&paragraph.paragraph_format)
            .map_err(|error| error.prefixed(&format!("{path}/paragraphFormat")))?;
        let resolved = resolve_paragraph_format(paragraph_format, &paragraph.paragraph_format)?;
        validate_resolved_paragraph_width(&resolved, context.nominal_height, width)
            .map_err(|error| error.prefixed(&path))?;
        validate_character_format(&paragraph.character_format, &validate_color)
            .map_err(|error| error.prefixed(&format!("{path}/characterFormat")))?;
        for (j, inline) in paragraph.inlines.iter().enumerate() {
            let inline_path = format!("{path}/inlines/{j}");
            match inline {
                MTextInline::Run {
                    text,
                    character_format,
                } => {
                    literal(text, &format!("{inline_path}/text"), false)?;
                    validate_character_format(character_format, &validate_color).map_err(
                        |error| error.prefixed(&format!("{inline_path}/characterFormat")),
                    )?;
                }
                MTextInline::Stack(stack) => {
                    literal(&stack.upper, &format!("{inline_path}/upper"), true)?;
                    literal(&stack.lower, &format!("{inline_path}/lower"), true)?;
                    if stack.upper.is_empty() && stack.lower.is_empty() {
                        return Err(TextValueError::new(
                            TextValueErrorCode::InvalidContent,
                            inline_path,
                            "stack parts cannot both be empty",
                        ));
                    }
                    positive(stack.text_scale, &format!("{inline_path}/textScale"))?;
                    match (stack.stack_kind, stack.separator) {
                        (TextStackKind::DecimalTolerance, Some(value)) => {
                            separator(value, &format!("{inline_path}/separator"))?
                        }
                        (TextStackKind::DecimalTolerance, None) | (_, Some(_)) => {
                            return Err(TextValueError::new(
                                TextValueErrorCode::InvalidContent,
                                format!("{inline_path}/separator"),
                                "only decimal tolerance requires a separator",
                            ))
                        }
                        (_, None) => {}
                    }
                    validate_character_format(&stack.character_format, &validate_color).map_err(
                        |error| error.prefixed(&format!("{inline_path}/characterFormat")),
                    )?;
                }
                MTextInline::ColumnBreak if context.columns.is_none() => {
                    return Err(TextValueError::new(
                        TextValueErrorCode::InvalidColumns,
                        inline_path,
                        "column break requires a column configuration",
                    ))
                }
                _ => {}
            }
        }
    }
    Ok(())
}

/// Checks the actual inherited heights/advances, including empty paragraphs.
/// Color semantics stay with the consuming drawing format.
pub fn validate_mtext_effective_formats<C: Clone>(
    content: &[MTextParagraph<C>],
    context: MTextValueContext<'_>,
    style: &TextStyleProperties,
    character_format: &CharacterFormat<C>,
    paragraph_format: &ParagraphFormat,
    validate_color: impl Fn(&C) -> bool,
) -> Result<(), TextValueError> {
    validate_mtext_content_with_paragraph_format(
        content,
        context,
        paragraph_format,
        &validate_color,
    )?;
    validate_character_format(character_format, &validate_color)
        .map_err(|error| error.prefixed("/characterFormat"))?;
    let empty = CharacterFormat::default();
    for (i, paragraph) in content.iter().enumerate() {
        if needs_paragraph_base_format(paragraph) {
            resolve_character_format(
                context.nominal_height,
                style,
                character_format,
                &paragraph.character_format,
                &empty,
            )
            .map_err(|error| error.prefixed(&format!("/content/{i}")))?;
        }
        for (j, inline) in paragraph.inlines.iter().enumerate() {
            let (format, scale) = match inline {
                MTextInline::Run {
                    character_format, ..
                } => (character_format, 1.0),
                MTextInline::Stack(stack) => (&stack.character_format, stack.text_scale),
                _ => continue,
            };
            let effective = resolve_character_format(
                context.nominal_height,
                style,
                character_format,
                &paragraph.character_format,
                format,
            )
            .map_err(|error| error.prefixed(&format!("/content/{i}/inlines/{j}")))?;
            let path = format!("/content/{i}/inlines/{j}/effectiveAdvance");
            let height = product(effective.height, scale, &path)?;
            let width = product(height, effective.width_factor, &path)?;
            product(width, effective.tracking, &path)?;
        }
    }
    Ok(())
}

pub(super) fn needs_paragraph_base_format<C>(paragraph: &MTextParagraph<C>) -> bool {
    let mut line_has_text = false;
    let mut empty_line = false;
    for inline in &paragraph.inlines {
        match inline {
            MTextInline::Run { .. } | MTextInline::Stack(_) => line_has_text = true,
            MTextInline::LineBreak | MTextInline::ColumnBreak => {
                empty_line |= !line_has_text;
                line_has_text = false;
            }
            MTextInline::Tab => {}
        }
    }
    empty_line || !line_has_text
}

pub fn validate_background<C>(
    background: &MTextBackground<C>,
    validate_color: impl Fn(&C) -> bool,
) -> Result<(), TextValueError> {
    match background.padding {
        TextPadding::Absolute { distance } => {
            nonnegative(distance, "/background/padding/distance")?
        }
        TextPadding::Relative { factor } => nonnegative(factor, "/background/padding/factor")?,
    }
    if !background.opacity.is_finite() || !(0.0..=1.0).contains(&background.opacity) {
        return Err(TextValueError::new(
            TextValueErrorCode::InvalidBackground,
            "/background/opacity",
            "opacity must be finite in 0..1",
        ));
    }
    if let MTextFill::Color(color) = &background.fill {
        if !validate_color(color) {
            return Err(TextValueError::new(
                TextValueErrorCode::InvalidColor,
                "/background/fill/color",
                "invalid explicit fill color",
            ));
        }
    }
    Ok(())
}
