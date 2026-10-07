#[path = "../examples/support/ifccad_consistency.rs"]
mod example_consistency;

#[test]
fn compact_examples_recover_sparse_ids_without_changing_content_or_fragments() {
    use serde_json::{json, Value};
    use std::collections::BTreeMap;
    let original: Value = serde_json::from_slice(include_bytes!(
        "../examples/ifccad/blocks-and-fragments.ifcx"
    ))
    .unwrap();
    let canonical = example_consistency::normalize(&original).unwrap();
    assert_eq!(
        canonical,
        example_consistency::normalize(&serde_json::from_slice(&canonical).unwrap()).unwrap()
    );
    let mut mapping = BTreeMap::new();
    for node in original["data"].as_array().unwrap() {
        let path = node["path"].as_str().unwrap();
        let split = if let Some(id) = path.strip_prefix("/cad/d1/e") {
            Some(("/cad/d1/e", id))
        } else {
            path.rsplit_once('/')
                .filter(|(prefix, _)| prefix.starts_with("/cad/d1/"))
        };
        if let Some((prefix, id)) = split {
            if let Ok(id) = id.parse::<u64>() {
                if id > 0 {
                    mapping.insert(
                        path.to_owned(),
                        format!(
                            "{prefix}{}{mapped}",
                            if prefix.ends_with("/e") { "" } else { "/" },
                            mapped = id * 17 + 101
                        ),
                    );
                }
            }
        }
    }
    fn sparse(value: &mut Value, mapping: &BTreeMap<String, String>) {
        match value {
            Value::String(text) => {
                if let Some(new) = mapping.get(text) {
                    *text = new.clone();
                }
            }
            Value::Array(values) => values.iter_mut().for_each(|v| sparse(v, mapping)),
            Value::Object(values) => values.values_mut().for_each(|v| sparse(v, mapping)),
            _ => {}
        }
    }
    let mut modified = original.clone();
    sparse(&mut modified, &mapping);
    for node in modified["data"].as_array_mut().unwrap() {
        if let Some(drawing) = node["attributes"].get_mut("ifccad::drawing") {
            for field in [
                "nextEntityId",
                "nextLayerId",
                "nextLayoutId",
                "nextBlockId",
                "nextLinePatternId",
            ] {
                drawing[field] = json!(10000);
            }
        }
    }
    let recovered: Value =
        serde_json::from_slice(&example_consistency::normalize(&modified).unwrap()).unwrap();
    assert_eq!(recovered, original);
    let fragments: Vec<_> = recovered["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["path"] == "/cad/d1/e1")
        .collect();
    assert_eq!(fragments.len(), 2);
    assert!(fragments[0]["attributes"]
        .get("ifccad::geom::lineSegment")
        .is_none());
    assert!(fragments[1]["attributes"]
        .get("ifccad::geom::lineSegment")
        .is_some());
    let foreign = recovered["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["path"] == "/reference/design-intent")
        .unwrap();
    assert_eq!(foreign["children"]["line"], "/cad/d1/e1");
    assert_eq!(foreign["inherits"]["reference"], "/reference/shared-note");
}

#[test]
fn every_ifccad_example_uses_compact_ids_and_writer_node_order() {
    use serde::de::{IgnoredAny, MapAccess, Visitor};
    use serde::{Deserialize, Deserializer};
    use std::fmt;
    struct NodeKeys(Vec<String>);
    impl<'de> Deserialize<'de> for NodeKeys {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct Keys;
            impl<'de> Visitor<'de> for Keys {
                type Value = NodeKeys;
                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str("an IFCX node")
                }
                fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<NodeKeys, M::Error> {
                    let mut keys = Vec::new();
                    while let Some(key) = map.next_key()? {
                        keys.push(key);
                        map.next_value::<IgnoredAny>()?;
                    }
                    Ok(NodeKeys(keys))
                }
            }
            deserializer.deserialize_map(Keys)
        }
    }
    #[derive(Deserialize)]
    struct FileKeys {
        data: Vec<NodeKeys>,
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/ifccad");
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "ifcx") {
            continue;
        }
        let bytes = std::fs::read(&path).unwrap();
        let loaded = ocdraw::ifccad::load_ifccad_bytes(&bytes, Default::default()).unwrap();
        let d = loaded.document();
        let mut papers: Vec<_> = d.paper_layouts.iter().collect();
        papers.sort_by_key(|p| p.tab_index);
        let ids: Vec<_> = d
            .model
            .entities
            .iter()
            .chain(papers.iter().flat_map(|p| &p.entities))
            .chain(d.blocks.iter().flat_map(|b| &b.entities))
            .map(|e| e.id)
            .collect();
        assert_eq!(
            ids,
            (1..=ids.len() as u64).collect::<Vec<_>>(),
            "{}",
            path.display()
        );
        assert_eq!(d.id_counters.next_entity_id, ids.len() as u64 + 1);
        for (mut ids, next, allow_zero) in [
            (
                std::iter::once(d.model.id)
                    .chain(papers.iter().map(|p| p.id))
                    .collect::<Vec<_>>(),
                d.id_counters.next_layout_id,
                false,
            ),
            (
                d.layers.iter().map(|l| l.id).collect(),
                d.id_counters.next_layer_id,
                true,
            ),
            (
                d.line_patterns.iter().map(|p| p.id.0).collect(),
                d.id_counters.next_line_pattern_id,
                true,
            ),
            (
                d.blocks.iter().map(|b| b.id).collect(),
                d.id_counters.next_block_id,
                false,
            ),
        ] {
            ids.sort();
            let first = if allow_zero && ids.first() == Some(&0) {
                0
            } else {
                1
            };
            assert_eq!(
                ids,
                (first..first + ids.len() as u64).collect::<Vec<_>>(),
                "{}",
                path.display()
            );
            assert_eq!(next, first + ids.len() as u64);
        }
        let keys: FileKeys = serde_json::from_slice(&bytes).unwrap();
        for NodeKeys(keys) in keys.data {
            assert_eq!(keys.first().map(String::as_str), Some("path"));
            let core: Vec<_> = keys
                .iter()
                .map(String::as_str)
                .filter(|k| ["path", "children", "attributes"].contains(k))
                .collect();
            assert_eq!(
                core,
                if core.contains(&"children") {
                    vec!["path", "children", "attributes"]
                } else {
                    vec!["path", "attributes"]
                }
            );
        }
    }
}

