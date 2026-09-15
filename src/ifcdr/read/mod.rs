mod codes;
pub(crate) mod decoded;
mod entity;
mod resource;
mod streams;
mod validation;

pub use entity::{EntityIterator, IfcdrEntityRef};
pub use resource::{IfcdrResourceRef, ScopeRef};
pub(crate) use resource::{LoadedIfcdrResource, ValidatedIfcdrResource};
pub use streams::{Line, LocalPointIterator, PointIterator, PolylineRef, ScopePointIterator};
pub(crate) use validation::validate_ifcdr;
