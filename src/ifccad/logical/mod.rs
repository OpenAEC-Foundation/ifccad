use super::*;
mod allocation;
mod bounds;
mod document_validation;
mod geometry;
mod model;
mod text;
pub use text::{IfccadMText, IfccadText, IfccadTextStyle, IfccadTextStyleId};
pub(crate) mod patterns;
mod viewports;
pub use allocation::{IfccadIdAllocationError, IfccadIdCounters, IfccadIdDomain};
pub use document_validation::validate_ifccad_document;
pub use model::*;
pub use patterns::validate_ifccad_line_patterns;
pub use viewports::*;

pub use bounds::{
    assess_ifccad_document_bounds, derive_ifccad_geometry_completeness,
    recompute_ifccad_document_bounds, IfccadBoundsAssessment, IfccadBoundsQuality,
    IfccadGeometryCompleteness, IfccadScopeBoundsAssessment, IfccadScopeId,
};

mod preservation;
pub use preservation::*;
mod preservation_validation;
mod workspace;
pub use workspace::*;
