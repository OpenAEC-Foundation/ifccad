use crate::{diagnostics::diagnostic, *};
use ocdraw::ifccad::*;
use opencadcodec::{CadDocument, EntityType, Handle};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct SourceScope<'a> {
    pub unit: &'a str,
    pub paper: Option<Option<&'a ocdraw::plot_kernel::PlotSettings>>,
    pub hatch_tolerance: ocdraw::geometry_kernel::hatch::HatchJoinToleranceRequest,
    pub document: &'a CadDocument,
    pub patterns: &'a crate::mapping::line_pattern::SourcePatterns,
    pub entities: &'a [Handle],
}
pub(super) fn from_cad(
    scope: SourceScope<'_>,
    preservation: &mut crate::preservation::Capture,
    ids: &mut IfccadIdCounters,
    mappings: &mut IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
    geometry: &mut crate::geometry_context::GeometryContext,
) -> Result<Vec<IfccadEntity>, IfccadConversionError> {
    let SourceScope {
        unit,
        paper,
        hatch_tolerance,
        document: source,
        patterns,
        entities: handles,
    } = scope;
    let mut retained = BTreeMap::new();
    for h in handles {
        let e = source.get_entity(*h).expect("inspected entity");
        if preservation.enabled && matches!(e, EntityType::Spline(_)) {
            continue;
        }
        if matches!(e, EntityType::Viewport(_)) {
            continue;
        }
        let loc = format!("entity/{h}");
        let appearance =
            crate::mapping::appearance::from_common(e.common(), patterns, &loc, issues);
        let mut kind = match e {
            EntityType::Hatch(h) => crate::mapping::hatch::from_cad(
                h,
                cad_geometry_convert::hatch::resolve_creation_tolerance(
                    hatch_tolerance,
                    unit,
                    paper,
                )?,
                geometry,
                issues,
            )?,
            EntityType::Text(_) | EntityType::MText(_) => {
                crate::mapping::text::from_entity(source, e, mappings, geometry, issues)?
            }
            EntityType::Insert(i) => mappings
                .blocks
                .ifccad_id(
                    source
                        .block_records
                        .get(&i.block_name)
                        .expect("inspected target")
                        .handle,
                )
                .and_then(|id| crate::mapping::blocks::from_insert(i, id, &loc, issues)),
            e => crate::mapping::geometry::from_entity(e, &loc, issues, geometry)?,
        };
        if let Some(kind) = &mut kind {
            if let (IfccadEntityKind::BlockInstance { transform, .. }, EntityType::Insert(insert)) =
                (kind, e)
            {
                let record = source
                    .block_records
                    .get(&insert.block_name)
                    .expect("inspected definition");
                let identity = crate::IfccadGeometryEntitySource::CadEntity {
                    handle: *h,
                    kind: "INSERT".into(),
                };
                let (native, source_map, target_map) =
                    cad_geometry_convert::geometry::blocks::from_cad_instance(
                        insert,
                        record.base_point,
                        &geometry.state.assessment,
                    )
                    .map_err(|e| geometry.numerical_failure(e))?;
                *transform = IfccadBlockTransform {
                    placement: crate::mapping::geometry::placement(native.placement()),
                    rotation: native.rotation(),
                    scale: [native.scale().x(), native.scale().y(), native.scale().z()],
                };
                geometry.state.register_identity(h.value(), identity);
                geometry.state.record_instance_parts(
                    h.value(),
                    record.handle.value(),
                    source_map,
                    target_map,
                );
                if cad_geometry_convert::geometry::stored_normal(native.placement())
                    != Some(insert.normal)
                {
                    issues.push(crate::diagnostics::modification(
                        "source-normal-normalized",
                        &loc,
                        "insert normal magnitude changed in native frame",
                    ));
                }
            }
        }
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
        if paper && crate::mapping::viewport::overall_canvas(source, v) {
            continue;
        }
        if !paper || !crate::source::workspace::authored(source, v) {
            issues.push(diagnostic(
                "viewport-camera",
                loc,
                "viewport owner or authored-versus-overall role cannot be established",
            ));
            continue;
        }
        let Some(mut view) =
            crate::mapping::viewport::from_cad(v, model_id, mappings, &loc, issues)
        else {
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
            let kind = retained.get(&boundary).map(|(kind, _)| kind);
            let opaque = preservation.enabled
                && matches!(source.get_entity(boundary), Some(EntityType::Spline(_)));
            if kind.is_none() && !(opaque && !view.paper_clip.enabled) {
                issues.push(diagnostic(
                    "viewport-clip",
                    loc,
                    "missing or unsupported active clip boundary; whole viewport omitted",
                ));
                continue;
            }
            if view.paper_clip.enabled
                && kind.is_some_and(|kind| {
                    validate_ifccad_viewport_boundary(&view.frame, kind).is_err()
                })
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
        let appearance = crate::mapping::appearance::from_common(&common, patterns, &loc, issues);
        retained.insert(*h, (IfccadEntityKind::Viewport(view), appearance));
    }
    let mut entities = Vec::new();
    for (index, h) in handles.iter().enumerate() {
        if preservation.enabled {
            if let Some(EntityType::Spline(s)) = source.get_entity(*h) {
                entities.push(preservation.entity(
                    s,
                    index as u64,
                    source,
                    patterns,
                    ids,
                    mappings,
                )?);
                continue;
            }
        }
        if let Some((kind, appearance)) = retained.remove(h) {
            let e = source.get_entity(*h).expect("inspected entity");
            let id = ids
                .allocate_entity_id()
                .map_err(IfccadConversionError::IdAllocation)?;
            mappings.entities.insert(id, *h);
            entities.push(IfccadEntity::Native(IfccadNativeEntity {
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
            }));
        }
    }
    for entity in entities.iter_mut().filter_map(IfccadEntity::as_native_mut) {
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
    crate::source::hatch::bind(&mut entities, source, mappings, issues);
    Ok(entities)
}
