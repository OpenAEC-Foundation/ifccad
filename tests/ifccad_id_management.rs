use ocdraw::ifccad::*;
use serde_json::{json, Value};

const FIELDS: [&str; 5] = [
    "nextEntityId",
    "nextLayerId",
    "nextLayoutId",
    "nextBlockId",
    "nextLinePatternId",
];

fn graph() -> Value {
    let mut document = load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-paper-layouts.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    // Keep sparse-ID/history coverage independent of the examples' compact IDs.
    for (entity, id) in document
        .model
        .entities
        .iter_mut()
        .chain(
            document
                .paper_layouts
                .iter_mut()
                .flat_map(|p| &mut p.entities),
        )
        .chain(document.blocks.iter_mut().flat_map(|b| &mut b.entities))
        .zip([42, 7, 90, 9, 91, 201, 202, 203, 100, 101, 102])
    {
        entity.as_native_mut().unwrap().id = id;
    }
    document.id_counters = IfccadIdCounters {
        next_text_style_id: 1,
        next_preservation_record_id: 1,
        next_ucs_id: 1,
        next_model_window_id: 1,

        next_entity_id: 1000,
        next_layer_id: 100,
        next_layout_id: 100,
        next_block_id: 100,
        next_line_pattern_id: 100,
    };
    serde_json::from_slice(encode_ifccad_document(&document).unwrap().bytes()).unwrap()
}

