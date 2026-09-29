use ifccad::experimental_ifcx::read_native_cad_ifcx;
use ifccad::experimental_ifcx::*;
use serde_json::{json, Value};

fn base() -> Value {
    let mut value = json!({
        "header": {"id":"demo", "ifcxVersion":"ifcx_alpha", "dataVersion":"0.1", "author":"test", "timestamp":"2026-09-29T00:00:00Z"},
        "imports": [], "schemas": {}, "data": [
            {"path":"</cad/d1>","children":{"model":"</cad/d1/layout/1>","layer0":"</cad/d1/layer/0>"},"attributes":{"ifccad::drawing":{"profileVersion":"0.1.0","lengthUnit":"mm"}}},
            {"path":"</cad/d1/layout/1>","children":{"0":"</cad/d1/e2>","1":"</cad/d1/e1>"},"attributes":{"ifccad::layout":{"kind":"Model"}}},
            {"path":"</cad/d1/layer/0>","attributes":{"ifccad::layer":{"name":"0","appearance":{"color":"#ffffff","opacity":1.0,"linePattern":"Continuous","lineWeight":0.25}}}},
            {"path":"</cad/d1/e1>","attributes":{"ifccad::entity":{"layer":"</cad/d1/layer/0>","appearance":{"color":{"mode":"ByLayer"},"opacity":{"mode":"ByLayer"},"linePattern":{"mode":"ByLayer"},"lineWeight":{"mode":"ByLayer"}}},"ifccad::geom::circle":{"radius":2.0},"ifccad::geom::placement":{"origin":[0,0,0],"xAxis":[1,0,0],"yAxis":[0,1,0]}}},
            {"path":"</cad/d1/e2>","attributes":{"ifccad::entity":{"layer":"</cad/d1/layer/0>","appearance":{"color":{"mode":"ByLayer"},"opacity":{"mode":"ByLayer"},"linePattern":{"mode":"ByLayer"},"lineWeight":{"mode":"ByLayer"}}},"ifccad::geom::lineSegment":{"start":[0,0,0],"end":[1,0,0]}}}
        ]
    });
    let module: Value = serde_json::from_str(include_str!(
        "../schemas/ifcx-native-cad/experimental-profile-0.1.0.ifcx"
    ))
    .unwrap();
    value["schemas"] = module["schemas"].clone();
    value
}

fn read(
    value: &Value,
) -> Result<ifccad::experimental_ifcx::ValidatedIfcxCad, ifccad::experimental_ifcx::IfcxCadReport> {
    read_native_cad_ifcx(&serde_json::to_vec(value).unwrap())
}

#[test]
fn profile_order_is_numeric_child_order() {
    let result = read(&base()).unwrap();
    assert_eq!(
        result
            .document()
            .model
            .entities
            .iter()
            .map(|e| e.id)
            .collect::<Vec<_>>(),
        vec![2, 1]
    );
}

#[test]
fn profile_circle_requires_placement() {
    let mut value = base();
    value["data"][3]["attributes"]
        .as_object_mut()
        .unwrap()
        .remove("ifccad::geom::placement");
    assert!(read(&value)
        .unwrap_err()
        .errors
        .iter()
        .any(|e| e.contains("placement")));
}

#[test]
fn profile_rejects_duplicate_owner() {
    let mut value = base();
    value["data"][0]["children"]["block1"] = json!("</cad/d1/block/1>");
    value["data"].as_array_mut().unwrap().push(json!({"path":"</cad/d1/block/1>","children":{"0":"</cad/d1/e1>"},"attributes":{"ifccad::blockDefinition":{"name":"B","basePoint":[0,0,0],"insertionUnit":"mm"}}}));
    assert!(read(&value)
        .unwrap_err()
        .errors
        .iter()
        .any(|e| e.contains("owner")));
}

