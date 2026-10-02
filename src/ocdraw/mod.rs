//! Standalone Open CAD Drawing document access.
//!
//! Fresh construction may use [`DrawingBuilder::finish`] directly or expose the
//! complete logical document for editing before encoding:
//!
//! ```
//! use ocdraw::ocdraw::{DrawingBuilder, DrawingOptions, encode_document,
//!     recompute_document_bounds, validate_document};
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut document = DrawingBuilder::new(DrawingOptions::new("drawing", "mm"))?
//!     .build_document()?;
//! // Edit typed records here. Recompute only when new bounds are wanted.
//! recompute_document_bounds(&mut document)?;
//! validate_document(&document)?;
//! let encoded = encode_document(&document)?;
//! // encoded.write_file("new-drawing.ocdraw.json")?;
//! # let _ = encoded;
//! # Ok(())
//! # }
//! # example().unwrap();
//! ```
//!
//! Encoding retains valid supplied bounds and all four allocation watermarks.
//! Reader snapshots expose [`ValidatedDrawing::document`] and can be consumed
//! through [`ValidatedDrawing::into_document`] for mutation.

mod codec;
mod display;
pub(crate) mod geometry;
mod layout;
mod logical;
pub use logical::recompute_document_bounds;
pub use logical::OcdrawDocument;
pub use logical::{validate_document, OcdrawValidationError};
mod encode;
pub(crate) mod names;
mod plot;
mod read;
pub(crate) mod types;
mod workspace;
mod write;
pub use display::{PointDisplay, PointGlyph, PointSize};
pub use encode::{encode_document, OcdrawEncodeError};
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
    PlotArea, PlotMapping, PlotMedia, PlotOffsetReference, PlotOptions, PlotOutput, PlotPlacement,
    PlotRotation, PlotScale, PlotSettings, PlotUnit, ShadedPlot, ShadedPlotMode, ShadedPlotQuality,
    ShadedPlotQualityMode,
};
pub use types::{BlockScaling, Bounds2d, DrawingLengthUnit, EntityId, LayerId, Point2, ScopeId};
pub use workspace::UcsDefinition;
pub use write::{
    ArcDefinition, BlockDefinition, BlockInstanceDefinition, CircleDefinition, DrawingBuildError,
    DrawingBuilder, DrawingOptions, DrawingSavedState, DrawingWriteError, EllipseDefinition,
    EncodedDrawing, GeometricEntityDefinition, LayerDefinition, LineDefinition,
    PlanarPolylineDefinition, PlotStyleMode, PointDefinition, RgbColor, SpatialPolylineDefinition,
    ViewportDefinition,
};

pub use read::{
    load_drawing_bytes, load_drawing_file, DrawingDiagnostic, DrawingLoadOutcome,
    DrawingLoadStatus, DrawingOpenError, ValidatedDrawing,
};
