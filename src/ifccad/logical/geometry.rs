//! The IFCCAD primitive adapter; ownership and file syntax remain independent.
use super::*;
use crate::geometry_kernel::{CoordinateFrame3, GeometryRef as G, PlanarVertices, Point3, Vector3};

impl IfccadPlacement {
    pub fn coordinate_frame(&self) -> Result<CoordinateFrame3, IfccadReport> {
        let o = self.origin;
        let x = self.x_axis;
        let y = self.y_axis;
        CoordinateFrame3::try_new(
            Point3::new(o[0], o[1], o[2]),
            Vector3::new(x[0], x[1], x[2]),
            Vector3::new(y[0], y[1], y[2]),
        )
        .map_err(|e| IfccadReport::one(format!("invalid placement: {e}")))
    }
}
impl IfccadEntityKind {
    /// Borrows primitive geometry without copying entity identity or file fields.
    pub fn as_shared_geometry(&self) -> Result<Option<G<'_>>, IfccadReport> {
        Ok(Some(match self {
            Self::LineSegment { start, end } => G::Line {
                start: *start,
                end: *end,
            },
            Self::Point { placement } => G::Point {
                placement: placement.coordinate_frame()?,
            },
            Self::Circle { placement, radius } => G::Circle {
                placement: placement.coordinate_frame()?,
                radius: *radius,
            },
            Self::Arc {
                placement,
                radius,
                start_parameter,
                sweep_parameter,
            } => G::Arc {
                placement: placement.coordinate_frame()?,
                radius: *radius,
                start: *start_parameter,
                sweep: *sweep_parameter,
            },
            Self::Ellipse {
                placement,
                semi_major_radius,
                semi_minor_radius,
            } => G::Ellipse {
                placement: placement.coordinate_frame()?,
                major: *semi_major_radius,
                minor: *semi_minor_radius,
                arc: None,
            },
            Self::EllipseArc {
                placement,
                semi_major_radius,
                semi_minor_radius,
                start_parameter,
                sweep_parameter,
            } => G::Ellipse {
                placement: placement.coordinate_frame()?,
                major: *semi_major_radius,
                minor: *semi_minor_radius,
                arc: Some((*start_parameter, *sweep_parameter)),
            },
            Self::PlanarPolyline {
                placement,
                vertices,
                bulges,
                closed,
                ..
            } => G::PlanarPolyline {
                placement: placement.coordinate_frame()?,
                vertices: PlanarVertices::Separate {
                    xy: vertices,
                    bulges,
                },
                closed: *closed,
            },
            Self::SpatialPolyline {
                vertices, closed, ..
            } => G::SpatialPolyline {
                vertices,
                closed: *closed,
            },
            Self::BlockInstance { .. } | Self::Viewport(_) => return Ok(None),
        }))
    }
}
