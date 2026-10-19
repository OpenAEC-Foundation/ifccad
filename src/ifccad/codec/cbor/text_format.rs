use super::*;
use crate::text::{
    CharacterFormat, FontRequest, MTextAttachment, MTextFlow, MTextInline, MTextParagraph,
    ParagraphFormat, TextColor, TextHeight, TextHorizontalAlignment, TextLayout, TextLineSpacing,
    TextParagraphAlignment, TextPosition, TextRun, TextStack, TextStackAlignment, TextStackKind,
    TextTabAlignment, TextTabStop, TextVerticalAlignment,
};
unit_enum!(MTextAttachment;
0 => TopLeft,
1 => TopCenter,
2 => TopRight,
3 => MiddleLeft,
4 => MiddleCenter,
5 => MiddleRight,
6 => BottomLeft,
7 => BottomCenter,
8 => BottomRight,
);
impl CborValue for TextLayout {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "Anchored"),
            (1, "WholeTextMiddle"),
            (2, "Aligned"),
            (3, "Fit"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Anchored {
                horizontal,
                vertical,
                height,
                width_factor,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if horizontal.present()
                    && (None as Option<TextHorizontalAlignment>)
                        .as_ref()
                        .is_none_or(|d| !horizontal.exact(d))
                {
                    __cbor_fields += 1;
                }
                if vertical.present()
                    && (None as Option<TextVerticalAlignment>)
                        .as_ref()
                        .is_none_or(|d| !vertical.exact(d))
                {
                    __cbor_fields += 1;
                }
                if height.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !height.exact(d))
                {
                    __cbor_fields += 1;
                }
                if width_factor.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !width_factor.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if horizontal.present()
                    && (None as Option<TextHorizontalAlignment>)
                        .as_ref()
                        .is_none_or(|d| !horizontal.exact(d))
                {
                    w.u64(0)?;
                    horizontal.encode(w)?;
                }
                if vertical.present()
                    && (None as Option<TextVerticalAlignment>)
                        .as_ref()
                        .is_none_or(|d| !vertical.exact(d))
                {
                    w.u64(1)?;
                    vertical.encode(w)?;
                }
                if height.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !height.exact(d))
                {
                    w.u64(2)?;
                    height.encode(w)?;
                }
                if width_factor.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !width_factor.exact(d))
                {
                    w.u64(3)?;
                    width_factor.encode(w)?;
                }
                Ok(())
            }
            Self::WholeTextMiddle {
                height,
                width_factor,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if height.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !height.exact(d))
                {
                    __cbor_fields += 1;
                }
                if width_factor.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !width_factor.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if height.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !height.exact(d))
                {
                    w.u64(0)?;
                    height.encode(w)?;
                }
                if width_factor.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !width_factor.exact(d))
                {
                    w.u64(1)?;
                    width_factor.encode(w)?;
                }
                Ok(())
            }
            Self::Aligned {
                length,
                width_factor,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if length.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !length.exact(d))
                {
                    __cbor_fields += 1;
                }
                if width_factor.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !width_factor.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if length.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !length.exact(d))
                {
                    w.u64(0)?;
                    length.encode(w)?;
                }
                if width_factor.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !width_factor.exact(d))
                {
                    w.u64(1)?;
                    width_factor.encode(w)?;
                }
                Ok(())
            }
            Self::Fit { length, height } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(3)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if length.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !length.exact(d))
                {
                    __cbor_fields += 1;
                }
                if height.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !height.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if length.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !length.exact(d))
                {
                    w.u64(0)?;
                    length.encode(w)?;
                }
                if height.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !height.exact(d))
                {
                    w.u64(1)?;
                    height.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(
                    &body,
                    &[
                        (0, "horizontal"),
                        (1, "vertical"),
                        (2, "height"),
                        (3, "width_factor"),
                    ],
                )?;
                let horizontal: TextHorizontalAlignment = b.take(0, None)?;
                let vertical: TextVerticalAlignment = b.take(1, None)?;
                let height: f64 = b.take(2, None)?;
                let width_factor: f64 = b.take(3, None)?;
                b.finish()?;
                Self::Anchored {
                    horizontal,
                    vertical,
                    height,
                    width_factor,
                }
            }
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "height"), (1, "width_factor")])?;
                let height: f64 = b.take(0, None)?;
                let width_factor: f64 = b.take(1, None)?;
                b.finish()?;
                Self::WholeTextMiddle {
                    height,
                    width_factor,
                }
            }
            2 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "length"), (1, "width_factor")])?;
                let length: f64 = b.take(0, None)?;
                let width_factor: f64 = b.take(1, None)?;
                b.finish()?;
                Self::Aligned {
                    length,
                    width_factor,
                }
            }
            3 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "length"), (1, "height")])?;
                let length: f64 = b.take(0, None)?;
                let height: f64 = b.take(1, None)?;
                b.finish()?;
                Self::Fit { length, height }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (
                Self::Anchored {
                    horizontal,
                    vertical,
                    height,
                    width_factor,
                },
                Self::Anchored {
                    horizontal: horizontal_other,
                    vertical: vertical_other,
                    height: height_other,
                    width_factor: width_factor_other,
                },
            ) => {
                horizontal.exact(horizontal_other)
                    && vertical.exact(vertical_other)
                    && height.exact(height_other)
                    && width_factor.exact(width_factor_other)
            }
            (
                Self::WholeTextMiddle {
                    height,
                    width_factor,
                },
                Self::WholeTextMiddle {
                    height: height_other,
                    width_factor: width_factor_other,
                },
            ) => height.exact(height_other) && width_factor.exact(width_factor_other),
            (
                Self::Aligned {
                    length,
                    width_factor,
                },
                Self::Aligned {
                    length: length_other,
                    width_factor: width_factor_other,
                },
            ) => length.exact(length_other) && width_factor.exact(width_factor_other),
            (
                Self::Fit { length, height },
                Self::Fit {
                    length: length_other,
                    height: height_other,
                },
            ) => length.exact(length_other) && height.exact(height_other),
            _ => false,
        }
    }
}
unit_enum!(TextHorizontalAlignment;
0 => Left,
1 => Center,
2 => Right,
);
unit_enum!(TextVerticalAlignment;
0 => Baseline,
1 => Bottom,
2 => Middle,
3 => Top,
);
record!(TextRun;
0 => text: String = None,
1 => underline: bool = None,
2 => overline: bool = None,
3 => strike_through: bool = None,
);
unit_enum!(MTextFlow;
0 => Horizontal,
1 => Vertical,
2 => ByStyle,
);
record!(CharacterFormat<IfccadColor>;
0 => font: Option<FontRequest> = Some(None),
1 => color: Option<TextColor<IfccadColor>> = Some(None),
2 => height: Option<TextHeight> = Some(None),
3 => width_factor: Option<f64> = Some(None),
4 => tracking: Option<f64> = Some(None),
5 => oblique_angle: Option<f64> = Some(None),
6 => underline: Option<bool> = Some(None),
7 => overline: Option<bool> = Some(None),
8 => strike_through: Option<bool> = Some(None),
9 => position: Option<TextPosition> = Some(None),
);
record!(ParagraphFormat;
0 => alignment: Option<TextParagraphAlignment> = Some(None),
1 => left_indent_factor: Option<f64> = Some(None),
2 => right_indent_factor: Option<f64> = Some(None),
3 => first_line_indent_factor: Option<f64> = Some(None),
4 => space_before: Option<f64> = Some(None),
5 => space_after: Option<f64> = Some(None),
6 => line_spacing: Option<TextLineSpacing> = Some(None),
7 => tab_stops: Option<Vec<TextTabStop>> = Some(None),
);
record!(MTextParagraph<IfccadColor>;
0 => paragraph_format: ParagraphFormat = None,
1 => character_format: CharacterFormat<IfccadColor> = None,
2 => inlines: Vec<MTextInline<IfccadColor>> = None,
);
impl CborValue for TextColor<IfccadColor> {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "Entity"),
            (1, "ByLayer"),
            (2, "ByBlock"),
            (3, "Explicit"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Entity => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::ByLayer => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(1)?;
                Ok(())
            }
            Self::ByBlock => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(2)?;
                Ok(())
            }
            Self::Explicit(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(3)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<IfccadColor>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<IfccadColor>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => Self::Entity,
            1 => Self::ByLayer,
            2 => Self::ByBlock,
            3 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: IfccadColor = b.take(0, None)?;
                b.finish()?;
                Self::Explicit(value_0)
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::Entity, Self::Entity) => true,
            (Self::ByLayer, Self::ByLayer) => true,
            (Self::ByBlock, Self::ByBlock) => true,
            (Self::Explicit(value_0), Self::Explicit(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            _ => false,
        }
    }
}
impl CborValue for TextHeight {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Relative"), (1, "Absolute")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Relative { factor } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if factor.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !factor.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if factor.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !factor.exact(d))
                {
                    w.u64(0)?;
                    factor.encode(w)?;
                }
                Ok(())
            }
            Self::Absolute { distance } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if distance.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !distance.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if distance.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !distance.exact(d))
                {
                    w.u64(0)?;
                    distance.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "factor")])?;
                let factor: f64 = b.take(0, None)?;
                b.finish()?;
                Self::Relative { factor }
            }
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "distance")])?;
                let distance: f64 = b.take(0, None)?;
                b.finish()?;
                Self::Absolute { distance }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (
                Self::Relative { factor },
                Self::Relative {
                    factor: factor_other,
                },
            ) => factor.exact(factor_other),
            (
                Self::Absolute { distance },
                Self::Absolute {
                    distance: distance_other,
                },
            ) => distance.exact(distance_other),
            _ => false,
        }
    }
}
unit_enum!(TextPosition;
0 => Bottom,
1 => Center,
2 => Top,
);
unit_enum!(TextParagraphAlignment;
0 => Left,
1 => Center,
2 => Right,
3 => Justified,
4 => Distributed,
);
impl CborValue for TextLineSpacing {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Exact"), (1, "AtLeast"), (2, "Multiple")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Exact { distance } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if distance.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !distance.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if distance.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !distance.exact(d))
                {
                    w.u64(0)?;
                    distance.encode(w)?;
                }
                Ok(())
            }
            Self::AtLeast { distance } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if distance.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !distance.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if distance.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !distance.exact(d))
                {
                    w.u64(0)?;
                    distance.encode(w)?;
                }
                Ok(())
            }
            Self::Multiple { factor } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if factor.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !factor.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if factor.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !factor.exact(d))
                {
                    w.u64(0)?;
                    factor.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "distance")])?;
                let distance: f64 = b.take(0, None)?;
                b.finish()?;
                Self::Exact { distance }
            }
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "distance")])?;
                let distance: f64 = b.take(0, None)?;
                b.finish()?;
                Self::AtLeast { distance }
            }
            2 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "factor")])?;
                let factor: f64 = b.take(0, None)?;
                b.finish()?;
                Self::Multiple { factor }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (
                Self::Exact { distance },
                Self::Exact {
                    distance: distance_other,
                },
            ) => distance.exact(distance_other),
            (
                Self::AtLeast { distance },
                Self::AtLeast {
                    distance: distance_other,
                },
            ) => distance.exact(distance_other),
            (
                Self::Multiple { factor },
                Self::Multiple {
                    factor: factor_other,
                },
            ) => factor.exact(factor_other),
            _ => false,
        }
    }
}
record!(TextTabStop;
0 => position_factor: f64 = None,
1 => alignment: TextTabAlignment = None,
);
impl CborValue for MTextInline<IfccadColor> {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "Run"),
            (1, "Tab"),
            (2, "LineBreak"),
            (3, "ColumnBreak"),
            (4, "Stack"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Run {
                text,
                character_format,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if text.present()
                    && (None as Option<String>)
                        .as_ref()
                        .is_none_or(|d| !text.exact(d))
                {
                    __cbor_fields += 1;
                }
                if character_format.present()
                    && (None as Option<CharacterFormat<IfccadColor>>)
                        .as_ref()
                        .is_none_or(|d| !character_format.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if text.present()
                    && (None as Option<String>)
                        .as_ref()
                        .is_none_or(|d| !text.exact(d))
                {
                    w.u64(0)?;
                    text.encode(w)?;
                }
                if character_format.present()
                    && (None as Option<CharacterFormat<IfccadColor>>)
                        .as_ref()
                        .is_none_or(|d| !character_format.exact(d))
                {
                    w.u64(1)?;
                    character_format.encode(w)?;
                }
                Ok(())
            }
            Self::Tab => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(1)?;
                Ok(())
            }
            Self::LineBreak => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(2)?;
                Ok(())
            }
            Self::ColumnBreak => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(3)?;
                Ok(())
            }
            Self::Stack(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(4)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<TextStack<IfccadColor>>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<TextStack<IfccadColor>>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "text"), (1, "character_format")])?;
                let text: String = b.take(0, None)?;
                let character_format: CharacterFormat<IfccadColor> = b.take(1, None)?;
                b.finish()?;
                Self::Run {
                    text,
                    character_format,
                }
            }
            1 => Self::Tab,
            2 => Self::LineBreak,
            3 => Self::ColumnBreak,
            4 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: TextStack<IfccadColor> = b.take(0, None)?;
                b.finish()?;
                Self::Stack(value_0)
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (
                Self::Run {
                    text,
                    character_format,
                },
                Self::Run {
                    text: text_other,
                    character_format: character_format_other,
                },
            ) => text.exact(text_other) && character_format.exact(character_format_other),
            (Self::Tab, Self::Tab) => true,
            (Self::LineBreak, Self::LineBreak) => true,
            (Self::ColumnBreak, Self::ColumnBreak) => true,
            (Self::Stack(value_0), Self::Stack(value_0_other)) => value_0.exact(value_0_other),
            _ => false,
        }
    }
}
impl CborValue for TextTabAlignment {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Left"), (1, "Center"), (2, "Right"), (3, "Decimal")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Left => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::Center => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(1)?;
                Ok(())
            }
            Self::Right => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(2)?;
                Ok(())
            }
            Self::Decimal { separator } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(3)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if separator.present()
                    && (None as Option<char>)
                        .as_ref()
                        .is_none_or(|d| !separator.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if separator.present()
                    && (None as Option<char>)
                        .as_ref()
                        .is_none_or(|d| !separator.exact(d))
                {
                    w.u64(0)?;
                    separator.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => Self::Left,
            1 => Self::Center,
            2 => Self::Right,
            3 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "separator")])?;
                let separator: char = b.take(0, None)?;
                b.finish()?;
                Self::Decimal { separator }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::Left, Self::Left) => true,
            (Self::Center, Self::Center) => true,
            (Self::Right, Self::Right) => true,
            (
                Self::Decimal { separator },
                Self::Decimal {
                    separator: separator_other,
                },
            ) => separator.exact(separator_other),
            _ => false,
        }
    }
}
record!(TextStack<IfccadColor>;
0 => stack_kind: TextStackKind = None,
1 => upper: String = None,
2 => lower: String = None,
3 => alignment: TextStackAlignment = None,
4 => text_scale: f64 = None,
5 => separator: Option<char> = Some(None),
6 => character_format: CharacterFormat<IfccadColor> = None,
);
unit_enum!(TextStackKind;
0 => Fraction,
1 => DiagonalFraction,
2 => Tolerance,
3 => DecimalTolerance,
);
unit_enum!(TextStackAlignment;
0 => Top,
1 => Center,
2 => Bottom,
);
