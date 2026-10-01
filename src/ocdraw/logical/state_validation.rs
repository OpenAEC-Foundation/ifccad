//! Cross-record view, clipping and workspace rules shared by reader and writer readback.
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
    [grid.spacing.x(), grid.spacing.y()]
        .into_iter()
        .all(|v| v.is_finite() && v >= 0.)
        && grid.major_line_frequency > 0
}
fn valid_snap(snap: DrawingSnap) -> bool {
    [
        snap.base.x(),
        snap.base.y(),
        snap.spacing.x(),
        snap.spacing.y(),
        snap.angle,
    ]
    .into_iter()
    .all(f64::is_finite)
        && snap.spacing.x() > 0.
        && snap.spacing.y() > 0.
}
fn valid_view(view: DrawingView, paper: bool) -> bool {
    let d = view.direction.components();
    let norm = d[0].hypot(d[1]).hypot(d[2]);
    let front = match view.front_clip.mode {
        DrawingClipMode::Disabled => None,
        DrawingClipMode::AtCamera => Some(norm),
        DrawingClipMode::AtDistance => view.front_clip.distance,
    };
    let back = if view.back_clip.mode == DrawingClipMode::AtDistance {
        view.back_clip.distance
    } else {
        None
    };
    [view.center.x(), view.center.y(), view.height, view.twist]
        .into_iter()
        .chain(view.target.components())
        .chain(d)
        .all(f64::is_finite)
        && norm.is_finite()
        && norm > 0.
        && view.height > 0.
        && (!paper || view.projection == DrawingProjection::Orthographic)
        && match (view.projection, view.lens_length) {
            (DrawingProjection::Orthographic, None) => true,
            (DrawingProjection::Orthographic, Some(v)) => v.is_finite() && v >= 0.,
            (DrawingProjection::Perspective, Some(v)) => v.is_finite() && v > 0.,
            _ => false,
        }
        && view.front_clip.distance.is_none_or(f64::is_finite)
        && view.back_clip.distance.is_none_or(f64::is_finite)
        && (view.front_clip.mode != DrawingClipMode::AtDistance || front.is_some())
        && view.back_clip.mode != DrawingClipMode::AtCamera
        && (view.back_clip.mode != DrawingClipMode::AtDistance || back.is_some())
        && front.zip(back).is_none_or(|(front, back)| back < front)
}
pub(crate) fn validate_state(document: &DrawingDocument) -> Vec<LogicalError> {
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
                let valid = geometry.get(&id).is_some_and(|entity| {
                    let EntityGeometry::PlanarPolyline {
                        placement,
                        vertices,
                        closed,
                    } = entity.geometry()
                    else {
                        return false;
                    };
                    if !closed || vertices.iter().any(|v| v[2] != 0.) {
                        return false;
                    }
                    let distinct = vertices
                        .iter()
                        .map(|v| {
                            let bits = |v: f64| if v == 0. { 0 } else { v.to_bits() };
                            (bits(v[0]), bits(v[1]))
                        })
                        .collect::<BTreeSet<_>>();
                    let Some(bounds) = super::viewport_bounds(viewport.frame) else {
                        return false;
                    };
                    distinct.len() >= 3
                        && vertices.iter().all(|v| {
                            placement.enclosed_by(crate::ocdraw::Point2::new(v[0], v[1]), bounds)
                        })
                });
                if !valid {
                    errors.push(error("VIEWPORT_CLIP",format!("{prefix}/paperClip/{index}"),"active boundary must be a closed straight polyline with three distinct vertices inside the viewport frame"));
                }
            }
        }
        if let Some(shading) = viewport.plot_shading_override {
            let custom = shading.quality.mode == crate::ocdraw::ShadedPlotQualityMode::Custom;
            if (custom
                && !shading
                    .quality
                    .dpi
                    .is_some_and(|dpi| (100..=32767).contains(&dpi)))
                || (!custom && shading.quality.dpi.is_some())
            {
                errors.push(error(
                    "VIEWPORT_SHADING",
                    format!("{prefix}/plotShadingOverride/{index}"),
                    "only Custom shading quality carries dpi 100..32767",
                ));
            }
        }
        for override_row in &viewport.layer_overrides {
            if !override_row.frozen
                && override_row.color.is_none()
                && override_row.opacity.is_none()
                && override_row.line_pattern.is_none()
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
    if document.view_state.is_none() && !document.model_windows.is_empty() {
        errors.push(error(
            "WORKSPACE_STATE",
            "/drawingViewState",
            "model windows require drawing view state",
        ));
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
            .find(|w| w.id == view.active_model_window_id)
            .is_some_and(|w| w.use_stored_ucs && w.stored_ucs != view.current_model_ucs)
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
        match canvas.active_context {
            DrawingPaperContext::Canvas if canvas.stored_ucs != canvas.current_ucs => {
                errors.push(error(
                    "WORKSPACE_STATE",
                    format!("/paperCanvases/{index}/currentUcs"),
                    "active canvas stored UCS conflicts with current UCS",
                ))
            }
            DrawingPaperContext::Viewport(id) => {
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
                } else if workspace
                    .is_some_and(|w| w.use_stored_ucs && w.stored_ucs != canvas.current_ucs)
                {
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
            || owners
                .get(&id)
                .is_none_or(|owner| !canvases.contains(owner))
        {
            errors.push(error(
                "WORKSPACE_STATE",
                format!("/viewportWorkspaces/{index}/viewportEntityId"),
                "workspace must select one unique viewport with a paper canvas",
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
