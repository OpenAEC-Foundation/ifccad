//! CAD view and workspace records independent of their JSON representation.

use crate::ocdraw::CoordinateFrame3;
pub use crate::workspace_kernel::{
    WorkspaceClip as DrawingClip, WorkspaceClipMode as DrawingClipMode,
    WorkspaceGrid as DrawingGrid, WorkspaceGridStyle as DrawingGridStyle,
    WorkspaceIsometricPlane as DrawingIsometricPlane, WorkspaceProjection as DrawingProjection,
    WorkspaceRenderMode as DrawingRenderMode, WorkspaceSnap as DrawingSnap,
    WorkspaceSnapStyle as DrawingSnapStyle, WorkspaceView as DrawingView,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrawingUcsSelection {
    World,
    Named(u32),
    Unnamed(CoordinateFrame3),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawingViewState {
    pub current_model_ucs: Option<DrawingUcsSelection>,
    pub active_model_window_id: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawingModelWindow {
    pub id: u32,
    pub rectangle: [f64; 4],
    pub view: DrawingView,
    pub aspect_ratio: f64,
    pub render_mode: DrawingRenderMode,
    pub grid: DrawingGrid,
    pub snap: DrawingSnap,
    pub stored_ucs: DrawingUcsSelection,
    pub use_stored_ucs: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawingPaperContext {
    Canvas,
    Viewport(u64),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawingPaperCanvas {
    pub frame: Option<crate::workspace_kernel::WorkspaceCanvasFrame>,
    pub use_stored_ucs: bool,
    pub scope_id: u32,
    pub view: DrawingView,
    pub grid: DrawingGrid,
    pub snap: DrawingSnap,
    pub stored_ucs: DrawingUcsSelection,
    pub current_ucs: Option<DrawingUcsSelection>,
    pub active_context: Option<DrawingPaperContext>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawingViewportWorkspace {
    pub viewport_entity_id: u64,
    pub grid: DrawingGrid,
    pub snap: DrawingSnap,
    pub stored_ucs: DrawingUcsSelection,
    pub use_stored_ucs: bool,
}
