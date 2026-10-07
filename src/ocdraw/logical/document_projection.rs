//! Derived structural checks; never an independently editable drawing.
use super::*;
use crate::ocdraw::PlotArea;

fn pair<T>(value: &AppearanceSelection<T>) -> AppearancePair {
    let mode = match value {
        AppearanceSelection::ByLayer => AppearanceMode::ByLayer,
        AppearanceSelection::ByBlock => AppearanceMode::ByBlock,
        AppearanceSelection::Explicit(_) => AppearanceMode::Explicit,
    };
    AppearancePair {
        mode,
        has_value: mode == AppearanceMode::Explicit,
    }
}

pub(crate) fn project_validation_model(
    doc: &OcdrawDocument,
    phase: ValidationPhase,
) -> DrawingModel {
    let completeness = derive_scope_geometry_completeness(doc);
    let mut refs = doc
        .layers
        .iter()
        .enumerate()
        .map(|(i, l)| (l.line_pattern_id, format!("/layers/{i}/linePatternId")))
        .collect::<Vec<_>>();
    let mut scales = vec![(doc.line_pattern_scale, "/linePatternScale".into())];
    let mut entities = Vec::new();
    for (id, layer_id, a, definition_scope_id) in doc
        .geometric_entities
        .iter()
        .map(|e| {
            (
                e.id,
                e.layer_id,
                &e.appearance,
                match e.geometry {
                    EntityGeometry::BlockInstance {
                        definition_scope_id,
                        ..
                    } => Some(definition_scope_id),
                    _ => None,
                },
            )
        })
        .chain(
            doc.viewports
                .iter()
                .map(|e| (e.id, e.layer_id, &e.appearance, None)),
        )
    {
        if let AppearanceSelection::Explicit(p) = a.line_pattern {
            refs.push((p, format!("/entities/{id}/linePatternId")));
        }
        scales.push((
            a.line_pattern_scale,
            format!("/entities/{id}/linePatternScale"),
        ));
        entities.push(Entity {
            id,
            layer_id: Some(layer_id),
            definition_scope_id,
            appearance: Some([
                pair(&a.color),
                pair(&a.opacity),
                pair(&a.line_pattern),
                pair(&a.line_weight),
            ]),
            location: format!("/entities/{id}"),
        });
    }
    for entity in &doc.opaque_entities {
        let id = entity.id;
        if let Some(a) = &entity.appearance {
            if let AppearanceSelection::Explicit(p) = a.line_pattern {
                refs.push((p, format!("/opaqueEntities/{id}/appearance/linePattern")));
            }
            scales.push((
                a.line_pattern_scale,
                format!("/opaqueEntities/{id}/appearance/linePatternScale"),
            ));
        }
        entities.push(Entity {
            id,
            layer_id: entity.layer_id,
            definition_scope_id: None,
            appearance: entity.appearance.as_ref().map(|a| {
                [
                    pair(&a.color),
                    pair(&a.opacity),
                    pair(&a.line_pattern),
                    pair(&a.line_weight),
                ]
            }),
            location: format!("/opaqueEntities/{id}"),
        });
    }
    for viewport in &doc.viewports {
        for row in &viewport.layer_overrides {
            if let Some(id) = row.line_pattern_id {
                refs.push((id, format!("/viewports/{}/overrides", viewport.id)));
            }
        }
    }
    let mut named_ucs_refs = Vec::new();
    let mut selection = |ucs, location: String| {
        if let DrawingUcsSelection::Named(id) = ucs {
            named_ucs_refs.push((Some(id), location));
        }
    };
    if let Some(s) = doc.view_state {
        selection(
            s.current_model_ucs,
            "/drawingViewState/currentModelUcs/ucsId".into(),
        );
    }
    for (i, s) in doc.model_windows.iter().enumerate() {
        selection(s.stored_ucs, format!("/modelWindows/{i}/storedUcs/ucsId"));
    }
    for (i, s) in doc.paper_canvases.iter().enumerate() {
        selection(s.stored_ucs, format!("/paperCanvases/{i}/storedUcs/ucsId"));
        selection(
            s.current_ucs,
            format!("/paperCanvases/{i}/currentUcs/ucsId"),
        );
    }
    for (i, s) in doc.viewport_workspaces.iter().enumerate() {
        selection(
            s.stored_ucs,
            format!("/viewportWorkspaces/{i}/storedUcs/ucsId"),
        );
    }
    DrawingModel {
        line_patterns: doc.line_patterns.clone(),
        next_line_pattern_id: doc.next_line_pattern_id,
        line_pattern_refs: refs,
        line_pattern_scales: scales,
        next_entity_id: doc.next_entity_id,
        next_layer_id: doc.next_layer_id,
        next_layout_id: doc.next_layout_id,
        layers: doc
            .layers
            .iter()
            .map(|l| NamedId {
                id: l.id,
                name: l.name.clone(),
            })
            .collect(),
        layouts: doc
            .layouts
            .iter()
            .map(|l| Layout {
                id: l.id,
                name: l.name.clone(),
                scope_id: l.scope_id,
                kind: match l.kind {
                    DrawingLayoutKind::Model => ScopeKind::Model,
                    DrawingLayoutKind::Paper => ScopeKind::Paper,
                },
                tab_index: l.tab_index,
                limits: l.settings.limits,
                plot_rectangles: l.settings.plot_settings.as_ref().map(|p| PlotRectangles {
                    printable_area: p.page.printable_area,
                    window: match p.area {
                        PlotArea::Window(r) => Some(r),
                        _ => None,
                    },
                }),
            })
            .collect(),
        scopes: doc
            .scopes
            .iter()
            .map(|s| Scope {
                geometry_completeness: completeness[&s.id],
                id: s.id,
                kind: match s.kind {
                    DrawingScopeKind::Model => ScopeKind::Model,
                    DrawingScopeKind::Paper => ScopeKind::Paper,
                    DrawingScopeKind::Block => ScopeKind::Block,
                },
                entities: s.entities.clone(),
                has_bounds: (phase == ValidationPhase::Complete).then_some(s.bounds.is_some()),
                bounds: if phase == ValidationPhase::Complete {
                    s.bounds
                } else {
                    None
                },
            })
            .collect(),
        blocks: doc
            .block_definitions
            .iter()
            .map(|d| BlockDefinition {
                scope_id: d.scope_id,
                name: d.name.clone(),
            })
            .collect(),
        entities,
        current_layer_id: doc.workspace_state.and_then(|s| s.current_layer_id),
        active_layout_id: doc.workspace_state.and_then(|s| s.active_layout_id),
        ucs_definitions: doc
            .ucs_definitions
            .iter()
            .map(|s| NamedUcs {
                id: s.id,
                name: s.definition.name.clone(),
                frame: Some(s.definition.frame),
            })
            .collect(),
        model_window_ids: doc.model_windows.iter().map(|w| w.id).collect(),
        active_model_window_id: doc.view_state.map(|s| s.active_model_window_id),
        named_ucs_refs,
        ucs_choices: Vec::new(),
    }
}
