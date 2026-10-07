//! Materialize CAD entities from validated OCDraw values, never JSON columns.

use super::{diagnostic, line_weight, OcdrawToCadDiagnostic, OcdrawToCadError};
use ocdraw::ocdraw::{
    AppearanceSelection, DrawingBlockDefinition, DrawingColor, EntityAppearance, OcdrawDocument,
};
use opencadcodec::{CadDocument, Color, EntityType, Handle, LineWeight, Transparency};
use std::collections::BTreeMap;

pub(super) fn color(
    value: &DrawingColor,
    location: &str,
    diagnostics: &mut Vec<OcdrawToCadDiagnostic>,
) -> (Color, Option<String>, Option<String>) {
    let rgb = value.rgb;
    let mapped = if let Some((system, index)) = &value.indexed {
        if system.eq_ignore_ascii_case("ACI") && (1..=255).contains(index) {
            Color::Index(*index as u8)
        } else {
            diagnostics.push(diagnostic(
                "COLOR_INDEX",
                location,
                "indexed color metadata cannot be represented in CAD",
            ));
            Color::from_rgb(rgb[0], rgb[1], rgb[2])
        }
    } else {
        Color::from_rgb(rgb[0], rgb[1], rgb[2])
    };
    let (catalog, name) = value
        .named
        .as_ref()
        .map(|(catalog, name)| (Some(catalog.clone()), Some(name.clone())))
        .unwrap_or((None, None));
    (mapped, catalog, name)
}

fn apply_common(
    patterns: &crate::mapping::line_pattern::ImportLinePatternMap,
    appearance: &EntityAppearance,
    visible: bool,
    common: &mut opencadcodec::entities::EntityCommon,
    layer_name: &str,
    location: &str,
    diagnostics: &mut Vec<OcdrawToCadDiagnostic>,
) {
    common.layer = layer_name.into();
    common.invisible = !visible;
    common.color_name = None;
    common.color = match &appearance.color {
        AppearanceSelection::ByLayer => Color::ByLayer,
        AppearanceSelection::ByBlock => Color::ByBlock,
        AppearanceSelection::Explicit(value) => {
            let (mapped, catalog, name) = color(value, location, diagnostics);
            if let (Some(catalog), Some(name)) = (catalog, name) {
                common.color_name = Some(format!("{catalog}${name}"));
            }
            mapped
        }
    };
    common.transparency = match appearance.opacity {
        AppearanceSelection::ByLayer => Transparency::ByLayer,
        AppearanceSelection::ByBlock => Transparency::ByBlock,
        AppearanceSelection::Explicit(value) => Transparency::from_percent(1.0 - value),
    };
    common.linetype_scale = appearance.line_pattern_scale;
    common.linetype_handle = match appearance.line_pattern {
        AppearanceSelection::Explicit(id) => Some(patterns[&id].1),
        _ => None,
    };
    common.linetype = match &appearance.line_pattern {
        AppearanceSelection::ByLayer => String::new(),
        AppearanceSelection::ByBlock => "ByBlock".into(),
        AppearanceSelection::Explicit(value) => patterns[value].0.clone(),
    };
    common.line_weight = match appearance.line_weight {
        AppearanceSelection::ByLayer => LineWeight::ByLayer,
        AppearanceSelection::ByBlock => LineWeight::ByBlock,
        AppearanceSelection::Explicit(value) => line_weight(value, location, diagnostics),
    };
}

