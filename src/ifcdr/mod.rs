//! Typed, read-only views over validated IFCDR drawing resources.

#![allow(dead_code)]

pub(crate) mod codec;
pub(crate) mod logical;
mod read;
mod types;
pub(crate) mod write;

pub(crate) use read::{validate_ifcdr, LoadedIfcdrResource, ValidatedIfcdrResource};
pub use read::{
    EntityIterator, IfcdrEntityRef, IfcdrResourceRef, Line, PointIterator, PolylineRef, ScopeRef,
};
pub use types::{AppearanceId, Bounds2d, EntityId, IfcdrLengthUnit, LayerId, Point2, ScopeId};
