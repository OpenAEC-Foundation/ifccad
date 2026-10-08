//! Format-owned workspace identity bindings and loss classification.
mod import;
use crate::{
    CadToOcdrawAction, CadToOcdrawDiagnostic, CadToOcdrawDiagnosticSource as Source,
    CadToOcdrawError, CadToOcdrawLossReason,
};
pub(crate) use import::apply;
use ocdraw::ocdraw::*;
use opencadcodec::objects::ObjectType;
use opencadcodec::{CadDocument, EntityType, Handle};
use std::collections::{BTreeMap, BTreeSet};

fn loss(diagnostics: &mut Vec<CadToOcdrawDiagnostic>, source: Source, field: &str, message: &str) {
    diagnostics.push(CadToOcdrawDiagnostic::loss(
        source,
        CadToOcdrawAction::PartiallyExported,
        vec![CadToOcdrawLossReason::UnsupportedSemantic {
            name: format!("{field}: {message}"),
        }],
    ));
}
fn frame(
    origin: opencadcodec::Vector3,
    x: opencadcodec::Vector3,
    y: opencadcodec::Vector3,
) -> Result<CoordinateFrame3, CadToOcdrawError> {
    Ok(cad_workspace_convert::prepare_ucs_frame_from_cad(
        origin, x, y,
    )?)
}

