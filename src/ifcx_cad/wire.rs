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
    color: IfcxCadMode<String>,
    opacity: IfcxCadMode<f64>,
    line_pattern: IfcxCadMode<String>,
    line_weight: IfcxCadMode<f64>,
}
fn resolve(
    path: &str,
    patterns: &BTreeMap<String, IfcxCadLinePatternId>,
) -> Result<IfcxCadLinePatternId, IfcxCadReport> {
    patterns
        .get(path)
        .copied()
        .ok_or_else(|| IfcxCadReport::one(format!("unresolved drawing-local line pattern {path}")))
}
impl LayerAppearance {
    pub(super) fn typed(
        self,
        patterns: &BTreeMap<String, IfcxCadLinePatternId>,
    ) -> Result<IfcxCadLayerAppearance, IfcxCadReport> {
        Ok(IfcxCadLayerAppearance {
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
        patterns: &BTreeMap<String, IfcxCadLinePatternId>,
    ) -> Result<IfcxCadEntityAppearance, IfcxCadReport> {
        let line_pattern = match self.line_pattern {
            IfcxCadMode::ByLayer => IfcxCadMode::ByLayer,
            IfcxCadMode::ByBlock => IfcxCadMode::ByBlock,
            IfcxCadMode::Explicit(p) => IfcxCadMode::Explicit(resolve(&p, patterns)?),
        };
        Ok(IfcxCadEntityAppearance {
            color: self.color,
            opacity: self.opacity,
            line_weight: self.line_weight,
            line_pattern,
        })
    }
}
pub(super) fn layer(a: &IfcxCadLayerAppearance, prefix: &str) -> Value {
    let mut v = serde_json::to_value(a).unwrap();
    v["linePattern"] = json!(format!("{prefix}/linePattern/{}", a.line_pattern.0));
    v
}
pub(super) fn entity(a: &IfcxCadEntityAppearance, prefix: &str) -> Value {
    let mut v = serde_json::to_value(a).unwrap();
    if let IfcxCadMode::Explicit(id) = a.line_pattern {
        v["linePattern"]["value"] = json!(format!("{prefix}/linePattern/{}", id.0));
    }
    v
}
