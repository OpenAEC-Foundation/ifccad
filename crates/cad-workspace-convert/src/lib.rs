//! Shared CAD workspace field adaptation; native identities belong to callers.
mod diagnostics;
mod read;
mod write;
pub use diagnostics::*;
use ocdraw::{geometry_kernel::CoordinateFrame3, workspace_kernel::*};
pub use read::*;
pub use write::*;

#[derive(Clone, Debug, PartialEq)]
pub struct PreparedWorkspace<T> {
    pub value: T,
    pub losses: Vec<WorkspaceFieldLoss>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ViewportAidValues {
    pub grid: WorkspaceGrid,
    pub snap: WorkspaceSnap,
    pub ucs_frame: CoordinateFrame3,
    pub ucs_elevation: f64,
    pub use_stored_ucs: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ModelWindowValues {
    pub rectangle: [f64; 4],
    pub view: WorkspaceView,
    pub aspect_ratio: f64,
    pub render_mode: WorkspaceRenderMode,
    pub aids: ViewportAidValues,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PaperCanvasValues {
    pub frame: Option<WorkspaceCanvasFrame>,
    pub view: WorkspaceView,
    pub aids: ViewportAidValues,
}
