//! Package construction and directory-writing implementation.
//!
//! Public writer types are re-exported by [`crate::package`]; this module is
//! private so callers do not encode the implementation direction in imports.

mod artifact;
mod builder;
mod error;
mod ifcx;
mod state;
mod types;

pub use artifact::EncodedPackage;
pub use builder::{
    DrawingAppearances, DrawingBuilder, DrawingLayers, ModelSpaceBuilder, PackageBuilder,
};
pub use error::{PackageBuildError, PackageWriteError};
pub use types::{
    AppearanceColor, AppearanceDefinition, AppearanceKey, AppearanceMode, DrawingOptions,
    EntityAppearance, LayerDefinition, LayerKey, LineDefinition, LinePatternDefinition,
    PackageOptions, PolylineDefinition,
};
