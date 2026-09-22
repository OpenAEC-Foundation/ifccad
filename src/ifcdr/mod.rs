//! Typed, read-only views over validated IFCDR drawing resources.

#![allow(dead_code)]

pub(crate) mod codec;
pub(crate) mod geometry;
pub(crate) mod logical;
mod read;
mod types;
pub(crate) mod write;

pub use geometry::{
    BlockTransform, BlockTransformError, Bounds3d, CoordinateAxis, GeometryEvaluationError,
    PlaneAxis, PlanePlacement, PlanePlacementError, PlanePlacementField, Point3, Scale3, Vector3,
};

pub(crate) use read::{validate_ifcdr, LoadedIfcdrResource, ValidatedIfcdrResource};
pub use read::{
    BlockDefinitionRef, BlockInstanceRef, EntityIterator, IfcdrEntityRef, IfcdrResourceRef, Line,
    LocalPointIterator, ModelSpaceRef, PaperSpaceRef, PointIterator, PolylineRef,
    ScopePointIterator, ScopeRef,
};
pub use types::{
    AppearanceId, BlockScaling, Bounds2d, EntityId, IfcdrLengthUnit, LayerId, Point2, ScopeId,
};
pub(crate) mod names;
