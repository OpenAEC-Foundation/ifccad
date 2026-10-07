use crate::content::{error, valid_color, MarkupColor};
use crate::markup::{escape_mtext_literal, TextMarkupError, TextMarkupErrorCode};
use ocdraw::text::*;

#[derive(Clone, Copy, Debug, Default)]
pub enum CadTextProfile {
    #[default]
    Pinned,
}

pub struct MTextMarkupInput<'a> {
    pub content: &'a [MTextParagraph<MarkupColor>],
    pub character_format: &'a CharacterFormat<MarkupColor>,
    pub paragraph_format: &'a ParagraphFormat,
}

pub fn emit_text(content: &[TextRun]) -> Result<String, TextMarkupError> {
    validate_text_runs(content)
        .map_err(|e| error(TextMarkupErrorCode::MalformedCode, 0, e.message))?;
    let mut result = String::new();
    let mut underline = false;
    let mut overline = false;
    for run in content {
        if run.strike_through {
            return Err(error(
                TextMarkupErrorCode::UnsupportedCode,
                0,
                "TEXT strikethrough emission is not qualified",
            ));
        }
        if underline != run.underline {
            result.push_str("%%u");
            underline = run.underline;
        }
        if overline != run.overline {
            result.push_str("%%o");
            overline = run.overline;
        }
        for c in run.text.chars() {
            match c {
                '\\' => result.push_str("%%092"),
                '%' => result.push_str("%%037"),
                '^' => result.push_str("%%094"),
                _ => result.push(c),
            }
        }
    }
    Ok(result)
}

pub fn emit_mtext(
    input: MTextMarkupInput<'_>,
    _profile: CadTextProfile,
) -> Result<String, TextMarkupError> {
    if input.content.is_empty() {
        return Err(error(
            TextMarkupErrorCode::MalformedCode,
            0,
            "MText requires at least one paragraph",
        ));
    }
    let mut result = emit_format(input.character_format, &CharacterFormat::default())?;
    for (index, paragraph) in input.content.iter().enumerate() {
        if index > 0 {
            result.push_str("\\P");
        }
        let pformat = merge_paragraph(input.paragraph_format, &paragraph.paragraph_format);
        result.push_str(&crate::paragraph::emit_paragraph(&pformat)?);
        if paragraph.inlines.is_empty() && paragraph.character_format != CharacterFormat::default()
        {
            return Err(error(
                TextMarkupErrorCode::UnsupportedCode,
                0,
                "formatted empty paragraph needs source-layout qualification",
            ));
        }
        result.push('{');
        result.push_str(&emit_format(
            &paragraph.character_format,
            input.character_format,
        )?);
        let parent = merge_character(input.character_format, &paragraph.character_format);
        for inline in &paragraph.inlines {
            match inline {
                MTextInline::Run {
                    text,
                    character_format,
                } => {
                    if text.is_empty() {
                        return Err(error(
                            TextMarkupErrorCode::MalformedCode,
                            0,
                            "empty run cannot be emitted",
                        ));
                    }
                    result.push('{');
                    result.push_str(&emit_format(character_format, &parent)?);
                    result.push_str(&escape_mtext_literal(text)?);
                    result.push('}');
                }
                MTextInline::Tab => result.push_str("^I"),
                MTextInline::ColumnBreak => result.push_str("\\N"),
                MTextInline::LineBreak => {
                    return Err(error(
                        TextMarkupErrorCode::UnsupportedCode,
                        0,
                        "native intra-paragraph line break has no qualified file-roundtrip mapping",
                    ))
                }
                MTextInline::Stack(stack) => {
                    if stack.upper.is_empty() && stack.lower.is_empty() {
                        return Err(error(
                            TextMarkupErrorCode::MalformedCode,
                            0,
                            "stack cannot have two empty parts",
                        ));
                    }
                    if stack.text_scale != 0.7
                        || stack.alignment != TextStackAlignment::Center
                        || stack.separator.is_some()
                    {
                        return Err(error(
                            TextMarkupErrorCode::UnsupportedCode,
                            0,
                            "stack scale/position/decimal options have no qualified mapping",
                        ));
                    }
                    let separator = match stack.stack_kind {
                        TextStackKind::Fraction => '/',
                        TextStackKind::DiagonalFraction => '#',
                        TextStackKind::Tolerance => '^',
                        TextStackKind::DecimalTolerance => {
                            return Err(error(
                                TextMarkupErrorCode::UnsupportedCode,
                                0,
                                "decimal tolerance has no qualified mapping",
                            ))
                        }
                    };
                    if stack
                        .upper
                        .chars()
                        .chain(stack.lower.chars())
                        .any(|c| matches!(c, '/' | '#' | '^'))
                    {
                        return Err(error(
                            TextMarkupErrorCode::UnsupportedCode,
                            0,
                            "stack delimiter escaping is not qualified",
                        ));
                    }
                    result.push('{');
                    result.push_str(&emit_format(&stack.character_format, &parent)?);
                    result.push_str("\\S");
                    result.push_str(&escape_mtext_literal(&stack.upper)?);
                    result.push(separator);
                    result.push_str(&escape_mtext_literal(&stack.lower)?);
                    result.push_str(";}");
                }
            }
        }
        result.push('}');
    }
    Ok(result)
}

