use crate::ifcdr::{IfcdrLengthUnit, Point2};
use crate::{PackageId, ResourceId};

#[derive(Clone, Debug)]
pub struct PackageOptions {
    pub package_id: PackageId,
    pub data_version: String,
    pub author: String,
    pub timestamp: String,
}

#[derive(Clone, Debug)]
pub struct DrawingOptions {
    pub model_layout_name: String,
    pub representation_resource_id: ResourceId,
    pub length_unit: IfcdrLengthUnit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexedColor {
    pub system: String,
    pub index: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedColor {
    pub catalog: String,
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppearanceColor {
    pub(crate) rgb: [u8; 3],
    pub(crate) indexed: Option<IndexedColor>,
    pub(crate) named: Option<NamedColor>,
}

impl AppearanceColor {
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            rgb: [red, green, blue],
            indexed: None,
            named: None,
        }
    }

    pub fn with_indexed(mut self, system: impl Into<String>, index: u32) -> Self {
        self.indexed = Some(IndexedColor {
            system: system.into(),
            index,
        });
        self
    }

    pub fn with_named(mut self, catalog: impl Into<String>, name: impl Into<String>) -> Self {
        self.named = Some(NamedColor {
            catalog: catalog.into(),
            name: name.into(),
        });
        self
    }
}

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntityAppearance {
    ByLayer,
    ByBlock,
    Explicit(AppearanceKey),
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
