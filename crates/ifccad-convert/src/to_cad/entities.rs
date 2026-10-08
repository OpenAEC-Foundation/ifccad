use crate::{diagnostics::diagnostic, *};
use ocdraw::ifccad::*;
use opencadcodec::{CadDocument, EntityType, Handle};
use std::collections::BTreeMap;

pub(super) fn to_cad(
    drawing: &IfccadDocument,
    restore: crate::IfccadPreservationRestore,
    document: &mut CadDocument,
    owners: &[(Handle, &[IfccadEntity])],
    mappings: &mut IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
    geometry: &mut crate::geometry_context::GeometryContext,
) -> Result<crate::IfccadPreservationReport, IfccadConversionError> {
    fn common(
        drawing: &IfccadDocument,
        document: &CadDocument,
        mappings: &IfccadMappings,
        e: &IfccadNativeEntity,
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
            Some(crate::mapping::line_pattern::target(document, mappings, id))
        } else {
            None
        };
        let mut common =
            crate::mapping::appearance::to_common(&e.appearance, pattern, layer, &loc, issues);
        common.linetype_scale = e.line_pattern_scale;
        common.invisible = !e.visible;
        common
    }
    let mut report = crate::IfccadPreservationReport::default();
    let mut retained = BTreeMap::new();
    for (owner, entities) in owners {
        let native_owner = if *owner == document.header.model_space_block_handle {
            crate::IfccadGeometryOwner::ModelLayout(drawing.model.id)
        } else if let Some(id) = mappings.blocks.ifccad_id(*owner) {
            crate::IfccadGeometryOwner::BlockDefinition(id)
        } else {
            let layout = document
                .objects
                .values()
                .find_map(|o| match o {
                    opencadcodec::objects::ObjectType::Layout(l) if l.block_record == *owner => {
                        Some(l.handle)
                    }
                    _ => None,
                })
                .expect("allocated Paper owner");
            crate::IfccadGeometryOwner::PaperLayout(
                mappings
                    .layouts
                    .ifccad_id(layout)
                    .expect("allocated native Paper"),
            )
        };
        geometry.select(native_owner);
        for e in *entities {
            if let Some(opaque) = e.as_opaque() {
                let record = drawing
                    .preservation
                    .as_ref()
                    .unwrap()
                    .records
                    .iter()
                    .find(|r| r.id == opaque.preservation_record_id)
                    .unwrap();
                let prepared = if restore == crate::IfccadPreservationRestore::Skip {
                    Err(crate::IfccadPreservationReason::UnsupportedContext)
                } else {
                    crate::preservation::restore_spline(record, opaque, drawing)
                };
                match prepared {
                    Ok(mut spline) => {
                        if let Some(id) = opaque.layer_id {
                            spline.common.layer = drawing
                                .layers
                                .iter()
                                .find(|l| l.id == id)
                                .unwrap()
                                .name
                                .clone();
                            spline.common.layer_handle = mappings.layers.cad_handle(id);
                        }
                        if let Some(a) = &opaque.appearance {
                            let pattern = if let IfccadMode::Explicit(id) =
                                a.appearance.line_pattern
                            {
                                Some(crate::mapping::line_pattern::target(document, mappings, id))
                            } else {
                                None
                            };
                            let c = crate::mapping::appearance::to_common(
                                &a.appearance,
                                pattern,
                                &spline.common.layer,
                                &format!("entity/{}", opaque.id),
                                issues,
                            );
                            spline.common.color = c.color;
                            spline.common.transparency = c.transparency;
                            spline.common.line_weight = c.line_weight;
                            spline.common.linetype = c.linetype;
                            spline.common.linetype_handle = c.linetype_handle;
                            spline.common.linetype_scale = a.line_pattern_scale;
                        }
                        spline.common.invisible = !opaque.visible;
                        spline.common.owner_handle = *owner;
                        spline.common.entity_mode =
                            Some(if *owner == document.header.model_space_block_handle {
                                2
                            } else if document
                                .block_records
                                .iter()
                                .any(|b| b.handle == *owner && b.is_paper_space())
                            {
                                1
                            } else {
                                0
                            });
                        spline.common.raw_record = None;
                        let handle = document.allocate_handle();
                        spline.common.handle = handle;
                        mappings.entities.insert(opaque.id, handle);
                        retained.insert(opaque.id, EntityType::Spline(spline));
                    }
                    Err(reason) => {
                        crate::preservation::report_restore(
                            &mut report,
                            drawing,
                            record,
                            Some(opaque.id),
                            Some(reason),
                        );
                        issues.push(diagnostic(
                            "preservation-not-restored",
                            format!("entity/{}", opaque.id),
                            format!("opaque SPLINE omitted: {reason:?}"),
                        ));
                    }
                }
                continue;
            }
            let e = e.as_native().expect("native branch");
            if matches!(e.kind, IfccadEntityKind::Viewport(_)) {
                continue;
            }
            let loc = format!("entity/{}", e.id);
            let common = common(drawing, document, mappings, e, issues);
            let target = match &e.kind {
                IfccadEntityKind::Text(_) | IfccadEntityKind::MText(_) => {
                    crate::mapping::text::to_entity(drawing, e, mappings, geometry, issues)?
                }
                IfccadEntityKind::BlockInstance {
                    definition_id,
                    transform,
                } if mappings.blocks.cad_handle(*definition_id).is_some() => {
                    let definition = drawing
                        .blocks
                        .iter()
                        .find(|b| b.id == *definition_id)
                        .expect("validated definition");
                    let frame = transform
                        .placement
                        .coordinate_frame()
                        .map_err(IfccadConversionError::CoreValidation)?;
                    let native = ocdraw::geometry_kernel::BlockTransform::try_new(
                        frame,
                        transform.rotation,
                        ocdraw::geometry_kernel::Scale3::new(
                            transform.scale[0],
                            transform.scale[1],
                            transform.scale[2],
                        ),
                    )
                    .map_err(|e| IfccadConversionError::CadConstruction(e.to_string()))?;
                    let identity = crate::IfccadGeometryEntitySource::NativeEntity {
                        owner: geometry.owner,
                        entity_id: e.id,
                    };
                    let (target, source_map, target_map, _) =
                        cad_geometry_convert::geometry::blocks::to_cad_instance_parts(
                            native,
                            &definition.name,
                            definition.base_point,
                            identity.clone(),
                            &geometry.state.assessment,
                        )
                        .map_err(|error| match error {
                            cad_geometry_convert::CadGeometryError::Geometry(failure) => {
                                geometry.numerical_failure(failure)
                            }
                            cad_geometry_convert::CadGeometryError::Cad(message) => {
                                let mut diagnostics = issues.clone();
                                diagnostics.push(diagnostic("scale-clamped", &loc, message));
                                IfccadConversionError::Unsupported(diagnostics)
                            }
                        })?;
                    geometry.state.register_identity(e.id, identity);
                    geometry.state.record_instance_parts(
                        e.id,
                        *definition_id,
                        source_map,
                        target_map,
                    );
                    Some(EntityType::Insert(target))
                }
                IfccadEntityKind::BlockInstance { .. } => None,
                IfccadEntityKind::Hatch(h) => {
                    crate::mapping::hatch::to_cad(h, e.id, geometry, issues)?
                }
                kind => crate::mapping::geometry::to_entity(kind, e.id, geometry, issues)?,
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
        for e in entities.iter().filter_map(IfccadEntity::as_native) {
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
            let Some(id) = crate::mapping::viewport::checked_viewport_number(authored_index) else {
                issues.push(diagnostic(
                    "viewport-number",
                    loc,
                    "CAD viewport numbering exhausted; whole viewport omitted",
                ));
                continue;
            };
            let Some(viewport) =
                crate::mapping::viewport::to_cad(v, id, boundary, mappings, &loc, issues)
            else {
                continue;
            };
            authored_index += 1;
            let mut target = EntityType::Viewport(viewport);
            let mut c = common(drawing, document, mappings, e, issues);
            c.owner_handle = *owner;
            c.handle = document.allocate_handle();
            let handle = c.handle;
            *target.common_mut() = c;
            retained.insert(e.id, target);
            mappings.entities.insert(e.id, handle);
        }
    }
    for (_, entities) in owners {
        for e in entities.iter().filter_map(IfccadEntity::as_native) {
            let IfccadEntityKind::Hatch(h) = &e.kind else {
                continue;
            };
            let Some(EntityType::Hatch(target)) = retained.get_mut(&e.id) else {
                continue;
            };
            for (index, (p, l)) in target.paths.iter_mut().zip(&h.loops).enumerate() {
                if let Some(id) = l.source_entity_id {
                    if let Some(handle) = mappings.entities.cad_handle(id) {
                        p.boundary_handles.push(handle);
                    } else {
                        issues.push(diagnostic(
                            "hatch-source",
                            format!("entity/{}/loops/{index}/source", e.id),
                            "source omitted; stored contour retained",
                        ));
                    }
                }
            }
            target.is_associative = target.paths.iter().any(|p| !p.boundary_handles.is_empty());
        }
    }
    // All native and eligible opaque handles now exist, including forward
    // references. Remove failed dependencies before emitting any ordered entity.
    loop {
        let mut removed = Vec::new();
        for (_, entities) in owners {
            for opaque in entities.iter().filter_map(IfccadEntity::as_opaque) {
                let Some(EntityType::Spline(spline)) = retained.get(&opaque.id) else {
                    continue;
                };
                let record = drawing
                    .preservation
                    .as_ref()
                    .unwrap()
                    .records
                    .iter()
                    .find(|r| r.id == opaque.preservation_record_id)
                    .unwrap();
                let mut spline = spline.clone();
                match crate::preservation::references::rebind_spline_references(
                    &mut spline,
                    record,
                    opaque,
                    drawing,
                    document,
                    mappings,
                ) {
                    Ok(()) => {
                        spline.common.layer_handle =
                            document.layers.get(&spline.common.layer).map(|l| l.handle);
                        retained.insert(opaque.id, EntityType::Spline(spline));
                    }
                    Err(reason) => removed.push((opaque.id, record, reason)),
                }
            }
        }
        let mut changed = !removed.is_empty();
        for (id, record, reason) in removed {
            retained.remove(&id);
            mappings.entities.remove(id);
            crate::preservation::report_restore(
                &mut report,
                drawing,
                record,
                Some(id),
                Some(reason),
            );
            issues.push(diagnostic(
                "preservation-not-restored",
                format!("entity/{id}"),
                format!("required constructed reference unavailable: {reason:?}"),
            ));
        }
        for (_, entities) in owners {
            for e in entities.iter().filter_map(IfccadEntity::as_native) {
                if let IfccadEntityKind::Viewport(v) = &e.kind {
                    if retained.contains_key(&e.id)
                        && v.paper_clip
                            .boundary_entity_id
                            .is_some_and(|id| mappings.entities.cad_handle(id).is_none())
                    {
                        retained.remove(&e.id);
                        mappings.entities.remove(e.id);
                        changed = true;
                        issues.push(diagnostic(
                            "viewport-clip",
                            format!("entity/{}", e.id),
                            "boundary failed restoration; dependent viewport omitted",
                        ));
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    for (_, entities) in owners {
        for opaque in entities.iter().filter_map(IfccadEntity::as_opaque) {
            if retained.contains_key(&opaque.id) {
                let record = drawing
                    .preservation
                    .as_ref()
                    .unwrap()
                    .records
                    .iter()
                    .find(|r| r.id == opaque.preservation_record_id)
                    .unwrap();
                crate::preservation::report_restore(
                    &mut report,
                    drawing,
                    record,
                    Some(opaque.id),
                    None,
                );
            }
        }
    }
    for (_, entities) in owners {
        for e in *entities {
            if let Some(target) = retained.remove(&e.id()) {
                document
                    .add_entity(target)
                    .map_err(|error| IfccadConversionError::CadConstruction(error.to_string()))?;
            }
        }
    }
    Ok(report)
}
