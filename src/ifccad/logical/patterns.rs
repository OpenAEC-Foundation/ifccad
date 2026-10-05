use super::*;
use std::collections::BTreeSet;

/// Validate the profile's drawing-local definitions without a CAD runtime.
pub fn validate_ifccad_line_patterns(patterns: &[IfccadLinePattern]) -> Result<(), IfccadReport> {
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for p in patterns {
        let name = crate::ocdraw::names::name_key(&p.name);
        let period = p.pattern.iter().map(|v| v.abs()).sum::<f64>();
        if p.name.is_empty()
            || !ids.insert(p.id)
            || !names.insert(name.clone())
            || name == "bylayer"
            || name == "byblock"
            || (name == "continuous" && !p.pattern.is_empty())
            || p.pattern.iter().any(|v| !v.is_finite())
            || (!p.pattern.is_empty()
                && (p.pattern.len() < 2
                    || p.pattern[0] < 0.
                    || !period.is_finite()
                    || period <= 0.))
        {
            return Err(IfccadReport::one(format!(
                "invalid or duplicate line pattern {} ({})",
                p.id.0, p.name
            )));
        }
    }
    Ok(())
}

pub(crate) fn scale(value: f64, context: &str) -> Result<(), IfccadReport> {
    if !value.is_finite() || value <= 0. {
        return Err(IfccadReport::one(format!(
            "{context} invalid line pattern scale"
        )));
    }
    Ok(())
}
pub(crate) fn one() -> f64 {
    1.
}
