mod access;
mod block_bounds;
mod blocks;
pub(crate) use blocks::BlockGraph;
mod diagnostic;
mod types;
mod validation;
mod viewport;
mod workspace;

pub(crate) use access::*;
pub(crate) use diagnostic::*;
pub(crate) use types::*;
pub use types::{AppearanceMode, IfcdrColor, IfcdrIndexedColor, IfcdrNamedColor};
pub use types::{
    BackClip, BackClipMode, FrontClip, FrontClipMode, PaperClip, ProjectionMode, ShadedPlot,
    ShadedPlotMode, ShadedPlotQuality, ShadedPlotQualityMode, ViewDefinition, ViewportFrame,
    ViewportLayerOverride, ViewportRenderMode,
};
pub(crate) use validation::*;
pub use workspace::{
    DrawingViewState, IfcdrWorkspace, IsometricPlane, ModelWindow, NormalizedRect2,
    PaperActiveContext, PaperCanvas, UcsDefinition, UcsSelection, ViewportWorkspace, WorkspaceGrid,
    WorkspaceGridStyle, WorkspaceSnap, WorkspaceSnapStyle,
};

#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
mod tests;
