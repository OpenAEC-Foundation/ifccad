//! Typed, read-only views over validated IFCDR drawing resources.

#![allow(dead_code)]

pub(crate) mod codec;
pub(crate) use crate::drawing::geometry;
pub(crate) mod logical;
mod read;
use crate::drawing::types;
pub(crate) mod write;

pub use geometry::{
    BlockTransform, BlockTransformError, Bounds3d, CoordinateAxis, CoordinateFrame3,
    CoordinateFrameError, CoordinateFrameField, GeometryEvaluationError, PlaneAxis, Point3, Scale3,
    Vector3,
};

pub(crate) use crate::drawing::names;
pub use logical::{
    BackClip, BackClipMode, FrontClip, FrontClipMode, PaperClip, ProjectionMode, ShadedPlot,
    ShadedPlotMode, ShadedPlotQuality, ShadedPlotQualityMode, ViewDefinition, ViewportFrame,
    ViewportLayerOverride, ViewportRenderMode,
};
pub use logical::{
    DrawingViewState, IfcdrWorkspace, IsometricPlane, ModelWindow, NormalizedRect2,
    PaperActiveContext, PaperCanvas, UcsDefinition, UcsSelection, ViewportWorkspace, WorkspaceGrid,
    WorkspaceGridStyle, WorkspaceSnap, WorkspaceSnapStyle,
};
pub use logical::{IfcdrColor, IfcdrIndexedColor, IfcdrNamedColor};
pub(crate) use read::{validate_ifcdr, LoadedIfcdrResource, ValidatedIfcdrResource};
pub use read::{
    AppearanceOverrideRef, ArcRef, BlockDefinitionRef, BlockInstanceRef, CircleRef, EllipseArcRef,
    EllipseRef, EntityIterator, IfcdrEntityRef, IfcdrResourceRef, Line, LocalPointIterator,
    ModelSpaceRef, PaperSpaceRef, PlanarPolylineRef, PointIterator, PointRef, ScopePointIterator,
    ScopeRef, SpatialPolylineRef, ViewportRef,
};
pub use types::{
    AppearanceId, BlockScaling, Bounds2d, EntityId, IfcdrLengthUnit, LayerId, Point2, ScopeId,
};
