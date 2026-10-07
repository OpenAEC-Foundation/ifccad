//! Standalone Open CAD Drawing document access.
//!
//! Fresh construction may use [`OcdrawBuilder::finish`] directly or expose the
//! complete logical document for editing before encoding:
//!
//! ```
//! use ocdraw::ocdraw::{OcdrawBuilder, OcdrawBuildOptions, encode_ocdraw_document,
//!     recompute_ocdraw_document_bounds, validate_ocdraw_document};
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut document = OcdrawBuilder::new(OcdrawBuildOptions::new("drawing", "mm"))?
//!     .build_document()?;
//! // Edit typed records here. Recompute only when new bounds are wanted.
//! recompute_ocdraw_document_bounds(&mut document)?;
//! validate_ocdraw_document(&document)?;
//! let encoded = encode_ocdraw_document(&document)?;
//! // encoded.write_file("new-drawing.ocdraw.json")?;
//! # let _ = encoded;
//! # Ok(())
//! # }
//! # example().unwrap();
//! ```
//!
//! Encoding retains valid supplied bounds and all four allocation watermarks.
//! Reader snapshots expose [`ValidatedOcdraw::document`] and can be consumed
//! through [`ValidatedOcdraw::into_document`] for mutation.

mod codec;
mod display;
pub(crate) mod geometry;
mod layout;
mod logical;
pub use logical::recompute_ocdraw_document_bounds;
pub use logical::validate_viewport_clip_boundary;
pub use logical::OcdrawDocument;
pub use logical::{validate_ocdraw_document, OcdrawValidationError};
pub use logical::{
    DrawingOpaqueEntity, OcdrawPreservation, OcdrawPreservationAllocationError,
    OcdrawPreservationBinding, OcdrawPreservationCategory, OcdrawPreservationCondition,
    OcdrawPreservationDependencyCoverage, OcdrawPreservationOrigin, OcdrawPreservationPayload,
    OcdrawPreservationPayloadKind, OcdrawPreservationRecord, OcdrawPreservationRecordId,
    OcdrawPreservationRepresentation, OcdrawPreservationRole, OcdrawPreservationSource,
    OcdrawPreservationTarget,
};
mod build;
mod encode;
pub(crate) mod names;
mod plot;
mod read;
mod storage;
pub(crate) mod types;
mod workspace;
pub use build::{
    ArcDefinition, BlockDefinition, BlockInstanceDefinition, CircleDefinition, DrawingSavedState,
    EllipseDefinition, GeometricEntityDefinition, LayerDefinition, LineDefinition,
    OcdrawBuildError, OcdrawBuildOptions, OcdrawBuilder, OpaqueEntityDefinition,
    PlanarPolylineDefinition, PlotStyleMode, PointDefinition, RgbColor, SpatialPolylineDefinition,
    ViewportDefinition,
};
pub use display::{PointDisplay, PointGlyph, PointSize};
pub use encode::{encode_ocdraw_document, OcdrawEncodeError};
pub use geometry::{
    BlockTransform, BlockTransformError, Bounds3d, CoordinateAxis, CoordinateFrame3,
    CoordinateFrameError, CoordinateFrameField, GeometryEvaluationError, PlaneAxis, Point3, Scale3,
    Vector3,
};
pub use layout::{LayoutRect, LayoutSettings};
pub use logical::{AppearanceSelection, DrawingColor, EntityAppearance};
pub use logical::{
    DrawingBlockDefinition, DrawingLayer, DrawingLayout, DrawingLayoutKind, DrawingScope,
    DrawingScopeKind, DrawingUcsDefinition, DrawingWorkspaceState,
};
pub use logical::{
    DrawingClip, DrawingClipMode, DrawingGrid, DrawingGridStyle, DrawingIsometricPlane,
    DrawingModelWindow, DrawingPaperCanvas, DrawingPaperContext, DrawingProjection,
    DrawingRenderMode, DrawingSnap, DrawingSnapStyle, DrawingUcsSelection, DrawingView,
    DrawingViewState, DrawingViewportWorkspace,
};
pub use logical::{DrawingGeometricEntity, EntityGeometry as DrawingGeometry};
pub use logical::{
    DrawingLinePattern, LinePatternDefinition, LinePatternGeneration, LinePatternId,
};
pub use logical::{
    DrawingPaperClip, DrawingViewport, DrawingViewportFrame, DrawingViewportLayerOverride,
};
pub use plot::{
    LayoutMedia, MediaUnit, PlotArea, PlotMapping, PlotOffsetReference, PlotOptions, PlotOutput,
    PlotPage, PlotPlacement, PlotRotation, PlotScale, PlotSettings, PlotUnit, ShadedPlot,
    ShadedPlotMode, ShadedPlotQuality, ShadedPlotQualityMode,
};
pub use types::{BlockScaling, Bounds2d, DrawingLengthUnit, EntityId, LayerId, Point2, ScopeId};
pub use workspace::UcsDefinition;

pub use read::{
    load_ocdraw_bytes, OcdrawDiagnostic, OcdrawReadError, OcdrawReadStatus, ValidatedOcdraw,
};

pub use storage::{load_ocdraw_file, EncodedOcdraw, OcdrawOpenError, OcdrawWriteError};
