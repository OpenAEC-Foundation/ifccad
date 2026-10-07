//! Evidence boundaries for text conversion; no font renderer is involved.
use ocdraw::ocdraw::OcdrawDocument;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OcdrawTextNumericCoverage {
    ActiveTextAnchors,
    MTextWcsAnchor,
    NotTransferred,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OcdrawTextGlyphCoverage {
    Empty,
    Unassessed,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OcdrawTextAssessmentEntry {
    pub entity_id: u64,
    pub numeric_coverage: OcdrawTextNumericCoverage,
    pub glyph_coverage: OcdrawTextGlyphCoverage,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OcdrawTextAssessment {
    entries: Vec<OcdrawTextAssessmentEntry>,
}
impl OcdrawTextAssessment {
    pub fn entries(&self) -> &[OcdrawTextAssessmentEntry] {
        &self.entries
    }
    pub(crate) fn new(doc: &OcdrawDocument, transferred: impl IntoIterator<Item = u64>) -> Self {
        let transferred = transferred.into_iter().collect::<BTreeSet<_>>();
        let mut entries = doc
            .text_entities
            .iter()
            .map(|t| OcdrawTextAssessmentEntry {
                entity_id: t.id,
                numeric_coverage: if transferred.contains(&t.id) {
                    OcdrawTextNumericCoverage::ActiveTextAnchors
                } else {
                    OcdrawTextNumericCoverage::NotTransferred
                },
                glyph_coverage: if t.content.is_empty() {
                    OcdrawTextGlyphCoverage::Empty
                } else {
                    OcdrawTextGlyphCoverage::Unassessed
                },
            })
            .collect::<Vec<_>>();
        entries.extend(
            doc.mtext_entities
                .iter()
                .map(|t| OcdrawTextAssessmentEntry {
                    entity_id: t.id,
                    numeric_coverage: if transferred.contains(&t.id) {
                        OcdrawTextNumericCoverage::MTextWcsAnchor
                    } else {
                        OcdrawTextNumericCoverage::NotTransferred
                    },
                    glyph_coverage: OcdrawTextGlyphCoverage::Unassessed,
                }),
        );
        Self { entries }
    }
}
