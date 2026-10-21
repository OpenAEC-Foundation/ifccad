//! Derived mathematical surface coverage, distinct from position and display evidence.
use crate::{
    IfccadConversionError as Error, IfccadGeometryEntitySource as Source,
    IfccadGeometryOwner as Owner, IfccadMappings,
};
use ifccad::*;
use opencadcodec::{CadDocument, EntityType};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadFilledSurfaceCoverage {
    Proven,
    Unassessed,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfccadFilledSurfaceAssessment {
    source: Source,
    coverage: IfccadFilledSurfaceCoverage,
}
impl IfccadFilledSurfaceAssessment {
    pub fn source(&self) -> &Source {
        &self.source
    }
    pub fn coverage(&self) -> IfccadFilledSurfaceCoverage {
        self.coverage
    }
}
pub(crate) fn collect_filled_surfaces(
    native: &IfccadDocument,
    cad: &CadDocument,
    mappings: &IfccadMappings,
) -> Result<Vec<IfccadFilledSurfaceAssessment>, Error> {
    let owners = std::iter::once((
        Owner::ModelLayout(native.model.id),
        native.model.entities.as_slice(),
    ))
    .chain(
        native
            .paper_layouts
            .iter()
            .map(|p| (Owner::PaperLayout(p.id), p.entities.as_slice())),
    )
    .chain(
        native
            .blocks
            .iter()
            .map(|b| (Owner::BlockDefinition(b.id), b.entities.as_slice())),
    )
    .collect::<Vec<_>>();
    let mut coverage = BTreeMap::new();
    for (_, entities) in &owners {
        for e in entities.iter().filter_map(IfccadEntity::as_native) {
            let (IfccadEntityKind::Solid(q) | IfccadEntityKind::Trace(q)) = &e.kind else {
                continue;
            };
            // Only retained mappings are assessed; diagnosed omissions have no target pair.
            let Some(h) = mappings.entities.cad_handle(e.id) else {
                continue;
            };
            let Some(EntityType::Solid(s)) = cad.get_entity(h) else {
                return Err(Error::InvalidStructure(format!(
                    "filled mapping {} has no SOLID/TRACE target",
                    e.id
                )));
            };
            if s.is_trace != matches!(e.kind, IfccadEntityKind::Trace(_)) {
                return Err(Error::InvalidStructure(
                    "filled mapping changes kind".into(),
                ));
            }
            let target = [
                s.first_corner,
                s.second_corner,
                s.third_corner,
                s.fourth_corner,
            ]
            .map(|c| [c.x, c.y]);
            let proven = cad_geometry_convert::geometry::filled::common_filled_surface_cover(
                &q.corners, &target,
            )
            .is_some();
            coverage.insert(
                e.id,
                if proven {
                    IfccadFilledSurfaceCoverage::Proven
                } else {
                    IfccadFilledSurfaceCoverage::Unassessed
                },
            );
        }
    }
    fn walk(
        d: &IfccadDocument,
        owner: Owner,
        entities: &[IfccadEntity],
        path: &mut Vec<Source>,
        coverage: &BTreeMap<u64, IfccadFilledSurfaceCoverage>,
        maps: &IfccadMappings,
        out: &mut Vec<IfccadFilledSurfaceAssessment>,
    ) {
        for e in entities.iter().filter_map(IfccadEntity::as_native) {
            if maps.entities.cad_handle(e.id).is_none() {
                continue;
            }
            let identity = Source::NativeEntity {
                owner,
                entity_id: e.id,
            };
            if let Some(&coverage) = coverage.get(&e.id) {
                let source = if path.is_empty() {
                    identity
                } else {
                    Source::BlockOccurrence {
                        path: path.clone(),
                        leaf: Box::new(identity),
                    }
                };
                out.push(IfccadFilledSurfaceAssessment { source, coverage });
            } else if let IfccadEntityKind::BlockInstance { definition_id, .. } = e.kind {
                if let Some(b) = d.blocks.iter().find(|b| b.id == definition_id) {
                    path.push(identity);
                    walk(
                        d,
                        Owner::BlockDefinition(b.id),
                        &b.entities,
                        path,
                        coverage,
                        maps,
                        out,
                    );
                    path.pop();
                }
            }
        }
    }
    let mut out = Vec::new();
    for (owner, entities) in owners {
        walk(
            native,
            owner,
            entities,
            &mut vec![],
            &coverage,
            mappings,
            &mut out,
        );
    }
    Ok(out)
}