#[test]
fn appearance_layer_zero_and_byblock_are_resolved_per_property() {
    use ifccad::experimental_ifcx::IfcxCadResolvedAppearance;
    let mut value = base();
    value["data"][0]["children"]["layer1"] = json!("</cad/d1/layer/1>");
    value["data"][0]["children"]["block1"] = json!("</cad/d1/block/1>");
    value["data"][2]["attributes"]["ifccad::layer"]["appearance"]["color"] = json!("#ff0000");
    value["data"][2]["attributes"]["ifccad::layer"]["appearance"]["lineWeight"] = json!(0.1);
    value["data"].as_array_mut().unwrap().push(json!({"path":"</cad/d1/layer/1>","attributes":{"ifccad::layer":{"name":"Walls","appearance":{"color":"#00ff00","opacity":0.8,"linePattern":"Continuous","lineWeight":0.5}}}}));
    value["data"].as_array_mut().unwrap().push(json!({"path":"</cad/d1/block/1>","children":{"0":"</cad/d1/e3>"},"attributes":{"ifccad::blockDefinition":{"name":"Mark","basePoint":[0,0,0],"insertionUnit":"mm"}}}));
    value["data"].as_array_mut().unwrap().push(json!({"path":"</cad/d1/e3>","attributes":{"ifccad::entity":{"layer":"</cad/d1/layer/0>","appearance":{"color":{"mode":"ByBlock"},"opacity":{"mode":"ByLayer"},"linePattern":{"mode":"ByLayer"},"lineWeight":{"mode":"ByLayer"}}},"ifccad::geom::lineSegment":{"start":[0,0,0],"end":[1,0,0]}}}));
    value["data"][1]["children"]["2"] = json!("</cad/d1/e4>");
    value["data"].as_array_mut().unwrap().push(json!({"path":"</cad/d1/e4>","attributes":{"ifccad::entity":{"layer":"</cad/d1/layer/1>","appearance":{"color":{"mode":"Explicit","value":"#0000ff"},"opacity":{"mode":"ByLayer"},"linePattern":{"mode":"ByLayer"},"lineWeight":{"mode":"ByLayer"}}},"ifccad::blockInstance":{"definition":"</cad/d1/block/1>","transform":{"placement":{"origin":[0,0,0],"xAxis":[1,0,0],"yAxis":[0,1,0]},"rotation":0,"scale":[1,1,1]}}}}));
    let loaded = read(&value).unwrap();
    assert_eq!(
        loaded
            .effective_appearance("</cad/d1/e3>", &["</cad/d1/e4>"])
            .unwrap(),
        IfcxCadResolvedAppearance {
            color: "#0000ff".into(),
            opacity: 0.8,
            line_pattern: "Continuous".into(),
            line_weight: 0.5
        }
    );
}

fn fixture_document() -> IfcxCadDocument {
    let by_layer = IfcxCadEntityAppearance {
        color: IfcxCadMode::ByLayer,
        opacity: IfcxCadMode::ByLayer,
        line_pattern: IfcxCadMode::ByLayer,
        line_weight: IfcxCadMode::ByLayer,
    };
    let placement = IfcxCadPlacement {
        origin: [10.0, 2.0, 0.0],
        x_axis: [1.0, 0.0, 0.0],
        y_axis: [0.0, 1.0, 0.0],
    };
    let instance = |id, x, color: &str| IfcxCadEntity {
        id,
        layer_id: 2,
        appearance: IfcxCadEntityAppearance {
            color: IfcxCadMode::Explicit(color.into()),
            ..by_layer.clone()
        },
        kind: IfcxCadEntityKind::BlockInstance {
            definition_id: 1,
            transform: IfcxCadBlockTransform {
                placement: IfcxCadPlacement {
                    origin: [x, 0.0, 0.0],
                    ..placement.clone()
                },
                rotation: 0.25,
                scale: [2.0, 1.0, 1.0],
            },
        },
    };
    IfcxCadDocument {
        header: IfcxCadHeader {
            id: "ifccad-experimental-hello".into(),
            data_version: "0.1.0".into(),
            author: "OpenAEC".into(),
            timestamp: "2026-09-29T00:00:00Z".into(),
        },
        drawing_id: 1,
        length_unit: "cm".into(),
        layers: vec![
            IfcxCadLayer {
                id: 0,
                name: "0".into(),
                appearance: IfcxCadLayerAppearance {
                    color: "#ffffff".into(),
                    opacity: 1.0,
                    line_pattern: "Continuous".into(),
                    line_weight: 0.1,
                },
            },
            IfcxCadLayer {
                id: 1,
                name: "Construction".into(),
                appearance: IfcxCadLayerAppearance {
                    color: "#00ff00".into(),
                    opacity: 0.8,
                    line_pattern: "Continuous".into(),
                    line_weight: 0.25,
                },
            },
            IfcxCadLayer {
                id: 2,
                name: "Symbols".into(),
                appearance: IfcxCadLayerAppearance {
                    color: "#ff0000".into(),
                    opacity: 1.0,
                    line_pattern: "Continuous".into(),
                    line_weight: 0.35,
                },
            },
        ],
        model: IfcxCadLayout {
            id: 1,
            entities: vec![
                IfcxCadEntity {
                    id: 42,
                    layer_id: 1,
                    appearance: by_layer.clone(),
                    kind: IfcxCadEntityKind::LineSegment {
                        start: [0.0, 0.0, 0.0],
                        end: [50.0, 0.0, 0.0],
                    },
                },
                IfcxCadEntity {
                    id: 7,
                    layer_id: 1,
                    appearance: by_layer.clone(),
                    kind: IfcxCadEntityKind::PlanarPolyline {
                        vertices: vec![[0.0, 0.0], [2.0, 0.0], [2.0, 3.0]],
                        closed: false,
                        placement: placement.clone(),
                    },
                },
                instance(90, 20.0, "#0000ff"),
                IfcxCadEntity {
                    id: 9,
                    layer_id: 2,
                    appearance: by_layer.clone(),
                    kind: IfcxCadEntityKind::Circle {
                        radius: 3.0,
                        placement: placement.clone(),
                    },
                },
                instance(91, 40.0, "#ffff00"),
            ],
        },
        blocks: vec![IfcxCadBlockDefinition {
            id: 1,
            name: "Marker".into(),
            base_point: [2.0, 0.0, 0.0],
            insertion_unit: "cm".into(),
            entities: vec![IfcxCadEntity {
                id: 100,
                layer_id: 0,
                appearance: IfcxCadEntityAppearance {
                    color: IfcxCadMode::ByBlock,
                    ..by_layer
                },
                kind: IfcxCadEntityKind::LineSegment {
                    start: [2.0, 0.0, 0.0],
                    end: [4.0, 0.0, 0.0],
                },
            }],
        }],
    }
}

