use super::*;

/// Escapes native literal content, including strings that resemble field syntax.
/// Lexical safety is tested here; actual CAD writer/readback is a separate gate.
pub fn escape_mtext_literal(text: &str) -> Result<String, TextMarkupError> {
    let mut result = String::new();
    for (offset, c) in text.char_indices() {
        match c {
            // Reader CIF decoding precedes MTEXT parsing and is not escape-aware.
            // A scope boundary prevents a following literal U+XXXX from becoming
            // a pre-decoded Unicode escape, while retaining the literal slash.
            '\\' if text[offset + 1..].starts_with("U+") => result.push_str(r"{\\}"),
            '\\' => result.push_str(r"\\"),
            '{' => result.push_str(r"\{"),
            '}' => result.push_str(r"\}"),
            '%' => result.push_str("%%%%"),
            '^' => result.push_str("{^}"),
            '\u{a0}' => result.push_str(r"\~"),
            c if c.is_control() || matches!(c, '\u{2028}' | '\u{2029}') => {
                return Err(TextMarkupError::new(
                    TextMarkupErrorCode::StructuralControl,
                    offset,
                    "literal contains a structural control",
                ))
            }
            c => result.push(c),
        }
    }
    Ok(result)
}
