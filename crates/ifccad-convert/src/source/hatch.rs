use crate::{diagnostics::diagnostic, *};
use ocdraw::ifccad::*;
use opencadcodec::{CadDocument, EntityType};
use std::collections::BTreeSet;
pub(crate) fn bind(
    entities: &mut [IfccadEntity],
    source: &CadDocument,
    mappings: &IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
) {
    let eligible = entities
        .iter()
        .filter_map(|e| e.as_native())
        .filter(|e| {
            matches!(
                e.kind,
                IfccadEntityKind::Circle { .. }
                    | IfccadEntityKind::Ellipse { .. }
                    | IfccadEntityKind::PlanarPolyline { closed: true, .. }
            )
        })
        .map(|e| e.id)
        .collect::<BTreeSet<_>>();
    for e in entities.iter_mut().filter_map(IfccadEntity::as_native_mut) {
        let IfccadEntityKind::Hatch(h) = &mut e.kind else {
            continue;
        };
        let Some(EntityType::Hatch(cad)) = mappings
            .entities
            .cad_handle(e.id)
            .and_then(|h| source.get_entity(h))
        else {
            unreachable!()
        };
        for (index, (l, p)) in h.loops.iter_mut().zip(&cad.paths).enumerate() {
            if p.boundary_handles.is_empty() {
                continue;
            }
            let target = if cad.is_associative && p.boundary_handles.len() == 1 {
                mappings
                    .entities
                    .ifccad_id(p.boundary_handles[0])
                    .filter(|id| eligible.contains(id))
            } else {
                None
            };
            if let Some(id) = target {
                l.source_entity_id = Some(id);
            } else {
                issues.push(diagnostic("hatch-source",format!("entity/{}/loops/{index}/source",e.id),"inactive, multiple, missing, wrong-owner or unsupported source relation omitted; stored contour retained"));
            }
        }
        if cad.is_associative && h.loops.iter().all(|l| l.source_entity_id.is_none()) {
            issues.push(diagnostic(
                "hatch-associative",
                format!("entity/{}/is_associative", e.id),
                "no supported active source relation retained",
            ));
        }
    }
}
