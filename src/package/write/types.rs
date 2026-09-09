use crate::ifcdr::{IfcdrLengthUnit, Point2};
use crate::{PackageId, ResourceId};

#[derive(Clone, Debug)]
/// Package-level identity and provenance written to the IFCX header.
pub struct PackageOptions {
    pub package_id: PackageId,
    pub data_version: String,
    pub author: String,
    pub timestamp: String,
}

#[derive(Clone, Debug)]
/// Configuration of the package's drawing resource.
pub struct DrawingOptions {
    /// Name of the model layout inside the drawing, not a drawing name.
    pub model_layout_name: String,
    /// Stable identifier assigned to the IFCDR resource, independent of storage.
    pub representation_resource_id: ResourceId,
    /// Coordinate length unit declared by that IFCDR resource.
    pub length_unit: IfcdrLengthUnit,
}

pub use crate::ifcdr::logical::IfcdrColor as AppearanceColor;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinePatternDefinition {
    pub(crate) name: String,
}

impl LinePatternDefinition {
    pub fn named(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppearanceDefinition {
    pub name: String,
    pub color: AppearanceColor,
    pub opacity: f64,
    pub line_pattern: LinePatternDefinition,
    pub line_weight: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AppearanceKey {
    pub(crate) builder_token: u64,
    pub(crate) local_id: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerDefinition {
    pub name: String,
    pub visible: bool,
    pub appearance: AppearanceKey,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayerKey {
    pub(crate) builder_token: u64,
    pub(crate) local_id: u32,
}

pub use crate::ifcdr::logical::AppearanceMode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntityAppearance {
    pub appearance: Option<AppearanceKey>,
    pub color_mode: AppearanceMode,
    pub opacity_mode: AppearanceMode,
    pub line_pattern_mode: AppearanceMode,
    pub line_weight_mode: AppearanceMode,
}

impl EntityAppearance {
    pub const fn by_layer() -> Self {
        Self {
            appearance: None,
            color_mode: AppearanceMode::ByLayer,
            opacity_mode: AppearanceMode::ByLayer,
            line_pattern_mode: AppearanceMode::ByLayer,
            line_weight_mode: AppearanceMode::ByLayer,
        }
    }

    pub const fn by_block() -> Self {
        Self {
            appearance: None,
            color_mode: AppearanceMode::ByBlock,
            opacity_mode: AppearanceMode::ByBlock,
            line_pattern_mode: AppearanceMode::ByBlock,
            line_weight_mode: AppearanceMode::ByBlock,
        }
    }

    pub const fn explicit(appearance: AppearanceKey) -> Self {
        Self {
            appearance: Some(appearance),
            color_mode: AppearanceMode::Explicit,
            opacity_mode: AppearanceMode::Explicit,
            line_pattern_mode: AppearanceMode::Explicit,
            line_weight_mode: AppearanceMode::Explicit,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LineDefinition {
    pub start: Point2,
    pub end: Point2,
    pub layer: LayerKey,
    pub appearance: EntityAppearance,
    pub visible: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PolylineDefinition {
    pub points: Vec<Point2>,
    pub closed: bool,
    pub layer: LayerKey,
    pub appearance: EntityAppearance,
    pub visible: bool,
}

/// Storage form of a drawing resource in an encoded package.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DrawingResourceStorage {
    /// Write a separate IFCDR file with an exact-byte checksum.
    #[default]
    External,
    /// Embed the IFCDR JSON object in the IFCX resource descriptor.
    Inline,
}
