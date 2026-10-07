use crate::{diagnostics::diagnostic, *};
use ocdraw::ifccad::*;
use opencadcodec::{CadDocument, EntityType, Handle};
use std::collections::BTreeMap;

pub(super) fn to_cad(
    drawing: &IfccadDocument,
    document: &mut CadDocument,
    owners: &[(Handle, &[IfccadEntity])],
    mappings: &mut IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
    geometry: &mut crate::geometry_context::GeometryContext,
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
            Some(crate::mapping::line_pattern::target(document, mappings, id))
        } else {
            None
        };
        let mut common =
            crate::mapping::appearance::to_common(&e.appearance, pattern, layer, &loc, issues);
        common.linetype_scale = e.line_pattern_scale;
        common
    }
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
