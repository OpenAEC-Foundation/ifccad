use crate::ifcdr::{IfcdrLengthUnit, PlanePlacement, Point2, Point3};
use crate::{PackageId, ResourceId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockDefinitionKey {
    pub(crate) builder_token: u64,
    pub(crate) local_id: u32,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaperSpaceKey {
    pub(crate) builder_token: u64,
    pub(crate) local_id: u32,
}

/// Definition metadata belongs to the local IFCDR resource, not an IFCX node.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockDefinitionOptions {
    pub name: String,
    pub base_point: Point3,
    pub description: String,
    pub anonymous: bool,
    pub insertion_unit: IfcdrLengthUnit,
    pub explodable: bool,
    pub scaling: crate::ifcdr::BlockScaling,
}
impl BlockDefinitionOptions {
    pub fn named(name: String) -> Self {
        Self {
            name,
            base_point: Point3::new(0., 0., 0.),
            description: String::new(),
            anonymous: false,
            insertion_unit: IfcdrLengthUnit::Unitless,
            explodable: true,
            scaling: crate::ifcdr::BlockScaling::Any,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct BlockInstanceDefinition {
    pub definition: BlockDefinitionKey,
    pub transform: crate::ifcdr::BlockTransform,
    pub layer: LayerKey,
    pub appearance: EntityAppearance,
    pub visible: bool,
}

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
    pub frozen: bool,
    pub locked: bool,
    pub plottable: bool,
    pub frozen_in_new_viewports: bool,
    pub description: Option<String>,
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
    pub start: Point3,
    pub end: Point3,
    pub layer: LayerKey,
    pub appearance: EntityAppearance,
    pub visible: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PolylineDefinition {
    pub points: Vec<Point2>,
    pub placement: PlanePlacement,
    pub closed: bool,
    pub layer: LayerKey,
    pub appearance: EntityAppearance,
    pub visible: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppearancePatch {
    pub color: Option<AppearanceColor>,
    pub opacity: Option<f64>,
    pub line_pattern: Option<LinePatternDefinition>,
    pub line_weight: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ViewportLayerOverrideDefinition {
    pub layer: LayerKey,
    pub frozen: bool,
    pub appearance: Option<AppearancePatch>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ViewportDefinition {
    pub frame: crate::ifcdr::ViewportFrame,
    pub view: crate::ifcdr::ViewDefinition,
    pub render_mode: crate::ifcdr::ViewportRenderMode,
    pub view_enabled: bool,
    pub view_locked: bool,
    pub paper_clip: crate::ifcdr::PaperClip,
    pub plot_shading_override: Option<crate::ifcdr::ShadedPlot>,
    pub layer: LayerKey,
    pub appearance: EntityAppearance,
    pub visible: bool,
    pub layer_overrides: Vec<ViewportLayerOverrideDefinition>,
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
