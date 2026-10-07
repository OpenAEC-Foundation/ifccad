use super::{EntityGeometry, OcdrawDocument};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScopeGeometryCompleteness {
    Empty,
    Complete,
    Unavailable,
}

/// Derives completeness without trusting stored bounds. Tolerates malformed links so
/// the shared structural validator can report them before numerical evaluation.
pub(crate) fn derive_scope_geometry_completeness(
    doc: &OcdrawDocument,
) -> BTreeMap<u32, ScopeGeometryCompleteness> {
    let opaque = doc
        .opaque_entities
        .iter()
        .map(|e| e.id)
        .collect::<BTreeSet<_>>();
    let estimates = super::text_bounds::text_estimates(doc).unwrap_or_default();
    let empty_text = estimates
        .iter()
        .filter(|(_, e)| e.status == crate::text::TextExtentStatus::Empty)
        .map(|(id, _)| *id)
        .collect::<BTreeSet<_>>();
    let unavailable_text = estimates
        .iter()
        .filter(|(_, e)| e.status == crate::text::TextExtentStatus::Unavailable)
        .map(|(id, _)| *id)
        .collect::<BTreeSet<_>>();
    let blocks = doc
        .geometric_entities
        .iter()
        .filter_map(|e| match e.geometry {
            EntityGeometry::BlockInstance {
                definition_scope_id,
                ..
            } => Some((e.id, definition_scope_id)),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    let mut unavailable = doc
        .scopes
        .iter()
        .filter(|s| {
            s.entities
                .iter()
                .any(|id| opaque.contains(id) || unavailable_text.contains(id))
        })
        .map(|s| s.id)
        .collect::<BTreeSet<_>>();
    loop {
        let next = doc
            .scopes
            .iter()
            .filter(|s| {
                s.entities.iter().any(|id| {
                    blocks
                        .get(id)
                        .is_some_and(|target| unavailable.contains(target))
                })
            })
            .map(|s| s.id)
            .collect::<Vec<_>>();
        let before = unavailable.len();
        unavailable.extend(next);
        if before == unavailable.len() {
            break;
        }
    }
    doc.scopes
        .iter()
        .map(|s| {
            (
                s.id,
                if unavailable.contains(&s.id) {
                    ScopeGeometryCompleteness::Unavailable
                } else if s.entities.iter().all(|id| empty_text.contains(id)) {
                    ScopeGeometryCompleteness::Empty
                } else {
                    ScopeGeometryCompleteness::Complete
                },
            )
        })
        .collect()
}
