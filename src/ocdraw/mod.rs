//! Standalone Open CAD Drawing document access.

mod codec;
mod display;
pub(crate) mod geometry;
mod layout;
mod logical;
pub(crate) mod names;
mod plot;
mod read;
pub(crate) mod types;
mod workspace;
mod write;
pub use display::{PointDisplay, PointGlyph, PointSize};
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
