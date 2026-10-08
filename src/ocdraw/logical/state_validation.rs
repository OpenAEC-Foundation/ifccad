//! Cross-record view, clipping and workspace rules shared by authored and decoded documents.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
fn error(code: &'static str, location: impl Into<String>, message: &str) -> LogicalError {
    LogicalError {
        code,
        location: location.into(),
        message: message.into(),
    }
}
fn valid_grid(grid: DrawingGrid) -> bool {
    crate::workspace_kernel::validate_grid(&grid).is_ok()
}
fn valid_snap(snap: DrawingSnap) -> bool {
    crate::workspace_kernel::validate_snap(&snap).is_ok()
}
fn valid_view(view: DrawingView, paper: bool) -> bool {
    use crate::workspace_kernel::{validate_view, WorkspaceViewKind};
    validate_view(
        &view,
        if paper {
            WorkspaceViewKind::PaperCanvas
        } else {
            WorkspaceViewKind::Model
        },
    )
    .is_ok()
}
pub(crate) fn validate_state(document: &OcdrawDocument) -> Vec<LogicalError> {
    let owners = owner_index(&document.scopes);
    let scopes = document
        .scopes
        .iter()
        .map(|s| (s.id, s))
        .collect::<BTreeMap<_, _>>();
    let geometry = document
        .geometric_entities
        .iter()
        .map(|e| (e.id(), e))
        .collect::<BTreeMap<_, _>>();
    let viewports = document
        .viewports
        .iter()
        .map(|v| (v.id, v))
        .collect::<BTreeMap<_, _>>();
    let mut errors = Vec::new();
    let mut claimed = BTreeSet::new();
    for (index, viewport) in document.viewports.iter().enumerate() {
        let prefix = "/streams/viewportStream";
        if !valid_view(viewport.view, false) {
            errors.push(error(
                "VIEWPORT_VIEW",
                format!("{prefix}/view/{index}"),
                "view requires finite values, a nonzero direction, valid lens and clip planes",
            ));
        }
        let clip = viewport.paper_clip;
        if clip.enabled && clip.boundary_entity_id.is_none() {
            errors.push(error(
                "VIEWPORT_CLIP",
                format!("{prefix}/paperClip/{index}"),
                "enabled clipping requires a boundary",
            ));
        }
        if let Some(id) = clip.boundary_entity_id {
            let same_owner = owners
                .get(&id)
                .is_some_and(|owner| Some(owner) == owners.get(&viewport.id));
            if !same_owner || !claimed.insert(id) {
                errors.push(error(
                    "VIEWPORT_CLIP",
                    format!("{prefix}/paperClip/{index}"),
                    "boundary must exist in the same paper scope and be unique to one viewport",
                ));
            } else if clip.enabled {
                if let Some(entity) = geometry.get(&id) {
                    if let Err(failure) =
                        super::validate_viewport_clip_boundary(viewport.frame, entity.geometry())
                    {
                        for diagnostic in failure.into_diagnostics() {
                            errors.push(error(
                                diagnostic.code,
                                format!("{prefix}/paperClip/{index}"),
                                &diagnostic.message,
                            ));
                        }
                    }
                } else {
                    errors.push(error(
                        "VIEWPORT_CLIP",
                        format!("{prefix}/paperClip/{index}"),
                        "active boundary must be a supported geometry entity",
                    ));
                }
            }
        }
        for override_row in &viewport.layer_overrides {
            if !override_row.frozen
                && override_row.color.is_none()
                && override_row.opacity.is_none()
                && override_row.line_pattern_id.is_none()
                && override_row.line_weight.is_none()
            {
                errors.push(error(
                    "VIEWPORT_LAYER",
                    format!("{prefix}/layerOverrideOffset/{index}"),
                    "override must contain an effective change",
                ));
            }
        }
    }
    for (index, window) in document.model_windows.iter().enumerate() {
        let rect = window.rectangle;
        if !rect
            .into_iter()
            .all(|v| v.is_finite() && (0.0..=1.).contains(&v))
            || rect[0] >= rect[2]
            || rect[1] >= rect[3]
        {
            errors.push(error(
                "WORKSPACE_STATE",
                format!("/modelWindows/{index}/rectangle"),
                "model window needs a positive normalized rectangle",
            ));
        }
        if !valid_view(window.view, false)
            || !window.aspect_ratio.is_finite()
            || window.aspect_ratio <= 0.
        {
            errors.push(error(
                "WORKSPACE_STATE",
                format!("/modelWindows/{index}/view"),
                "invalid model view or aspect ratio",
            ));
        }
        if !valid_grid(window.grid) || !valid_snap(window.snap) {
            errors.push(error(
                "WORKSPACE_STATE",
                format!("/modelWindows/{index}/grid"),
                "invalid grid or snap settings",
            ));
        }
    }
    if let Some(view) = document.view_state {
        if document
            .model_windows
            .iter()
            .find(|w| Some(w.id) == view.active_model_window_id)
            .is_some_and(|w| {
                w.use_stored_ucs
                    && view
                        .current_model_ucs
                        .is_some_and(|current| w.stored_ucs != current)
            })
        {
            errors.push(error(
                "WORKSPACE_STATE",
                "/drawingViewState/currentModelUcs",
                "active model window stored UCS conflicts with current UCS",
            ));
        }
    }
    let mut canvases = BTreeSet::new();
    for (index, canvas) in document.paper_canvases.iter().enumerate() {
        if !canvases.insert(canvas.scope_id)
            || scopes
                .get(&canvas.scope_id)
                .is_none_or(|s| s.kind != DrawingScopeKind::Paper)
        {
            errors.push(error(
                "WORKSPACE_STATE",
                format!("/paperCanvases/{index}/scopeId"),
                "canvas must select one unique paper scope",
            ));
        }
        if !valid_view(canvas.view, true) || !valid_grid(canvas.grid) || !valid_snap(canvas.snap) {
            errors.push(error(
                "WORKSPACE_STATE",
                format!("/paperCanvases/{index}/view"),
                "invalid paper view, grid or snap",
            ));
        }
        if canvas
            .frame
            .as_ref()
            .is_some_and(|frame| crate::workspace_kernel::validate_canvas_frame(frame).is_err())
        {
            errors.push(error(
                "WORKSPACE_STATE",
                format!("/paperCanvases/{index}/frame"),
                "invalid canvas frame",
            ));
        }
        if canvas.current_ucs.is_some() && canvas.active_context.is_none() {
            errors.push(error(
                "WORKSPACE_STATE",
                format!("/paperCanvases/{index}/currentUcs"),
                "current UCS requires a known active context",
            ));
        }
        match canvas.active_context {
            Some(DrawingPaperContext::Canvas)
                if canvas.use_stored_ucs
                    && canvas
                        .current_ucs
                        .is_some_and(|current| canvas.stored_ucs != current) =>
            {
                errors.push(error(
                    "WORKSPACE_STATE",
                    format!("/paperCanvases/{index}/currentUcs"),
                    "active canvas stored UCS conflicts with current UCS",
                ));
            }
            Some(DrawingPaperContext::Viewport(id)) => {
                let workspace = document
                    .viewport_workspaces
                    .iter()
                    .find(|w| w.viewport_entity_id == id);
                if !viewports.contains_key(&id)
                    || owners.get(&id) != Some(&canvas.scope_id)
                    || workspace.is_none()
                {
                    errors.push(error(
                        "WORKSPACE_STATE",
                        format!("/paperCanvases/{index}/activeContext"),
                        "active paper viewport must belong to this canvas and have a workspace row",
                    ));
                } else if workspace.is_some_and(|w| {
                    w.use_stored_ucs
                        && canvas
                            .current_ucs
                            .is_some_and(|current| w.stored_ucs != current)
                }) {
                    errors.push(error(
                        "WORKSPACE_STATE",
                        format!("/paperCanvases/{index}/currentUcs"),
                        "active viewport stored UCS conflicts with current UCS",
                    ));
                }
            }
            _ => {}
        }
    }
    let mut workspace_ids = BTreeSet::new();
    for (index, workspace) in document.viewport_workspaces.iter().enumerate() {
        let id = workspace.viewport_entity_id;
        if !workspace_ids.insert(id)
            || !viewports.contains_key(&id)
            || owners.get(&id).is_none_or(|owner| {
                scopes
                    .get(owner)
                    .is_none_or(|scope| scope.kind != DrawingScopeKind::Paper)
            })
        {
            errors.push(error(
                "WORKSPACE_STATE",
                format!("/viewportWorkspaces/{index}/viewportEntityId"),
                "workspace must select one unique viewport in a paper scope",
            ));
        }
        if !valid_grid(workspace.grid) || !valid_snap(workspace.snap) {
            errors.push(error(
                "WORKSPACE_STATE",
                format!("/viewportWorkspaces/{index}/grid"),
                "invalid grid or snap settings",
            ));
        }
    }
    errors
}
