use crate::{diagnostics::diagnostic, *};
use ocdraw::ifccad::*;
use opencadcodec::{CadDocument, EntityType, Handle};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn from_cad(
    source: &CadDocument,
    patterns: &crate::patterns::SourcePatterns,
    handles: &[Handle],
    ids: &mut IfccadIdCounters,
    mappings: &mut IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<Vec<IfccadEntity>, IfccadConversionError> {
    let mut retained = BTreeMap::new();
    for h in handles {
        let e = source.get_entity(*h).expect("inspected entity");
        if matches!(e, EntityType::Viewport(_)) {
            continue;
        }
        let loc = format!("entity/{h}");
        let appearance = crate::appearance::from_common(e.common(), patterns, &loc, issues);
        let kind = match e {
            EntityType::Insert(i) => mappings
                .blocks
                .ifccad_id(
                    source
                        .block_records
                        .get(&i.block_name)
                        .expect("inspected target")
                        .handle,
                )
                .and_then(|id| crate::blocks::from_insert(i, id, &loc, issues)),
            e => crate::geometry::from_entity(e, &loc, issues),
        };
        if let Some(kind) = kind {
            retained.insert(*h, (kind, appearance));
        } else {
            issues.push(diagnostic(
                "entity-skipped",
                loc,
                "whole entity omitted; unsupported geometry or entity family",
            ));
        }
    }
    let model_id = mappings
        .layouts
        .ifccad_id(
            source
                .objects
                .values()
                .find_map(|o| match o {
                    opencadcodec::objects::ObjectType::Layout(l)
                        if source
                            .block_records
                            .iter()
                            .any(|b| b.handle == l.block_record && b.is_model_space()) =>
                    {
                        Some(l.handle)
                    }
                    _ => None,
                })
                .expect("inspected model layout"),
        )
        .expect("allocated model layout");
    let mut boundaries = BTreeMap::new();
    let mut claimed = BTreeSet::new();
    for h in handles {
        let Some(EntityType::Viewport(v)) = source.get_entity(*h) else {
            continue;
        };
        let loc = format!("entity/{h}");
        let paper = source
            .block_records
            .iter()
            .any(|b| b.handle == v.common.owner_handle && b.is_paper_space());
        if !paper || crate::viewports::overall_canvas(source, v) {
            issues.push(diagnostic(
                "viewport-camera",
                loc,
                "Model-owned or authored overall Paper canvas is outside the viewport contract",
            ));
            continue;
        }
        let Some(mut view) = crate::viewports::from_cad(v, model_id, mappings, &loc, issues) else {
            continue;
        };
        if !v.clip_boundary_handle.is_null() {
            let boundary = v.clip_boundary_handle;
            if let Some(target) = source.get_entity(boundary) {
                if target.common().owner_handle != v.common.owner_handle {
                    return Err(IfccadConversionError::InvalidStructure(format!(
                        "{loc}: clip boundary has a different owner"
                    )));
                }
            }
            let Some((kind, _)) = retained.get(&boundary) else {
                issues.push(diagnostic(
                    "viewport-clip",
                    loc,
                    "missing or unsupported clip boundary; whole viewport omitted",
                ));
                continue;
            };
            if view.paper_clip.enabled
                && validate_ifccad_viewport_boundary(&view.frame, kind).is_err()
            {
                issues.push(diagnostic("viewport-clip",loc,"active boundary is not an eligible enclosed Paper Z=0 curve; whole viewport omitted"));
                continue;
            }
            if !claimed.insert(boundary) {
                return Err(IfccadConversionError::InvalidStructure(format!(
                    "{loc}: clip boundary is shared by viewports"
                )));
            }
            view.paper_clip.boundary_entity_id = Some(0); // resolved after final retained IDs
            boundaries.insert(*h, boundary);
        } else if view.paper_clip.enabled {
            issues.push(diagnostic(
                "viewport-clip",
                loc,
                "active clip boundary is missing; whole viewport omitted",
            ));
            continue;
        }
        let mut common = v.common.clone();
        common.invisible = false;
        let appearance = crate::appearance::from_common(&common, patterns, &loc, issues);
        retained.insert(*h, (IfccadEntityKind::Viewport(view), appearance));
    }
    let mut entities = Vec::new();
    for h in handles {
        if let Some((kind, appearance)) = retained.remove(h) {
            let e = source.get_entity(*h).expect("inspected entity");
            let id = ids
                .allocate_entity_id()
                .map_err(IfccadConversionError::IdAllocation)?;
            mappings.entities.insert(id, *h);
            entities.push(IfccadEntity {
                id,
                kind,
                appearance,
                line_pattern_scale: e.common().linetype_scale,
                layer_id: mappings
                    .layers
                    .ifccad_id(
                        source
                            .layers
                            .get(&e.common().layer)
                            .expect("inspected layer")
                            .handle,
                    )
                    .expect("allocated layer"),
            });
        }
    }
    for entity in &mut entities {
        if let IfccadEntityKind::Viewport(view) = &mut entity.kind {
            if let Some(boundary) =
                boundaries.get(&mappings.entities.cad_handle(entity.id).unwrap())
            {
                view.paper_clip.boundary_entity_id = Some(
                    mappings
                        .entities
                        .ifccad_id(*boundary)
                        .expect("retained boundary"),
                );
            }
        }
    }
    Ok(entities)
}

pub(crate) fn to_cad(
    drawing: &IfccadDocument,
    document: &mut CadDocument,
    owners: &[(Handle, &[IfccadEntity])],
    mappings: &mut IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<(), IfccadConversionError> {
    fn common(
        drawing: &IfccadDocument,
        document: &CadDocument,
        mappings: &IfccadMappings,
        e: &IfccadEntity,
        issues: &mut Vec<IfccadDiagnostic>,
    ) -> opencadcodec::entities::EntityCommon {
        let loc = format!("entity/{}", e.id);
        let layer = &drawing
            .layers
            .iter()
            .find(|l| l.id == e.layer_id)
            .expect("validated layer")
            .name;
        let pattern = if let IfccadMode::Explicit(id) = e.appearance.line_pattern {
            Some(crate::patterns::target(document, mappings, id))
        } else {
            None
        };
        let mut common = crate::appearance::to_common(&e.appearance, pattern, layer, &loc, issues);
        common.linetype_scale = e.line_pattern_scale;
        common
    }
    let mut retained = BTreeMap::new();
    for (owner, entities) in owners {
        for e in *entities {
            if matches!(e.kind, IfccadEntityKind::Viewport(_)) {
                continue;
            }
            let loc = format!("entity/{}", e.id);
            let common = common(drawing, document, mappings, e, issues);
            let target = match &e.kind {
                IfccadEntityKind::BlockInstance {
                    definition_id,
                    transform,
                } if mappings.blocks.cad_handle(*definition_id).is_some() => {
                    let before = issues.len();
                    let insert = crate::blocks::to_insert(
                        &drawing
                            .blocks
                            .iter()
                            .find(|b| b.id == *definition_id)
                            .expect("validated definition")
                            .name,
                        transform,
                        &loc,
                        issues,
                    );
                    (issues.len() == before).then_some(insert)
                }
                IfccadEntityKind::BlockInstance { .. } => None,
                kind => crate::geometry::to_entity(kind, &loc, issues),
            };
            if let Some(mut target) = target {
                *target.common_mut() = common;
                target.common_mut().owner_handle = *owner;
                let handle = document.allocate_handle();
                target.common_mut().handle = handle;
                mappings.entities.insert(e.id, handle);
                retained.insert(e.id, target);
            } else {
                issues.push(diagnostic(
                    "entity-skipped",
                    loc,
                    "whole entity omitted; unsupported geometry or omitted block target",
                ));
            }
        }
    }
    for (owner, entities) in owners {
        let mut authored_index = 0usize;
        for e in *entities {
            let IfccadEntityKind::Viewport(v) = &e.kind else {
                continue;
            };
            let loc = format!("entity/{}", e.id);
            let boundary = if let Some(id) = v.paper_clip.boundary_entity_id {
                let Some(handle) = mappings.entities.cad_handle(id) else {
                    issues.push(diagnostic(
                        "viewport-clip",
                        loc,
                        "boundary was omitted during CAD conversion; whole viewport omitted",
                    ));
                    continue;
                };
                handle
            } else {
                Handle::NULL
            };
            let Some(id) = crate::viewports::checked_viewport_number(authored_index) else {
                issues.push(diagnostic(
                    "viewport-number",
                    loc,
                    "CAD viewport numbering exhausted; whole viewport omitted",
                ));
                continue;
            };
            let Some(viewport) = crate::viewports::to_cad(v, id, boundary, mappings, &loc, issues)
            else {
                continue;
            };
            authored_index += 1;
            let mut target = EntityType::Viewport(viewport);
            let mut c = common(drawing, document, mappings, e, issues);
            c.invisible = !v.visible;
            c.owner_handle = *owner;
            c.handle = document.allocate_handle();
            let handle = c.handle;
            *target.common_mut() = c;
            retained.insert(e.id, target);
            mappings.entities.insert(e.id, handle);
        }
    }
    for (_, entities) in owners {
        for e in *entities {
            if let Some(target) = retained.remove(&e.id) {
                document
                    .add_entity(target)
                    .map_err(|error| IfccadConversionError::CadConstruction(error.to_string()))?;
            }
        }
    }
    Ok(())
}
