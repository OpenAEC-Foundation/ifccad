//! Geometric values and predicates shared by independent drawing formats.
//! This module has no CAD-runtime or file-encoding dependency.

#[allow(dead_code)] // Preserve the existing internal block component construction helpers.
mod block;
mod bounds;
mod extrusion;
mod filled;
pub use filled::{filled_quadrilateral_surface_cover, FilledQuadrilateralSurfaceCover};
mod bulge;
mod circular;
pub(crate) mod clip_containment;
pub mod hatch;
pub(crate) mod numeric;
mod placement;
mod polyline_width;
mod polyline_width_bounds;
mod primitive;
pub use polyline_width::{
    PlanarPolylineWidths, PolylineVertexWidthOverride, PolylineWidthValidationError,
};
pub use polyline_width_bounds::{
    polyline_width_display_bounds, PolylineWidthBoundsReason, PolylineWidthDisplayBounds,
};
mod trig;
mod units;
mod validation;
mod values;

pub(crate) use block::PreparedBlockTransform;
pub use block::{BlockTransform, BlockTransformError, Scale3};
pub use bounds::{geometry_bounds, geometry_centerline_bounds, paper_frame_bounds};
pub(crate) use bulge::bulge_segment_bounds;
pub(crate) use circular::{circular_bounds, elliptic_bounds};
pub(crate) use placement::CoordinateFrameComponents;
pub use placement::{
    CoordinateFrame3, CoordinateFrameError, CoordinateFrameField, GeometryEvaluationError,
    PlaneAxis,
};
pub use primitive::{GeometryRef, OwnedGeometry, PaperFrame, PlanarVertices};
pub use units::CoordinateLengthUnit;
pub use validation::{validate_geometry, validate_paper_boundary, GeometryValidationError};
pub use values::{Bounds2d, Bounds3d, CoordinateAxis, Point2, Point3, Vector3};
