use super::*;
use ocdraw::ocdraw::*;
use opencadcodec::{CadDocument, Handle};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn bind(
    d: &mut OcdrawDocument,
    mapping: &BTreeMap<Handle, u64>,
    pending: Vec<(Handle, u64, Vec<Vec<Handle>>, bool)>,
    diagnostics: &mut Vec<CadToOcdrawDiagnostic>,
) -> Result<(), OcdrawBuildError> {
    let owners = d
        .scopes
        .iter()
        .flat_map(|s| s.entities.iter().map(move |id| (*id, s.id)))
        .collect::<BTreeMap<_, _>>();
    let eligible = d
        .geometric_entities
        .iter()
        .filter(|e| e.geometry.is_hatch_boundary_source())
        .map(|e| e.id)
        .chain(d.text_entities.iter().map(|e| e.id))
        .chain(d.mtext_entities.iter().map(|e| e.id))
        .chain(d.viewports.iter().map(|e| e.id))
        .collect::<BTreeSet<_>>();
    for (handle, id, handles, enabled) in pending {
        let h = d
            .hatch_entities
            .iter_mut()
            .find(|h| h.id == id)
            .expect("prepared Hatch");
        if h.loops.len() != handles.len() {
            return Err(OcdrawBuildError::Invalid(
                "Hatch source loop count mismatch".into(),
            ));
        }
        h.association_enabled = enabled;
        let mut reasons = Vec::new();
        for (index, (l, handles)) in h.loops.iter_mut().zip(handles).enumerate() {
            let mut ids = Vec::new();
            let mut failed = false;
            for (entry, source) in handles.iter().enumerate() {
                if let Some(target) = mapping.get(source).copied().filter(|target| {
                    eligible.contains(target) && owners.get(target) == owners.get(&id)
                }) {
                    ids.push(target);
                } else {
                    failed = true;
                    reasons.push(CadToOcdrawLossReason::UnsupportedSemantic{name:format!("Hatch loops/{index}/sourceEntityIds/{entry}: missing, skipped, wrong-owner or unsupported source")});
                }
            }
            if failed {
                l.source_entity_ids.clear();
                reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
                    name: format!("Hatch loops/{index}/sourceEntityIds: source group omitted"),
                });
            } else {
                l.source_entity_ids = ids;
            }
        }
        if !reasons.is_empty() {
            diagnostics.push(CadToOcdrawDiagnostic::loss(
                CadToOcdrawDiagnosticSource::Entity {
                    handle,
                    kind: "HATCH".into(),
                },
                CadToOcdrawAction::PartiallyExported,
                reasons,
            ));
        }
    }
    Ok(())
}

pub(crate) fn account_reactors(
    d: &OcdrawDocument,
    source: &CadDocument,
    mapping: &BTreeMap<Handle, u64>,
    diagnostics: &mut Vec<CadToOcdrawDiagnostic>,
) {
    let reverse = mapping
        .iter()
        .map(|(h, id)| (*id, *h))
        .collect::<BTreeMap<_, _>>();
    let mut pairs = BTreeSet::new();
    for h in &d.hatch_entities {
        if let Some(hh) = reverse.get(&h.id) {
            for l in &h.loops {
                for id in &l.source_entity_ids {
                    if let Some(sh) = reverse.get(id) {
                        pairs.insert((*sh, *hh));
                    }
                }
            }
        }
    }
    for diagnostic in diagnostics.iter_mut() {
        let CadToOcdrawDiagnostic::Loss {
            source: CadToOcdrawDiagnosticSource::Entity { handle, .. },
            reasons,
            ..
        } = diagnostic
        else {
            continue;
        };
        if source.get_entity(*handle).is_some_and(|e| {
            !e.common().reactors.is_empty()
                && e.common()
                    .reactors
                    .iter()
                    .all(|h| pairs.contains(&(*handle, *h)))
        }) {
            reasons.retain(|r| !matches!(r, CadToOcdrawLossReason::EntityReactors));
        }
    }
    diagnostics.retain(|d| !d.reasons().is_empty());
}
