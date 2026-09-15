//! Typed, read-only views over validated IFCDR drawing resources.

#![allow(dead_code)]

pub(crate) mod codec;
pub(crate) mod geometry;
pub(crate) mod logical;
mod read;
mod types;
pub(crate) mod write;

pub use geometry::{
    Bounds3d, CoordinateAxis, GeometryEvaluationError, PlaneAxis, PlanePlacement,
    PlanePlacementError, PlanePlacementField, Point3, Vector3,
};

pub(crate) use read::{validate_ifcdr, LoadedIfcdrResource, ValidatedIfcdrResource};
pub use read::{
    EntityIterator, IfcdrEntityRef, IfcdrResourceRef, Line, LocalPointIterator, PointIterator,
    PolylineRef, ScopePointIterator, ScopeRef,
};
pub use types::{AppearanceId, Bounds2d, EntityId, IfcdrLengthUnit, LayerId, Point2, ScopeId};
