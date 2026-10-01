//! Decoded drawing content independent of JSON field names and stream columns.

use super::{
    DrawingBlockDefinition, DrawingGeometricEntity, DrawingLayer, DrawingLayout,
    DrawingModelWindow, DrawingPaperCanvas, DrawingScope, DrawingUcsDefinition, DrawingViewState,
    DrawingViewport, DrawingViewportWorkspace, DrawingWorkspaceState,
};
use crate::ocdraw::{PlotStyleMode, PointDisplay};

#[derive(Clone, Debug)]
pub(crate) struct DrawingDocument {
    pub drawing_id: String,
    pub unit: String,
    pub plot_style_mode: PlotStyleMode,
    pub geometric_entities: Vec<DrawingGeometricEntity>,
    pub viewports: Vec<DrawingViewport>,
    pub line_patterns: Vec<super::DrawingLinePattern>,
    pub line_pattern_scale: f64,
    pub layers: Vec<DrawingLayer>,
    pub layouts: Vec<DrawingLayout>,
    pub ucs_definitions: Vec<DrawingUcsDefinition>,
    pub workspace_state: Option<DrawingWorkspaceState>,
    pub point_display: Option<PointDisplay>,
    pub block_definitions: Vec<DrawingBlockDefinition>,
    pub scopes: Vec<DrawingScope>,
    pub view_state: Option<DrawingViewState>,
    pub model_windows: Vec<DrawingModelWindow>,
    pub paper_canvases: Vec<DrawingPaperCanvas>,
    pub viewport_workspaces: Vec<DrawingViewportWorkspace>,
}
