use super::*;
use crate::dimension::*;
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IfccadDimensionStyleId(pub u64);
pub type IfccadDimensionSettings =
    DimensionStyleSettings<IfccadTextStyleId, u64, IfccadColor, IfccadLinePatternId>;
pub type IfccadDimensionOverrides =
    DimensionStyleOverrides<IfccadTextStyleId, u64, IfccadColor, IfccadLinePatternId>;
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadDimensionStyle {
    pub id: IfccadDimensionStyleId,
    pub name: String,
    pub settings: IfccadDimensionSettings,
    pub type_overrides: BTreeMap<DimensionKind, IfccadDimensionOverrides>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadDimensionAssociation {
    pub role: DimensionRole,
    pub source_entity_id: u64,
    pub selector: DimensionPointSelector,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadDimension {
    pub placement: IfccadPlacement,
    pub style_id: IfccadDimensionStyleId,
    pub definition: DimensionDefinition,
    pub text_placement: DimensionTextPlacement,
    pub label: DimensionLabel,
    pub overrides: IfccadDimensionOverrides,
    pub associations: Vec<IfccadDimensionAssociation>,
    /// Optional independent worked-out picture, in the Dimension frame.
    pub graphics: Option<IfccadDimensionGraphics>,
}
