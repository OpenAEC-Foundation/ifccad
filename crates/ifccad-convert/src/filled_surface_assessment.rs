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
#[derive(Clone, Copy)]
pub(crate) enum FilledSourceDirection {
    FromCad,
    ToCad,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CoverageKey {
    Entity(u64),
    GraphicsPart(u64, usize),
}
pub(crate) fn collect_filled_surfaces(
    native: &IfccadDocument,
    cad: &CadDocument,
    mappings: &IfccadMappings,
    direction: FilledSourceDirection,
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
                CoverageKey::Entity(e.id),
                if proven {
                    IfccadFilledSurfaceCoverage::Proven
                } else {
                    IfccadFilledSurfaceCoverage::Unassessed
                },
            );
        }
    }
    let mut part_sources = BTreeMap::new();
    for (owner, entities) in &owners {
        for e in entities.iter().filter_map(IfccadEntity::as_native) {
            let IfccadEntityKind::Dimension(d) = &e.kind else {
                continue;
            };
            let Some(g) = &d.graphics else { continue };
            let Some(handle) = mappings.entities.cad_handle(e.id) else {
                continue;
            };
            let Some(EntityType::Dimension(c)) = cad.get_entity(handle) else {
                continue;
            };
            let Some(block) = cad.block_records.get(&c.base().block_name) else {
                continue;
            };
            if block.entity_handles.len() != g.parts.len() {
                return Err(Error::InvalidStructure(
                    "complete graphics picture has mismatched part count".into(),
                ));
            }
            let dimension = match direction {
                FilledSourceDirection::FromCad => Source::CadEntity {
                    handle,
                    kind: "DIMENSION".into(),
                },
                FilledSourceDirection::ToCad => Source::NativeEntity {
                    owner: *owner,
                    entity_id: e.id,
                },
            };
            for (index, part) in g.parts.iter().enumerate() {
                let actual = cad.get_entity(block.entity_handles[index]).ok_or_else(|| {
                    Error::InvalidStructure("missing filled graphics part".into())
                })?;
                let source = match direction {
                    FilledSourceDirection::FromCad => Source::CadEntity {
                        handle: actual.common().handle,
                        kind: actual.as_entity().entity_type().into(),
                    },
                    FilledSourceDirection::ToCad => dimension.clone(),
                };
                part_sources.insert(
                    (e.id, index),
                    Source::DimensionGraphicsPart {
                        owner: *owner,
                        dimension: Box::new(dimension.clone()),
                        part_index: index,
                        source: Box::new(source),
                    },
                );
                let (IfccadDimensionGraphicsKind::Solid(q) | IfccadDimensionGraphicsKind::Trace(q)) =
                    &part.kind
                else {
                    continue;
                };
                let EntityType::Solid(s) = actual else {
                    return Err(Error::InvalidStructure(
                        "filled graphics kind changed".into(),
                    ));
                };
                if s.is_trace != matches!(part.kind, IfccadDimensionGraphicsKind::Trace(_)) {
                    return Err(Error::InvalidStructure(
                        "filled graphics CAD type changed".into(),
                    ));
                }
                let target = [
                    s.first_corner,
                    s.second_corner,
                    s.third_corner,
                    s.fourth_corner,
                ]
                .map(|p| [p.x, p.y]);
                let proven = cad_geometry_convert::geometry::filled::common_filled_surface_cover(
                    &q.corners, &target,
                )
                .is_some();
                coverage.insert(
                    CoverageKey::GraphicsPart(e.id, index),
                    if proven {
                        IfccadFilledSurfaceCoverage::Proven
                    } else {
                        IfccadFilledSurfaceCoverage::Unassessed
                    },
                );
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn walk(
        d: &IfccadDocument,
        owner: Owner,
        entities: &[IfccadEntity],
        path: &mut Vec<Source>,
        coverage: &BTreeMap<CoverageKey, IfccadFilledSurfaceCoverage>,
        part_sources: &BTreeMap<(u64, usize), Source>,
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
            if let Some(&coverage) = coverage.get(&CoverageKey::Entity(e.id)) {
                let source = if path.is_empty() {
                    identity
                } else {
                    Source::BlockOccurrence {
                        path: path.clone(),
                        leaf: Box::new(identity),
                    }
                };
                out.push(IfccadFilledSurfaceAssessment { source, coverage });
            } else if let IfccadEntityKind::Dimension(dimension) = &e.kind {
                if let Some(g) = &dimension.graphics {
                    for (index, p) in g.parts.iter().enumerate() {
                        let Some(identity) = part_sources.get(&(e.id, index)) else {
                            continue;
                        };
                        if let Some(&coverage) =
                            coverage.get(&CoverageKey::GraphicsPart(e.id, index))
                        {
                            let source = if path.is_empty() {
                                identity.clone()
                            } else {
                                Source::BlockOccurrence {
                                    path: path.clone(),
                                    leaf: Box::new(identity.clone()),
                                }
                            };
                            out.push(IfccadFilledSurfaceAssessment { source, coverage });
                        } else if let IfccadDimensionGraphicsKind::BlockInstance {
                            definition_id,
                            ..
                        } = &p.kind
                        {
                            if let Some(b) = d.blocks.iter().find(|b| b.id == *definition_id) {
                                path.push(identity.clone());
                                walk(
                                    d,
                                    Owner::BlockDefinition(b.id),
                                    &b.entities,
                                    path,
                                    coverage,
                                    part_sources,
                                    maps,
                                    out,
                                );
                                path.pop();
                            }
                        }
                    }
                }
            } else if let IfccadEntityKind::BlockInstance { definition_id, .. } = e.kind {
                if let Some(b) = d.blocks.iter().find(|b| b.id == definition_id) {
                    path.push(identity);
                    walk(
                        d,
                        Owner::BlockDefinition(b.id),
                        &b.entities,
                        path,
                        coverage,
                        part_sources,
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
            &part_sources,
            mappings,
            &mut out,
        );
    }
    Ok(out)
}
