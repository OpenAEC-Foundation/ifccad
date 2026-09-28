use super::access::IfcdrResourceAccess;
use super::diagnostic::{
    IfcdrDiagnostic, IFCCAD_IFCDR_REFERENCE_MISSING, IFCCAD_IFCDR_WORKSPACE_INVALID,
};
use super::types::{IfcdrScopeKind, ProjectionMode, ViewDefinition, ViewportRenderMode};
use super::validation::{diagnostic, IfcdrEntityKind, IfcdrEvidence};
use crate::ifcdr::names::name_key;
use crate::ifcdr::{CoordinateFrame3, Point2};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UcsSelection {
    World,
    Named { ucs_id: u32 },
    Unnamed { frame: CoordinateFrame3 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct UcsDefinition {
    pub ucs_id: u32,
    pub name: String,
    pub frame: CoordinateFrame3,
    pub elevation: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceGridStyle {
    Lines,
    Dots,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorkspaceGrid {
    pub enabled: bool,
    pub spacing: Point2,
    pub style: WorkspaceGridStyle,
    pub major_line_frequency: u32,
    pub beyond_limits: bool,
    pub adaptive: bool,
    pub subdivision: bool,
    pub follows_workplane: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceSnapStyle {
    Rectangular,
    Isometric,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IsometricPlane {
    Left,
    Top,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorkspaceSnap {
    pub enabled: bool,
    pub base: Point2,
    pub spacing: Point2,
    pub angle: f64,
    pub style: WorkspaceSnapStyle,
    pub isometric_plane: IsometricPlane,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NormalizedRect2 {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawingViewState {
    pub current_model_ucs: UcsSelection,
    pub active_model_window_id: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModelWindow {
    pub model_window_id: u32,
    pub rectangle: NormalizedRect2,
    pub view: ViewDefinition,
    pub aspect_ratio: f64,
    pub render_mode: ViewportRenderMode,
    pub grid: WorkspaceGrid,
    pub snap: WorkspaceSnap,
    pub stored_ucs: UcsSelection,
    pub use_stored_ucs: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaperActiveContext {
    Canvas,
    Viewport { viewport_entity_id: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaperCanvas {
    pub scope_id: u32,
    pub view: ViewDefinition,
    pub grid: WorkspaceGrid,
    pub snap: WorkspaceSnap,
    pub stored_ucs: UcsSelection,
    pub current_ucs: UcsSelection,
    pub active_context: PaperActiveContext,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewportWorkspace {
    pub viewport_entity_id: u64,
    pub grid: WorkspaceGrid,
    pub snap: WorkspaceSnap,
    pub stored_ucs: UcsSelection,
    pub use_stored_ucs: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct IfcdrWorkspace {
    pub drawing_view_state: Option<DrawingViewState>,
    pub ucs_definitions: Vec<UcsDefinition>,
    pub model_windows: Vec<ModelWindow>,
    pub paper_canvases: Vec<PaperCanvas>,
    pub viewport_workspaces: Vec<ViewportWorkspace>,
}

fn valid_frame(frame: CoordinateFrame3) -> bool {
    CoordinateFrame3::try_new(frame.origin(), frame.x_axis(), frame.y_axis()).is_ok()
}

fn check_selection<R: IfcdrResourceAccess>(
    r: &R,
    errors: &mut Vec<IfcdrDiagnostic>,
    ids: &BTreeSet<u32>,
    choice: UcsSelection,
    collection: &'static str,
    row: Option<usize>,
    property: &'static str,
) {
    let problem = match choice {
        UcsSelection::World => None,
        UcsSelection::Named { ucs_id } if ids.contains(&ucs_id) => None,
        UcsSelection::Named { .. } => Some("missing named UCS"),
        UcsSelection::Unnamed { frame } if valid_frame(frame) => None,
        UcsSelection::Unnamed { .. } => Some("invalid unnamed UCS frame"),
    };
    if let Some(message) = problem {
        errors.push(diagnostic(
            r,
            IFCCAD_IFCDR_WORKSPACE_INVALID,
            collection,
            row,
            property,
            message,
        ));
    }
}

fn valid_grid(grid: WorkspaceGrid) -> bool {
    grid.spacing.x().is_finite()
        && grid.spacing.y().is_finite()
        && grid.spacing.x() >= 0.0
        && grid.spacing.y() >= 0.0
        && grid.major_line_frequency > 0
}

fn valid_snap(snap: WorkspaceSnap) -> bool {
    [
        snap.base.x(),
        snap.base.y(),
        snap.spacing.x(),
        snap.spacing.y(),
        snap.angle,
    ]
    .into_iter()
    .all(f64::is_finite)
        && snap.spacing.x() > 0.0
        && snap.spacing.y() > 0.0
}

fn valid_view(view: ViewDefinition, paper: bool) -> bool {
    let d = view.direction.components();
    let norm = d[0].hypot(d[1]).hypot(d[2]);
    let target = view.target.components();
    view.center.x().is_finite()
        && view.center.y().is_finite()
        && target.into_iter().all(f64::is_finite)
        && norm.is_finite()
        && norm > 0.0
        && view.height.is_finite()
        && view.height > 0.0
        && view.twist.is_finite()
        && (!paper || view.projection == ProjectionMode::Orthographic)
        && match (view.projection, view.lens_length) {
            (ProjectionMode::Orthographic, None) => true,
            (ProjectionMode::Orthographic, Some(v)) => v.is_finite() && v >= 0.0,
            (ProjectionMode::Perspective, Some(v)) => v.is_finite() && v > 0.0,
            _ => false,
        }
        && view.front_clip.distance.is_none_or(f64::is_finite)
        && view.back_clip.distance.is_none_or(f64::is_finite)
        && (view.front_clip.mode != super::types::FrontClipMode::AtDistance
            || view.front_clip.distance.is_some())
        && (view.back_clip.mode != super::types::BackClipMode::AtDistance
            || view.back_clip.distance.is_some())
}

pub(super) fn check_workspace<R: IfcdrResourceAccess>(
    r: &R,
    evidence: &IfcdrEvidence,
    errors: &mut Vec<IfcdrDiagnostic>,
) {
    let Some(state) = r.workspace() else { return };
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for (row, definition) in state.ucs_definitions.iter().enumerate() {
        if !ids.insert(definition.ucs_id) {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_WORKSPACE_INVALID,
                "ucsDefinition",
                Some(row),
                "ucsId",
                "duplicate UCS ID",
            ));
        }
        if definition.name.is_empty() || !names.insert(name_key(&definition.name)) {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_WORKSPACE_INVALID,
                "ucsDefinition",
                Some(row),
                "name",
                "empty or duplicate folded UCS name",
            ));
        }
        if !valid_frame(definition.frame) || !definition.elevation.is_finite() {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_WORKSPACE_INVALID,
                "ucsDefinition",
                Some(row),
                "frame",
                "invalid UCS frame or elevation",
            ));
        }
    }
    if state.drawing_view_state.is_none() && !state.model_windows.is_empty() {
        errors.push(diagnostic(
            r,
            IFCCAD_IFCDR_WORKSPACE_INVALID,
            "drawingViewState",
            None,
            "activeModelWindowId",
            "model windows require drawing view state",
        ));
    }
    let mut model_ids = BTreeSet::new();
    for (row, window) in state.model_windows.iter().enumerate() {
        if !model_ids.insert(window.model_window_id) {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_WORKSPACE_INVALID,
                "modelWindow",
                Some(row),
                "modelWindowId",
                "duplicate model window ID",
            ));
        }
        let rect = window.rectangle;
        if ![rect.min_x, rect.min_y, rect.max_x, rect.max_y]
            .into_iter()
            .all(|v| v.is_finite() && (0.0..=1.0).contains(&v))
            || rect.min_x >= rect.max_x
            || rect.min_y >= rect.max_y
        {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_WORKSPACE_INVALID,
                "modelWindow",
                Some(row),
                "rectangle",
                "invalid normalized rectangle",
            ));
        }
        if !valid_view(window.view, false)
            || !window.aspect_ratio.is_finite()
            || window.aspect_ratio <= 0.0
        {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_WORKSPACE_INVALID,
                "modelWindow",
                Some(row),
                "view",
                "invalid model view or aspect ratio",
            ));
        }
        if !valid_grid(window.grid) || !valid_snap(window.snap) {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_WORKSPACE_INVALID,
                "modelWindow",
                Some(row),
                "grid",
                "invalid grid or snap settings",
            ));
        }
        check_selection(
            r,
            errors,
            &ids,
            window.stored_ucs,
            "modelWindow",
            Some(row),
            "storedUcs",
        );
    }
    if let Some(view) = state.drawing_view_state {
        check_selection(
            r,
            errors,
            &ids,
            view.current_model_ucs,
            "drawingViewState",
            None,
            "currentModelUcs",
        );
        if !model_ids.contains(&view.active_model_window_id)
            || !r
                .scopes()
                .iter()
                .any(|s| s.kind == IfcdrScopeKind::ModelSpace)
        {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_REFERENCE_MISSING,
                "drawingViewState",
                None,
                "activeModelWindowId",
                "missing active model window or ModelSpace",
            ));
        } else if let Some(active) = state
            .model_windows
            .iter()
            .find(|w| w.model_window_id == view.active_model_window_id)
        {
            if active.use_stored_ucs && active.stored_ucs != view.current_model_ucs {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_WORKSPACE_INVALID,
                    "drawingViewState",
                    None,
                    "currentModelUcs",
                    "active model window UCS conflicts with current UCS",
                ));
            }
        }
    }
    let mut canvases = BTreeMap::new();
    for (row, canvas) in state.paper_canvases.iter().enumerate() {
        if canvases.insert(canvas.scope_id, row).is_some()
            || !r
                .scopes()
                .iter()
                .any(|s| s.id == canvas.scope_id && s.kind == IfcdrScopeKind::PaperSpace)
        {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_WORKSPACE_INVALID,
                "paperCanvas",
                Some(row),
                "scopeId",
                "duplicate or non-paper canvas scope",
            ));
        }
        if !valid_view(canvas.view, true) || !valid_grid(canvas.grid) || !valid_snap(canvas.snap) {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_WORKSPACE_INVALID,
                "paperCanvas",
                Some(row),
                "view",
                "invalid paper view, grid or snap",
            ));
        }
        check_selection(
            r,
            errors,
            &ids,
            canvas.stored_ucs,
            "paperCanvas",
            Some(row),
            "storedUcs",
        );
        check_selection(
            r,
            errors,
            &ids,
            canvas.current_ucs,
            "paperCanvas",
            Some(row),
            "currentUcs",
        );
        if canvas.active_context == PaperActiveContext::Canvas
            && canvas.stored_ucs != canvas.current_ucs
        {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_WORKSPACE_INVALID,
                "paperCanvas",
                Some(row),
                "currentUcs",
                "active canvas UCS conflicts with current UCS",
            ));
        }
    }
    let mut viewport_ids = BTreeSet::new();
    for (row, viewport) in state.viewport_workspaces.iter().enumerate() {
        let location = evidence.entities.get(&viewport.viewport_entity_id);
        if !viewport_ids.insert(viewport.viewport_entity_id)
            || !matches!(location, Some(loc) if matches!(loc.kind, IfcdrEntityKind::Viewport)
                && canvases.contains_key(&loc.scope))
        {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_REFERENCE_MISSING,
                "viewportWorkspace",
                Some(row),
                "viewportEntityId",
                "duplicate or missing paper viewport/canvas",
            ));
        }
        if !valid_grid(viewport.grid) || !valid_snap(viewport.snap) {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_WORKSPACE_INVALID,
                "viewportWorkspace",
                Some(row),
                "grid",
                "invalid grid or snap",
            ));
        }
        check_selection(
            r,
            errors,
            &ids,
            viewport.stored_ucs,
            "viewportWorkspace",
            Some(row),
            "storedUcs",
        );
    }
    for (row, canvas) in state.paper_canvases.iter().enumerate() {
        if let PaperActiveContext::Viewport { viewport_entity_id } = canvas.active_context {
            let matching = state
                .viewport_workspaces
                .iter()
                .find(|v| v.viewport_entity_id == viewport_entity_id);
            let in_scope = evidence
                .entities
                .get(&viewport_entity_id)
                .is_some_and(|loc| {
                    matches!(loc.kind, IfcdrEntityKind::Viewport) && loc.scope == canvas.scope_id
                });
            if !in_scope || matching.is_none() {
                errors.push(diagnostic(
                    r,
                    IFCCAD_IFCDR_REFERENCE_MISSING,
                    "paperCanvas",
                    Some(row),
                    "activeContext",
                    "active paper viewport or workspace row is missing",
                ));
            } else if let Some(viewport) = matching {
                if viewport.use_stored_ucs && viewport.stored_ucs != canvas.current_ucs {
                    errors.push(diagnostic(
                        r,
                        IFCCAD_IFCDR_WORKSPACE_INVALID,
                        "paperCanvas",
                        Some(row),
                        "currentUcs",
                        "active viewport UCS conflicts with current paper UCS",
                    ));
                }
            }
        }
    }
}
