//! CAD view and workspace records independent of their JSON representation.

use crate::ocdraw::{CoordinateFrame3, Point2, Point3, Vector3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrawingUcsSelection {
    World,
    Named(u32),
    Unnamed(CoordinateFrame3),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawingProjection {
    Orthographic,
    Perspective,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawingClip {
    pub mode: DrawingClipMode,
    pub distance: Option<f64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawingClipMode {
    Disabled,
    AtCamera,
    AtDistance,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawingView {
    pub center: Point2,
    pub target: Point3,
    pub direction: Vector3,
    pub height: f64,
    pub twist: f64,
    pub projection: DrawingProjection,
    pub lens_length: Option<f64>,
    pub front_clip: DrawingClip,
    pub back_clip: DrawingClip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawingGridStyle {
    Lines,
    Dots,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawingGrid {
    pub enabled: bool,
    pub spacing: Point2,
    pub style: DrawingGridStyle,
    pub major_line_frequency: u32,
    pub beyond_limits: bool,
    pub adaptive: bool,
    pub subdivision: bool,
    pub follows_workplane: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawingSnapStyle {
    Rectangular,
    Isometric,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawingIsometricPlane {
    Left,
    Top,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawingSnap {
    pub enabled: bool,
    pub base: Point2,
    pub spacing: Point2,
    pub angle: f64,
    pub style: DrawingSnapStyle,
    pub isometric_plane: DrawingIsometricPlane,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawingViewState {
    pub current_model_ucs: DrawingUcsSelection,
    pub active_model_window_id: u32,
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
pub enum DrawingRenderMode {
    TwoDimensional,
    Wireframe,
    HiddenLine,
    FlatShadedWithoutEdges,
    FlatShadedWithEdges,
    SmoothShadedWithoutEdges,
    SmoothShadedWithEdges,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawingPaperContext {
    Canvas,
    Viewport(u64),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawingPaperCanvas {
    pub scope_id: u32,
    pub view: DrawingView,
    pub grid: DrawingGrid,
    pub snap: DrawingSnap,
    pub stored_ucs: DrawingUcsSelection,
    pub current_ucs: DrawingUcsSelection,
    pub active_context: DrawingPaperContext,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawingViewportWorkspace {
    pub viewport_entity_id: u64,
    pub grid: DrawingGrid,
    pub snap: DrawingSnap,
    pub stored_ucs: DrawingUcsSelection,
    pub use_stored_ucs: bool,
}
