use super::*;
use crate::geometry_kernel::CoordinateFrame3;
use crate::workspace_kernel::*;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IfccadUcsId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IfccadModelWindowId(pub u64);
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadUcsDefinition {
    pub id: IfccadUcsId,
    pub name: String,
    pub frame: CoordinateFrame3,
    pub elevation: f64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IfccadUcsSelection {
    World,
    Named(IfccadUcsId),
    Unnamed(CoordinateFrame3),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadPaperContext {
    Canvas,
    Viewport(u64),
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct IfccadDrawingWorkspaceState {
    pub current_layer_id: Option<u64>,
    pub active_layout_id: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Default)]
pub struct IfccadModelViewState {
    pub current_model_ucs: Option<IfccadUcsSelection>,
    pub active_model_window_id: Option<IfccadModelWindowId>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadModelWindow {
    pub id: IfccadModelWindowId,
    pub rectangle: [f64; 4],
    pub view: WorkspaceView,
    pub aspect_ratio: f64,
    pub render_mode: WorkspaceRenderMode,
    pub grid: WorkspaceGrid,
    pub snap: WorkspaceSnap,
    pub stored_ucs: IfccadUcsSelection,
    pub use_stored_ucs: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadPaperCanvas {
    pub view: WorkspaceView,
    pub frame: Option<WorkspaceCanvasFrame>,
    pub grid: WorkspaceGrid,
    pub snap: WorkspaceSnap,
    pub stored_ucs: IfccadUcsSelection,
    pub use_stored_ucs: bool,
    pub current_ucs: Option<IfccadUcsSelection>,
    pub active_context: Option<IfccadPaperContext>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadViewportWorkspace {
    pub grid: WorkspaceGrid,
    pub snap: WorkspaceSnap,
    pub stored_ucs: IfccadUcsSelection,
    pub use_stored_ucs: bool,
}

pub(crate) fn validate_workspace(document: &IfccadDocument) -> Result<(), IfccadReport> {
    let prefix = format!("/cad/d{}", document.drawing_id);
    let fail = |path: &str, message: &str| {
        crate::ifccad::diagnostics::failure("IFCCAD-WORKSPACE-001", path, message)
    };
    let mut ucs_ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for ucs in &document.ucs_definitions {
        let path = format!("{prefix}/ucs/{}", ucs.id.0);
        if !ucs_ids.insert(ucs.id)
            || ucs.name.trim().is_empty()
            || !names.insert(crate::ocdraw::names::name_key(&ucs.name))
            || !ucs.elevation.is_finite()
        {
            return Err(fail(&path, "invalid or duplicate UCS definition"));
        }
    }
    let choice = |value: IfccadUcsSelection, path: &str| -> Result<(), IfccadReport> {
        if matches!(value, IfccadUcsSelection::Named(id) if !ucs_ids.contains(&id)) {
            Err(fail(path, "named UCS does not resolve"))
        } else {
            Ok(())
        }
    };
    let mut window_ids = BTreeSet::new();
    for window in &document.model_windows {
        let path = format!("{prefix}/modelWindow/{}", window.id.0);
        if !window_ids.insert(window.id)
            || !window
                .rectangle
                .into_iter()
                .all(|v| v.is_finite() && (0.0..=1.).contains(&v))
            || window.rectangle[0] >= window.rectangle[2]
            || window.rectangle[1] >= window.rectangle[3]
            || !window.aspect_ratio.is_finite()
            || window.aspect_ratio <= 0.
        {
            return Err(fail(
                &path,
                "invalid model window rectangle, aspect ratio or identity",
            ));
        }
        validate_view(&window.view, WorkspaceViewKind::Model)
            .map_err(|e| fail(&path, &e.to_string()))?;
        validate_grid(&window.grid).map_err(|e| fail(&path, &e.to_string()))?;
        validate_snap(&window.snap).map_err(|e| fail(&path, &e.to_string()))?;
        choice(window.stored_ucs, &format!("{path}/storedUcs"))?;
    }
    if let Some(state) = &document.workspace_state {
        if state
            .current_layer_id
            .is_some_and(|id| !document.layers.iter().any(|v| v.id == id))
            || state.active_layout_id.is_some_and(|id| {
                id != document.model.id && !document.paper_layouts.iter().any(|v| v.id == id)
            })
        {
            return Err(fail(&prefix, "workspace layer/layout does not resolve"));
        }
    }
    if let Some(state) = &document.model_view_state {
        if let Some(current) = state.current_model_ucs {
            choice(current, &format!("{prefix}/currentModelUcs"))?;
        }
        if let Some(active) = state.active_model_window_id {
            let window = document
                .model_windows
                .iter()
                .find(|v| v.id == active)
                .ok_or_else(|| fail(&prefix, "active model window does not resolve"))?;
            if window.use_stored_ucs
                && state
                    .current_model_ucs
                    .is_some_and(|current| current != window.stored_ucs)
            {
                return Err(fail(
                    &prefix,
                    "active model window stored/current UCS conflict",
                ));
            }
        }
    }
    for paper in &document.paper_layouts {
        let path = format!("{prefix}/layout/{}", paper.id);
        if let Some(canvas) = &paper.canvas {
            validate_view(&canvas.view, WorkspaceViewKind::PaperCanvas)
                .map_err(|e| fail(&path, &e.to_string()))?;
            validate_grid(&canvas.grid).map_err(|e| fail(&path, &e.to_string()))?;
            validate_snap(&canvas.snap).map_err(|e| fail(&path, &e.to_string()))?;
            if let Some(frame) = &canvas.frame {
                validate_canvas_frame(frame).map_err(|e| fail(&path, &e.to_string()))?;
            }
            choice(canvas.stored_ucs, &format!("{path}/storedUcs"))?;
            if let Some(current) = canvas.current_ucs {
                choice(current, &format!("{path}/currentUcs"))?;
            }
            if canvas.current_ucs.is_some() && canvas.active_context.is_none() {
                return Err(fail(&path, "current UCS requires a known context"));
            }
            match canvas.active_context {
                Some(IfccadPaperContext::Canvas)
                    if canvas.use_stored_ucs
                        && canvas.current_ucs.is_some_and(|v| v != canvas.stored_ucs) =>
                {
                    return Err(fail(&path, "active canvas stored/current UCS conflict"))
                }
                Some(IfccadPaperContext::Viewport(id)) => {
                    let workspace = paper
                        .entities
                        .iter()
                        .filter_map(IfccadEntity::as_native)
                        .find_map(|entity| match &entity.kind {
                            IfccadEntityKind::Viewport(v) if entity.id == id => {
                                v.workspace.as_ref()
                            }
                            _ => None,
                        })
                        .ok_or_else(|| {
                            fail(&path, "active viewport needs a same-layout workspace")
                        })?;
                    if workspace.use_stored_ucs
                        && canvas
                            .current_ucs
                            .is_some_and(|v| v != workspace.stored_ucs)
                    {
                        return Err(fail(&path, "active viewport stored/current UCS conflict"));
                    }
                }
                _ => {}
            }
        }
        for entity in paper.entities.iter().filter_map(IfccadEntity::as_native) {
            if let IfccadEntityKind::Viewport(viewport) = &entity.kind {
                if let Some(workspace) = &viewport.workspace {
                    let path = format!("{prefix}/e{}", entity.id);
                    validate_grid(&workspace.grid).map_err(|e| fail(&path, &e.to_string()))?;
                    validate_snap(&workspace.snap).map_err(|e| fail(&path, &e.to_string()))?;
                    choice(workspace.stored_ucs, &format!("{path}/storedUcs"))?;
                }
            }
        }
    }
    Ok(())
}

pub type IfccadWorkspaceView = WorkspaceView;
pub type IfccadWorkspaceGrid = WorkspaceGrid;
pub type IfccadWorkspaceSnap = WorkspaceSnap;
pub type IfccadWorkspaceCanvasFrame = WorkspaceCanvasFrame;
