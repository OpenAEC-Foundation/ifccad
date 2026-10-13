//! Local symbol source identities. Filenames are requests, not resolved assets.
use super::{DrawingLinePatternMarker, LogicalError, OcdrawDocument};
use crate::line_pattern::{validate_line_pattern_transform, validate_symbol_source, SymbolSource};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct SymbolSourceId(pub u32);

#[derive(Clone, Debug, PartialEq)]
pub struct DrawingSymbolSource {
    pub id: SymbolSourceId,
    pub source: SymbolSource,
}

pub(crate) fn validate_symbol_sources(doc: &OcdrawDocument) -> Vec<LogicalError> {
    let mut errors = Vec::new();
    let mut ids = BTreeSet::new();
    for (i, row) in doc.symbol_sources.iter().enumerate() {
        let location = format!("/symbolSources/{i}");
        if !ids.insert(row.id)
            || row.id.0 >= doc.next_symbol_source_id
            || validate_symbol_source(&row.source).is_err()
        {
            errors.push(LogicalError {
                code: "SYMBOL_SOURCE_VALUE",
                location,
                message: "invalid source, identity or allocation watermark".into(),
            });
        }
    }
    let styles: BTreeSet<_> = doc.text_styles.iter().map(|s| s.id).collect();
    for (i, pattern) in doc.line_patterns.iter().enumerate() {
        for (j, element) in pattern.pattern.iter().enumerate() {
            if let Some(marker) = &element.marker {
                let (transform, resolved) = match marker {
                    DrawingLinePatternMarker::Text {
                        text_style_id,
                        transform,
                        ..
                    } => (transform, styles.contains(text_style_id)),
                    DrawingLinePatternMarker::Symbol {
                        symbol_source_id,
                        transform,
                        ..
                    } => (transform, ids.contains(symbol_source_id)),
                };
                if !resolved || validate_line_pattern_transform(transform).is_err() {
                    errors.push(LogicalError {
                        code: "LINE_PATTERN_MARKER",
                        location: format!("/linePatterns/{i}/pattern/{j}/marker"),
                        message: "invalid marker transform or unresolved local reference".into(),
                    });
                }
            }
        }
    }
    errors
}
