#[path = "../examples/support/explorer_catalogue.rs"]
mod catalogue;
#[path = "../examples/support/ifccad_consistency.rs"]
mod consistency;
#[path = "../examples/support/explorer_drawing.rs"]
mod drawing;
#[path = "../examples/support/explorer_house.rs"]
mod house;

#[test]
fn curated_files_match_their_strictly_read_back_recipes() {
    for (name, bytes) in [
        ("entities", catalogue::build().unwrap()),
        ("house", house::build().unwrap()),
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("examples/ifccad/{name}.ifcx"));
        assert_eq!(
            bytes,
            std::fs::read(path).unwrap(),
            "regenerate {name} after recipe changes"
        );
    }
}

#[test]
fn adding_catalogue_samples_grows_the_grid_and_saved_views() {
    let samples: Vec<_> = catalogue::SAMPLES
        .iter()
        .chain(std::iter::repeat_n(&catalogue::SAMPLES[0], 5))
        .map(|s| catalogue::Sample {
            title: s.title,
            note: s.note,
            draw: s.draw,
        })
        .collect();
    let bytes = catalogue::build_with_samples(&samples).unwrap();
    let loaded = ifccad::load_ifccad_bytes(&bytes, Default::default()).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let nodes = json["data"].as_array().unwrap();
    for number in [
        "22  Point",
        "23  Point",
        "24  Point",
        "25  Point",
        "26  Point",
    ] {
        assert!(String::from_utf8_lossy(&bytes).contains(number));
    }
    let viewport = nodes
        .iter()
        .find_map(|n| n["attributes"].get("ifccad::viewport"))
        .unwrap();
    assert_eq!(viewport["view"]["height"].as_f64(), Some(616.)); // Seven rows * 78 + 70.
    assert_eq!(loaded.document().model_windows[0].view.height, 736.);
}

// Exhaustive on purpose: a new native kind requires a catalogue coverage decision.
fn family(kind: &ifccad::IfccadEntityKind) -> &'static str {
    use ifccad::IfccadEntityKind::*;
    match kind {
        Solid(_) => "solid",
        Trace(_) => "trace",
        Dimension(_) => "dimension",
        Hatch(_) => "hatch",
        Text(_) => "text",
        MText(_) => "mtext",
        Viewport(_) => "viewport",
        Point { .. } => "point",
        Arc { .. } => "arc",
        Ellipse { .. } => "ellipse",
        EllipseArc { .. } => "ellipseArc",
        LineSegment { .. } => "line",
        PlanarPolyline { .. } => "planarPolyline",
        SpatialPolyline { .. } => "spatialPolyline",
        Circle { .. } => "circle",
        BlockInstance { .. } => "blockInstance",
    }
}

