use super::*;

/// Escapes native literal content, including strings that resemble field syntax.
/// Lexical safety is tested here; actual CAD writer/readback is a separate gate.
pub fn escape_mtext_literal(text: &str) -> Result<String, TextMarkupError> {
    let mut result = String::new();
    for (offset, c) in text.char_indices() {
        match c {
            '\\' => result.push_str(r"\\"),
            '{' => result.push_str(r"\{"),
            '}' => result.push_str(r"\}"),
            '%' => result.push_str(r"\U+0025"),
            '^' => result.push_str(r"\U+005E"),
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
