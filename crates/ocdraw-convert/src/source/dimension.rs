use super::*;
use ocdraw::ocdraw::*;
use opencadcodec::Handle;
use std::collections::{BTreeMap, BTreeSet};
pub(crate) fn bind(
    doc: &mut OcdrawDocument,
    mapping: &BTreeMap<Handle, u64>,
    pending: Vec<(Handle, u64, Vec<cad_dimension::CadDimensionPointReference>)>,
    diagnostics: &mut Vec<CadToOcdrawDiagnostic>,
) {
    let owners = doc
        .scopes
        .iter()
        .flat_map(|s| s.entities.iter().map(move |id| (*id, s.id)))
        .collect::<BTreeMap<_, _>>();
    let eligible = doc
        .geometric_entities
        .iter()
        .map(|e| e.id())
        .collect::<BTreeSet<_>>();
    for (handle, id, refs) in pending {
        let target = doc
            .dimension_entities
            .iter_mut()
            .find(|d| d.id == id)
            .expect("prepared Dimension");
        for r in refs {
            if let Some(source) = mapping
                .get(&r.source)
                .copied()
                .filter(|source| eligible.contains(source) && owners.get(source) == owners.get(&id))
            {
                target.associations.push(OcdrawDimensionAssociation {
                    role: r.role,
                    source_entity_id: source,
                    selector: r.selector,
                });
            } else {
                diagnostics.push(CadToOcdrawDiagnostic::loss(CadToOcdrawDiagnosticSource::Entity{handle,kind:"DIMENSION".into()},CadToOcdrawAction::PartiallyExported,vec![CadToOcdrawLossReason::UnsupportedSemantic{name:format!("Dimension associations/{:?}: source not transferred as same-owner geometry; stored definition retained",r.role)}]));
            }
        }
    }
}
