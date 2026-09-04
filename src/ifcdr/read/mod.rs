mod codes;
mod entity;
mod registry;
mod resource;
mod streams;
mod validation;

pub use entity::{EntityIterator, IfcdrEntityRef, UnmodeledEntityRef};
pub use resource::{IfcdrResourceRef, ScopeRef};
pub(crate) use resource::{LoadedIfcdrResource, ValidatedIfcdrResource};
pub use streams::{Line, PointIterator, PolylineRef};
pub(crate) use validation::validate_ifcdr;
