use ocdraw::ifcx_cad::read_native_cad_ifcx;
use ocdraw::ifcx_cad::*;
use serde_json::{json, Value};

fn base() -> Value {
    let mut value = json!({
        "header": {"id":"demo", "ifcxVersion":"ifcx_alpha", "dataVersion":"0.1", "author":"test", "timestamp":"2026-09-29T00:00:00Z"},
        "imports": [], "schemas": {}, "data": [
            {"path":"/cad/d1","children":{"model":"/cad/d1/layout/1","layer0":"/cad/d1/layer/0"},"attributes":{"ifccad::drawing":{"profileVersion":"0.1.0","lengthUnit":"mm"}}},
            {"path":"/cad/d1/layout/1","children":{"0":"/cad/d1/e2","1":"/cad/d1/e1"},"attributes":{"ifccad::layout":{"kind":"Model"}}},
            {"path":"/cad/d1/layer/0","attributes":{"ifccad::layer":{"name":"0","appearance":{"color":"#ffffff","opacity":1.0,"linePattern":"/cad/d1/linePattern/0","lineWeight":0.25}}}},
            {"path":"/cad/d1/e1","attributes":{"ifccad::entity":{"layer":"/cad/d1/layer/0","appearance":{"color":{"mode":"ByLayer"},"opacity":{"mode":"ByLayer"},"linePattern":{"mode":"ByLayer"},"lineWeight":{"mode":"ByLayer"}}},"ifccad::geom::circle":{"radius":2.0},"ifccad::geom::placement":{"origin":[0,0,0],"xAxis":[1,0,0],"yAxis":[0,1,0]}}},
            {"path":"/cad/d1/e2","attributes":{"ifccad::entity":{"layer":"/cad/d1/layer/0","appearance":{"color":{"mode":"ByLayer"},"opacity":{"mode":"ByLayer"},"linePattern":{"mode":"ByLayer"},"lineWeight":{"mode":"ByLayer"}}},"ifccad::geom::lineSegment":{"start":[0,0,0],"end":[1,0,0]}}}
        ]
    });
    value["data"][0]["children"]["linePattern0"] = json!("/cad/d1/linePattern/0");
    value["data"].as_array_mut().unwrap().push(json!({"path":"/cad/d1/linePattern/0","attributes":{"ifccad::linePattern":{"name":"Continuous","pattern":[]}}}));
    let module: Value = serde_json::from_str(include_str!(
        "../schemas/ifcx-native-cad/experimental-profile-0.1.0.ifcx"
    ))
    .unwrap();
    value["schemas"] = module["schemas"].clone();
    value
}