#[test]
fn roundtrip_complete_example_and_determinism() {
    let document = fixture_document();
    let bytes = write_native_cad_ifcx(&document).unwrap();
    assert_paths_first(&bytes);
    let file: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(file["schemas"].as_object().unwrap().is_empty());
    assert_eq!(
        file["imports"][0]["uri"],
        "urn:example:ifccad:experimental-cad:0.1.0"
    );
    assert_eq!(read_native_cad_ifcx(&bytes).unwrap().document(), &document);
    assert_eq!(write_native_cad_ifcx(&document).unwrap(), bytes);
    if std::env::var_os("GENERATE_IFCX_FIXTURE").is_some() {
        std::fs::write("examples/ifcx-native-cad/hello-cad.ifcx", &bytes).unwrap();
    }
}

fn assert_paths_first(bytes: &[u8]) {
    let source = std::str::from_utf8(bytes).unwrap();
    let data = source.split("\"data\": [").nth(1).unwrap();
    let mut nodes = 0;
    let mut lines = data.lines();
    while let Some(line) = lines.next() {
        if line == "  ]," || line == "  ]" {
            break;
        }
        if line == "    {" {
            nodes += 1;
            assert!(lines.next().unwrap().starts_with("      \"path\": "));
        }
    }
    assert!(nodes > 0);
}

#[test]
fn profile_rejects_child_gap_and_unused_block_cycle() {
    let mut value = base();
    value["data"][1]["children"]
        .as_object_mut()
        .unwrap()
        .remove("0");
    assert!(read(&value)
        .unwrap_err()
        .errors
        .iter()
        .any(|e| e.contains("gap")));
    let mut value = base();
    value["data"][0]["children"]["block1"] = json!("</cad/d1/block/1>");
    value["data"].as_array_mut().unwrap().push(json!({"path":"</cad/d1/block/1>","children":{"0":"</cad/d1/e3>"},"attributes":{"ifccad::blockDefinition":{"name":"Loop","basePoint":[0,0,0],"insertionUnit":"mm"}}}));
    value["data"].as_array_mut().unwrap().push(json!({"path":"</cad/d1/e3>","attributes":{"ifccad::entity":{"layer":"</cad/d1/layer/0>","appearance":{"color":{"mode":"ByLayer"},"opacity":{"mode":"ByLayer"},"linePattern":{"mode":"ByLayer"},"lineWeight":{"mode":"ByLayer"}}},"ifccad::blockInstance":{"definition":"</cad/d1/block/1>","transform":{"placement":{"origin":[0,0,0],"xAxis":[1,0,0],"yAxis":[0,1,0]},"rotation":0,"scale":[1,1,1]}}}}));
    assert!(read(&value)
        .unwrap_err()
        .errors
        .iter()
        .any(|e| e.contains("cycle")));
}

