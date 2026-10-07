use super::*;
mod allocation;
mod bounds;
mod document_validation;
mod geometry;
mod model;
pub(crate) mod patterns;
mod viewports;
pub use allocation::{IfccadIdAllocationError, IfccadIdCounters, IfccadIdDomain};
pub use document_validation::validate_ifccad_document;
pub use model::*;
pub use patterns::validate_ifccad_line_patterns;
pub use viewports::*;

pub use bounds::{
    derive_ifccad_geometry_completeness, recompute_ifccad_document_bounds,
    IfccadGeometryCompleteness, IfccadScopeId,
};

mod preservation;
pub use preservation::*;
mod preservation_validation;
