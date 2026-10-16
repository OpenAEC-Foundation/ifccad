use crate::{diagnostics::diagnostic, *};
use ocdraw::ifccad::*;
use opencadcodec::{CadDocument, EntityType, Handle};
use std::collections::BTreeSet;
pub(crate) fn bind(
    entities: &mut [IfccadEntity],
    source: &CadDocument,
    mappings: &IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<(), IfccadConversionError> {
    let eligible = entities
        .iter()
        .filter_map(IfccadEntity::as_native)
        .filter(|e| e.kind.is_hatch_boundary_source())
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
        if h.loops.len() != cad.paths.len() {
            return Err(IfccadConversionError::InvalidStructure(format!(
                "entity/{}: Hatch source loop count mismatch",
                e.id
            )));
        }
        h.association_enabled = cad.is_associative;
        for (index, (l, p)) in h.loops.iter_mut().zip(&cad.paths).enumerate() {
            let mut ids = Vec::new();
            let mut failed = false;
            for (entry, source_handle) in p.boundary_handles.iter().enumerate() {
                if let Some(id) = mappings
                    .entities
                    .ifccad_id(*source_handle)
                    .filter(|id| eligible.contains(id))
                {
                    ids.push(id);
                } else {
                    failed = true;
                    issues.push(diagnostic(
                        "hatch-source-entry-loss",
                        format!("entity/{}/loops/{index}/sources/{entry}", e.id),
                        "missing, skipped, wrong-owner or unsupported source",
                    ));
                }
            }
            if failed {
                l.source_entity_ids.clear();
                issues.push(diagnostic(
                    "hatch-source-group-loss",
                    format!("entity/{}/loops/{index}/sources", e.id),
                    "source group omitted; stored contour retained",
                ));
            } else {
                l.source_entity_ids = ids;
            }
        }
    }
    Ok(())
}

pub(crate) fn confirmed_reactors(
    d: &IfccadDocument,
    m: &IfccadMappings,
) -> BTreeSet<(Handle, Handle)> {
    let entities = d
        .model
        .entities
        .iter()
        .chain(d.paper_layouts.iter().flat_map(|p| p.entities.iter()))
        .chain(d.blocks.iter().flat_map(|b| b.entities.iter()));
    let mut pairs = BTreeSet::new();
    for e in entities.filter_map(IfccadEntity::as_native) {
        let IfccadEntityKind::Hatch(h) = &e.kind else {
            continue;
        };
        if let Some(hh) = m.entities.cad_handle(e.id) {
            for l in &h.loops {
                for id in &l.source_entity_ids {
                    if let Some(sh) = m.entities.cad_handle(*id) {
                        pairs.insert((sh, hh));
                    }
                }
            }
        }
    }
    pairs
}