#[test]
fn profile_schema_and_extensions() {
    let mut value = base();
    value["data"].as_array_mut().unwrap().push(json!({"path":"</project/site>","children":{"cad":"</cad/d1/e1>"},"attributes":{"example::tag":{"value":"kept"}}}));
    value["data"][3]["attributes"]["example::entityNote"] = json!(42);
    let loaded = read(&value).unwrap();
    assert_eq!(loaded.raw_ifcx()["data"].as_array().unwrap().len(), 6);
    assert_eq!(
        loaded.raw_ifcx()["data"][3]["attributes"]["example::entityNote"],
        42
    );
    value["schemas"]
        .as_object_mut()
        .unwrap()
        .remove("ifccad::geom::circle");
    assert!(read(&value)
        .unwrap_err()
        .errors
        .iter()
        .any(|e| e.contains("schema")));
}

#[test]
fn profile_rejects_invalid_unit_and_unknown_geometry() {
    let mut value = base();
    value["data"][0]["attributes"]["ifccad::drawing"]["lengthUnit"] = json!("metres");
    assert!(read(&value)
        .unwrap_err()
        .errors
        .iter()
        .any(|e| e.contains("unit")));
    let mut value = base();
    value["data"][3]["attributes"]["ifccad::geom::spline"] = json!({"knots":[]});
    assert!(read(&value)
        .unwrap_err()
        .errors
        .iter()
        .any(|e| e.contains("unsupported")));
}

#[test]
fn profile_rejects_ambiguous_appearance_mode() {
    let mut value = base();
    value["data"][3]["attributes"]["ifccad::entity"]["appearance"]["color"]["value"] =
        json!("#ff0000");
    assert!(read(&value)
        .unwrap_err()
        .errors
        .iter()
        .any(|e| e.contains("mode")));
    let mut value = base();
    value["data"][3]["attributes"]["ifccad::entity"]["appearance"]["color"] =
        json!({"mode":"Explicit"});
    assert!(read(&value)
        .unwrap_err()
        .errors
        .iter()
        .any(|e| e.contains("value")));
}

#[test]
fn fixture_is_a_complete_strictly_readable_ifcx_file() {
    let bytes = include_bytes!("../examples/ifcx-native-cad/hello-cad.ifcx");
    assert_paths_first(bytes);
    let loaded = read_native_cad_ifcx(bytes).unwrap();
    assert_eq!(loaded.document(), &fixture_document());
    let schema_module: Value = serde_json::from_str(include_str!(
        "../schemas/ifcx-native-cad/experimental-profile-0.1.0.ifcx"
    ))
    .unwrap();
    let schemas = schema_module["schemas"].as_object().unwrap();
    assert_eq!(
        loaded.raw_ifcx()["imports"][0]["uri"],
        schema_module["header"]["id"]
    );
    for node in loaded.raw_ifcx()["data"].as_array().unwrap() {
        if let Some(attrs) = node.get("attributes").and_then(Value::as_object) {
            for key in attrs.keys().filter(|k| k.starts_with("ifccad::")) {
                assert!(schemas.contains_key(key), "missing {key}");
            }
        }
    }
}

#[test]
fn profile_imported_schema_resolves_offline_and_missing_import_fails() {
    let mut value = base();
    value["schemas"] = json!({});
    value["imports"] = json!([{"uri":"urn:example:ifccad:experimental-cad:0.1.0"}]);
    assert_eq!(read(&value).unwrap().document().model.entities.len(), 2);
    value["imports"] = json!([]);
    assert!(read(&value)
        .unwrap_err()
        .errors
        .iter()
        .any(|e| e.contains("schema")));
}

#[test]
fn exploratory_line_count_probe() {
    let mut document = fixture_document();
    document.model.entities = (1..=1000)
        .map(|id| IfcxCadEntity {
            id,
            layer_id: 1,
            appearance: IfcxCadEntityAppearance {
                color: IfcxCadMode::ByLayer,
                opacity: IfcxCadMode::ByLayer,
                line_pattern: IfcxCadMode::ByLayer,
                line_weight: IfcxCadMode::ByLayer,
            },
            kind: IfcxCadEntityKind::LineSegment {
                start: [id as f64, 0.0, 0.0],
                end: [id as f64, 10.0, 0.0],
            },
        })
        .collect();
    document.blocks.clear();
    let bytes = write_native_cad_ifcx(&document).unwrap();
    let start = std::time::Instant::now();
    let loaded = read_native_cad_ifcx(&bytes).unwrap();
    let elapsed = start.elapsed();
    assert_eq!(loaded.document(), &document);
    println!(
        "1000 lines: {} complete JSON bytes, strict read {} ms",
        bytes.len(),
        elapsed.as_millis()
    );
}
