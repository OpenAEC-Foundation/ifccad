use super::*;
use ocdraw::ocdraw::*;
use opencadcodec::Handle;
use std::collections::BTreeMap;

pub(crate) fn bind(
    d: &mut OcdrawDocument,
    mapping: &BTreeMap<Handle, u64>,
    pending: Vec<(Handle, u64, Vec<Vec<Handle>>, bool)>,
    diagnostics: &mut Vec<CadToOcdrawDiagnostic>,
) {
    let owners = d
        .scopes
        .iter()
        .flat_map(|s| s.entities.iter().map(move |id| (*id, s.id)))
        .collect::<BTreeMap<_, _>>();
    let eligible = d
        .geometric_entities
        .iter()
        .filter(|e| {
            matches!(
                e.geometry(),
                DrawingGeometry::Circle { .. }
                    | DrawingGeometry::Ellipse { arc: None, .. }
                    | DrawingGeometry::PlanarPolyline { closed: true, .. }
            )
        })
        .map(|e| e.id())
        .collect::<std::collections::BTreeSet<_>>();
    for (handle, id, handles, active) in pending {
        let h = d
            .hatch_entities
            .iter_mut()
            .find(|h| h.id == id)
            .expect("prepared Hatch");
        let mut reasons = Vec::new();
        for (index, (l, handles)) in h.loops.iter_mut().zip(handles).enumerate() {
            if handles.is_empty() {
                continue;
            }
            let target = if active && handles.len() == 1 {
                mapping.get(&handles[0]).copied().filter(|target| {
                    eligible.contains(target) && owners.get(target) == owners.get(&id)
                })
            } else {
                None
            };
            if let Some(target) = target {
                l.source_entity_id = Some(target);
            } else {
                reasons.push(CadToOcdrawLossReason::UnsupportedSemantic{name:format!("Hatch loops/{index}/source: inactive, multiple, missing, wrong-owner or unsupported source relation omitted")});
            }
        }
        if active && h.loops.iter().all(|l| l.source_entity_id.is_none()) {
            reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
                name: "Hatch is_associative: no supported active source relation retained".into(),
            });
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
}
