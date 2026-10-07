//! Deterministic fontless MText layout estimates, never a rendering contract.
use super::extent::{checked, finish, transform};
use super::*;

#[derive(Clone, Copy, Default)]
struct Measure {
    width: f64,
    height: f64,
    shear: f64,
    ink: bool,
}
impl Measure {
    fn append(
        &mut self,
        width: f64,
        height: f64,
        shear: f64,
        ink: bool,
    ) -> Result<(), TextExtentError> {
        self.width = checked(self.width + width)?;
        self.height = self.height.max(height);
        self.shear = self.shear.max(shear);
        self.ink |= ink;
        Ok(())
    }
}

pub fn estimate_mtext_extent<C: Clone>(
    input: &ResolvedMTextExtentInput<'_, C>,
) -> Result<TextExtentEstimate, TextExtentError> {
    let context = MTextValueContext {
        nominal_height: input.height,
        wrap_width: input.wrap_width,
        columns: input.columns,
    };
    validate_mtext_effective_formats(
        input.content,
        context,
        input.style,
        input.character_format,
        input.paragraph_format,
        |_| true,
    )?;
    if let Some(background) = input.background {
        validate_background(background, |_| true)?;
    }
    if let Some(stroke) = input.frame_stroke_width {
        if !stroke.is_finite() || stroke < 0. {
            return Err(TextExtentError::OutOfRange);
        }
    }
    let prepared = transform(
        input.placement,
        input.rotation,
        input.backward,
        input.upside_down,
    )?;
    let mut layout = Layout::new(input)?;
    for paragraph in input.content {
        layout.paragraph(paragraph)?;
    }
    layout.include_configured_columns()?;
    let mut status = TextExtentStatus::Estimated;
    if let Some(bg) = input.background {
        if !matches!(bg.fill, MTextFill::None) || bg.frame {
            let pad = match bg.padding {
                TextPadding::Absolute { distance } => distance,
                TextPadding::Relative { factor } => checked(factor * input.height)?,
            };
            let stroke = if bg.frame {
                match input.frame_stroke_width {
                    Some(width) => width / 2.,
                    None => {
                        status = TextExtentStatus::Unavailable;
                        layout.reason(TextExtentReason::StrokeExtentUnavailable);
                        0.
                    }
                }
            } else {
                0.
            };
            let margin = checked(pad + stroke)?;
            layout.include(
                [-margin, checked(-layout.depth - margin)?],
                [checked(layout.width + margin)?, margin],
            )?;
        }
    }
    let vertical = input.flow == MTextFlow::Vertical
        || (input.flow == MTextFlow::ByStyle && input.style.vertical);
    if vertical {
        layout.reason(TextExtentReason::VerticalFlowEstimated);
        let (min, max) = (layout.min, layout.max);
        layout.min = [-max[1], -max[0]];
        layout.max = [-min[1], -min[0]];
        std::mem::swap(&mut layout.width, &mut layout.depth);
    }
    let horizontal = match input.attachment {
        MTextAttachment::TopLeft | MTextAttachment::MiddleLeft | MTextAttachment::BottomLeft => 0.,
        MTextAttachment::TopCenter
        | MTextAttachment::MiddleCenter
        | MTextAttachment::BottomCenter => -layout.width / 2.,
        _ => -layout.width,
    };
    let vertical = match input.attachment {
        MTextAttachment::TopLeft | MTextAttachment::TopCenter | MTextAttachment::TopRight => 0.,
        MTextAttachment::MiddleLeft
        | MTextAttachment::MiddleCenter
        | MTextAttachment::MiddleRight => layout.depth / 2.,
        _ => layout.depth,
    };
    finish(
        [
            checked(layout.min[0] + horizontal)?,
            checked(layout.min[1] + vertical)?,
            0.,
        ],
        [
            checked(layout.max[0] + horizontal)?,
            checked(layout.max[1] + vertical)?,
            0.,
        ],
        prepared,
        status,
        layout.reasons,
    )
}

