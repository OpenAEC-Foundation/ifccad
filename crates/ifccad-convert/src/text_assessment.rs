//! Text anchor evidence is independent of font and layout fidelity.
use crate::IfccadGeometryOwner;
use ocdraw::ifccad::*;
use std::collections::BTreeSet;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadTextNumericCoverage {
    ActiveTextAnchors,
    MTextWcsAnchor,
    NotTransferred,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadTextGlyphCoverage {
    Empty,
    Unassessed,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfccadTextAssessmentEntry {
    pub owner: IfccadGeometryOwner,
    pub entity_id: u64,
    pub numeric_coverage: IfccadTextNumericCoverage,
    pub glyph_coverage: IfccadTextGlyphCoverage,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IfccadTextAssessment {
    entries: Vec<IfccadTextAssessmentEntry>,
}
impl IfccadTextAssessment {
    pub fn entries(&self) -> &[IfccadTextAssessmentEntry] {
        &self.entries
    }
    pub(crate) fn new(d: &IfccadDocument, transferred: impl IntoIterator<Item = u64>) -> Self {
        let transferred = transferred.into_iter().collect::<BTreeSet<_>>();
        let mut entries = Vec::new();
        let scopes = std::iter::once((
            IfccadGeometryOwner::ModelLayout(d.model.id),
            &d.model.entities,
        ))
        .chain(
            d.paper_layouts
                .iter()
                .map(|p| (IfccadGeometryOwner::PaperLayout(p.id), &p.entities)),
        )
        .chain(
            d.blocks
                .iter()
                .map(|b| (IfccadGeometryOwner::BlockDefinition(b.id), &b.entities)),
        );
        for (owner, entities) in scopes {
            for e in entities {
                let Some(e) = e.as_native() else {
                    continue;
                };
                let (numeric, glyph) = match &e.kind {
                    IfccadEntityKind::Text(t) => (
                        IfccadTextNumericCoverage::ActiveTextAnchors,
                        if t.content.is_empty() {
                            IfccadTextGlyphCoverage::Empty
                        } else {
                            IfccadTextGlyphCoverage::Unassessed
                        },
                    ),
                    IfccadEntityKind::MText(_) => (
                        IfccadTextNumericCoverage::MTextWcsAnchor,
                        IfccadTextGlyphCoverage::Unassessed,
                    ),
                    _ => continue,
                };
                entries.push(IfccadTextAssessmentEntry {
                    owner,
                    entity_id: e.id,
                    numeric_coverage: if transferred.contains(&e.id) {
                        numeric
                    } else {
                        IfccadTextNumericCoverage::NotTransferred
                    },
                    glyph_coverage: glyph,
                });
            }
        }
        Self { entries }
    }
}
