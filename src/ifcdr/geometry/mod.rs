pub(crate) mod numeric;
mod placement;
pub(crate) use placement::PlanePlacementComponents;
mod values;

pub use placement::{
    GeometryEvaluationError, PlaneAxis, PlanePlacement, PlanePlacementError, PlanePlacementField,
};
pub use values::{Bounds3d, CoordinateAxis, Point3, Vector3};
