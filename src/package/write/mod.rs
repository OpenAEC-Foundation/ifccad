//! Package construction and directory-writing implementation.
//!
//! Public writer types are re-exported by [`crate::package`]; this module is
//! private so callers do not encode the implementation direction in imports.

mod artifact;
mod builder;
mod error;
mod ifcx;
mod layout;
mod prepare;
mod state;
mod types;

pub use artifact::EncodedPackage;
pub use builder::{
    DrawingAppearances, DrawingBuilder, DrawingLayers, ModelSpaceBuilder, PackageBuilder,
    ScopeEntitiesBuilder,
};
pub use error::{PackageBuildError, PackageWriteError};
pub use layout::{
    LayoutSettings, PlotArea, PlotMapping, PlotMedia, PlotOffsetReference, PlotOptions, PlotOutput,
    PlotPlacement, PlotRect, PlotRotation, PlotScale, PlotSettings, PlotStyleMode, PlotUnit,
};
pub use types::{
    AppearanceColor, AppearanceDefinition, AppearanceKey, AppearanceMode, AppearancePatch,
    ArcDefinition, BlockDefinitionKey, BlockDefinitionOptions, BlockInstanceDefinition,
    CircleDefinition, DrawingOptions, DrawingResourceStorage, EllipseArcDefinition,
    EllipseDefinition, EntityAppearance, LayerDefinition, LayerKey, LineDefinition,
    LinePatternDefinition, PackageOptions, PaperSpaceKey, PlanarPolylineDefinition,
    PointDefinition, PointDisplay, PointGlyph, PointSize, SpatialPolylineDefinition,
    ViewportDefinition, ViewportLayerOverrideDefinition,
};
