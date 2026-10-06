//! Compatibility reexports for the shared format-neutral geometry kernel.
pub(crate) use crate::geometry_kernel::{
    bulge_segment_bounds, circular_bounds, elliptic_bounds, numeric, PreparedBlockTransform,
};
pub use crate::geometry_kernel::{
    BlockTransform, BlockTransformError, Bounds3d, CoordinateAxis, CoordinateFrame3,
    CoordinateFrameError, CoordinateFrameField, GeometryEvaluationError, PlaneAxis, Point3, Scale3,
    Vector3,
};
