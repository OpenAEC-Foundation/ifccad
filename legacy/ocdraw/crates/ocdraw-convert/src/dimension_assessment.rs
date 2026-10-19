//! Evidence for definition anchors; no dimension picture or font renderer is involved.
use ocdraw::ocdraw::OcdrawDocument;
use std::collections::BTreeSet;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OcdrawDimensionNumericCoverage {
    StoredDefinitionAnchors,
    StoredDefinitionAndExplicitTextAnchors,
    NotTransferred,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OcdrawDimensionDisplayCoverage {
    Unassessed,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OcdrawDimensionAssessmentEntry {
    pub entity_id: u64,
    pub numeric_coverage: OcdrawDimensionNumericCoverage,
    pub display_coverage: OcdrawDimensionDisplayCoverage,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OcdrawDimensionAssessment {
    entries: Vec<OcdrawDimensionAssessmentEntry>,
}
impl OcdrawDimensionAssessment {
    pub fn entries(&self) -> &[OcdrawDimensionAssessmentEntry] {
        &self.entries
    }
    pub(crate) fn new(doc: &OcdrawDocument, transferred: impl IntoIterator<Item = u64>) -> Self {
        let transferred = transferred.into_iter().collect::<BTreeSet<_>>();
        Self {
            entries: doc
                .dimension_entities
                .iter()
                .map(|d| OcdrawDimensionAssessmentEntry {
                    entity_id: d.id,
                    numeric_coverage: if !transferred.contains(&d.id) {
                        OcdrawDimensionNumericCoverage::NotTransferred
                    } else if matches!(
                        d.text_placement,
                        ocdraw::dimension::DimensionTextPlacement::Explicit { .. }
                    ) {
                        OcdrawDimensionNumericCoverage::StoredDefinitionAndExplicitTextAnchors
                    } else {
                        OcdrawDimensionNumericCoverage::StoredDefinitionAnchors
                    },
                    display_coverage: OcdrawDimensionDisplayCoverage::Unassessed,
                })
                .collect(),
        }
    }
}