fn unnamed(value: CoordinateFrame3) -> DrawingUcsSelection {
    if value == CoordinateFrame3::default() {
        DrawingUcsSelection::World
    } else {
        DrawingUcsSelection::Unnamed(value)
    }
}
fn selection(
    handle: Handle,
    value: CoordinateFrame3,
    elevation: f64,
    document: &CadDocument,
    handles: &BTreeMap<Handle, u32>,
    diagnostics: &mut Vec<CadToOcdrawDiagnostic>,
    source: Source,
) -> DrawingUcsSelection {
    if let Some(&id) = handles.get(&handle) {
        if document.ucss.iter().any(|u| {
            u.handle == handle
                && frame(u.origin, u.x_axis, u.y_axis).ok() == Some(value)
                && u.elevation == elevation
        }) {
            return DrawingUcsSelection::Named(id);
        }
        loss(
            diagnostics,
            source.clone(),
            "storedUcs",
            "named definition disagrees with stored frame/elevation; retained unnamed frame",
        );
    } else if !handle.is_null() {
        loss(
            diagnostics,
            source.clone(),
            "storedUcs",
            "unresolved named handle; retained unnamed frame",
        );
    }
    if elevation != 0. {
        loss(
            diagnostics,
            source,
            "storedUcs.elevation",
            "unnamed/World selection cannot retain separate elevation",
        );
    }
    unnamed(value)
}
pub(crate) fn export(
    document: &CadDocument,
    drawing: &mut OcdrawBuilder,
    paper_scopes: &BTreeMap<Handle, u32>,
    names: &BTreeMap<String, u32>,
    handles: &BTreeMap<Handle, u32>,
    diagnostics: &mut Vec<CadToOcdrawDiagnostic>,
) -> Result<BTreeSet<Handle>, CadToOcdrawError> {
    for entity in document.entities() {
        if let EntityType::Viewport(view) = entity {
            if paper_scopes.contains_key(&view.common.owner_handle) {
                cad_workspace_convert::prepare_viewport_aids_from_cad(view)?;
            }
        }
    }
    let mut saved = DrawingSavedState::default();
    let mut mapped = BTreeSet::new();
    let header_frame = frame(
        document.header.model_space_ucs_origin,
        document.header.model_space_ucs_x_axis,
        document.header.model_space_ucs_y_axis,
    )?;
    let current = if document.header.model_space_ucs_name.is_empty() {
        Some(unnamed(header_frame))
    } else if let Some(&id) = names.get(&document.header.model_space_ucs_name.to_lowercase()) {
        let candidates: Vec<_> = document
            .ucss
            .iter()
            .filter(|u| {
                u.name
                    .eq_ignore_ascii_case(&document.header.model_space_ucs_name)
            })
            .collect();
        if candidates.len() == 1
            && frame(
                candidates[0].origin,
                candidates[0].x_axis,
                candidates[0].y_axis,
            )? == header_frame
        {
            Some(DrawingUcsSelection::Named(id))
        } else {
            loss(
                diagnostics,
                Source::DocumentField {
                    name: "header.model_space_ucs".into(),
                },
                "currentModelUcs",
                "ambiguous name or inconsistent named frame",
            );
            None
        }
    } else {
        loss(
            diagnostics,
            Source::DocumentField {
                name: "header.model_space_ucs".into(),
            },
            "currentModelUcs",
            "unresolved name",
        );
        None
    };
    for v in document
        .vports
        .iter()
        .filter(|v| v.name.eq_ignore_ascii_case("*active"))
    {
        let prepared = cad_workspace_convert::prepare_model_window_from_cad(v)?;
        let value = prepared.value;
        let source = Source::Object {
            handle: v.handle,
            kind: "VPORT".into(),
        };
        let stored_ucs = selection(
            v.named_ucs_handle,
            value.aids.ucs_frame,
            value.aids.ucs_elevation,
            document,
            handles,
            diagnostics,
            source.clone(),
        );
        for issue in prepared.losses {
            loss(diagnostics, source.clone(), issue.field, &issue.message);
        }
        let id = u32::try_from(saved.model_windows.len()).map_err(|_| {
            cad_workspace_convert::WorkspaceNumericError {
                field: "modelWindows",
                message: "too many windows".into(),
            }
        })?;
        saved.model_windows.push(DrawingModelWindow {
            id,
            rectangle: value.rectangle,
            view: value.view,
            aspect_ratio: value.aspect_ratio,
            render_mode: value.render_mode,
            grid: value.aids.grid,
            snap: value.aids.snap,
            stored_ucs,
            use_stored_ucs: value.aids.use_stored_ucs,
        });
        mapped.insert(v.handle);
    }
    let active = crate::source::workspace::active_model_window(&saved.model_windows, current);
    if !saved.model_windows.is_empty() && active.is_none() {
        loss(
            diagnostics,
            Source::DocumentField {
                name: "modelWindows.active".into(),
            },
            "activeModelWindow",
            "association unavailable or inconsistent; windows retained",
        );
    }
    saved.view_state = Some(DrawingViewState {
        current_model_ucs: current,
        active_model_window_id: active,
    });
    for layout in document.objects.values().filter_map(|o| match o {
        ObjectType::Layout(l) => Some(l),
        _ => None,
    }) {
        let Some(&scope_id) = paper_scopes.get(&layout.block_record) else {
            continue;
        };
        let Some(handle) = crate::source::overall_viewport_handle(document, layout) else {
            if document.entities().any(|e| matches!(e,EntityType::Viewport(v) if v.common.owner_handle == layout.block_record)) {
                loss(diagnostics,Source::Object{handle:layout.handle,kind:"LAYOUT".into()},"canvas","overall viewport role unavailable");
            }
            continue;
        };
        let Some(EntityType::Viewport(v)) = document.get_entity(handle) else {
            continue;
        };
        let prepared = cad_workspace_convert::prepare_canvas_from_cad(v)?;
        let value = prepared.value;
        let source = Source::Entity {
            handle,
            kind: "VIEWPORT".into(),
        };
        let stored_ucs = selection(
            v.ucs_handle,
            value.aids.ucs_frame,
            value.aids.ucs_elevation,
            document,
            handles,
            diagnostics,
            source.clone(),
        );
        for issue in prepared.losses {
            loss(diagnostics, source.clone(), issue.field, &issue.message);
        }
        loss(
            diagnostics,
            source.clone(),
            "activeContext/currentUcs",
            "codec does not expose current Paper selection",
        );
        for issue in crate::mapping::viewport::deferred_canvas_losses(v) {
            diagnostics.push(CadToOcdrawDiagnostic::loss(
                source.clone(),
                CadToOcdrawAction::PartiallyExported,
                vec![issue],
            ));
        }
        saved.paper_canvases.push(DrawingPaperCanvas {
            scope_id,
            view: value.view,
            frame: value.frame,
            grid: value.aids.grid,
            snap: value.aids.snap,
            stored_ucs,
            use_stored_ucs: value.aids.use_stored_ucs,
            current_ucs: None,
            active_context: None,
        });
    }
    drawing.set_saved_state(saved);
    Ok(mapped)
}
pub(crate) fn bind_viewport_workspaces(
    document: &CadDocument,
    drawing: &mut OcdrawDocument,
    entities: &BTreeMap<Handle, u64>,
    handles: &BTreeMap<Handle, u32>,
    diagnostics: &mut Vec<CadToOcdrawDiagnostic>,
) -> Result<(), CadToOcdrawError> {
    for entity in document.entities() {
        let EntityType::Viewport(v) = entity else {
            continue;
        };
        if !drawing
            .viewports
            .iter()
            .any(|p| entities.get(&v.common.handle) == Some(&p.id))
        {
            continue;
        }
        let prepared = cad_workspace_convert::prepare_viewport_aids_from_cad(v)?;
        let value = prepared.value;
        let source = Source::Entity {
            handle: v.common.handle,
            kind: "VIEWPORT".into(),
        };
        let stored_ucs = selection(
            v.ucs_handle,
            value.ucs_frame,
            value.ucs_elevation,
            document,
            handles,
            diagnostics,
            source.clone(),
        );
        for issue in prepared.losses {
            loss(diagnostics, source.clone(), issue.field, &issue.message);
        }
        drawing.viewport_workspaces.push(DrawingViewportWorkspace {
            viewport_entity_id: entities[&v.common.handle],
            grid: value.grid,
            snap: value.snap,
            stored_ucs,
            use_stored_ucs: value.use_stored_ucs,
        });
    }
    Ok(())
}
