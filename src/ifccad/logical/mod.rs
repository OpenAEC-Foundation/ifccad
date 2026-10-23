use super::*;
pub use crate::geometry_kernel::hatch::{
    HatchBoundaryConstruction, HatchClassicFill, HatchClassicSettings, HatchCreationType,
    HatchLoopProvenance,
};
mod allocation;
mod dimension;
mod dimension_graphics;
mod dimension_graphics_validation;
pub use dimension_graphics::*;
mod dimension_validation;
pub use dimension::*;
mod bounds;
mod document_validation;
mod geometry;
mod hatch;
pub(crate) mod line_pattern_bounds;
pub use hatch::{
    resolve_ifccad_hatch_join_tolerance, IfccadHatch, IfccadHatchGradient, IfccadHatchLoop,
};
mod model;
mod symbol_source;
pub use symbol_source::*;
mod presentation;
pub use presentation::{
    IfccadColor, IfccadColorMetadataError, IfccadPointDisplay, IfccadPointGlyph, IfccadPointSize,
};
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
