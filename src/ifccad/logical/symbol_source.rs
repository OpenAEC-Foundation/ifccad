//! Independent drawing-local symbol identities for the experimental profile.
use super::*;
use crate::line_pattern::*;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IfccadSymbolSourceId(pub u64);
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadSymbolSource {
    pub id: IfccadSymbolSourceId,
    pub source: SymbolSource,
}
#[derive(Clone, Debug, PartialEq)]
pub enum IfccadLinePatternMarker {
    Text {
        text: String,
        text_style_id: IfccadTextStyleId,
        transform: LinePatternTransform,
    },
    Symbol {
        symbol_source_id: IfccadSymbolSourceId,
        shape_number: u32,
        transform: LinePatternTransform,
    },
}
pub type IfccadLinePatternElement = LinePatternElement<IfccadLinePatternMarker>;

pub(crate) fn validate_sources(document: &IfccadDocument) -> Result<(), IfccadReport> {
    let prefix = format!("/cad/d{}", document.drawing_id);
    let mut ids = BTreeSet::new();
    for row in &document.symbol_sources {
        if !ids.insert(row.id) || validate_symbol_source(&row.source).is_err() {
            return Err(crate::ifccad::diagnostics::failure(
                "IFCCAD-SYMBOL-001",
                &format!("{prefix}/symbolSource/{}", row.id.0),
                "invalid source or duplicate identity",
            ));
        }
    }
    let styles: BTreeSet<_> = document.text_styles.iter().map(|s| s.id).collect();
    for pattern in &document.line_patterns {
        for (i, e) in pattern.pattern.iter().enumerate() {
            let Some(m) = &e.marker else {
                continue;
            };
            let (transform, resolved) = match m {
                IfccadLinePatternMarker::Text {
                    text_style_id,
                    transform,
                    ..
                } => (transform, styles.contains(text_style_id)),
                IfccadLinePatternMarker::Symbol {
                    symbol_source_id,
                    transform,
                    ..
                } => (transform, ids.contains(symbol_source_id)),
            };
            if !resolved || validate_line_pattern_transform(transform).is_err() {
                return Err(crate::ifccad::diagnostics::failure(
                    "IFCCAD-PATTERN-002",
                    &format!("{prefix}/linePattern/{}/pattern/{i}/marker", pattern.id.0),
                    "invalid transform or unresolved marker reference",
                ));
            }
        }
    }
    Ok(())
}
