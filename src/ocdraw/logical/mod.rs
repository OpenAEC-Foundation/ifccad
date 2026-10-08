mod block_validation;
pub(crate) mod hatch;
mod hatch_validation;
pub(crate) use block_validation::validate_blocks;
pub use hatch::{resolve_ocdraw_hatch_join_tolerance, DrawingHatchEntity, OcdrawHatchLoop};
mod line_pattern;
pub(crate) mod line_pattern_validation;
pub use line_pattern::*;
mod appearance;
mod bounds_preparation;
mod definitions;
mod document;
mod text;
mod text_bounds;
mod text_validation;
pub use text::*;
pub use text_bounds::{OcdrawBoundsQuality, OcdrawScopeBoundsAssessment};
pub(crate) use text_validation::validate_text_parts;
mod opaque;
mod preservation;
mod preservation_validation;
pub use opaque::DrawingOpaqueEntity;
pub use preservation::*;
mod document_projection;
mod document_validation;
mod field_validation;
pub use bounds_preparation::assess_ocdraw_document_bounds;
pub use bounds_preparation::recompute_ocdraw_document_bounds;
pub(crate) use document_validation::{validate_logical_document, ValidationPhase};
pub use document_validation::{validate_ocdraw_document, OcdrawValidationError};
mod geometry;
mod geometry_completeness;
mod geometry_validation;
pub(crate) use geometry_completeness::{
    derive_scope_geometry_completeness, ScopeGeometryCompleteness,
};
mod model;
mod validation;
mod view_state;
mod viewport;
mod viewport_clip;
pub use viewport_clip::validate_viewport_clip_boundary;

pub use appearance::{AppearanceSelection, DrawingColor, EntityAppearance};
pub use definitions::{
    DrawingBlockDefinition, DrawingLayer, DrawingLayout, DrawingLayoutKind, DrawingScope,
    DrawingScopeKind, DrawingUcsDefinition, DrawingWorkspaceState,
};
pub use document::OcdrawDocument;
pub(crate) use geometry::DrawingEntityRecord;
pub use geometry::{DrawingGeometricEntity, EntityGeometry};
pub(crate) use geometry_validation::{
    enclosure, validate_geometry_bounds, validate_viewport_bounds,
};
pub(crate) use model::*;
pub use view_state::{
    DrawingClip, DrawingClipMode, DrawingGrid, DrawingGridStyle, DrawingIsometricPlane,
    DrawingModelWindow, DrawingPaperCanvas, DrawingPaperContext, DrawingProjection,
    DrawingRenderMode, DrawingSnap, DrawingSnapStyle, DrawingUcsSelection, DrawingView,
    DrawingViewState, DrawingViewportWorkspace,
};
pub use viewport::{
    DrawingPaperClip, DrawingViewport, DrawingViewportFrame, DrawingViewportLayerOverride,
};

pub(crate) use definitions::owner_index;

mod state_validation;
pub(crate) use geometry_validation::viewport_bounds;
pub(crate) use state_validation::validate_state;