fn emit_format(
    format: &CharacterFormat<MarkupColor>,
    parent: &CharacterFormat<MarkupColor>,
) -> Result<String, TextMarkupError> {
    validate_character_format(format, valid_color)
        .map_err(|e| error(TextMarkupErrorCode::MalformedCode, 0, e.message))?;
    let mut out = String::new();
    if let Some(font) = &format.font {
        if font.big_font_name.is_some() || (font.family.is_some() && font.cad_font_name.is_some()) {
            return Err(error(
                TextMarkupErrorCode::UnsupportedCode,
                0,
                "inline combined/big-font requests have no qualified code",
            ));
        }
        let (prefix, name) = if let Some(name) = &font.family {
            ("\\f", name)
        } else {
            ("\\FN", font.cad_font_name.as_ref().unwrap())
        };
        if name
            .chars()
            .any(|c| matches!(c, '|' | ';' | '\\' | '{' | '}'))
        {
            return Err(error(
                TextMarkupErrorCode::UnsupportedCode,
                0,
                "font-name delimiter escaping is not qualified",
            ));
        }
        out.push_str(prefix);
        out.push_str(name);
        if let Some(value) = font.bold {
            out.push_str(if value { "|b1" } else { "|b0" });
        }
        if let Some(value) = font.italic {
            out.push_str(if value { "|i1" } else { "|i0" });
        }
        if let Some(value) = font.charset {
            out.push_str(&format!("|c{value}"));
        }
        if let Some(value) = font.pitch {
            out.push_str(&format!("|p{value}"));
        }
        out.push(';');
    }
    if let Some(color) = &format.color {
        match color {
            TextColor::ByLayer => out.push_str("\\C256;"),
            TextColor::ByBlock => out.push_str("\\C0;"),
            TextColor::Explicit(MarkupColor::Index(index)) => out.push_str(&format!("\\C{index};")),
            TextColor::Explicit(MarkupColor::Rgb { red, green, blue }) => out.push_str(&format!(
                "\\c{};",
                u32::from(*red) | (u32::from(*green) << 8) | (u32::from(*blue) << 16)
            )),
            TextColor::Entity
                if parent
                    .color
                    .as_ref()
                    .is_none_or(|color| *color == TextColor::Entity) => {}
            TextColor::Entity => {
                return Err(error(
                    TextMarkupErrorCode::UnsupportedCode,
                    0,
                    "entity-color reset under an active color override needs a qualified context",
                ))
            }
        }
    }
    if let Some(height) = format.height {
        match height {
            TextHeight::Absolute { distance } => out.push_str(&format!("\\H{distance};")),
            TextHeight::Relative { factor } => {
                let parent = match parent.height { Some(TextHeight::Absolute {..}) => return Err(error(TextMarkupErrorCode::UnsupportedCode,0,"relative-to-nominal height cannot be expressed below an absolute source height")), Some(TextHeight::Relative {factor}) => factor, None => 1. };
                let ratio = factor / parent;
                let ratio = [ratio, ratio.next_down(), ratio.next_up()]
                    .into_iter()
                    .find(|ratio| ratio.is_finite() && *ratio > 0. && parent * ratio == factor)
                    .ok_or_else(|| {
                        error(
                            TextMarkupErrorCode::UnsupportedCode,
                            0,
                            "height factor cannot be emitted without parameter loss",
                        )
                    })?;
                out.push_str(&format!("\\H{ratio}x;"));
            }
        }
    }
    if let Some(value) = format.width_factor {
        out.push_str(&format!("\\W{value};"));
    }
    if let Some(value) = format.tracking {
        out.push_str(&format!("\\T{value};"));
    }
    if let Some(value) = format.oblique_angle {
        let degrees = value.to_degrees();
        if degrees.to_radians() != value {
            return Err(error(
                TextMarkupErrorCode::UnsupportedCode,
                0,
                "oblique degree/radian emission would change the parameter",
            ));
        }
        out.push_str(&format!("\\Q{degrees};"));
    }
    for (value, on, off) in [
        (format.underline, "\\L", "\\l"),
        (format.overline, "\\O", "\\o"),
        (format.strike_through, "\\K", "\\k"),
    ] {
        if let Some(value) = value {
            out.push_str(if value { on } else { off });
        }
    }
    if let Some(position) = format.position {
        out.push_str(match position {
            TextPosition::Bottom => "\\A0;",
            TextPosition::Center => "\\A1;",
            TextPosition::Top => "\\A2;",
        });
    }
    Ok(out)
}

fn merge_character(
    parent: &CharacterFormat<MarkupColor>,
    child: &CharacterFormat<MarkupColor>,
) -> CharacterFormat<MarkupColor> {
    let mut result = parent.clone();
    macro_rules! replace { ($($field:ident),+) => { $(if child.$field.is_some() { result.$field = child.$field.clone(); })+ }; }
    replace!(
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
    result
}
fn merge_paragraph(parent: &ParagraphFormat, child: &ParagraphFormat) -> ParagraphFormat {
    let mut result = parent.clone();
    macro_rules! replace { ($($field:ident),+) => { $(if child.$field.is_some() { result.$field = child.$field.clone(); })+ }; }
    replace!(
        alignment,
        left_indent_factor,
        right_indent_factor,
        first_line_indent_factor,
        space_before,
        space_after,
        line_spacing,
        tab_stops
    );
    result
}
