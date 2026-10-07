//! Typed content normalization, separate from drawing-owned style/reference lookup.
use crate::markup::*;
use ocdraw::text::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MarkupColor {
    Index(u16),
    Rgb { red: u8, green: u8, blue: u8 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParsedMTextContent {
    /// Source state-machine overrides were factored into a paragraph basis.
    /// Current formatting is retained; authored editing dependencies can change.
    pub character_basis_normalized: bool,
    pub character_format: CharacterFormat<MarkupColor>,
    pub paragraph_format: ParagraphFormat,
    pub content: Vec<MTextParagraph<MarkupColor>>,
}

pub fn parse_text(source: &str, limits: MarkupLimits) -> Result<Vec<TextRun>, TextMarkupError> {
    let mut result: Vec<TextRun> = Vec::new();
    let mut decorations = [false; 3];
    let mut scopes = Vec::new();
    for token in lex_text(source, limits)? {
        match token.kind {
            MarkupTokenKind::Literal(text) => {
                if let Some(last) = result.last_mut().filter(|last| {
                    [last.underline, last.overline, last.strike_through] == decorations
                }) {
                    last.text.push_str(&text);
                } else {
                    result.push(TextRun {
                        text,
                        underline: decorations[0],
                        overline: decorations[1],
                        strike_through: decorations[2],
                    });
                }
            }
            MarkupTokenKind::ScopeStart => scopes.push(decorations),
            MarkupTokenKind::ScopeEnd => {
                decorations = scopes.pop().expect("lexer checked balanced scopes")
            }
            MarkupTokenKind::Decoration {
                decoration,
                enabled,
                toggle,
            } => {
                if decoration == MarkupDecoration::StrikeThrough {
                    return Err(error(
                        TextMarkupErrorCode::UnsupportedCode,
                        token.range.start,
                        "TEXT legacy strikethrough is not qualified",
                    ));
                }
                let index = match decoration {
                    MarkupDecoration::Underline => 0,
                    MarkupDecoration::Overline => 1,
                    MarkupDecoration::StrikeThrough => 2,
                };
                decorations[index] = if toggle { !decorations[index] } else { enabled };
            }
            _ => {
                return Err(error(
                    TextMarkupErrorCode::UnsupportedCode,
                    token.range.start,
                    "Text supports literal content and run decorations only",
                ))
            }
        }
    }
    validate_text_runs(&result)
        .map_err(|e| error(TextMarkupErrorCode::MalformedCode, 0, e.message))?;
    Ok(result)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Level {
    Global,
    Paragraph,
    Inline,
}
#[derive(Clone)]
struct State {
    format: CharacterFormat<MarkupColor>,
    levels: [Level; 10],
}
impl Default for State {
    fn default() -> Self {
        Self {
            format: Default::default(),
            levels: [Level::Global; 10],
        }
    }
}

impl State {
    fn selected(&self, level: Level) -> CharacterFormat<MarkupColor> {
        let mut f = CharacterFormat::default();
        macro_rules! copy {
            ($i:expr, $field:ident) => {
                if self.levels[$i] == level {
                    f.$field = self.format.$field.clone();
                }
            };
        }
        copy!(0, font);
        copy!(1, color);
        copy!(2, height);
        copy!(3, width_factor);
        copy!(4, tracking);
        copy!(5, oblique_angle);
        copy!(6, underline);
        copy!(7, overline);
        copy!(8, strike_through);
        copy!(9, position);
        f
    }
}

pub fn parse_mtext(
    source: &str,
    limits: MarkupLimits,
) -> Result<ParsedMTextContent, TextMarkupError> {
    let tokens = lex(source, limits)?;
    let mut result = ParsedMTextContent {
        character_basis_normalized: false,
        character_format: Default::default(),
        paragraph_format: Default::default(),
        content: vec![MTextParagraph::default()],
    };
    let mut state = State::default();
    let mut scopes = Vec::new();
    let mut started = false;
    let mut font_bytes = 0usize;
    for token in tokens {
        let offset = token.range.start;
        let level = if !scopes.is_empty() {
            Level::Inline
        } else if !started {
            Level::Global
        } else if result.content.last().unwrap().inlines.is_empty() {
            Level::Paragraph
        } else {
            Level::Inline
        };
        let field = match token.kind {
            MarkupTokenKind::ScopeStart => {
                charge(
                    &mut font_bytes,
                    font_size(&state.format),
                    limits.max_bytes,
                    offset,
                )?;
                scopes.push(state.clone());
                continue;
            }
            MarkupTokenKind::ScopeEnd => {
                state = scopes.pop().expect("lexer checked scope balance");
                continue;
            }
            MarkupTokenKind::Literal(text) => {
                started = true;
                let f = state.selected(Level::Inline);
                charge(&mut font_bytes, font_size(&f), limits.max_bytes, offset)?;
                let paragraph = result.content.last_mut().unwrap();
                if let Some(MTextInline::Run {
                    text: previous,
                    character_format,
                }) = paragraph.inlines.last_mut()
                {
                    if *character_format == f {
                        previous.push_str(&text);
                        continue;
                    }
                }
                paragraph.inlines.push(MTextInline::Run {
                    text,
                    character_format: f,
                });
                continue;
            }
            MarkupTokenKind::ParagraphBreak
            | MarkupTokenKind::RawLineFeed
            | MarkupTokenKind::RawCarriageReturn
            | MarkupTokenKind::RawCarriageReturnLineFeed => {
                started = true;
                let inherited_level = if scopes.is_empty() {
                    Level::Paragraph
                } else {
                    Level::Inline
                };
                for level in &mut state.levels {
                    if *level != Level::Global {
                        *level = inherited_level;
                    }
                }
                // Saved contexts can belong to an earlier paragraph. Inside a
                // still-open group, keep their overrides on inlines so a later
                // scope pop restores the true source baseline in the new one.
                for saved in &mut scopes {
                    for level in &mut saved.levels {
                        if *level != Level::Global {
                            *level = Level::Inline;
                        }
                    }
                }
                let f = state.selected(Level::Paragraph);
                charge(&mut font_bytes, font_size(&f), limits.max_bytes, offset)?;
                result.content.push(MTextParagraph {
                    character_format: f,
                    ..Default::default()
                });
                continue;
            }
            MarkupTokenKind::CaretControl(_) => {
                return Err(error(
                    TextMarkupErrorCode::UnsupportedCode,
                    offset,
                    "caret line/carriage-return semantics are not qualified",
                ))
            }
            MarkupTokenKind::Tab => {
                started = true;
                result
                    .content
                    .last_mut()
                    .unwrap()
                    .inlines
                    .push(MTextInline::Tab);
                continue;
            }
            MarkupTokenKind::ColumnBreak => {
                started = true;
                result
                    .content
                    .last_mut()
                    .unwrap()
                    .inlines
                    .push(MTextInline::ColumnBreak);
                continue;
            }
            MarkupTokenKind::Stack(body) => {
                started = true;
                let (separator, index) = body
                    .char_indices()
                    .find_map(|(i, c)| {
                        if matches!(c, '/' | '#' | '^') {
                            Some((c, i))
                        } else {
                            None
                        }
                    })
                    .unwrap();
                let upper = literal_stack_part(&body[..index], limits, offset)?;
                let lower = literal_stack_part(&body[index + 1..], limits, offset)?;
                let format = state.selected(Level::Inline);
                charge(
                    &mut font_bytes,
                    font_size(&format),
                    limits.max_bytes,
                    offset,
                )?;
                result
                    .content
                    .last_mut()
                    .unwrap()
                    .inlines
                    .push(MTextInline::Stack(TextStack {
                        upper,
                        lower,
                        stack_kind: match separator {
                            '/' => TextStackKind::Fraction,
                            '#' => TextStackKind::DiagonalFraction,
                            _ => TextStackKind::Tolerance,
                        },
                        character_format: format,
                        ..Default::default()
                    }));
                continue;
            }
            MarkupTokenKind::ParagraphProperties(body) => {
                let paragraph = result.content.last_mut().unwrap();
                if !paragraph.inlines.is_empty() {
                    return Err(error(
                        TextMarkupErrorCode::UnsupportedCode,
                        offset,
                        "mid-paragraph layout mutation requires a qualified source profile",
                    ));
                }
                crate::paragraph::parse_paragraph(&body, &mut paragraph.paragraph_format, offset)?;
                continue;
            }
            MarkupTokenKind::Font { body, shx } => {
                let font = font(&body, shx, offset)?;
                charge(
                    &mut font_bytes,
                    font.family.as_ref().map_or(0, String::len)
                        + font.cad_font_name.as_ref().map_or(0, String::len),
                    limits.max_bytes,
                    offset,
                )?;
                state.format.font = Some(font);
                0
            }
            MarkupTokenKind::IndexedColor(index) => {
                state.format.color = Some(match index {
                    0 => TextColor::ByBlock,
                    256 => TextColor::ByLayer,
                    _ => TextColor::Explicit(MarkupColor::Index(index)),
                });
                1
            }
            MarkupTokenKind::PackedBgrColor(value) => {
                state.format.color = Some(TextColor::Explicit(MarkupColor::Rgb {
                    red: value as u8,
                    green: (value >> 8) as u8,
                    blue: (value >> 16) as u8,
                }));
                1
            }
            MarkupTokenKind::Scalar {
                property,
                value,
                relative,
            } => match property {
                MarkupScalarProperty::Height => {
                    state.format.height = Some(if relative {
                        match state
                            .format
                            .height
                            .unwrap_or(TextHeight::Relative { factor: 1. })
                        {
                            TextHeight::Relative { factor } => TextHeight::Relative {
                                factor: multiply(factor, value, offset)?,
                            },
                            TextHeight::Absolute { distance } => TextHeight::Absolute {
                                distance: multiply(distance, value, offset)?,
                            },
                        }
                    } else {
                        TextHeight::Absolute { distance: value }
                    });
                    2
                }
                MarkupScalarProperty::Width => {
                    state.format.width_factor = Some(if relative {
                        multiply(state.format.width_factor.unwrap_or(1.), value, offset)?
                    } else {
                        value
                    });
                    3
                }
                MarkupScalarProperty::Tracking => {
                    state.format.tracking = Some(if relative {
                        multiply(state.format.tracking.unwrap_or(1.), value, offset)?
                    } else {
                        value
                    });
                    4
                }
                MarkupScalarProperty::ObliqueDegrees => {
                    state.format.oblique_angle = Some(value.to_radians());
                    5
                }
            },
            MarkupTokenKind::Decoration {
                decoration,
                enabled,
                toggle,
            } => {
                if toggle && decoration == MarkupDecoration::StrikeThrough {
                    return Err(error(
                        TextMarkupErrorCode::UnsupportedCode,
                        offset,
                        "legacy percent strikethrough is not qualified",
                    ));
                }
                let (slot, field) = match decoration {
                    MarkupDecoration::Underline => (6, &mut state.format.underline),
                    MarkupDecoration::Overline => (7, &mut state.format.overline),
                    MarkupDecoration::StrikeThrough => (8, &mut state.format.strike_through),
                };
                *field = Some(if toggle {
                    !field.unwrap_or(false)
                } else {
                    enabled
                });
                slot
            }
            MarkupTokenKind::LineAlignment(value) => {
                state.format.position = Some(match value {
                    0 => TextPosition::Bottom,
                    1 => TextPosition::Center,
                    _ => TextPosition::Top,
                });
                9
            }
        };
        state.levels[field] = level;
        validate_character_format(&state.format, valid_color)
            .map_err(|e| error(TextMarkupErrorCode::MalformedCode, offset, e.message))?;
        match level {
            Level::Global => result.character_format = state.selected(Level::Global),
            Level::Paragraph => {
                result.content.last_mut().unwrap().character_format =
                    state.selected(Level::Paragraph)
            }
            Level::Inline => {}
        }
    }
    for paragraph in &mut result.content {
        result.character_basis_normalized |= factor_paragraph_character_basis(paragraph);
    }
    Ok(result)
}

fn font(body: &str, shx: bool, offset: usize) -> Result<FontRequest, TextMarkupError> {
    let mut parts = body.split('|');
    let name = parts.next().unwrap();
    if name.chars().any(|c| matches!(c, '\\' | '{' | '}')) {
        return Err(error(
            TextMarkupErrorCode::UnsupportedCode,
            offset,
            "escaped font-name syntax is not qualified",
        ));
    }
    let mut f = if shx {
        FontRequest {
            cad_font_name: Some(name.into()),
            ..Default::default()
        }
    } else {
        FontRequest::family(name)
    };
    for part in parts {
        if part.len() < 2 {
            return Err(error(
                TextMarkupErrorCode::MalformedCode,
                offset,
                "empty or incomplete font option",
            ));
        }
        let (flag, value) = part.split_at(1);
        match flag {
            "b" | "i" => {
                let value = match value {
                    "0" => false,
                    "1" => true,
                    _ => {
                        return Err(error(
                            TextMarkupErrorCode::MalformedCode,
                            offset,
                            "font boolean must be 0 or 1",
                        ))
                    }
                };
                if flag == "b" {
                    f.bold = Some(value);
                } else {
                    f.italic = Some(value);
                }
            }
            "c" => {
                f.charset = Some(value.parse().map_err(|_| {
                    error(
                        TextMarkupErrorCode::InvalidNumber,
                        offset,
                        "font charset exceeds uint16",
                    )
                })?)
            }
            "p" => {
                f.pitch = Some(value.parse().map_err(|_| {
                    error(
                        TextMarkupErrorCode::InvalidNumber,
                        offset,
                        "font pitch exceeds uint8",
                    )
                })?)
            }
            _ => {
                return Err(error(
                    TextMarkupErrorCode::UnsupportedCode,
                    offset,
                    "font option has no native mapping",
                ))
            }
        }
    }
    validate_font_request(&f)
        .map_err(|e| error(TextMarkupErrorCode::MalformedCode, offset, e.message))?;
    Ok(f)
}

fn literal_stack_part(
    source: &str,
    limits: MarkupLimits,
    offset: usize,
) -> Result<String, TextMarkupError> {
    let mut result = String::new();
    for token in lex(source, limits)? {
        if let MarkupTokenKind::Literal(text) = token.kind {
            result.push_str(&text);
        } else {
            return Err(error(
                TextMarkupErrorCode::UnsupportedCode,
                offset,
                "stack parts cannot contain nested formatting or structure",
            ));
        }
    }
    Ok(result)
}
fn multiply(a: f64, b: f64, offset: usize) -> Result<f64, TextMarkupError> {
    let result = a * b;
    if result.is_finite() && result > 0. {
        Ok(result)
    } else {
        Err(error(
            TextMarkupErrorCode::InvalidNumber,
            offset,
            "cumulative scalar exceeds finite positive range",
        ))
    }
}
fn font_size(f: &CharacterFormat<MarkupColor>) -> usize {
    f.font.as_ref().map_or(0, |font| {
        [&font.family, &font.cad_font_name, &font.big_font_name]
            .iter()
            .filter_map(|name| name.as_ref())
            .map(|name| name.len())
            .sum()
    })
}
fn charge(
    used: &mut usize,
    bytes: usize,
    limit: usize,
    offset: usize,
) -> Result<(), TextMarkupError> {
    *used = used
        .checked_add(bytes)
        .filter(|total| *total <= limit)
        .ok_or_else(|| {
            error(
                TextMarkupErrorCode::ResourceLimit,
                offset,
                "materialized font strings exceed byte budget",
            )
        })?;
    Ok(())
}
fn factor_paragraph_character_basis(paragraph: &mut MTextParagraph<MarkupColor>) -> bool {
    if paragraph
        .inlines
        .iter()
        .any(|inline| matches!(inline, MTextInline::LineBreak | MTextInline::ColumnBreak))
    {
        return false;
    }
    fn format(inline: &MTextInline<MarkupColor>) -> Option<&CharacterFormat<MarkupColor>> {
        match inline {
            MTextInline::Run {
                character_format, ..
            } => Some(character_format),
            MTextInline::Stack(stack) => Some(&stack.character_format),
            _ => None,
        }
    }
    let Some(first) = paragraph.inlines.iter().find_map(format).cloned() else {
        return false;
    };
    let mut normalized = false;
    macro_rules! factor {
        ($($field:ident),+) => { $(
            if paragraph.character_format.$field.is_none() && first.$field.is_some()
                && paragraph.inlines.iter().filter_map(format).all(|f| f.$field == first.$field) {
                paragraph.character_format.$field = first.$field.clone();
                normalized = true;
                for inline in &mut paragraph.inlines {
                    match inline { MTextInline::Run {character_format,..} => character_format.$field = None, MTextInline::Stack(stack) => stack.character_format.$field = None, _=>{} }
                }
            }
        )+ };
    }
    factor!(
        font,
        color,
        height,
        width_factor,
        tracking,
        oblique_angle,
        underline,
        overline,
        strike_through,
        position
    );
    normalized
}

pub(crate) fn valid_color(color: &MarkupColor) -> bool {
    match color {
        MarkupColor::Index(index) => (1..=255).contains(index),
        MarkupColor::Rgb { .. } => true,
    }
}
pub(crate) fn error(
    code: TextMarkupErrorCode,
    offset: usize,
    message: &'static str,
) -> TextMarkupError {
    TextMarkupError {
        code,
        offset,
        message,
    }
}
