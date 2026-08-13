// The IFCDR model is intentionally crate-internal until package-wide validation
// and the first conversion experiments establish its stable consumer surface.
#![allow(dead_code)]

mod entity;
mod registry;
mod resource;
mod streams;
mod validation;

pub(crate) use resource::{LoadedIfcdrResource, ValidatedIfcdrResource};
pub(crate) use validation::validate_ifcdr;
