use super::*;
pub use crate::geometry_kernel::hatch::{
    HatchBoundaryConstruction, HatchClassicFill, HatchClassicSettings, HatchCreationType,
    HatchLoopProvenance,
};
pub use crate::geometry_kernel::{
    PlanarPolylineWidths as IfccadPlanarPolylineWidths,
    PolylineVertexWidthOverride as IfccadPolylineVertexWidthOverride,
};
mod allocation;
mod annotation;
mod annotation_policy;
pub use annotation_policy::*;
mod annotation_dimension;
pub use annotation::*;
pub use annotation_dimension::*;
mod annotation_views;
pub use annotation_views::*;
mod dimension;
mod dimension_graphics;
mod leader_graphics;
pub use leader_graphics::*;
mod leader;
pub use leader::*;
mod leader_validation;
mod multi_leader;
pub use multi_leader::*;
mod multi_leader_content;
pub use multi_leader_content::*;
mod multi_leader_settings;
pub use multi_leader_settings::*;
mod dimension_graphics_validation;
pub use dimension_graphics::*;
mod dimension_validation;
pub use dimension::*;
mod bounds;
mod document_validation;
mod geometry;
mod spline;
pub use spline::*;
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
    recompute_ifccad_document_bounds, IfccadAnnotationBoundsIssue, IfccadAnnotationBoundsReason,
    IfccadBoundsAssessment, IfccadBoundsQuality, IfccadGeometryCompleteness,
    IfccadPolylineWidthBoundsIssue, IfccadScopeBoundsAssessment, IfccadScopeId,
};

mod preservation;
pub use preservation::*;
mod preservation_validation;
mod workspace;
pub use workspace::*;

mod annotation_text;
pub use annotation_text::*;

mod annotation_hatch;

mod attributes;
pub use attributes::*;
mod annotation_attributes;
pub use annotation_attributes::*;
mod attribute_fields;
pub use attribute_fields::*;