struct Layout<'a, C> {
    input: &'a ResolvedMTextExtentInput<'a, C>,
    min: [f64; 2],
    max: [f64; 2],
    width: f64,
    depth: f64,
    column: u64,
    x: f64,
    y: f64,
    line_start: f64,
    line_height: f64,
    base_height: f64,
    row_touched: bool,
    row_ink: bool,
    row_extent: Option<([f64; 2], [f64; 2])>,
    alignment: TextParagraphAlignment,
    right_indent: f64,
    reasons: Vec<TextExtentReason>,
}

impl<'a, C: Clone> Layout<'a, C> {
    fn new(input: &'a ResolvedMTextExtentInput<'a, C>) -> Result<Self, TextExtentError> {
        let mut result = Self {
            input,
            min: [0., 0.],
            max: [0., 0.],
            width: 0.,
            depth: 0.,
            column: 0,
            x: 0.,
            y: 0.,
            line_start: 0.,
            line_height: input.height,
            base_height: input.height,
            row_touched: false,
            row_ink: false,
            row_extent: None,
            alignment: TextParagraphAlignment::Left,
            right_indent: 0.,
            reasons: vec![
                TextExtentReason::FontMetricsUnavailable,
                TextExtentReason::LayoutEstimated,
            ],
        };
        result.width = if input.columns.is_some() {
            checked(
                result.column_width().unwrap() * result.count() as f64
                    + result.gutter() * (result.count() - 1) as f64,
            )?
        } else {
            input.wrap_width.unwrap_or(0.)
        };
        Ok(result)
    }
    fn reason(&mut self, reason: TextExtentReason) {
        if !self.reasons.contains(&reason) {
            self.reasons.push(reason);
        }
    }
    fn count(&self) -> u64 {
        match self.input.columns {
            Some(MTextColumns::Static { count, .. }) => u64::from(*count),
            Some(MTextColumns::DynamicAutoHeight {
                current_column_count,
                ..
            }) => u64::from(*current_column_count),
            Some(MTextColumns::DynamicManualHeight { column_heights, .. }) => {
                column_heights.len() as u64
            }
            None => 1,
        }
    }
    fn column_width(&self) -> Option<f64> {
        match self.input.columns {
            Some(
                MTextColumns::Static { column_width, .. }
                | MTextColumns::DynamicAutoHeight { column_width, .. }
                | MTextColumns::DynamicManualHeight { column_width, .. },
            ) => Some(*column_width),
            None => self.input.wrap_width,
        }
    }
    fn gutter(&self) -> f64 {
        match self.input.columns {
            Some(
                MTextColumns::Static { gutter, .. }
                | MTextColumns::DynamicAutoHeight { gutter, .. }
                | MTextColumns::DynamicManualHeight { gutter, .. },
            ) => *gutter,
            None => 0.,
        }
    }
    fn column_height(&self) -> Option<f64> {
        match self.input.columns {
            Some(
                MTextColumns::Static { column_height, .. }
                | MTextColumns::DynamicAutoHeight { column_height, .. },
            ) => Some(*column_height),
            Some(MTextColumns::DynamicManualHeight { column_heights, .. }) => {
                match column_heights[self.column as usize] {
                    MTextColumnHeight::Fixed { distance } => Some(distance),
                    MTextColumnHeight::Auto => None,
                }
            }
            None => None,
        }
    }
    fn column_origin(&self) -> Result<f64, TextExtentError> {
        let reversed = match self.input.columns {
            Some(
                MTextColumns::Static { flow_reversed, .. }
                | MTextColumns::DynamicAutoHeight { flow_reversed, .. }
                | MTextColumns::DynamicManualHeight { flow_reversed, .. },
            ) => *flow_reversed,
            None => false,
        };
        let index = if reversed {
            self.count() - 1 - self.column
        } else {
            self.column
        };
        checked(index as f64 * checked(self.column_width().unwrap_or(0.) + self.gutter())?)
    }
    fn include(&mut self, min: [f64; 2], max: [f64; 2]) -> Result<(), TextExtentError> {
        for i in 0..2 {
            checked(min[i])?;
            checked(max[i])?;
            self.min[i] = self.min[i].min(min[i]);
            self.max[i] = self.max[i].max(max[i]);
        }
        Ok(())
    }
    fn next_column(&mut self) -> Result<(), TextExtentError> {
        if self.column + 1 < self.count() {
            self.column += 1;
            self.y = 0.;
        } else {
            self.reason(TextExtentReason::Overflow);
        }
        self.x = self.line_start;
        self.line_height = self.base_height;
        self.row_touched = false;
        self.row_ink = false;
        self.row_extent = None;
        Ok(())
    }
    fn finish_row(&mut self) -> Result<(), TextExtentError> {
        if let (Some((min, max)), Some(width)) = (self.row_extent, self.column_width()) {
            let shift = match self.alignment {
                TextParagraphAlignment::Left => 0.,
                TextParagraphAlignment::Center => {
                    checked((width - self.right_indent - self.x) / 2.)?
                }
                TextParagraphAlignment::Right
                | TextParagraphAlignment::Justified
                | TextParagraphAlignment::Distributed => {
                    checked(width - self.right_indent - self.x)?
                }
            };
            // Roomy union includes natural and aligned/stretched ink overhangs;
            // it is not a font-layout certificate or a renderer's geometry.
            self.include(
                [checked(min[0] + shift)?, min[1]],
                [checked(max[0] + shift)?, max[1]],
            )?;
        }
        let origin = self.column_origin()?;
        let bottom = checked(self.y + checked(3. * self.line_height)?)?;
        if self.column_height().is_some_and(|height| bottom > height) {
            self.reason(TextExtentReason::Overflow);
        }
        self.depth = self.depth.max(bottom);
        let width = self.column_width().unwrap_or(self.x.max(0.));
        if self.input.columns.is_none() {
            self.width = self.width.max(self.x.max(0.));
        }
        self.include([origin, -bottom], [checked(origin + width)?, -self.y])
    }
    fn newline(&mut self, format: &ResolvedParagraphFormat) -> Result<(), TextExtentError> {
        self.finish_row()?;
        let natural = checked(3. * self.line_height.max(self.input.height))?;
        let advance = match format.line_spacing {
            TextLineSpacing::Exact { distance } => distance,
            TextLineSpacing::AtLeast { distance } => distance.max(natural),
            TextLineSpacing::Multiple { factor } => checked(factor * natural)?,
        };
        self.y = checked(self.y + advance)?;
        self.line_start = checked(format.left_indent_factor * self.input.height)?;
        self.x = self.line_start;
        self.line_height = self.base_height;
        self.row_touched = false;
        self.row_ink = false;
        self.row_extent = None;
        Ok(())
    }
    fn place(
        &mut self,
        m: Measure,
        format: &ResolvedParagraphFormat,
        tab_start: Option<f64>,
    ) -> Result<(), TextExtentError> {
        let available = self
            .column_width()
            .map(|width| checked(width - format.right_indent_factor * self.input.height))
            .transpose()?;
        let mut start = tab_start.unwrap_or(self.x);
        let mut end = checked(start + m.width)?;
        if let Some(available) = available {
            if end > available && self.row_touched && tab_start.is_none() {
                self.newline(format)?;
                start = self.x;
                end = checked(start + m.width)?;
            }
            if end > available {
                self.reason(TextExtentReason::Overflow);
            }
        }
        if let Some(cap) = self.column_height() {
            if checked(self.y + 3. * m.height.max(self.line_height))? > cap {
                if self.y > 0. && self.column + 1 < self.count() {
                    self.finish_row()?;
                    self.next_column()?;
                    start = tab_start.unwrap_or(self.x);
                    end = checked(start + m.width)?;
                }
                if checked(self.y + 3. * m.height.max(self.line_height))? > cap {
                    self.reason(TextExtentReason::Overflow);
                }
            }
        }
        let origin = self.column_origin()?;
        if m.ink {
            let band = checked(4. * m.height)?;
            let shear = checked(2. * m.height * m.shear)?;
            let min = [checked(origin + start - shear)?, checked(-self.y - band)?];
            let max = [checked(origin + end + shear)?, -self.y];
            if let Some((lo, hi)) = &mut self.row_extent {
                for i in 0..2 {
                    lo[i] = lo[i].min(min[i]);
                    hi[i] = hi[i].max(max[i]);
                }
            } else {
                self.row_extent = Some((min, max));
            }
            self.include(min, max)?;
        }
        self.x = end;
        self.line_height = self.line_height.max(m.height);
        self.row_touched = true;
        self.row_ink |= m.ink;
        Ok(())
    }
    fn effective(
        &self,
        paragraph: &MTextParagraph<C>,
        format: &CharacterFormat<C>,
    ) -> Result<ResolvedCharacterFormat<C>, TextExtentError> {
        Ok(resolve_character_format(
            self.input.height,
            self.input.style,
            self.input.character_format,
            &paragraph.character_format,
            format,
        )?)
    }
    fn run_measure(
        &self,
        paragraph: &MTextParagraph<C>,
        text: &str,
        format: &CharacterFormat<C>,
    ) -> Result<Measure, TextExtentError> {
        let effective = self.effective(paragraph, format)?;
        Ok(Measure {
            width: checked(
                2. * effective.height
                    * effective.width_factor
                    * effective.tracking
                    * text.chars().count() as f64,
            )?,
            height: effective.height,
            shear: effective.oblique_angle.tan().abs(),
            ink: !text.is_empty(),
        })
    }
    fn stack_measure(
        &self,
        paragraph: &MTextParagraph<C>,
        stack: &TextStack<C>,
    ) -> Result<Measure, TextExtentError> {
        let effective = self.effective(paragraph, &stack.character_format)?;
        let height = checked(effective.height * stack.text_scale)?;
        let count = stack.upper.chars().count().max(stack.lower.chars().count());
        Ok(Measure {
            width: checked(
                2. * height * effective.width_factor * effective.tracking * count as f64
                    + effective.height,
            )?,
            height: checked((2. * height).max(effective.height))?,
            shear: effective.oblique_angle.tan().abs(),
            ink: true,
        })
    }
    fn field_measure(
        &self,
        paragraph: &MTextParagraph<C>,
        field: &[MTextInline<C>],
        separator: Option<char>,
    ) -> Result<(Measure, Option<f64>), TextExtentError> {
        let mut result = Measure::default();
        let mut prefix = None;
        for inline in field {
            let m = match inline {
                MTextInline::Run {
                    text,
                    character_format,
                } => {
                    let m = self.run_measure(paragraph, text, character_format)?;
                    if prefix.is_none() {
                        if let Some(separator) = separator {
                            if let Some(index) = text.find(separator) {
                                prefix = Some(checked(
                                    result.width
                                        + self
                                            .run_measure(
                                                paragraph,
                                                &text[..index],
                                                character_format,
                                            )?
                                            .width,
                                )?);
                            }
                        }
                    }
                    m
                }
                MTextInline::Stack(stack) => self.stack_measure(paragraph, stack)?,
                _ => unreachable!("field ends at a structural boundary"),
            };
            result.append(m.width, m.height, m.shear, m.ink)?;
        }
        result.height = result.height.max(self.input.height);
        Ok((result, prefix))
    }
    fn tab_field(
        &mut self,
        paragraph: &MTextParagraph<C>,
        field: &[MTextInline<C>],
        format: &ResolvedParagraphFormat,
    ) -> Result<(), TextExtentError> {
        let left = checked(format.left_indent_factor * self.input.height)?;
        let stop = next_tab_stop(
            &format.tab_stops,
            checked((self.x - left) / self.input.height)?,
        )?;
        let separator = match stop.alignment {
            TextTabAlignment::Decimal { separator } => Some(separator),
            _ => None,
        };
        let (measure, prefix) = self.field_measure(paragraph, field, separator)?;
        let offset = match stop.alignment {
            TextTabAlignment::Left => 0.,
            TextTabAlignment::Center => measure.width / 2.,
            TextTabAlignment::Right => measure.width,
            TextTabAlignment::Decimal { .. } => prefix.unwrap_or(measure.width),
        };
        let start = checked(left + stop.position_factor * self.input.height - offset)?;
        if let Some(width) = self.column_width() {
            let end = checked(start + measure.width)?;
            let available = checked(width - format.right_indent_factor * self.input.height)?;
            if end > available && self.row_touched {
                self.newline(format)?;
                return self.tab_field(paragraph, field, format);
            }
        }
        self.place(measure, format, Some(start))
    }
    fn paragraph(&mut self, paragraph: &MTextParagraph<C>) -> Result<(), TextExtentError> {
        self.base_height = if super::validation::needs_paragraph_base_format(paragraph) {
            self.effective(paragraph, &CharacterFormat::default())?
                .height
                .max(self.input.height)
        } else {
            self.input.height
        };
        let format =
            resolve_paragraph_format(self.input.paragraph_format, &paragraph.paragraph_format)?;
        self.alignment = format.alignment;
        self.right_indent = checked(format.right_indent_factor * self.input.height)?;
        self.y = checked(self.y + format.space_before)?;
        self.line_start = checked(
            (format.left_indent_factor + format.first_line_indent_factor) * self.input.height,
        )?;
        self.x = self.line_start;
        self.line_height = self.base_height;
        self.row_touched = false;
        self.row_ink = false;
        self.row_extent = None;
        let mut word = Measure::default();
        let mut index = 0;
        while index < paragraph.inlines.len() {
            match &paragraph.inlines[index] {
                MTextInline::Run {
                    text,
                    character_format,
                } => {
                    for part in text.split_inclusive(' ') {
                        let m = self.run_measure(paragraph, part, character_format)?;
                        word.append(m.width, m.height, m.shear, m.ink)?;
                        if part.ends_with(' ') {
                            self.place(word, &format, None)?;
                            word = Measure::default();
                        }
                    }
                }
                MTextInline::Stack(stack) => {
                    let m = self.stack_measure(paragraph, stack)?;
                    word.append(m.width, m.height, m.shear, m.ink)?;
                }
                boundary => {
                    if word.ink {
                        self.place(word, &format, None)?;
                        word = Measure::default();
                    }
                    match boundary {
                        MTextInline::LineBreak => self.newline(&format)?,
                        MTextInline::ColumnBreak => {
                            self.newline(&format)?;
                            self.next_column()?;
                        }
                        MTextInline::Tab => {
                            let start = index + 1;
                            let end = paragraph.inlines[start..]
                                .iter()
                                .position(|inline| {
                                    matches!(
                                        inline,
                                        MTextInline::Tab
                                            | MTextInline::LineBreak
                                            | MTextInline::ColumnBreak
                                    )
                                })
                                .map_or(paragraph.inlines.len(), |count| start + count);
                            self.tab_field(paragraph, &paragraph.inlines[start..end], &format)?;
                            index = end;
                            continue;
                        }
                        _ => unreachable!(),
                    }
                }
            }
            index += 1;
        }
        if word.ink {
            self.place(word, &format, None)?;
        }
        self.newline(&format)?;
        self.y = checked(self.y + format.space_after)?;
        self.depth = self.depth.max(self.y);
        Ok(())
    }
    fn include_configured_columns(&mut self) -> Result<(), TextExtentError> {
        let height = match self.input.columns {
            Some(
                MTextColumns::Static { column_height, .. }
                | MTextColumns::DynamicAutoHeight { column_height, .. },
            ) => *column_height,
            Some(MTextColumns::DynamicManualHeight { column_heights, .. }) => column_heights
                .iter()
                .filter_map(|height| match height {
                    MTextColumnHeight::Fixed { distance } => Some(*distance),
                    MTextColumnHeight::Auto => None,
                })
                .fold(self.depth, f64::max),
            None => self.depth,
        };
        self.depth = self.depth.max(height);
        self.include([0., -self.depth], [self.width, 0.])
    }
}
