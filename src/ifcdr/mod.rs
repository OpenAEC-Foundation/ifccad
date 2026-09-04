//! Typed, read-only views over validated IFCDR drawing resources.

#![allow(dead_code)]

mod read;
mod types;

pub(crate) use read::{validate_ifcdr, LoadedIfcdrResource, ValidatedIfcdrResource};
pub use read::{
    EntityIterator, IfcdrEntityRef, IfcdrResourceRef, Line, PointIterator, PolylineRef, ScopeRef,
    UnmodeledEntityRef,
};
pub use types::{AppearanceId, Bounds2d, EntityId, IfcdrLengthUnit, LayerId, Point2, ScopeId};