pub(super) struct TargetIndex<'a> {
    pub patterns: &'a crate::mapping::line_pattern::ImportLinePatternMap,
    pub layers: &'a BTreeMap<u64, String>,
    pub layouts: &'a BTreeMap<u64, Option<String>>,
    pub block_handles: &'a BTreeMap<u64, Handle>,
    pub blocks: &'a BTreeMap<u64, DrawingBlockDefinition>,
}
pub(super) fn append_entities(
    drawing: &OcdrawDocument,
    document: &mut CadDocument,
    target: &TargetIndex<'_>,
    state: &mut crate::mapping::geometry::ExchangeState<u64>,
    diagnostics: &mut Vec<OcdrawToCadDiagnostic>,
    restore: crate::OcdrawPreservationRestore,
    preservation_report: &mut crate::OcdrawPreservationReport,
) -> Result<BTreeMap<u64, Handle>, OcdrawToCadError> {
    let TargetIndex {
        patterns,
        layers: layer_names,
        layouts: scope_layouts,
        block_handles,
        blocks,
    } = target;
    let mut entity_mapping = BTreeMap::new();
    let mut pending_clips = Vec::new();
    let mut pending_splines = Vec::new();
    let geometry = drawing
        .geometric_entities
        .iter()
        .map(|row| (row.id(), row))
        .collect::<BTreeMap<_, _>>();
    let viewports = drawing
        .viewports
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    let opaque = drawing
        .opaque_entities
        .iter()
        .map(|e| (e.id, e))
        .collect::<BTreeMap<_, _>>();
    for (owner, id) in drawing
        .scopes
        .iter()
        .flat_map(|scope| scope.entities.iter().map(move |id| (scope.id, *id)))
    {
        let location = format!("/entities/{id}");
        if let Some(source) = opaque.get(&id) {
            let record = drawing
                .preservation
                .as_ref()
                .and_then(|p| {
                    p.records
                        .iter()
                        .find(|r| r.id == source.preservation_record_id)
                })
                .expect("validated opaque record link");
            let prepared = if restore == crate::OcdrawPreservationRestore::Skip {
                Err(crate::OcdrawPreservationReason::UnsupportedContext)
            } else {
                crate::preservation::restore_spline(record, source, drawing)
            };
            let prepared = prepared.and_then(|mut spline| {
                if spline
                    .common
                    .extended_data
                    .records()
                    .iter()
                    .any(|r| document.app_ids.get(&r.application_name).is_none())
                {
                    return Err(crate::OcdrawPreservationReason::UnsupportedContext);
                }
                let layer = if let Some(id) = source.layer_id {
                    layer_names.get(&u64::from(id)).cloned()
                } else {
                    record
                        .bindings
                        .iter()
                        .find(|b| b.slot == "common.layer")
                        .and_then(|b| match b.target {
                            ocdraw::ocdraw::OcdrawPreservationTarget::Layer(id) => {
                                layer_names.get(&u64::from(id)).cloned()
                            }
                            _ => None,
                        })
                }
                .ok_or(crate::OcdrawPreservationReason::MissingDependency)?;
                if let Some(appearance) = &source.appearance {
                    apply_common(
                        patterns,
                        appearance,
                        source.visible,
                        &mut spline.common,
                        &layer,
                        &location,
                        diagnostics,
                    );
                    // Canonical byte-derived native opacity maps back exactly; arbitrary edits
                    // retain the existing quantization policy.
                    if let AppearanceSelection::Explicit(value) = appearance.opacity {
                        if let Some(alpha) = (0..=255_u16).find(|alpha| {
                            (1. - f64::from(*alpha) / 255.).to_bits() == value.to_bits()
                        }) {
                            spline.common.transparency = Transparency::Explicit(alpha as u8);
                        }
                    }
                } else {
                    spline.common.layer = layer;
                    spline.common.invisible = !source.visible;
                    if !spline.common.linetype.is_empty()
                        && !spline.common.linetype.eq_ignore_ascii_case("ByLayer")
                        && !spline.common.linetype.eq_ignore_ascii_case("ByBlock")
                    {
                        let binding = record
                            .bindings
                            .iter()
                            .find(|b| b.slot == "common.linetype")
                            .ok_or(crate::OcdrawPreservationReason::MissingDependency)?;
                        let ocdraw::ocdraw::OcdrawPreservationTarget::LinePattern(id) =
                            binding.target
                        else {
                            return Err(crate::OcdrawPreservationReason::UnresolvedReference);
                        };
                        let pattern = patterns
                            .get(&id)
                            .ok_or(crate::OcdrawPreservationReason::MissingDependency)?;
                        spline.common.linetype = pattern.0.clone();
                        spline.common.linetype_handle = Some(pattern.1);
                    } else {
                        spline.common.linetype_handle = None;
                    }
                }
                spline.common.layer_handle =
                    document.layers.get(&spline.common.layer).map(|l| l.handle);
                spline.common.handle = Handle::NULL;
                spline.common.owner_handle = if let Some(handle) =
                    block_handles.get(&u64::from(owner))
                {
                    *handle
                } else if let Some(Some(layout)) = scope_layouts.get(&u64::from(owner)) {
                    document
                        .objects
                        .values()
                        .find_map(|o| match o {
                            opencadcodec::objects::ObjectType::Layout(l) if &l.name == layout => {
                                Some(l.block_record)
                            }
                            _ => None,
                        })
                        .ok_or(crate::OcdrawPreservationReason::MissingDependency)?
                } else {
                    document.header.model_space_block_handle
                };
                let kind = drawing
                    .scopes
                    .iter()
                    .find(|s| s.id == owner)
                    .expect("validated owner")
                    .kind;
                spline.common.entity_mode = Some(match kind {
                    ocdraw::ocdraw::DrawingScopeKind::Model => 2,
                    ocdraw::ocdraw::DrawingScopeKind::Paper => 1,
                    ocdraw::ocdraw::DrawingScopeKind::Block => 0,
                });
                spline.common.raw_record = None;
                Ok(spline)
            });
            match prepared {
                Err(reason) => {
                    crate::preservation::report_restore(
                        preservation_report,
                        record,
                        Some(id),
                        Some(reason),
                    );
                    diagnostics.push(diagnostic(
                        "PRESERVATION_NOT_RESTORED",
                        &location,
                        format!("live opaque content not restored: {reason:?}"),
                    ));
                }
                Ok(spline) => {
                    let entity = EntityType::Spline(spline);
                    let handle = match scope_layouts.get(&u64::from(owner)) {
                        Some(Some(layout)) => document.add_entity_to_layout(entity, layout),
                        _ => document.add_entity(entity),
                    }
                    .map_err(|error| {
                        OcdrawToCadError::Cad(format!("opaque entity {id}: {error}"))
                    })?;
                    entity_mapping.insert(id, handle);
                    pending_splines.push((id, handle));
                }
            }
            continue;
        }
        let (scope_id, layer_id, appearance, visible, mut entity) =
            if let Some(source) = geometry.get(&id) {
                let (entity, bound, changed) =
                    crate::mapping::geometry::to_cad(source, owner, blocks, state)?;
                if bound > 0.0 {
                    diagnostics.push(diagnostic(
                        "GEOMETRY_ROUNDED_WITHIN_TOLERANCE",
                        &location,
                        format!("geometry deviation is at most {bound} drawing units"),
                    ));
                }
                if changed {
                    diagnostics.push(diagnostic(
                        "PARAMETERIZATION_CHANGED",
                        &location,
                        "geometric parameterization was changed to the CAD coordinate convention",
                    ));
                }
                (
                    u64::from(owner),
                    source.layer_id(),
                    source.appearance(),
                    source.visible(),
                    entity,
                )
            } else if let Some(source) = viewports.get(&id) {
                let Some(entity) =
                    crate::mapping::viewport::to_cad(source, document, layer_names, diagnostics)
                else {
                    continue;
                };
                (
                    u64::from(owner),
                    source.layer_id,
                    &source.appearance,
                    source.visible,
                    EntityType::Viewport(entity),
                )
            } else {
                continue;
            };
        let layer_name = &layer_names[&u64::from(layer_id)];
        apply_common(
            patterns,
            appearance,
            visible,
            entity.common_mut(),
            layer_name,
            &location,
            diagnostics,
        );
        let handle = match scope_layouts.get(&scope_id) {
            Some(None) => document.add_entity(entity),
            Some(Some(layout)) => document.add_entity_to_layout(entity, layout),
            None => {
                entity.common_mut().owner_handle = block_handles[&scope_id];
                document.add_entity(entity)
            }
        }
        .map_err(|error| OcdrawToCadError::Cad(format!("entity {id}: {error}")))?;
        entity_mapping.insert(id, handle);
        if let Some(source) = viewports.get(&id) {
            if let Some(boundary) = source.paper_clip.boundary_entity_id {
                pending_clips.push((handle, boundary));
            }
        }
    }
    // Rebind only after construction, propagating missing targets through live dependants.
    loop {
        let mut failed = Vec::new();
        for (id, handle) in &pending_splines {
            if !entity_mapping.contains_key(id) {
                continue;
            }
            let source = opaque[id];
            let record = drawing
                .preservation
                .as_ref()
                .unwrap()
                .records
                .iter()
                .find(|r| r.id == source.preservation_record_id)
                .unwrap();
            let Some(EntityType::Spline(stored)) = document.get_entity(*handle) else {
                return Err(OcdrawToCadError::Cad("constructed spline missing".into()));
            };
            let mut spline = stored.clone();
            let original = crate::preservation::decode_spline_snapshot(&record.payload.bytes)
                .map_err(|e| OcdrawToCadError::Cad(e.to_string()))?
                .to_source();
            spline.common.extended_data = original.common.extended_data;
            if let Err(reason) = crate::preservation::rebind_spline_references(
                &mut spline,
                record,
                source,
                drawing,
                document,
                &entity_mapping,
            ) {
                failed.push((*id, *handle, reason));
            } else if let Some(EntityType::Spline(target)) = document.get_entity_mut(*handle) {
                *target = spline;
            }
        }
        if failed.is_empty() {
            break;
        }
        for (id, handle, reason) in failed {
            document.remove_entity(handle);
            for record in document.block_records.iter_mut() {
                record.entity_handles.retain(|h| *h != handle);
            }
            entity_mapping.remove(&id);
            let source = opaque[&id];
            let record = drawing
                .preservation
                .as_ref()
                .unwrap()
                .records
                .iter()
                .find(|r| r.id == source.preservation_record_id)
                .unwrap();
            crate::preservation::report_restore(
                preservation_report,
                record,
                Some(id),
                Some(reason),
            );
            diagnostics.push(diagnostic(
                "PRESERVATION_NOT_RESTORED",
                format!("/entities/{id}"),
                format!("required target was not constructed: {reason:?}"),
            ));
        }
    }
    for (id, _) in pending_splines {
        if entity_mapping.contains_key(&id) {
            let source = opaque[&id];
            let record = drawing
                .preservation
                .as_ref()
                .unwrap()
                .records
                .iter()
                .find(|r| r.id == source.preservation_record_id)
                .unwrap();
            crate::preservation::report_restore(preservation_report, record, Some(id), None);
        }
    }
    for (viewport, boundary) in pending_clips {
        let boundary = entity_mapping.get(&boundary).copied().ok_or_else(|| {
            OcdrawToCadError::Cad(format!(
                "viewport {viewport} clip boundary was not constructed"
            ))
        })?;
        let Some(EntityType::Viewport(target)) = document.get_entity_mut(viewport) else {
            return Err(OcdrawToCadError::Cad(format!(
                "viewport {viewport} was not constructed"
            )));
        };
        target.clip_boundary_handle = boundary;
    }
    Ok(entity_mapping)
}