#[test]
fn catalogue_covers_every_native_family_and_both_dimension_kinds() {
    use std::collections::BTreeSet;
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/ifccad/entities.ifcx"
    ))
    .unwrap();
    let loaded = ifccad::load_ifccad_bytes(&bytes, Default::default()).unwrap();
    let d = loaded.document();
    let entities: Vec<_> = d
        .model
        .entities
        .iter()
        .chain(d.paper_layouts.iter().flat_map(|p| &p.entities))
        .filter_map(ifccad::IfccadEntity::as_native)
        .collect();
    let actual: BTreeSet<_> = entities.iter().map(|e| family(&e.kind)).collect();
    assert_eq!(
        actual,
        BTreeSet::from([
            "dimension",
            "hatch",
            "text",
            "mtext",
            "viewport",
            "point",
            "arc",
            "ellipse",
            "ellipseArc",
            "line",
            "planarPolyline",
            "spatialPolyline",
            "circle",
            "blockInstance",
            "solid",
            "trace"
        ])
    );
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    for kind in ["linear", "aligned"] {
        assert!(json["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n["attributes"]["ifccad::dimension"]["definition"]["kind"] == kind));
    }
    for kind in ["solid", "linePattern", "gradient"] {
        assert!(json["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n["attributes"]["ifccad::hatch"]["fill"]["kind"] == kind));
    }
}

#[test]
fn examples_use_native_dimensions_without_duplicate_text_stand_ins() {
    for name in ["entities", "house"] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("examples/ifccad/{name}.ifcx"));
        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        for node in value["data"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["attributes"].get("ifccad::entity").is_some())
        {
            if let Some(runs) = node["attributes"]["ifccad::text"]["content"].as_array() {
                for run in runs {
                    assert!(
                        !["58", "56.92 mm", "10000", "12000", "6100", "3900"]
                            .contains(&run["text"].as_str().unwrap_or_default()),
                        "{name} duplicates a native measurement in ordinary Text"
                    );
                }
            }
        }
        let style = value["data"]
            .as_array()
            .unwrap()
            .iter()
            .find_map(|n| n["attributes"].get("ifccad::dimensionStyle"))
            .unwrap();
        let numbers = &style["settings"]["numbers"]["primary"];
        assert_eq!(numbers["measurementFactor"].as_f64(), Some(1.));
        assert_eq!(
            numbers["precision"].as_u64(),
            Some(if name == "house" { 0 } else { 2 })
        );
        assert_eq!(numbers["suffix"], "");
        assert_eq!(numbers["zeroSuppression"]["trailing"], true);
    }
}

#[test]
fn house_is_in_millimetres_and_has_a_scaled_a3_sheet_and_reused_blocks() {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/ifccad/house.ifcx"
    ))
    .unwrap();
    let loaded = ifccad::load_ifccad_bytes(&bytes, Default::default()).unwrap();
    let d = loaded.document();
    let canvas = d.paper_layouts[0]
        .canvas
        .as_ref()
        .expect("A3 sheet needs a saved initial view");
    assert_eq!(canvas.view.center.x(), 210.);
    assert_eq!(canvas.view.center.y(), 148.5);
    assert!(canvas.view.height >= 340.);
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let nodes = json["data"].as_array().unwrap();
    assert!(nodes
        .iter()
        .any(|n| n["attributes"]["ifccad::drawing"]["lengthUnit"] == "mm"));
    assert!(nodes
        .iter()
        .any(|n| n["attributes"]["ifccad::layout"]["media"]["width"].as_f64() == Some(420.)));
    let viewport = nodes
        .iter()
        .find_map(|n| n["attributes"].get("ifccad::viewport"))
        .unwrap();
    assert_eq!(
        viewport["view"]["height"].as_f64().unwrap()
            / viewport["frame"]["height"].as_f64().unwrap(),
        100.0
    );
    assert!(
        d.model
            .entities
            .iter()
            .filter_map(ifccad::IfccadEntity::as_native)
            .filter(|e| family(&e.kind) == "blockInstance")
            .count()
            >= 8
    );
    for room in ["Living room", "Kitchen", "Hall", "WC"] {
        assert!(String::from_utf8_lossy(&bytes).contains(room));
    }
}

#[test]
fn compact_examples_recover_sparse_ids_without_changing_content_or_fragments() {
    use serde_json::{json, Value};
    use std::collections::BTreeMap;
    let original: Value = serde_json::from_slice(include_bytes!(
        "../tests/fixtures/ifccad/blocks-and-fragments.ifcx"
    ))
    .unwrap();
    let canonical = consistency::normalize(&original).unwrap();
    assert_eq!(
        canonical,
        consistency::normalize(&serde_json::from_slice(&canonical).unwrap()).unwrap()
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
        serde_json::from_slice(&consistency::normalize(&modified).unwrap()).unwrap();
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
        let loaded = ifccad::load_ifccad_bytes(&bytes, Default::default()).unwrap();
        let d = loaded.document();
        let mut papers: Vec<_> = d.paper_layouts.iter().collect();
        papers.sort_by_key(|p| p.tab_index);
        let ids: Vec<_> = d
            .model
            .entities
            .iter()
            .chain(papers.iter().flat_map(|p| &p.entities))
            .chain(d.blocks.iter().flat_map(|b| &b.entities))
            .map(|e| e.id())
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
    for bytes in [
        include_bytes!("../examples/ifccad/overview.ifcx").as_slice(),
        include_bytes!("../tests/fixtures/ifccad/layouts-viewports.ifcx").as_slice(),
        include_bytes!("../tests/fixtures/ifccad/blocks-and-fragments.ifcx").as_slice(),
    ] {
        ifccad::load_ifccad_bytes(bytes, Default::default()).unwrap();
    }
}

#[test]
fn curated_overviews_include_complex_patterns_without_expanding_the_chooser() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let ifccad = ifccad::load_ifccad_bytes(
        &std::fs::read(root.join("examples/ifccad/overview.ifcx")).unwrap(),
        Default::default(),
    )
    .unwrap();
    assert_eq!(ifccad.document().symbol_sources.len(), 3);
    assert!(ifccad
        .document()
        .line_patterns
        .iter()
        .any(|p| p.name == "Mixed" && p.pattern.iter().any(|e| e.marker.is_some())));
}
