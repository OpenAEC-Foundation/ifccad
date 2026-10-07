//! Presentation and compact IDs for authored examples, never a production codec.
use ocdraw::ifccad::load_ifccad_bytes;
use serde::{ser::SerializeMap, Serialize, Serializer};
use serde_json::{json, Value};
use std::{collections::BTreeMap, error::Error};

struct OrderedObject<'a> {
    value: &'a Value,
    keys: &'static [&'static str],
}
impl Serialize for OrderedObject<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let object = self.value.as_object().expect("validated example object");
        let mut map = serializer.serialize_map(Some(object.len()))?;
        for key in self.keys {
            if let Some(value) = object.get(*key) {
                if *key == "data" {
                    let nodes: Vec<_> = value
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|node| OrderedObject {
                            value: node,
                            keys: &["path", "children", "attributes"],
                        })
                        .collect();
                    map.serialize_entry(key, &nodes)?;
                } else if *key == "header" {
                    map.serialize_entry(
                        key,
                        &OrderedObject {
                            value,
                            keys: &["id", "ifcxVersion", "dataVersion", "author", "timestamp"],
                        },
                    )?;
                } else {
                    map.serialize_entry(key, value)?;
                }
            }
        }
        for (key, value) in object {
            if !self.keys.contains(&key.as_str()) {
                map.serialize_entry(key, value)?;
            }
        }
        map.end()
    }
}
fn assign_paths(
    paths: &mut BTreeMap<String, String>,
    prefix: &str,
    role: &str,
    ids: impl Iterator<Item = u64>,
    keep_zero: bool,
) -> u64 {
    let mut next = 1;
    for old in ids {
        let new = if keep_zero && old == 0 {
            0
        } else {
            let id = next;
            next += 1;
            id
        };
        let separator = if role == "e" { "" } else { "/" };
        paths.insert(
            format!("{prefix}/{role}{separator}{old}"),
            format!("/cad/d1/{role}{separator}{new}"),
        );
    }
    next
}
fn rewrite_paths(value: &mut Value, paths: &BTreeMap<String, String>) {
    match value {
        Value::String(text) => {
            if let Some(new) = paths.get(text) {
                *text = new.clone();
            }
        }
        Value::Array(values) => values.iter_mut().for_each(|v| rewrite_paths(v, paths)),
        Value::Object(values) => values.values_mut().for_each(|v| rewrite_paths(v, paths)),
        _ => {}
    }
}

/// Retain every source fragment and foreign value while assigning example IDs.
pub fn normalize(value: &Value) -> Result<Vec<u8>, Box<dyn Error>> {
    let loaded = load_ifccad_bytes(&serde_json::to_vec(value)?, Default::default())?;
    let d = loaded.document();
    let prefix = format!("/cad/d{}", d.drawing_id);
    let mut papers: Vec<_> = d.paper_layouts.iter().collect();
    papers.sort_by_key(|p| p.tab_index);
    let mut paths = BTreeMap::from([(prefix.clone(), "/cad/d1".to_owned())]);
    let entities = d
        .model
        .entities
        .iter()
        .chain(papers.iter().flat_map(|p| &p.entities))
        .chain(d.blocks.iter().flat_map(|b| &b.entities));
    let next_entity = assign_paths(&mut paths, &prefix, "e", entities.map(|e| e.id), false);
    let next_layout = assign_paths(
        &mut paths,
        &prefix,
        "layout",
        std::iter::once(d.model.id).chain(papers.iter().map(|p| p.id)),
        false,
    );
    let next_layer = assign_paths(
        &mut paths,
        &prefix,
        "layer",
        d.layers.iter().map(|l| l.id),
        true,
    );
    let next_pattern = assign_paths(
        &mut paths,
        &prefix,
        "linePattern",
        d.line_patterns.iter().map(|p| p.id.0),
        true,
    );
    let next_block = assign_paths(
        &mut paths,
        &prefix,
        "block",
        d.blocks.iter().map(|b| b.id),
        false,
    );
    let mut result = value.clone();
    rewrite_paths(&mut result, &paths);
    for node in result["data"].as_array_mut().unwrap() {
        if let Some(drawing) = node["attributes"].get_mut("ifccad::drawing") {
            for (key, next) in [
                ("nextEntityId", next_entity),
                ("nextLayoutId", next_layout),
                ("nextLayerId", next_layer),
                ("nextLinePatternId", next_pattern),
                ("nextBlockId", next_block),
            ] {
                drawing[key] = json!(next);
            }
            if let Some(children) = node.get_mut("children").and_then(Value::as_object_mut) {
                let mut renamed = serde_json::Map::new();
                for (key, target) in std::mem::take(children) {
                    let name = target
                        .as_str()
                        .and_then(|p| p.strip_prefix("/cad/d1/"))
                        .and_then(|p| {
                            let (role, id) = p.split_once('/')?;
                            Some(match role {
                                "layout" if id == "1" => "model".to_owned(),
                                "layout" => format!("paper{id}"),
                                "layer" | "linePattern" | "block" => format!("{role}{id}"),
                                _ => return None,
                            })
                        })
                        .unwrap_or(key);
                    if renamed.insert(name, target).is_some() {
                        return Err("Duplicate example drawing child alias".into());
                    }
                }
                *children = renamed;
            }
        }
    }
    let bytes = serde_json::to_vec_pretty(&OrderedObject {
        value: &result,
        keys: &["header", "imports", "schemas", "data"],
    })?;
    load_ifccad_bytes(&bytes, Default::default())?;
    Ok(bytes)
}
