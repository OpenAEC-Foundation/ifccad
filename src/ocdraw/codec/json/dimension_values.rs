//! Physical color and local reference backings for Dimension nested values.
use crate::{dimension::*, ocdraw::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ConcreteWireColor {
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
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum WireColor {
    Foreground(ForegroundWireColor),
    Concrete(ConcreteWireColor),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ForegroundWireColor {
    kind: ForegroundKind,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
enum ForegroundKind {
    Foreground,
}
impl WireColor {
    fn encode(c: &DrawingColor) -> Self {
        match c {
            DrawingColor::Foreground => Self::Foreground(ForegroundWireColor {
                kind: ForegroundKind::Foreground,
            }),
            DrawingColor::Concrete {
                rgb,
                indexed,
                named,
            } => Self::Concrete(ConcreteWireColor {
                rgb: *rgb,
                indexed_color: indexed.as_ref().map(|(s, i)| WireIndexed {
                    system: s.clone(),
                    index: *i,
                }),
                named_color: named.as_ref().map(|(c, n)| WireNamed {
                    catalog: c.clone(),
                    name: n.clone(),
                }),
            }),
        }
    }
    fn decode(&self) -> DrawingColor {
        match self {
            Self::Foreground(_) => DrawingColor::Foreground,
            Self::Concrete(c) => DrawingColor::Concrete {
                rgb: c.rgb,
                indexed: c
                    .indexed_color
                    .as_ref()
                    .map(|v| (v.system.clone(), v.index)),
                named: c
                    .named_color
                    .as_ref()
                    .map(|v| (v.catalog.clone(), v.name.clone())),
            },
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
