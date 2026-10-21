//! Canonical owned graph projection, separate from ordinary entity ownership.
use super::*;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
fn fail(path: &str, m: impl ToString) -> IfccadReport {
    crate::ifccad::diagnostics::failure("IFCCAD-DIMENSION-GRAPHICS-003", path, m.to_string())
}
pub(super) fn encode(
    g: &IfccadDimensionGraphics,
    path: &str,
    prefix: &str,
) -> Vec<super::encode::NodeOut> {
    let mut nodes = vec![super::encode::NodeOut {
        path: path.into(),
        children: Some(
            g.parts
                .iter()
                .enumerate()
                .map(|(i, _)| (i.to_string(), json!(format!("{path}/p{i}"))))
                .collect(),
        ),
        attributes: Map::from_iter([(
            "ifccad::dimensionGraphics".into(),
            json!({"origin":match g.origin {IfccadDimensionGraphicsOrigin::SourceSnapshot=>"sourceSnapshot",IfccadDimensionGraphicsOrigin::Generated=>"generated"}}),
        )]),
    }];
    for (i, p) in g.parts.iter().enumerate() {
        let mut value = json!({});
        if let Some(id) = p.layer_id {
            value["layer"] = json!(format!("{prefix}/layer/{id}"));
        }
        if let Some(a) = super::wire::native_entity(&p.appearance, prefix) {
            value["appearance"] = a;
        }
        if !p.visible {
            value["visible"] = json!(false);
        }
        if !super::defaults::is_one(p.line_pattern_scale) {
            value["linePatternScale"] = json!(p.line_pattern_scale);
        }
        if p.purpose == IfccadDimensionGraphicsPurpose::ReferencePoint {
            value["purpose"] = json!("referencePoint");
        }
        let mut attrs = Map::from_iter([("ifccad::dimensionGraphicsPart".into(), value)]);
        super::encode::encode_payload(&p.kind.clone().into(), prefix, &mut attrs);
        nodes.push(super::encode::NodeOut {
            path: format!("{path}/p{i}"),
            children: None,
            attributes: attrs,
        });
    }
    nodes
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Part {
    #[serde(default, deserialize_with = "super::presentation::present")]
    layer: Option<String>,
    #[serde(default, deserialize_with = "super::presentation::present")]
    appearance: Option<Value>,
    #[serde(default = "yes")]
    visible: bool,
    #[serde(default = "crate::ifccad::logical::patterns::one")]
    line_pattern_scale: f64,
    #[serde(default)]
    purpose: Purpose,
}
fn yes() -> bool {
    true
}
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Purpose {
    #[default]
    Display,
    ReferencePoint,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Group {
    origin: Origin,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum Origin {
    SourceSnapshot,
    Generated,
}
pub(super) struct Reader<'a> {
    pub nodes: &'a BTreeMap<String, &'a Value>,
    pub prefix: &'a str,
    pub layers: &'a BTreeMap<String, u64>,
    pub blocks: &'a BTreeMap<String, u64>,
    pub patterns: &'a BTreeMap<String, IfccadLinePatternId>,
    pub styles: &'a BTreeMap<String, IfccadTextStyleId>,
    pub seen: BTreeSet<String>,
}
impl Reader<'_> {
    pub fn attach(&mut self, e: &mut IfccadEntity, node: &Value) -> Result<(), IfccadReport> {
        let owner = node["path"].as_str().expect("indexed path");
        let Some(link) = node.get("children").and_then(|c| c.get("graphics")) else {
            return Ok(());
        };
        let Some(IfccadNativeEntity {
            kind: IfccadEntityKind::Dimension(v),
            ..
        }) = e.as_native_mut()
        else {
            return Err(fail(owner, "graphics child requires Dimension"));
        };
        let path = format!("{owner}/graphics");
        if link.as_str() != Some(&path) {
            return Err(fail(owner, "noncanonical graphics link"));
        }
        let n = self
            .nodes
            .get(&path)
            .ok_or_else(|| fail(&path, "missing graphics group"))?;
        self.role(n, "ifccad::dimensionGraphics", &[])?;
        let raw = &n["attributes"]["ifccad::dimensionGraphics"];
        super::supplemental::validate_value("ifccad::dimensionGraphics", raw, &path)?;
        let g: Group = serde_json::from_value(raw.clone()).map_err(|e| fail(&path, e))?;
        let children = n
            .get("children")
            .and_then(Value::as_object)
            .ok_or_else(|| fail(&path, "graphics children map required"))?;
        let mut parts = Vec::with_capacity(children.len());
        for i in 0..children.len() {
            let target = format!("{path}/p{i}");
            if children.get(&i.to_string()).and_then(Value::as_str) != Some(&target) {
                return Err(fail(
                    &path,
                    "graphics roles must be contiguous and canonical",
                ));
            }
            let n = self
                .nodes
                .get(&target)
                .ok_or_else(|| fail(&target, "missing graphics part"))?;
            if n.get("children")
                .is_some_and(|c| !c.as_object().is_some_and(|c| c.is_empty()))
            {
                return Err(fail(&target, "part cannot have children"));
            }
            let attrs = n["attributes"]
                .as_object()
                .ok_or_else(|| fail(&target, "missing part attributes"))?;
            let extra = super::geometry::PAYLOADS
                .iter()
                .copied()
                .chain([
                    "ifccad::blockInstance",
                    "ifccad::text",
                    "ifccad::mText",
                    "ifccad::hatch",
                    "ifccad::geom::placement",
                ])
                .collect::<Vec<_>>();
            self.role(n, "ifccad::dimensionGraphicsPart", &extra)?;
            super::supplemental::validate_value(
                "ifccad::dimensionGraphicsPart",
                &attrs["ifccad::dimensionGraphicsPart"],
                &target,
            )?;
            let p: Part = serde_json::from_value(attrs["ifccad::dimensionGraphicsPart"].clone())
                .map_err(|e| fail(&target, e))?;
            if attrs.contains_key("ifccad::blockInstance")
                && attrs.contains_key("ifccad::geom::placement")
            {
                return Err(fail(&target, "BlockInstance forbids separate placement"));
            }
            let kind = super::validate::decode_payload(
                attrs,
                self.prefix,
                &target,
                self.blocks,
                self.styles,
            )?
            .try_into()?;
            let layer_id = p
                .layer
                .map(|r| {
                    self.layers
                        .get(&r)
                        .copied()
                        .ok_or_else(|| fail(&target, "unresolved graphics layer"))
                })
                .transpose()?;
            parts.push(IfccadDimensionGraphicsPart {
                layer_id,
                kind,
                appearance: super::wire::decode_native_appearance(
                    p.appearance.as_ref(),
                    self.patterns,
                )?,
                visible: p.visible,
                line_pattern_scale: p.line_pattern_scale,
                purpose: match p.purpose {
                    Purpose::Display => IfccadDimensionGraphicsPurpose::Display,
                    Purpose::ReferencePoint => IfccadDimensionGraphicsPurpose::ReferencePoint,
                },
            });
        }
        v.graphics = Some(IfccadDimensionGraphics {
            origin: match g.origin {
                Origin::SourceSnapshot => IfccadDimensionGraphicsOrigin::SourceSnapshot,
                Origin::Generated => IfccadDimensionGraphicsOrigin::Generated,
            },
            parts,
        });
        Ok(())
    }
    fn role(&mut self, n: &Value, role: &str, extra: &[&str]) -> Result<(), IfccadReport> {
        let path = n["path"].as_str().expect("indexed path");
        if !self.seen.insert(path.into()) {
            return Err(fail(path, "multiple graphics owners"));
        }
        let a = n["attributes"]
            .as_object()
            .ok_or_else(|| fail(path, "missing graphics role"))?;
        if !a.contains_key(role)
            || a.keys()
                .any(|k| k.starts_with("ifccad::") && k != role && !extra.contains(&k.as_str()))
        {
            return Err(fail(path, "incompatible graphics role"));
        }
        Ok(())
    }
    pub fn finish(&self) -> Result<(), IfccadReport> {
        for (path, n) in self.nodes {
            let a = &n["attributes"];
            let declared = a.get("ifccad::dimensionGraphics").is_some()
                || a.get("ifccad::dimensionGraphicsPart").is_some();
            if declared && !self.seen.contains(path) {
                return Err(fail(path, "unowned graphics role"));
            }
            // Count every incoming link, including foreign/shared references to known roles.
            if declared {
                let count = self
                    .nodes
                    .values()
                    .filter_map(|n| n.get("children").and_then(Value::as_object))
                    .flat_map(|c| c.values())
                    .filter(|v| v.as_str() == Some(path))
                    .count();
                if count != 1 {
                    return Err(fail(
                        path,
                        "graphics node must have exactly one incoming child link",
                    ));
                }
            }
        }
        Ok(())
    }
}
