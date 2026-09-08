use crate::ifcdr::Point2;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct IfcdrScope {
    pub id: u32,
    pub kind: u32,
    pub name: String,
    pub base: Point2,
    pub flags: u32,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct IfcdrLayerBinding {
    pub id: u32,
    pub ifcx_layer: String,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct IfcdrAppearanceBinding {
    pub id: u32,
    pub ifcx_appearance: Option<String>,
    pub modes: [u32; 4],
    pub override_id: Option<u32>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct IfcdrAppearanceOverride {
    pub id: u32,
    pub color: Option<IfcdrColor>,
    pub opacity: Option<f64>,
    pub ifcx_line_pattern: Option<String>,
    pub line_weight: Option<f64>,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct IfcdrScopeOrder {
    pub scope_id: u32,
    pub entities: Vec<u64>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IfcdrColor {
    pub(crate) rgb: [u8; 3],
    pub(crate) indexed: Option<IfcdrIndexedColor>,
    pub(crate) named: Option<IfcdrNamedColor>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IfcdrIndexedColor {
    pub system: String,
    pub index: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IfcdrNamedColor {
    pub catalog: String,
    pub name: String,
}

impl IfcdrColor {
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            rgb: [red, green, blue],
            indexed: None,
            named: None,
        }
    }
    pub fn with_indexed(mut self, system: impl Into<String>, index: u64) -> Self {
        self.indexed = Some(IfcdrIndexedColor {
            system: system.into(),
            index,
        });
        self
    }
    pub fn with_named(mut self, catalog: impl Into<String>, name: impl Into<String>) -> Self {
        self.named = Some(IfcdrNamedColor {
            catalog: catalog.into(),
            name: name.into(),
        });
        self
    }
}

pub(crate) fn valid_polyline_vertex_count(count: usize) -> bool {
    count >= 2
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IfcdrEntityRow {
    pub entity_id: u64,
    pub scope_id: u32,
    pub layer_id: u32,
    pub appearance_id: u32,
    pub visible: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IfcdrLineRow {
    pub entity: IfcdrEntityRow,
    pub start: Point2,
    pub end: Point2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppearanceMode {
    ByLayer,
    ByBlock,
    Explicit,
}

pub(crate) fn appearance_mode(value: u32) -> Option<AppearanceMode> {
    match value {
        0 => Some(AppearanceMode::ByLayer),
        1 => Some(AppearanceMode::Explicit),
        2 => Some(AppearanceMode::ByBlock),
        _ => None,
    }
}
pub(crate) fn length_unit(value: &str) -> Option<crate::ifcdr::IfcdrLengthUnit> {
    use crate::ifcdr::IfcdrLengthUnit::*;
    Some(match value {
        "unitless" => Unitless,
        "mm" => Millimetre,
        "cm" => Centimetre,
        "m" => Metre,
        "km" => Kilometre,
        "in" => Inch,
        "ft" => Foot,
        _ => return None,
    })
}
pub(crate) fn rgb_channel(value: u64) -> Option<u8> {
    u8::try_from(value).ok()
}
pub(crate) fn valid_opacity(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}
pub(crate) fn valid_line_weight(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}
pub(crate) fn valid_point(point: Point2) -> bool {
    point.x().is_finite() && point.y().is_finite()
}
pub(crate) fn valid_color(color: &IfcdrColor) -> bool {
    color.indexed.as_ref().is_none_or(|v| !v.system.is_empty())
        && color
            .named
            .as_ref()
            .is_none_or(|v| !v.catalog.is_empty() && !v.name.is_empty())
}
