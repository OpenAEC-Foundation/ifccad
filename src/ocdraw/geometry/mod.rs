#![allow(dead_code)]

mod block;
mod bulge;
mod circular;
pub(crate) mod numeric;
mod placement;
mod trig;
pub(crate) use bulge::bulge_segment_bounds;
pub(crate) use circular::{circular_bounds, elliptic_bounds};
pub(crate) use placement::CoordinateFrameComponents;
mod values;

pub(crate) use block::PreparedBlockTransform;
pub use block::{BlockTransform, BlockTransformError, Scale3};

pub use placement::{
    CoordinateFrame3, CoordinateFrameError, CoordinateFrameField, GeometryEvaluationError,
    PlaneAxis,
};
pub use values::{Bounds3d, CoordinateAxis, Point3, Vector3};
