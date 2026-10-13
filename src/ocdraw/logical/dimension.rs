use super::{DrawingColor, EntityAppearance, LinePatternId, OcdrawTextStyleId};
use crate::{dimension::*, geometry_kernel::CoordinateFrame3};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OcdrawDimensionStyleId(pub u32);
pub type OcdrawDimensionSettings =
    DimensionStyleSettings<OcdrawTextStyleId, u32, DrawingColor, LinePatternId>;
pub type OcdrawDimensionOverrides =
    DimensionStyleOverrides<OcdrawTextStyleId, u32, DrawingColor, LinePatternId>;

#[derive(Clone, Debug, PartialEq)]
pub struct DrawingDimensionStyle {
    pub id: OcdrawDimensionStyleId,
    pub name: String,
    pub settings: OcdrawDimensionSettings,
    pub type_overrides: BTreeMap<DimensionKind, OcdrawDimensionOverrides>,
}
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OcdrawDimensionAssociation {
    pub role: DimensionRole,
    pub source_entity_id: u64,
    pub selector: DimensionPointSelector,
}
#[derive(Clone, Debug, PartialEq)]
pub struct DrawingDimensionEntity {
    pub id: u64,
    pub layer_id: u32,
    pub visible: bool,
    pub appearance: EntityAppearance,
    pub placement: CoordinateFrame3,
    pub style_id: OcdrawDimensionStyleId,
    pub definition: DimensionDefinition,
    pub text_placement: DimensionTextPlacement,
    pub label: DimensionLabel,
    pub overrides: OcdrawDimensionOverrides,
    pub associations: Vec<OcdrawDimensionAssociation>,
}
