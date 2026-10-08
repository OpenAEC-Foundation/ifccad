use crate::geometry_kernel::{Point2, Point3, Vector3};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceProjection {
    Orthographic,
    Perspective,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceClipMode {
    Disabled,
    AtCamera,
    AtDistance,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorkspaceClip {
    pub mode: WorkspaceClipMode,
    pub distance: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorkspaceView {
    pub center: Point2,
    pub target: Point3,
    pub direction: Vector3,
    pub height: f64,
    pub twist: f64,
    pub projection: WorkspaceProjection,
    pub lens_length: Option<f64>,
    pub front_clip: WorkspaceClip,
    pub back_clip: WorkspaceClip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceRenderMode {
    TwoDimensional,
    Wireframe,
    HiddenLine,
    FlatShadedWithoutEdges,
    FlatShadedWithEdges,
    SmoothShadedWithoutEdges,
    SmoothShadedWithEdges,
}
