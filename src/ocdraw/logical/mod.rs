mod block_validation;
pub(crate) use block_validation::validate_blocks;
mod line_pattern;
pub(crate) mod line_pattern_validation;
pub use line_pattern::*;
mod appearance;
mod bounds_preparation;
mod definitions;
mod document;
mod document_projection;
mod document_validation;
mod field_validation;
pub use bounds_preparation::recompute_ocdraw_document_bounds;
pub(crate) use document_validation::{validate_logical_document, ValidationPhase};
pub use document_validation::{validate_ocdraw_document, OcdrawValidationError};
mod geometry;
mod geometry_validation;
mod model;
mod validation;
mod view_state;
mod viewport;

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
