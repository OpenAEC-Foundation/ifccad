use super::{DrawingLinePattern, LinePatternId, LogicalError};
use crate::ocdraw::names::name_key;
use std::collections::BTreeSet;

pub(crate) fn validate_line_patterns(
    rows: &[DrawingLinePattern],
    next: u32,
    refs: &[(LinePatternId, String)],
    scales: &[(f64, String)],
) -> Vec<LogicalError> {
    let mut errors = Vec::new();
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    let mut fail = |code, location, message: &str| {
        errors.push(LogicalError {
            code,
            location,
            message: message.into(),
        })
    };
    for (index, row) in rows.iter().enumerate() {
        let location = format!("/linePatterns/{index}");
        let key = name_key(&row.name);
        if !ids.insert(row.id) || !names.insert(key.clone()) || row.name.is_empty() {
            fail(
                "LINE_PATTERN_IDENTITY",
                location.clone(),
                "invalid or duplicate line pattern identity",
            );
        }
        if row.id.0 >= next {
            fail(
                "LINE_PATTERN_WATERMARK",
                "/header/nextLinePatternId".into(),
                "line pattern allocation watermark is not greater than allocated IDs",
            );
        }
        let total: f64 = row.pattern.iter().map(|v| v.abs()).sum();
        if key == name_key("ByLayer")
            || key == name_key("ByBlock")
            || (key == name_key("Continuous") && !row.pattern.is_empty())
            || row.pattern.iter().any(|v| !v.is_finite())
            || (!row.pattern.is_empty()
                && (row.pattern.len() < 2
                    || row.pattern[0] < 0.0
                    || !total.is_finite()
                    || total <= 0.0))
        {
            fail(
                "LINE_PATTERN_VALUE",
                location,
                "invalid simple line pattern",
            );
        }
    }
    for (id, location) in refs {
        if !ids.contains(id) {
            fail(
                "LINE_PATTERN_REFERENCE",
                location.clone(),
                "line pattern ID does not resolve locally",
            );
        }
    }
    for (scale, location) in scales {
        if !scale.is_finite() || *scale <= 0.0 {
            fail(
                "LINE_PATTERN_SCALE",
                location.clone(),
                "line pattern scale must be finite and positive",
            );
        }
    }
    errors
}
