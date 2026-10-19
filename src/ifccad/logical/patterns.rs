use super::*;
use std::collections::BTreeSet;

/// Validate the profile's drawing-local definitions without a CAD runtime.
pub fn validate_ifccad_line_patterns(patterns: &[IfccadLinePattern]) -> Result<(), IfccadReport> {
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for p in patterns {
        let name = crate::names::name_key(&p.name);
        if p.name.is_empty()
            || !ids.insert(p.id)
            || !names.insert(name.clone())
            || name == "bylayer"
            || name == "byblock"
            || (name == "continuous" && !p.pattern.is_empty())
            || crate::line_pattern::validate_line_pattern_values(&p.pattern).is_err()
        {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-PATTERN-002",
                &format!("/linePattern/{}", p.id.0),
                format!("invalid or duplicate line pattern {} ({})", p.id.0, p.name),
            ));
        }
    }
    Ok(())
}

pub(crate) fn scale(value: f64, context: &str) -> Result<(), IfccadReport> {
    if !value.is_finite() || value <= 0. {
        return Err(crate::ifccad::diagnostics::failure(
            "IFCCAD-PATTERN-001",
            context,
            format!("{context} invalid line pattern scale"),
        ));
    }
    Ok(())
}
pub(crate) fn one() -> f64 {
    1.
}
