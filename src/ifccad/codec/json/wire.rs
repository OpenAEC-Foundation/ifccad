use super::*;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LayerAppearance {
    color: String,
    opacity: f64,
    line_pattern: String,
    line_weight: f64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EntityAppearance {
    color: IfccadMode<String>,
    opacity: IfccadMode<f64>,
    line_pattern: IfccadMode<String>,
    line_weight: IfccadMode<f64>,
}
fn resolve(
    path: &str,
    patterns: &BTreeMap<String, IfccadLinePatternId>,
) -> Result<IfccadLinePatternId, IfccadReport> {
    patterns
        .get(path)
        .copied()
        .ok_or_else(|| IfccadReport::one(format!("unresolved drawing-local line pattern {path}")))
}
impl LayerAppearance {
    pub(super) fn typed(
        self,
        patterns: &BTreeMap<String, IfccadLinePatternId>,
    ) -> Result<IfccadLayerAppearance, IfccadReport> {
        Ok(IfccadLayerAppearance {
            color: self.color,
            opacity: self.opacity,
            line_weight: self.line_weight,
            line_pattern: resolve(&self.line_pattern, patterns)?,
        })
    }
}
impl EntityAppearance {
    pub(super) fn typed(
        self,
        patterns: &BTreeMap<String, IfccadLinePatternId>,
    ) -> Result<IfccadEntityAppearance, IfccadReport> {
        let line_pattern = match self.line_pattern {
            IfccadMode::ByLayer => IfccadMode::ByLayer,
            IfccadMode::ByBlock => IfccadMode::ByBlock,
            IfccadMode::Explicit(p) => IfccadMode::Explicit(resolve(&p, patterns)?),
        };
        Ok(IfccadEntityAppearance {
            color: self.color,
            opacity: self.opacity,
            line_weight: self.line_weight,
            line_pattern,
        })
    }
}
pub(super) fn layer(a: &IfccadLayerAppearance, prefix: &str) -> Value {
    let mut v = serde_json::to_value(a).unwrap();
    v["linePattern"] = json!(format!("{prefix}/linePattern/{}", a.line_pattern.0));
    v
}
pub(super) fn entity(a: &IfccadEntityAppearance, prefix: &str) -> Value {
    let mut v = serde_json::to_value(a).unwrap();
    if let IfccadMode::Explicit(id) = a.line_pattern {
        v["linePattern"]["value"] = json!(format!("{prefix}/linePattern/{}", id.0));
    }
    v
}
