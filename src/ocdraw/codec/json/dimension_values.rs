//! Physical color and local reference backings for Dimension nested values.
use crate::{dimension::*, ocdraw::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct WireColor {
    rgb: [u8; 3],
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::dimension::present_option"
    )]
    indexed_color: Option<WireIndexed>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::dimension::present_option"
    )]
    named_color: Option<WireNamed>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireIndexed {
    system: String,
    index: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireNamed {
    catalog: String,
    name: String,
}
impl WireColor {
    pub(super) fn encode(c: &DrawingColor) -> Self {
        Self {
            rgb: c.rgb,
            indexed_color: c.indexed.as_ref().map(|(s, i)| WireIndexed {
                system: s.clone(),
                index: *i,
            }),
            named_color: c.named.as_ref().map(|(c, n)| WireNamed {
                catalog: c.clone(),
                name: n.clone(),
            }),
        }
    }
    pub(super) fn decode(&self) -> DrawingColor {
        DrawingColor {
            rgb: self.rgb,
            indexed: self
                .indexed_color
                .as_ref()
                .map(|i| (i.system.clone(), i.index)),
            named: self
                .named_color
                .as_ref()
                .map(|n| (n.catalog.clone(), n.name.clone())),
        }
    }
}
pub(super) type WireSettings = DimensionStyleSettings<u32, u32, WireColor, u32>;
pub(super) type WireOverrides = DimensionStyleOverrides<u32, u32, WireColor, u32>;
pub(super) fn encode_settings(v: &OcdrawDimensionSettings) -> WireSettings {
    v.map_references(|id| id.0, |id| *id, WireColor::encode, |id| id.0)
}
pub(super) fn decode_settings(v: &WireSettings) -> OcdrawDimensionSettings {
    v.map_references(
        |id| OcdrawTextStyleId(*id),
        |id| *id,
        WireColor::decode,
        |id| LinePatternId(*id),
    )
}
pub(super) fn encode_overrides(v: &OcdrawDimensionOverrides) -> WireOverrides {
    v.map_references(|id| id.0, |id| *id, WireColor::encode, |id| id.0)
}
pub(super) fn decode_overrides(v: &WireOverrides) -> OcdrawDimensionOverrides {
    v.map_references(
        |id| OcdrawTextStyleId(*id),
        |id| *id,
        WireColor::decode,
        |id| LinePatternId(*id),
    )
}