fn read(
    value: &Value,
) -> Result<ocdraw::ifcx_cad::ValidatedIfcxCad, ocdraw::ifcx_cad::IfcxCadReport> {
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
fn profile_uses_later_geometry_and_order_fragments() {
    let mut value = base();
    value["data"].as_array_mut().unwrap().push(json!({
        "path": "/cad/d1/e1",
        "attributes": {"ifccad::geom::circle": {"radius": 5.0}}
    }));
    value["data"].as_array_mut().unwrap().push(json!({
        "path": "/cad/d1/layout/1",
        "children": {"0": "/cad/d1/e1", "1": "/cad/d1/e2"}
    }));
    let loaded = read(&value).unwrap();
    assert_eq!(
        loaded
            .document()
            .model
            .entities
            .iter()
            .map(|e| e.id)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert!(matches!(
        loaded.document().model.entities[0].kind,
        IfcxCadEntityKind::Circle { radius: 5.0, .. }
    ));
}

#[test]
fn reader_policy_can_reject_the_same_cad_overwrite() {
    let mut value = base();
    value["data"].as_array_mut().unwrap().push(json!({
        "path": "/cad/d1/e1",
        "attributes": {"ifccad::geom::circle": {"radius": 5.0}}
    }));
    let bytes = serde_json::to_vec(&value).unwrap();
    let default = read_native_cad_ifcx(&bytes).unwrap();
    let explicit_later =
        read_native_cad_ifcx_with_policy(&bytes, IfcxCompositionPolicy::LaterWins).unwrap();
    assert_eq!(default.document(), explicit_later.document());
    assert!(
        read_native_cad_ifcx_with_policy(&bytes, IfcxCompositionPolicy::RejectConflicts)
            .unwrap_err()
            .errors
            .iter()
            .any(|e| e.contains("conflict at /cad/d1/e1/attributes/ifccad::geom::circle"))
    );
}

#[test]
fn profile_validates_after_later_geometry_fragment() {
    let mut value = base();
    value["data"].as_array_mut().unwrap().push(json!({
        "path": "/cad/d1/e1",
        "attributes": {"ifccad::geom::circle": {"radius": -1.0}}
    }));
    assert!(read(&value)
        .unwrap_err()
        .errors
        .iter()
        .any(|e| e.contains("invalid circle radius")));
}

#[test]
fn profile_keeps_later_foreign_attribute_fragment() {
    let mut value = base();
    value["data"]
        .as_array_mut()
        .unwrap()
        .push(json!({"path":"/project/site","attributes":{"example::tag":"before"}}));
    value["data"]
        .as_array_mut()
        .unwrap()
        .push(json!({"path":"/project/site","attributes":{"example::tag":"after"}}));
    let loaded = read(&value).unwrap();
    let foreign = loaded.raw_ifcx()["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["path"] == "/project/site")
        .unwrap();
    assert_eq!(foreign["attributes"]["example::tag"], "after");
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
    value["data"][0]["children"]["block1"] = json!("/cad/d1/block/1");
    value["data"].as_array_mut().unwrap().push(json!({"path":"/cad/d1/block/1","children":{"0":"/cad/d1/e1"},"attributes":{"ifccad::blockDefinition":{"name":"B","basePoint":[0,0,0],"insertionUnit":"mm"}}}));
    assert!(read(&value)
        .unwrap_err()
        .errors
        .iter()
        .any(|e| e.contains("owner")));
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
        line_pattern_scale: 1.,
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
        line_patterns: vec![IfcxCadLinePattern {
            id: IfcxCadLinePatternId(0),
            name: "Continuous".into(),
            description: None,
            pattern: vec![],
        }],
        line_pattern_scale: 1.,
        header: IfcxCadHeader {
            id: "ifccad-experimental-hello".into(),
            data_version: "0.1.0".into(),
            author: "OpenAEC".into(),
            timestamp: "2026-09-29T00:00:00Z".into(),
        },
        drawing_id: 1,
        length_unit: "cm".into(),
        paper_layouts: Vec::new(),
        layers: vec![
            IfcxCadLayer {
                id: 0,
                name: "0".into(),
                appearance: IfcxCadLayerAppearance {
                    color: "#ffffff".into(),
                    opacity: 1.0,
                    line_pattern: IfcxCadLinePatternId(0),
                    line_weight: 0.1,
                },
            },
            IfcxCadLayer {
                id: 1,
                name: "Construction".into(),
                appearance: IfcxCadLayerAppearance {
                    color: "#00ff00".into(),
                    opacity: 0.8,
                    line_pattern: IfcxCadLinePatternId(0),
                    line_weight: 0.25,
                },
            },
            IfcxCadLayer {
                id: 2,
                name: "Symbols".into(),
                appearance: IfcxCadLayerAppearance {
                    color: "#ff0000".into(),
                    opacity: 1.0,
                    line_pattern: IfcxCadLinePatternId(0),
                    line_weight: 0.35,
                },
            },
        ],
        model: IfcxCadLayout {
            id: 1,
            entities: vec![
                IfcxCadEntity {
                    line_pattern_scale: 1.,
                    id: 42,
                    layer_id: 1,
                    appearance: by_layer.clone(),
                    kind: IfcxCadEntityKind::LineSegment {
                        start: [0.0, 0.0, 0.0],
                        end: [50.0, 0.0, 0.0],
                    },
                },
                IfcxCadEntity {
                    line_pattern_scale: 1.,
                    id: 7,
                    layer_id: 1,
                    appearance: by_layer.clone(),
                    kind: IfcxCadEntityKind::PlanarPolyline {
                        line_pattern_generation: IfcxCadLinePatternGeneration::PerSegment,
                        vertices: vec![[0.0, 0.0], [2.0, 0.0], [2.0, 3.0]],
                        closed: false,
                        placement: placement.clone(),
                    },
                },
                instance(90, 20.0, "#0000ff"),
                IfcxCadEntity {
                    line_pattern_scale: 1.,
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
                line_pattern_scale: 1.,
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

fn nested_document() -> IfcxCadDocument {
    let mut document = fixture_document();
    let placement = IfcxCadPlacement {
        origin: [5.0, 0.0, 0.0],
        x_axis: [1.0, 0.0, 0.0],
        y_axis: [0.0, 1.0, 0.0],
    };
    document.blocks.push(IfcxCadBlockDefinition {
        id: 2,
        name: "Nested marker".into(),
        base_point: [0.0, 0.0, 0.0],
        insertion_unit: "mm".into(),
        entities: vec![
            IfcxCadEntity {
                line_pattern_scale: 1.,
                id: 101,
                layer_id: 0,
                appearance: IfcxCadEntityAppearance {
                    color: IfcxCadMode::ByBlock,
                    opacity: IfcxCadMode::Explicit(0.6),
                    line_pattern: IfcxCadMode::ByLayer,
                    line_weight: IfcxCadMode::ByLayer,
                },
                kind: IfcxCadEntityKind::BlockInstance {
                    definition_id: 1,
                    transform: IfcxCadBlockTransform {
                        placement,
                        rotation: 0.0,
                        scale: [1.0, 1.0, 1.0],
                    },
                },
            },
            IfcxCadEntity {
                line_pattern_scale: 1.,
                id: 102,
                layer_id: 1,
                appearance: IfcxCadEntityAppearance {
                    color: IfcxCadMode::ByLayer,
                    opacity: IfcxCadMode::ByLayer,
                    line_pattern: IfcxCadMode::ByLayer,
                    line_weight: IfcxCadMode::ByLayer,
                },
                kind: IfcxCadEntityKind::LineSegment {
                    start: [0.0, 0.0, 0.0],
                    end: [0.0, 1.0, 0.0],
                },
            },
        ],
    });
    for entity in &mut document.model.entities {
        if let IfcxCadEntityKind::BlockInstance { definition_id, .. } = &mut entity.kind {
            *definition_id = 2;
        }
    }
    document
}

#[test]
fn nested_blocks_keep_shared_definitions_order_and_stored_appearance() {
    let document = nested_document();
    let fixture = include_bytes!("../examples/ifcx-native-cad/hello-nested-blocks.ifcx");
    assert_paths_first(fixture);
    assert_eq!(read_native_cad_ifcx(fixture).unwrap().document(), &document);
    let bytes = write_native_cad_ifcx(&document).unwrap();
    let loaded = read_native_cad_ifcx(&bytes).unwrap();
    assert_eq!(loaded.document(), &document);
    assert_eq!(
        loaded.document().blocks[1]
            .entities
            .iter()
            .map(|entity| entity.id)
            .collect::<Vec<_>>(),
        vec![101, 102]
    );
    let nodes = loaded.raw_ifcx()["data"].as_array().unwrap();
    let outer = nodes
        .iter()
        .find(|node| node["path"] == "/cad/d1/block/2")
        .unwrap();
    assert_eq!(outer["children"]["0"], "/cad/d1/e101");
    assert_eq!(outer["children"]["1"], "/cad/d1/e102");
    let inner_instance = nodes
        .iter()
        .find(|node| node["path"] == "/cad/d1/e101")
        .unwrap();
    assert_eq!(
        inner_instance["attributes"]["ifccad::blockInstance"]["definition"],
        "/cad/d1/block/1"
    );
    assert_eq!(
        inner_instance["attributes"]["ifccad::entity"]["appearance"]["color"]["mode"],
        "ByBlock"
    );
    assert_eq!(
        inner_instance["attributes"]["ifccad::entity"]["appearance"]["opacity"],
        json!({"mode":"Explicit","value":0.6})
    );
    let inner_line = nodes
        .iter()
        .find(|node| node["path"] == "/cad/d1/e100")
        .unwrap();
    assert_eq!(
        inner_line["attributes"]["ifccad::entity"]["layer"],
        "/cad/d1/layer/0"
    );
    assert_eq!(
        inner_line["attributes"]["ifccad::entity"]["appearance"]["color"]["mode"],
        "ByBlock"
    );
    for id in [90, 91] {
        let outer_instance = nodes
            .iter()
            .find(|node| node["path"] == format!("/cad/d1/e{id}").as_str())
            .unwrap();
        assert_eq!(
            outer_instance["attributes"]["ifccad::blockInstance"]["definition"],
            "/cad/d1/block/2"
        );
    }
    if std::env::var_os("GENERATE_IFCX_NESTED_FIXTURE").is_some() {
        std::fs::write("examples/ifcx-native-cad/hello-nested-blocks.ifcx", &bytes).unwrap();
    }
}

#[test]
fn nested_block_definition_cycle_is_rejected() {
    let mut document = nested_document();
    let mut back_reference = document.blocks[1].entities[0].clone();
    back_reference.id = 103;
    if let IfcxCadEntityKind::BlockInstance { definition_id, .. } = &mut back_reference.kind {
        *definition_id = 2;
    }
    document.blocks[0].entities.push(back_reference);
    assert!(write_native_cad_ifcx(&document)
        .unwrap_err()
        .errors
        .iter()
        .any(|error| error.contains("cycle")));
}

fn paper_layout_file() -> Value {
    let mut value: Value =
        serde_json::from_slice(&write_native_cad_ifcx(&nested_document()).unwrap()).unwrap();
    value["data"][0]["children"]["paper2"] = json!("/cad/d1/layout/2");
    value["data"][0]["children"]["paper3"] = json!("/cad/d1/layout/3");
    value["data"].as_array_mut().unwrap().extend([
        json!({"path":"/cad/d1/layout/2","children":{"0":"/cad/d1/e201","1":"/cad/d1/e202"},"attributes":{"ifccad::layout":{"kind":"Paper","name":"A3","paper":{"width":297.0,"height":420.0,"lengthUnit":"mm"}}}}),
        json!({"path":"/cad/d1/layout/3","children":{"0":"/cad/d1/e203"},"attributes":{"ifccad::layout":{"kind":"Paper","name":"Letter","paper":{"width":8.5,"height":11.0,"lengthUnit":"in"}},"example::note":"retained"}}),
        json!({"path":"/cad/d1/e201","attributes":{"ifccad::entity":{"layer":"/cad/d1/layer/1","appearance":{"color":{"mode":"ByLayer"},"opacity":{"mode":"ByLayer"},"linePattern":{"mode":"ByLayer"},"lineWeight":{"mode":"ByLayer"}}},"ifccad::geom::lineSegment":{"start":[10,10,0],"end":[287,10,0]}}}),
        json!({"path":"/cad/d1/e202","attributes":{"ifccad::entity":{"layer":"/cad/d1/layer/2","appearance":{"color":{"mode":"Explicit","value":"#ff00ff"},"opacity":{"mode":"ByLayer"},"linePattern":{"mode":"ByLayer"},"lineWeight":{"mode":"ByLayer"}}},"ifccad::blockInstance":{"definition":"/cad/d1/block/2","transform":{"placement":{"origin":[20,20,0],"xAxis":[1,0,0],"yAxis":[0,1,0]},"rotation":0,"scale":[10,10,10]}}}}),
        json!({"path":"/cad/d1/e203","attributes":{"ifccad::entity":{"layer":"/cad/d1/layer/1","appearance":{"color":{"mode":"ByLayer"},"opacity":{"mode":"ByLayer"},"linePattern":{"mode":"ByLayer"},"lineWeight":{"mode":"ByLayer"}}},"ifccad::geom::circle":{"radius":0.5},"ifccad::geom::placement":{"origin":[1,1,0],"xAxis":[1,0,0],"yAxis":[0,1,0]}}}),
    ]);
    value
}

#[test]
fn paper_layouts_accept_distinct_units_and_shared_block_definitions() {
    let loaded = read(&paper_layout_file()).unwrap();
    assert_eq!(loaded.document().length_unit, "cm");
    assert_eq!(loaded.document().model.entities.len(), 5);
    assert_eq!(loaded.document().blocks.len(), 2);
    let paper = &loaded.document().paper_layouts;
    assert_eq!(paper.len(), 2);
    assert_eq!((paper[0].id, paper[0].name.as_str()), (2, "A3"));
    assert_eq!(
        (paper[0].paper.width, paper[0].paper.height),
        (297.0, 420.0)
    );
    assert_eq!(paper[0].paper.length_unit, "mm");
    assert_eq!(paper[1].paper.length_unit, "in");
    assert_eq!(
        paper[0].entities.iter().map(|e| e.id).collect::<Vec<_>>(),
        vec![201, 202]
    );
    assert_eq!(paper[1].entities[0].id, 203);
    let IfcxCadEntityKind::BlockInstance {
        definition_id,
        transform,
    } = &paper[0].entities[1].kind
    else {
        panic!("paper block instance")
    };
    assert_eq!(*definition_id, 2);
    assert_eq!(transform.scale, [10.0; 3]);
    assert_eq!(transform.placement.origin, [20.0, 20.0, 0.0]);
    let raw_paper = loaded.raw_ifcx()["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["path"] == "/cad/d1/layout/3")
        .unwrap();
    assert_eq!(raw_paper["attributes"]["example::note"], "retained");
}

#[test]
fn writer_roundtrips_paper_layouts_without_unit_conversion() {
    let document = read(&paper_layout_file()).unwrap().document().clone();
    let bytes = write_native_cad_ifcx(&document).unwrap();
    assert_paths_first(&bytes);
    assert_eq!(read_native_cad_ifcx(&bytes).unwrap().document(), &document);
    let mut reversed = document.clone();
    reversed.paper_layouts.reverse();
    assert_eq!(write_native_cad_ifcx(&reversed).unwrap(), bytes);
    if std::env::var_os("GENERATE_IFCX_PAPER_FIXTURE").is_some() {
        std::fs::write("examples/ifcx-native-cad/hello-paper-layouts.ifcx", bytes).unwrap();
    }
}

#[test]
fn paper_layout_fixture_is_strictly_readable() {
    let fixture = include_bytes!("../examples/ifcx-native-cad/hello-paper-layouts.ifcx");
    assert_paths_first(fixture);
    let expected = read(&paper_layout_file()).unwrap().document().clone();
    assert_eq!(read_native_cad_ifcx(fixture).unwrap().document(), &expected);
}

#[test]
fn paper_layouts_allow_empty_scopes_but_require_one_model() {
    let mut value = paper_layout_file();
    value["data"]
        .as_array_mut()
        .unwrap()
        .retain(|node| node["path"] != "/cad/d1/e203");
    let paper = value["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["path"] == "/cad/d1/layout/3")
        .unwrap();
    paper["children"] = json!({});
    assert!(read(&value).unwrap().document().paper_layouts[1]
        .entities
        .is_empty());
    value["data"][1]["attributes"]["ifccad::layout"] =
        json!({"kind":"Paper","name":"Extra","paper":{"width":1,"height":1,"lengthUnit":"mm"}});
    assert!(read(&value).unwrap_err().errors[0].contains("missing Model"));
    let mut value = paper_layout_file();
    let paper = value["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["path"] == "/cad/d1/layout/3")
        .unwrap();
    paper["attributes"]["ifccad::layout"] = json!({"kind":"Model"});
    assert!(read(&value).unwrap_err().errors[0].contains("exactly one Model"));
}

#[test]
fn paper_layouts_require_metadata_and_one_owner() {
    for (field, replacement) in [
        ("name", json!("  ")),
        ("paper", json!(null)),
        ("paper", json!({"width":0,"height":420,"lengthUnit":"mm"})),
        ("paper", json!({"width":297,"height":-1,"lengthUnit":"mm"})),
        (
            "paper",
            json!({"width":297,"height":420,"lengthUnit":"unitless"}),
        ),
        (
            "paper",
            json!({"width":297,"height":420,"lengthUnit":"unknown"}),
        ),
        ("kind", json!("Unknown")),
    ] {
        let mut value = paper_layout_file();
        // Locate by identity so this test does not depend on writer node order.
        let node = value["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|node| node["path"] == "/cad/d1/layout/2")
            .unwrap();
        node["attributes"]["ifccad::layout"][field] = replacement;
        assert!(read(&value).is_err(), "invalid paper {field}");
    }
    let mut value = paper_layout_file();
    value["data"][0]["children"]
        .as_object_mut()
        .unwrap()
        .remove("paper2");
    assert!(read(&value).is_err());
    let mut value = paper_layout_file();
    let node = value["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["path"] == "/cad/d1/layout/2")
        .unwrap();
    node["children"]["0"] = json!("/cad/d1/e101");
    assert!(read(&value).is_err());
    let mut value = paper_layout_file();
    value["data"][1]["attributes"]["ifccad::layout"]["paper"] =
        json!({"width":297,"height":420,"lengthUnit":"mm"});
    assert!(read(&value).is_err());
}

#[test]
fn writer_rejects_duplicate_layout_ids() {
    let mut document = read(&paper_layout_file()).unwrap().document().clone();
    document.paper_layouts[0].id = document.model.id;
    assert!(write_native_cad_ifcx(&document).is_err());
    document.paper_layouts[0].id = document.paper_layouts[1].id;
    assert!(write_native_cad_ifcx(&document).is_err());
}

#[test]
fn roundtrip_complete_example_and_determinism() {
    let document = fixture_document();
    let bytes = write_native_cad_ifcx(&document).unwrap();
    assert_paths_first(&bytes);
    let file: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(file["schemas"].as_object().unwrap().is_empty());
    assert_eq!(file["imports"][0]["uri"], "urn:example:ifccad:0.1.0");
    assert_eq!(read_native_cad_ifcx(&bytes).unwrap().document(), &document);
    assert_eq!(write_native_cad_ifcx(&document).unwrap(), bytes);
    if std::env::var_os("GENERATE_IFCX_FIXTURE").is_some() {
        std::fs::write("examples/ifcx-native-cad/hello-cad.ifcx", &bytes).unwrap();
    }
}

#[test]
fn writer_uses_unwrapped_cad_paths_and_references() {
    let bytes = write_native_cad_ifcx(&fixture_document()).unwrap();
    assert!(!bytes.windows(2).any(|pair| pair == b"</"));
    let file: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(file["data"][0]["path"], "/cad/d1");
    assert_eq!(file["data"][0]["children"]["model"], "/cad/d1/layout/1");
    assert_eq!(file["data"][1]["children"]["2"], "/cad/d1/e90");
}

#[test]
fn profile_rejects_usd_wrapped_cad_path() {
    let mut value = base();
    value["data"][0]["path"] = json!("</cad/d1>");
    assert!(read(&value)
        .unwrap_err()
        .errors
        .iter()
        .any(|error| error.contains("invalid CAD path")));
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
    value["data"][0]["children"]["block1"] = json!("/cad/d1/block/1");
    value["data"].as_array_mut().unwrap().push(json!({"path":"/cad/d1/block/1","children":{"0":"/cad/d1/e3"},"attributes":{"ifccad::blockDefinition":{"name":"Loop","basePoint":[0,0,0],"insertionUnit":"mm"}}}));
    value["data"].as_array_mut().unwrap().push(json!({"path":"/cad/d1/e3","attributes":{"ifccad::entity":{"layer":"/cad/d1/layer/0","appearance":{"color":{"mode":"ByLayer"},"opacity":{"mode":"ByLayer"},"linePattern":{"mode":"ByLayer"},"lineWeight":{"mode":"ByLayer"}}},"ifccad::blockInstance":{"definition":"/cad/d1/block/1","transform":{"placement":{"origin":[0,0,0],"xAxis":[1,0,0],"yAxis":[0,1,0]},"rotation":0,"scale":[1,1,1]}}}}));
    assert!(read(&value)
        .unwrap_err()
        .errors
        .iter()
        .any(|e| e.contains("cycle")));
}

#[test]
fn profile_schema_and_extensions() {
    let mut value = base();
    value["data"].as_array_mut().unwrap().push(json!({"path":"/project/site","children":{"cad":"/cad/d1/e1"},"attributes":{"example::tag":{"value":"kept"}}}));
    value["data"][3]["attributes"]["example::entityNote"] = json!(42);
    let loaded = read(&value).unwrap();
    assert_eq!(loaded.raw_ifcx()["data"].as_array().unwrap().len(), 7);
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
    value["imports"] = json!([{"uri":"urn:example:ifccad:0.1.0"}]);
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
            line_pattern_scale: 1.,
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
