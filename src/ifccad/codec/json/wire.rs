use super::*;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct LayerAppearance {
    color: IfccadColor,
    #[serde(default = "crate::ifccad::logical::patterns::one")]
    opacity: f64,
    line_pattern: String,
    line_weight: f64,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct EntityAppearance {
    color: IfccadMode<IfccadColor>,
    opacity: IfccadMode<f64>,
    line_pattern: IfccadMode<String>,
    line_weight: IfccadMode<f64>,
}

pub(super) fn decode_native_appearance(
    value: Option<&Value>,
    patterns: &BTreeMap<String, IfccadLinePatternId>,
) -> Result<IfccadEntityAppearance, IfccadReport> {
    let mut expanded = match value {
        None => serde_json::Map::new(),
        Some(value) => value.as_object().cloned().ok_or_else(|| {
            crate::ifccad::diagnostics::failure(
                "IFCCAD-APPEARANCE-002",
                "/appearance",
                "native appearance must be an object",
            )
        })?,
    };
    for property in ["color", "opacity", "linePattern", "lineWeight"] {
        expanded
            .entry(property)
            .or_insert_with(|| json!({"mode":"ByLayer"}));
    }
    let expanded = Value::Object(expanded);
    super::validate::appearance_modes(&expanded, "/appearance")?;
    let appearance: EntityAppearance = serde_json::from_value(expanded).map_err(|error| {
        crate::ifccad::diagnostics::failure(
            "IFCCAD-APPEARANCE-002",
            "/appearance",
            error.to_string(),
        )
    })?;
    appearance.typed(patterns)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct BlockTransformValue {
    placement: IfccadPlacement,
    #[serde(default)]
    rotation: f64,
    #[serde(default = "unit_scale")]
    scale: [f64; 3],
}
fn unit_scale() -> [f64; 3] {
    [1.; 3]
}
impl BlockTransformValue {
    pub(super) fn into_typed(self) -> IfccadBlockTransform {
        IfccadBlockTransform {
            placement: self.placement,
            rotation: self.rotation,
            scale: self.scale,
        }
    }
}
fn resolve(
    path: &str,
    patterns: &BTreeMap<String, IfccadLinePatternId>,
) -> Result<IfccadLinePatternId, IfccadReport> {
    patterns.get(path).copied().ok_or_else(|| {
        crate::ifccad::diagnostics::failure(
            "IFCCAD-APPEARANCE-001",
            "/linePattern",
            format!("unresolved drawing-local line pattern {path}"),
        )
    })
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
    super::defaults::omit_number(&mut v, "opacity", 1.);
    v
}
pub(super) fn native_entity(a: &IfccadEntityAppearance, prefix: &str) -> Option<Value> {
    let mut value = entity(a, prefix);
    let fields = value.as_object_mut().expect("encoded appearance");
    fields.retain(|_, v| v["mode"] != "ByLayer");
    (!fields.is_empty()).then_some(value)
}
pub(super) fn block_transform(value: &IfccadBlockTransform) -> Value {
    let mut encoded = json!({"placement": value.placement});
    if !super::defaults::is_positive_zero(value.rotation) {
        encoded["rotation"] = json!(value.rotation);
    }
    if !value.scale.iter().all(|v| super::defaults::is_one(*v)) {
        encoded["scale"] = json!(value.scale);
    }
    encoded
}
pub(super) fn entity(a: &IfccadEntityAppearance, prefix: &str) -> Value {
    let mut v = serde_json::to_value(a).unwrap();
    if let IfccadMode::Explicit(id) = a.line_pattern {
        v["linePattern"]["value"] = json!(format!("{prefix}/linePattern/{}", id.0));
    }
    v
}