fn drawing(graph: &mut Value) -> &mut Value {
    &mut graph["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|node| node["path"] == "/cad/d1")
        .unwrap()["attributes"]["ifccad::drawing"]
}

fn read(graph: &Value) -> Result<ValidatedIfccad, IfccadReadError> {
    load_ifccad_bytes(&serde_json::to_vec(graph).unwrap(), Default::default())
}

#[test]
fn counter_fields_are_required_and_exact_u64() {
    for field in FIELDS {
        let mut missing = graph();
        drawing(&mut missing).as_object_mut().unwrap().remove(field);
        assert!(read(&missing).is_err(), "missing {field}");
        for invalid in [json!(-1), json!(1.5), json!("1000"), Value::Null] {
            let mut malformed = graph();
            drawing(&mut malformed)[field] = invalid;
            assert!(read(&malformed).is_err(), "invalid {field}");
        }
        let mut overflow = graph();
        drawing(&mut overflow)[field] = json!(12345);
        let text = serde_json::to_string(&overflow).unwrap().replace(
            &format!("\"{field}\":12345"),
            &format!("\"{field}\":18446744073709551616"),
        );
        assert!(
            load_ifccad_bytes(text.as_bytes(), Default::default()).is_err(),
            "overflow {field}"
        );
    }
}

#[test]
fn stale_counters_cover_every_domain_and_owner() {
    for (field, maximum) in FIELDS.into_iter().zip([203, 2, 3, 2, 0]) {
        let mut stale = graph();
        drawing(&mut stale)[field] = json!(maximum);
        assert!(read(&stale).is_err(), "stale {field}");
        let mut document = read(&graph()).unwrap().document().clone();
        match field {
            "nextEntityId" => document.id_counters.next_entity_id = maximum,
            "nextLayerId" => document.id_counters.next_layer_id = maximum,
            "nextLayoutId" => document.id_counters.next_layout_id = maximum,
            "nextBlockId" => document.id_counters.next_block_id = maximum,
            "nextLinePatternId" => document.id_counters.next_line_pattern_id = maximum,
            _ => unreachable!(),
        }
        assert!(
            encode_ifccad_document(&document).is_err(),
            "writer stale {field}"
        );
    }
}

#[test]
fn block_contents_share_the_entity_domain_even_without_layout_entities() {
    let mut document = read(&graph()).unwrap().document().clone();
    document.model.entities.clear();
    for layout in &mut document.paper_layouts {
        layout.entities.clear();
    }
    // e102 exists only in the second block definition.
    document.id_counters.next_entity_id = 102;
    assert!(encode_ifccad_document(&document).is_err());
    document.id_counters.next_entity_id = 103;
    let mut saved: Value =
        serde_json::from_slice(encode_ifccad_document(&document).unwrap().bytes()).unwrap();
    drawing(&mut saved)["nextEntityId"] = json!(102);
    assert!(read(&saved).is_err());
}

#[test]
fn sparse_ids_and_advanced_counters_roundtrip() {
    let document = read(&graph()).unwrap().document().clone();
    let bytes = encode_ifccad_document(&document).unwrap();
    let mut saved: Value = serde_json::from_slice(bytes.bytes()).unwrap();
    assert_eq!(drawing(&mut saved)["nextEntityId"], json!(1000));
    for field in &FIELDS[1..] {
        assert_eq!(drawing(&mut saved)[*field], json!(100));
    }
    assert_eq!(
        load_ifccad_bytes(bytes.bytes(), Default::default())
            .unwrap()
            .document(),
        &document
    );
    assert!(saved["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n["path"] == "/cad/d1/linePattern/0"));
}

#[test]
fn drawing_fragment_replacement_validates_final_counters() {
    let mut graph = graph();
    let mut replacement = drawing(&mut graph).clone();
    replacement["nextEntityId"] = json!(2000);
    graph["data"].as_array_mut().unwrap().push(json!({
        "path": "/cad/d1", "attributes": {"ifccad::drawing": replacement}
    }));
    assert!(read(&graph).is_ok());
    assert!(load_ifccad_bytes(
        &serde_json::to_vec(&graph).unwrap(),
        IfccadReadOptions {
            composition_policy: IfccadCompositionPolicy::RejectConflicts
        }
    )
    .is_err());
    graph["data"].as_array_mut().unwrap().last_mut().unwrap()["attributes"]["ifccad::drawing"]
        ["nextEntityId"] = json!(203);
    assert!(read(&graph).is_err());
    graph["data"].as_array_mut().unwrap().last_mut().unwrap()["attributes"]["ifccad::drawing"] =
        json!({"nextEntityId":2000});
    assert!(read(&graph).is_err());
}

fn reopen(document: &IfccadDocument) -> IfccadDocument {
    load_ifccad_bytes(
        encode_ifccad_document(document).unwrap().bytes(),
        Default::default(),
    )
    .unwrap()
    .document()
    .clone()
}

#[test]
fn delete_save_reopen_allocate_never_reuses_entity_id() {
    let mut document = read(&graph()).unwrap().document().clone();
    let template = document.model.entities[0].clone();
    document.model.entities = [1, 3, 4]
        .into_iter()
        .map(|id| {
            IfccadEntity::Native(IfccadNativeEntity {
                id,
                ..template.as_native().unwrap().clone()
            })
        })
        .collect();
    document.paper_layouts.clear();
    document.blocks.clear();
    document.id_counters.next_entity_id = 5;
    document.model.entities.pop();
    let mut loaded = reopen(&document);
    assert_eq!(loaded.id_counters.next_entity_id, 5);
    let id = loaded.id_counters.allocate_entity_id().unwrap();
    assert_eq!(id, 5);
    loaded
        .model
        .entities
        .push(IfccadEntity::Native(IfccadNativeEntity {
            id,
            ..template.as_native().unwrap().clone()
        }));
    let loaded = reopen(&loaded);
    assert_eq!(loaded.id_counters.next_entity_id, 6);
    assert_eq!(
        loaded
            .model
            .entities
            .iter()
            .map(|entity| entity.id())
            .collect::<Vec<_>>(),
        [1, 3, 5]
    );
}

#[test]
fn deleted_definitions_do_not_reset_their_watermarks() {
    let document = read(&graph()).unwrap().document().clone();
    let mut layer = document.clone();
    let id = layer.id_counters.allocate_layer_id().unwrap();
    assert_eq!(id, 100);
    layer.layers.push(IfccadLayer {
        id,
        name: "Unused".into(),
        ..layer.layers[0].clone()
    });
    let mut layer = reopen(&layer);
    layer.layers.retain(|layer| layer.id != 100);
    let mut layer = reopen(&layer);
    assert_eq!(layer.id_counters.allocate_layer_id().unwrap(), 101);

    let mut pattern = document.clone();
    let id = pattern.id_counters.allocate_line_pattern_id().unwrap();
    assert_eq!(id.0, 100);
    pattern.line_patterns.push(IfccadLinePattern {
        id,
        name: "Unused".into(),
        description: None,
        pattern: vec![],
    });
    let mut pattern = reopen(&pattern);
    pattern.line_patterns.retain(|pattern| pattern.id.0 != 100);
    let mut pattern = reopen(&pattern);
    assert_eq!(
        pattern.id_counters.allocate_line_pattern_id().unwrap().0,
        101
    );

    let mut layout = document.clone();
    let id = layout.id_counters.allocate_layout_id().unwrap();
    assert_eq!(id, 100);
    layout.paper_layouts.push(IfccadPaperLayout {
        canvas: None,

        settings: ocdraw::ifccad::IfccadLayoutSettings {
            media: None,
            ..Default::default()
        },
        id,
        tab_index: 3,
        name: "Unused".into(),
        entities: vec![],
        ..layout.paper_layouts[0].clone()
    });
    let mut layout = reopen(&layout);
    layout.paper_layouts.retain(|layout| layout.id != 100);
    let mut layout = reopen(&layout);
    assert_eq!(layout.id_counters.allocate_layout_id().unwrap(), 101);

    let mut block = document;
    let id = block.id_counters.allocate_block_id().unwrap();
    assert_eq!(id, 100);
    block.blocks.push(IfccadBlockDefinition {
        id,
        name: "Unused".into(),
        entities: vec![],
        ..block.blocks[0].clone()
    });
    let mut block = reopen(&block);
    block.blocks.retain(|block| block.id != 100);
    let mut block = reopen(&block);
    assert_eq!(block.id_counters.allocate_block_id().unwrap(), 101);
}

#[test]
fn empty_domains_preserve_reserved_watermarks() {
    let mut document = read(&graph()).unwrap().document().clone();
    document.model.entities.clear();
    document.paper_layouts.clear();
    document.blocks.clear();
    document.layers.clear();
    document.line_patterns.clear();
    let loaded = reopen(&document);
    assert_eq!(loaded.id_counters, document.id_counters);
    assert_eq!(loaded.id_counters.next_entity_id, 1000);
    assert_eq!(loaded.id_counters.next_line_pattern_id, 100);
}

#[test]
fn reorder_move_copy_and_rename_preserve_identity() {
    let mut document = read(&graph()).unwrap().document().clone();
    let counters = document.id_counters;
    document.model.entities.reverse();
    let mut moved = document.model.entities.pop().unwrap();
    let id = moved.id();
    moved.as_native_mut().unwrap().layer_id = document.layers[1].id;
    moved.as_native_mut().unwrap().appearance.color = IfccadMode::Explicit("#123456".into());
    let IfccadEntityKind::LineSegment { end, .. } = &mut moved.as_native_mut().unwrap().kind else {
        panic!("line fixture")
    };
    *end = [60., 1., 0.];
    document.blocks[0].entities.push(moved.clone());
    let mut document = reopen(&document);
    assert_eq!(document.id_counters, counters);
    document.blocks[0]
        .entities
        .retain(|entity| entity.id() != id);
    document.paper_layouts[0].entities.push(moved.clone());
    document.layers[1].name = "Renamed layer".into();
    document.blocks[0].name = "Renamed block".into();
    document.header.id = "saved-under-another-name".into();
    let copy_id = document.id_counters.allocate_entity_id().unwrap();
    assert_eq!(copy_id, 1000);
    document
        .model
        .entities
        .push(IfccadEntity::Native(IfccadNativeEntity {
            id: copy_id,
            ..moved.as_native().unwrap().clone()
        }));
    let bytes = encode_ifccad_document(&document).unwrap();
    let loaded = load_ifccad_bytes(bytes.bytes(), Default::default()).unwrap();
    assert_eq!(
        loaded.document().paper_layouts[0].entities.last().unwrap(),
        &moved
    );
    assert_eq!(loaded.document().id_counters.next_entity_id, 1001);
    assert!(loaded.graph().composed_ifcx()["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|node| node["path"] == format!("/cad/d1/e{id}")));
    assert_eq!(loaded.document().model.entities.last().unwrap().id(), 1000);
}

#[test]
fn wire_roundtrip_preserves_full_width_counters_and_rejects_maximum_ids() {
    for next in [9_007_199_254_740_993, 9_223_372_036_854_775_809, u64::MAX] {
        let mut document = read(&graph()).unwrap().document().clone();
        document.id_counters = IfccadIdCounters {
            next_text_style_id: 1,
            next_preservation_record_id: 1,
            next_ucs_id: 1,
            next_model_window_id: 1,

            next_entity_id: next,
            next_layer_id: next,
            next_layout_id: next,
            next_block_id: next,
            next_line_pattern_id: next,
        };
        let bytes = encode_ifccad_document(&document).unwrap();
        let mut saved: Value = serde_json::from_slice(bytes.bytes()).unwrap();
        for field in FIELDS {
            assert_eq!(drawing(&mut saved)[field].as_u64(), Some(next));
        }
        assert_eq!(
            load_ifccad_bytes(bytes.bytes(), Default::default())
                .unwrap()
                .document(),
            &document
        );
    }
    let mut document = read(&graph()).unwrap().document().clone();
    document.model.entities[0].as_native_mut().unwrap().id = u64::MAX;
    document.id_counters.next_entity_id = u64::MAX;
    assert!(encode_ifccad_document(&document).is_err());
    document.model.entities[0].as_native_mut().unwrap().id = u64::MAX - 1;
    assert_eq!(reopen(&document).model.entities[0].id(), u64::MAX - 1);
}

#[test]
fn snapshot_validation_does_not_claim_historical_monotonicity() {
    let mut graph = graph();
    let identical = drawing(&mut graph).clone();
    graph["data"]
        .as_array_mut()
        .unwrap()
        .push(json!({"path":"/cad/d1","attributes":{"ifccad::drawing":identical}}));
    assert!(load_ifccad_bytes(
        &serde_json::to_vec(&graph).unwrap(),
        IfccadReadOptions {
            composition_policy: IfccadCompositionPolicy::RejectConflicts
        }
    )
    .is_ok());
    graph["data"].as_array_mut().unwrap().last_mut().unwrap()["attributes"]["ifccad::drawing"]
        ["nextEntityId"] = json!(500);
    assert_eq!(
        read(&graph).unwrap().document().id_counters.next_entity_id,
        500
    );
}