#[test]
fn representative_native_examples_strict_read_and_cover_entity_families() {
    use ocdraw::ocdraw::{load_ocdraw_bytes, DrawingGeometry};
    for bytes in [
        include_bytes!("../examples/ocdraw/overview.ocdraw.json").as_slice(),
        include_bytes!("../examples/ocdraw/layouts-viewports.ocdraw.json").as_slice(),
        include_bytes!("../examples/ocdraw/state-and-storage.ocdraw.json").as_slice(),
    ] {
        load_ocdraw_bytes(bytes).unwrap();
    }
    let drawing =
        load_ocdraw_bytes(include_bytes!("../examples/ocdraw/overview.ocdraw.json")).unwrap();
    let kinds: std::collections::BTreeSet<_> = drawing
        .geometric_entities()
        .iter()
        .map(|e| match e.geometry {
            DrawingGeometry::Line { .. } => "line",
            DrawingGeometry::Point { .. } => "point",
            DrawingGeometry::Circle { .. } => "circle",
            DrawingGeometry::Arc { .. } => "arc",
            DrawingGeometry::Ellipse { .. } => "ellipse",
            DrawingGeometry::PlanarPolyline { .. } => "planar",
            DrawingGeometry::SpatialPolyline { .. } => "spatial",
            DrawingGeometry::BlockInstance { .. } => "block",
        })
        .collect();
    assert_eq!(kinds.len(), 8);
    for bytes in [
        include_bytes!("../examples/ifccad/overview.ifcx").as_slice(),
        include_bytes!("../examples/ifccad/layouts-viewports.ifcx").as_slice(),
        include_bytes!("../examples/ifccad/blocks-and-fragments.ifcx").as_slice(),
    ] {
        ocdraw::ifccad::load_ifccad_bytes(bytes, Default::default()).unwrap();
    }
}
