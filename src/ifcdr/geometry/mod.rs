mod block;
pub(crate) mod numeric;
mod placement;
mod trig;
pub(crate) use placement::PlanePlacementComponents;
mod values;

pub use block::{BlockTransform, BlockTransformError, Scale3};
pub(crate) use block::{BlockTransformComponents, PreparedBlockTransform};

pub use placement::{
    GeometryEvaluationError, PlaneAxis, PlanePlacement, PlanePlacementError, PlanePlacementField,
};
pub use values::{Bounds3d, CoordinateAxis, Point3, Vector3};
