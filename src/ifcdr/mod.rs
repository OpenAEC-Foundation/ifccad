//! Typed, read-only views over validated IFCDR drawing resources.

#![allow(dead_code)]

mod entity;
mod registry;
mod resource;
mod streams;
mod validation;

pub use entity::{EntityId, EntityIterator, IfcdrEntityRef, ScopeId, UnmodeledEntityRef};
pub use resource::{
    AppearanceId, Bounds2d, IfccadLengthUnit, IfcdrResourceRef, LayerId, Point2, ScopeRef,
};
pub(crate) use resource::{LoadedIfcdrResource, ValidatedIfcdrResource};
pub use streams::{Line, PointIterator, PolylineRef};
pub(crate) use validation::validate_ifcdr;
