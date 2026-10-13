use crate::{dimension::*, ifccad::*};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct WireColor {
    rgb: [u8; 3],
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::dimension::present_option"
    )]
    indexed_color: Option<Indexed>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::dimension::present_option"
    )]
    named_color: Option<Named>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Indexed {
    system: String,
    index: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Named {
    catalog: String,
    name: String,
}
impl WireColor {
    fn encode(c: &IfccadColor) -> Self {
        Self {
            rgb: c.rgb,
            indexed_color: c.indexed.as_ref().map(|(s, i)| Indexed {
                system: s.clone(),
                index: *i,
            }),
            named_color: c.named.as_ref().map(|(c, n)| Named {
                catalog: c.clone(),
                name: n.clone(),
            }),
        }
    }
    fn decode(&self) -> IfccadColor {
        IfccadColor {
            rgb: self.rgb,
            indexed: self
                .indexed_color
                .as_ref()
                .map(|c| (c.system.clone(), c.index)),
            named: self
                .named_color
                .as_ref()
                .map(|c| (c.catalog.clone(), c.name.clone())),
        }
    }
}
pub(super) type WireSettings = DimensionStyleSettings<String, String, WireColor, String>;
pub(super) type WireOverrides = DimensionStyleOverrides<String, String, WireColor, String>;
pub(super) fn reference(
    prefix: &str,
    stem: &str,
    v: &str,
    path: &str,
) -> Result<u64, IfccadReport> {
    let suffix = v.strip_prefix(&format!("{prefix}/{stem}")).ok_or_else(|| {
        crate::ifccad::diagnostics::failure(
            "IFCCAD-DIMENSION-002",
            path,
            "reference must be a complete same-drawing path",
        )
    })?;
    let id = suffix
        .parse::<u64>()
        .ok()
        .filter(|id| id.to_string() == suffix)
        .ok_or_else(|| {
            crate::ifccad::diagnostics::failure(
                "IFCCAD-DIMENSION-002",
                path,
                "reference needs a canonical uint64 target",
            )
        })?;
    Ok(id)
}
pub(super) fn encode_settings(v: &IfccadDimensionSettings, prefix: &str) -> WireSettings {
    v.map_references(
        |id| format!("{prefix}/textStyle/{}", id.0),
        |id| format!("{prefix}/block/{id}"),
        WireColor::encode,
        |id| format!("{prefix}/linePattern/{}", id.0),
    )
}
pub(super) fn encode_overrides(v: &IfccadDimensionOverrides, prefix: &str) -> WireOverrides {
    v.map_references(
        |id| format!("{prefix}/textStyle/{}", id.0),
        |id| format!("{prefix}/block/{id}"),
        WireColor::encode,
        |id| format!("{prefix}/linePattern/{}", id.0),
    )
}
pub(super) fn decode_settings(
    v: &WireSettings,
    prefix: &str,
    path: &str,
) -> Result<IfccadDimensionSettings, IfccadReport> {
    let error = RefCell::new(None);
    let parse = |s: &str, stem: &str| match reference(prefix, stem, s, path) {
        Ok(v) => Some(v),
        Err(e) => {
            *error.borrow_mut() = Some(e);
            None
        }
    };
    let partial = v.map_references(
        |s| parse(s, "textStyle/"),
        |s| parse(s, "block/"),
        WireColor::decode,
        |s| parse(s, "linePattern/"),
    );
    if let Some(e) = error.into_inner() {
        return Err(e);
    };
    Ok(partial.map_references(
        |v| IfccadTextStyleId(v.expect("checked references")),
        |v| v.expect("checked references"),
        Clone::clone,
        |v| IfccadLinePatternId(v.expect("checked references")),
    ))
}
pub(super) fn decode_overrides(
    v: &WireOverrides,
    prefix: &str,
    path: &str,
) -> Result<IfccadDimensionOverrides, IfccadReport> {
    let error = RefCell::new(None);
    let parse = |s: &str, stem: &str| match reference(prefix, stem, s, path) {
        Ok(v) => Some(v),
        Err(e) => {
            *error.borrow_mut() = Some(e);
            None
        }
    };
    let partial = v.map_references(
        |s| parse(s, "textStyle/"),
        |s| parse(s, "block/"),
        WireColor::decode,
        |s| parse(s, "linePattern/"),
    );
    if let Some(e) = error.into_inner() {
        return Err(e);
    };
    Ok(partial.map_references(
        |v| IfccadTextStyleId(v.expect("checked references")),
        |v| v.expect("checked references"),
        Clone::clone,
        |v| IfccadLinePatternId(v.expect("checked references")),
    ))
}
