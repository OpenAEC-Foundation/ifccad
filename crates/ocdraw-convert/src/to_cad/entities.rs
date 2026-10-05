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
    for (owner, id) in drawing
        .scopes
        .iter()
        .flat_map(|scope| scope.entities.iter().map(move |id| (scope.id, *id)))
    {
        let location = format!("/entities/{id}");
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
