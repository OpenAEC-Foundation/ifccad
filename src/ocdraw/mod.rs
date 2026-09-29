//! Standalone Open CAD Drawing document access.

mod display;
pub(crate) mod geometry;
mod layout;
mod logical;
pub(crate) mod names;
mod plot;
mod read;
pub(crate) mod types;
mod validate;
mod workspace;
mod write;
pub use display::{PointDisplay, PointGlyph, PointSize};
pub use geometry::{
    BlockTransform, BlockTransformError, Bounds3d, CoordinateAxis, CoordinateFrame3,
    CoordinateFrameError, CoordinateFrameField, GeometryEvaluationError, PlaneAxis, Point3, Scale3,
    Vector3,
};
pub use layout::{LayoutRect, LayoutSettings};
pub use plot::{
    PlotArea, PlotMapping, PlotMedia, PlotOffsetReference, PlotOptions, PlotOutput, PlotPlacement,
    PlotRotation, PlotScale, PlotSettings, PlotUnit, ShadedPlot, ShadedPlotMode, ShadedPlotQuality,
    ShadedPlotQualityMode,
};
pub use types::{
    AppearanceId, BlockScaling, Bounds2d, EntityId, IfcdrLengthUnit, LayerId, Point2, ScopeId,
};
pub use workspace::UcsDefinition;
pub use write::{
    AppearanceSelection, ArcDefinition, BlockDefinition, BlockInstanceDefinition, CircleDefinition,
    DrawingBuildError, DrawingBuilder, DrawingColor, DrawingOptions, DrawingWriteError,
    EllipseDefinition, EncodedDrawing, EntityAppearance, LayerDefinition, LineDefinition,
    PlanarPolylineDefinition, PlotStyleMode, PointDefinition, RgbColor, SpatialPolylineDefinition,
};

pub use read::{
    load_drawing_bytes, load_drawing_file, DrawingDiagnostic, DrawingLoadOutcome,
    DrawingLoadStatus, DrawingOpenError, ValidatedDrawing,
};
