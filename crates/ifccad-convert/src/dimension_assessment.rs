//! Definition-anchor evidence remains separate from unassessed dimension display.
use crate::IfccadGeometryOwner;
use ocdraw::{dimension::DimensionTextPlacement, ifccad::*};
use std::collections::BTreeSet;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadDimensionNumericCoverage {
    StoredDefinitionAnchors,
    StoredDefinitionAndExplicitTextAnchors,
    NotTransferred,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadDimensionDisplayCoverage {
    Unassessed,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfccadDimensionAssessmentEntry {
    pub owner: IfccadGeometryOwner,
    pub entity_id: u64,
    pub numeric_coverage: IfccadDimensionNumericCoverage,
    pub display_coverage: IfccadDimensionDisplayCoverage,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IfccadDimensionAssessment {
    entries: Vec<IfccadDimensionAssessmentEntry>,
}
impl IfccadDimensionAssessment {
    pub fn entries(&self) -> &[IfccadDimensionAssessmentEntry] {
        &self.entries
    }
    pub(crate) fn new(doc: &IfccadDocument, transferred: impl IntoIterator<Item = u64>) -> Self {
        let transferred = transferred.into_iter().collect::<BTreeSet<_>>();
        let mut entries = Vec::new();
        let scopes = std::iter::once((
            IfccadGeometryOwner::ModelLayout(doc.model.id),
            &doc.model.entities,
        ))
        .chain(
            doc.paper_layouts
                .iter()
                .map(|p| (IfccadGeometryOwner::PaperLayout(p.id), &p.entities)),
        )
        .chain(
            doc.blocks
                .iter()
                .map(|b| (IfccadGeometryOwner::BlockDefinition(b.id), &b.entities)),
        );
        for (owner, entities) in scopes {
            for e in entities.iter().filter_map(IfccadEntity::as_native) {
                let IfccadEntityKind::Dimension(d) = &e.kind else {
                    continue;
                };
                entries.push(IfccadDimensionAssessmentEntry {
                    owner,
                    entity_id: e.id,
                    numeric_coverage: if !transferred.contains(&e.id) {
                        IfccadDimensionNumericCoverage::NotTransferred
                    } else if matches!(d.text_placement, DimensionTextPlacement::Explicit { .. }) {
                        IfccadDimensionNumericCoverage::StoredDefinitionAndExplicitTextAnchors
                    } else {
                        IfccadDimensionNumericCoverage::StoredDefinitionAnchors
                    },
                    display_coverage: IfccadDimensionDisplayCoverage::Unassessed,
                });
            }
        }
        Self { entries }
    }
}
