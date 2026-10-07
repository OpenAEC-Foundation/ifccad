use crate::content::error;
use crate::markup::{TextMarkupError, TextMarkupErrorCode};
use ocdraw::text::*;

pub(crate) fn parse_paragraph(
    body: &str,
    out: &mut ParagraphFormat,
    offset: usize,
) -> Result<(), TextMarkupError> {
    if body.is_empty() {
        return Err(error(
            TextMarkupErrorCode::UnsupportedCode,
            offset,
            "legacy empty paragraph/reset code is not qualified",
        ));
    }
    let bytes = body.as_bytes();
    let mut i = usize::from(bytes.first() == Some(&b'x'));
    while i < bytes.len() {
        if bytes[i] == b',' {
            i += 1;
            continue;
        }
        let key = bytes[i];
        i += 1;
        match key {
            b'q' => {
                let value = *bytes.get(i).ok_or_else(|| {
                    error(
                        TextMarkupErrorCode::MalformedCode,
                        offset,
                        "paragraph alignment value missing",
                    )
                })?;
                i += 1;
                out.alignment = Some(match value {
                    b'l' => TextParagraphAlignment::Left,
                    b'c' => TextParagraphAlignment::Center,
                    b'r' => TextParagraphAlignment::Right,
                    b'j' | b'*' => TextParagraphAlignment::Justified,
                    b'd' => TextParagraphAlignment::Distributed,
                    _ => {
                        return Err(error(
                            TextMarkupErrorCode::UnsupportedCode,
                            offset,
                            "unknown paragraph alignment",
                        ))
                    }
                });
            }
            b'i' | b'l' | b'r' => {
                let start = i;
                while bytes
                    .get(i)
                    .is_some_and(|c| c.is_ascii_digit() || matches!(c, b'.' | b'-' | b'+'))
                {
                    i += 1;
                }
                let value = body[start..i].parse::<f64>().map_err(|_| {
                    error(
                        TextMarkupErrorCode::InvalidNumber,
                        offset,
                        "invalid indent factor",
                    )
                })?;
                if !value.is_finite() {
                    return Err(error(
                        TextMarkupErrorCode::InvalidNumber,
                        offset,
                        "indent must be finite",
                    ));
                }
                match key {
                    b'i' => out.first_line_indent_factor = Some(value),
                    b'l' => out.left_indent_factor = Some(value),
                    _ => out.right_indent_factor = Some(value),
                }
            }
            b't' => {
                // Only left/center/right stops are qualified. Decimal syntax
                // can include a comma marker; never guess by splitting that list.
                let remaining = &body[i..];
                if remaining.bytes().any(|c| matches!(c, b'D' | b'd')) {
                    return Err(error(
                        TextMarkupErrorCode::UnsupportedCode,
                        offset,
                        "decimal tab separator syntax is not qualified",
                    ));
                }
                let mut stops = Vec::new();
                if !remaining.is_empty() {
                    for part in remaining.split(',') {
                        if part.is_empty() {
                            return Err(error(
                                TextMarkupErrorCode::MalformedCode,
                                offset,
                                "empty tab stop in source list",
                            ));
                        }
                        let (alignment, number) = if let Some(number) =
                            part.strip_prefix('c').or_else(|| part.strip_prefix('C'))
                        {
                            (TextTabAlignment::Center, number)
                        } else if let Some(number) =
                            part.strip_prefix('r').or_else(|| part.strip_prefix('R'))
                        {
                            (TextTabAlignment::Right, number)
                        } else {
                            (TextTabAlignment::Left, part)
                        };
                        let position_factor = number.parse::<f64>().map_err(|_| {
                            error(
                                TextMarkupErrorCode::InvalidNumber,
                                offset,
                                "invalid tab factor",
                            )
                        })?;
                        stops.push(TextTabStop {
                            position_factor,
                            alignment,
                        });
                    }
                }
                out.tab_stops = Some(stops);
                i = bytes.len();
            }
            b'a' | b'b' | b's' => {
                return Err(error(
                    TextMarkupErrorCode::UnsupportedCode,
                    offset,
                    "local paragraph spacing/line-spacing units or semantics are unqualified",
                ))
            }
            _ => {
                return Err(error(
                    TextMarkupErrorCode::UnsupportedCode,
                    offset,
                    "unknown paragraph property",
                ))
            }
        }
    }
    validate_paragraph_format(out)
        .map_err(|e| error(TextMarkupErrorCode::MalformedCode, offset, e.message))
}

pub(crate) fn emit_paragraph(format: &ParagraphFormat) -> Result<String, TextMarkupError> {
    validate_paragraph_format(format)
        .map_err(|e| error(TextMarkupErrorCode::MalformedCode, 0, e.message))?;
    if format.space_before.is_some()
        || format.space_after.is_some()
        || format.line_spacing.is_some()
    {
        return Err(error(
            TextMarkupErrorCode::UnsupportedCode,
            0,
            "local paragraph spacing has no qualified CAD mapping",
        ));
    }
    let mut parts = Vec::new();
    if let Some(alignment) = format.alignment {
        parts.push(format!(
            "q{}",
            match alignment {
                TextParagraphAlignment::Left => 'l',
                TextParagraphAlignment::Center => 'c',
                TextParagraphAlignment::Right => 'r',
                TextParagraphAlignment::Justified => 'j',
                TextParagraphAlignment::Distributed => 'd',
            }
        ));
    }
    for (prefix, value) in [
        ('i', format.first_line_indent_factor),
        ('l', format.left_indent_factor),
        ('r', format.right_indent_factor),
    ] {
        if let Some(value) = value {
            parts.push(format!("{prefix}{value}"));
        }
    }
    if let Some(stops) = &format.tab_stops {
        let mut values = Vec::new();
        for stop in stops {
            let prefix = match stop.alignment {
                TextTabAlignment::Left => "",
                TextTabAlignment::Center => "c",
                TextTabAlignment::Right => "r",
                TextTabAlignment::Decimal { .. } => {
                    return Err(error(
                        TextMarkupErrorCode::UnsupportedCode,
                        0,
                        "decimal tab emission is not qualified",
                    ))
                }
            };
            values.push(format!("{prefix}{}", stop.position_factor));
        }
        parts.push(format!("t{}", values.join(",")));
    }
    if parts.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("\\px{};", parts.join(",")))
    }
}
