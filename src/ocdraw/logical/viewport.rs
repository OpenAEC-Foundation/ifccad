//! Paper viewport records and their per-layer overrides.

use super::{DrawingColor, DrawingRenderMode, DrawingView, EntityAppearance};
use crate::ocdraw::{Point2, ShadedPlot};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawingViewportFrame {
    pub center: Point2,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawingPaperClip {
    pub enabled: bool,
    pub boundary_entity_id: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrawingViewportLayerOverride {
    pub layer_id: u32,
    pub frozen: bool,
    pub color: Option<DrawingColor>,
    pub opacity: Option<f64>,
    pub line_pattern: Option<String>,
    pub line_weight: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrawingViewport {
    pub id: u64,
    pub view_scope_id: u32,
    pub layer_id: u32,
    pub frame: DrawingViewportFrame,
    pub view: DrawingView,
    pub render_mode: DrawingRenderMode,
    pub view_enabled: bool,
    pub view_locked: bool,
    pub paper_clip: DrawingPaperClip,
    pub plot_shading_override: Option<ShadedPlot>,
    pub appearance: EntityAppearance,
    pub visible: bool,
    pub layer_overrides: Vec<DrawingViewportLayerOverride>,
}
