use crate::geometry_kernel::{Point2, Point3};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceSnapStyle {
    Rectangular,
    Isometric,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceIsometricPlane {
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
    pub isometric_plane: WorkspaceIsometricPlane,
}

/// Saved overall Paper canvas frame, independent of drawable geometry/bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorkspaceCanvasFrame {
    pub center: Point3,
    pub width: f64,
    pub height: f64,
}
