//! Explicit recovery boundaries for partial CAD import. Never suppress shared failures.
use crate::*;
use opencadcodec::{CadDocument, Handle};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

/// Source entities omitted from a successfully validated partial import.
/// Other field/group omissions remain in the outcome's complete diagnostics.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CadPartialOpenReport {
    pub entities: Vec<CadAuditEntity>,
    /// Original error category, cause chain, source identity and numeric evidence.
    pub errors: Vec<CadAuditEvent>,
}

impl IfccadConversionError {
    /// Eligible for explicit omission when raised inside one entity's production
    /// preparation transaction. This does not make shared/finalization errors recoverable.
    pub fn allows_partial_open_retry(&self) -> bool {
        matches!(
            self,
            Self::EntitySourceValidation { .. }
                | Self::Geometry(_)
                | Self::HatchPreparation { .. }
                | Self::DimensionPreparation { .. }
                | Self::TextPreparation { .. }
        )
    }
}

pub(crate) fn assess_partial_occurrences(
    source: &CadDocument,
    drawing: &mut ifccad::IfccadDocument,
    mappings: &mut IfccadMappings,
    geometry: &mut crate::geometry_context::GeometryContext,
    audit: &mut crate::audit::AuditRun<'_>,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<(), IfccadConversionError> {
    // Repeat after omissions so every retained parent is rechecked against the
    // final membership, regardless of source definition order.
    loop {
        let members = drawing
            .blocks
            .iter()
            .map(|b| {
                (
                    mappings.blocks.cad_handle(b.id).unwrap().value(),
                    b.entities
                        .iter()
                        .filter_map(|e| mappings.entities.cad_handle(e.id()).map(|h| h.value()))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let mut omitted = BTreeSet::new();
        let scopes = std::iter::once((
            IfccadGeometryOwner::ModelLayout(drawing.model.id),
            &mut drawing.model.entities,
        ))
        .chain(
            drawing
                .blocks
                .iter_mut()
                .map(|b| (IfccadGeometryOwner::BlockDefinition(b.id), &mut b.entities)),
        )
        .chain(
            drawing
                .paper_layouts
                .iter_mut()
                .map(|p| (IfccadGeometryOwner::PaperLayout(p.id), &mut p.entities)),
        );
        for (owner, entities) in scopes {
            geometry.select(owner);
            for e in entities.iter() {
                if !e.as_native().is_some_and(|e| {
                    matches!(e.kind, ifccad::IfccadEntityKind::BlockInstance { .. })
                }) {
                    continue;
                }
                let h = mappings
                    .entities
                    .cad_handle(e.id())
                    .expect("prepared instance");
                if let Err(error) = geometry.assess_roots(&members, &[h.value()], issues) {
                    if !error.allows_partial_open_retry() {
                        return Err(error);
                    }
                    audit.failure("block-occurrence", Some(h), &error, None)?;
                    issues.push(crate::diagnostics::diagnostic(
                        "partial-entity-skipped",
                        format!("entity/{h}"),
                        error.to_string(),
                    ));
                    omitted.insert(e.id());
                }
            }
        }
        if omitted.is_empty() {
            break;
        }
        for entities in std::iter::once(&mut drawing.model.entities)
            .chain(drawing.blocks.iter_mut().map(|b| &mut b.entities))
            .chain(drawing.paper_layouts.iter_mut().map(|p| &mut p.entities))
        {
            entities.retain(|e| !omitted.contains(&e.id()));
        }
        for id in omitted {
            let h = mappings.entities.cad_handle(id).unwrap();
            geometry.state.discard_entity(h.value());
            mappings.entities.remove(id);
        }
    }
    // Hatch source associations are whole groups. A later occurrence omission
    // must detach the entire affected group rather than retain dangling IDs or
    // fabricate a substitute contour. Dimension/workspace binding runs afterward.
    let retained = mappings
        .entities
        .iter()
        .map(|(id, _)| id)
        .collect::<BTreeSet<_>>();
    for e in drawing
        .model
        .entities
        .iter_mut()
        .chain(
            drawing
                .blocks
                .iter_mut()
                .flat_map(|b| b.entities.iter_mut()),
        )
        .chain(
            drawing
                .paper_layouts
                .iter_mut()
                .flat_map(|p| p.entities.iter_mut()),
        )
    {
        let id = e.id();
        let Some(e) = e.as_native_mut() else { continue };
        let ifccad::IfccadEntityKind::Hatch(h) = &mut e.kind else {
            continue;
        };
        for (index, l) in h.loops.iter_mut().enumerate() {
            if l.source_entity_ids.iter().any(|id| !retained.contains(id)) {
                l.source_entity_ids.clear();
                issues.push(crate::diagnostics::diagnostic("partial-reference-group",format!("entity/{}/loops/{index}/sources",mappings.entities.cad_handle(id).unwrap()),"whole Hatch source group omitted because a referenced entity failed; stored contour and association switch retained"));
            }
        }
    }
    // A shared definition with missing members is deliberately partial. Report
    // every referring instance, including nested parents; keep unaffected members.
    let members = source
        .block_records
        .iter()
        .map(|b| {
            (
                b.handle.value(),
                b.entity_handles.iter().map(|h| h.value()).collect(),
            )
        })
        .collect();
    let incomplete = source
        .block_records
        .iter()
        .filter(|b| !b.is_model_space() && !b.is_paper_space())
        .filter(|b| b.entity_handles.iter().any(|h| audit.entity_failed(*h)))
        .map(|b| b.handle.value())
        .collect();
    let affected = geometry.state.affected_instances(&members, &incomplete);
    for (definition, roots) in affected {
        for root in roots {
            issues.push(crate::diagnostics::diagnostic("partial-block-content",format!("entity/{}",Handle::new(root)),
                format!("referenced definition {} contains omitted failed entities; retained members and occurrences were checked",Handle::new(definition))));
        }
    }
    Ok(())
}
