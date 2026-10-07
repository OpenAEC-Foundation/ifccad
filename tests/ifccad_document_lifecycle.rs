use ocdraw::ifccad::*;
use serde_json::{json, Value};

const HELLO: &[u8] = include_bytes!("../examples/ifccad/hello-cad.ifcx");

#[test]
fn encoder_validation_retains_original_report() {
    use std::error::Error;

    let mut document = load_ifccad_bytes(HELLO, Default::default())
        .unwrap()
        .into_document();
    document.header.id.clear();
    let expected = validate_ifccad_document(&document).unwrap_err();
    assert_eq!(expected.errors, ["incomplete IFCX header"]);
    let error = encode_ifccad_document(&document).unwrap_err();
    let IfccadEncodeError::InvalidDocument(report) = &error else {
        panic!("expected logical validation phase");
    };
    assert_eq!(report, &expected);
    assert_eq!(error.report(), Some(&expected));
    assert_eq!(
        error.source().unwrap().downcast_ref::<IfccadReport>(),
        Some(&expected)
    );
}

fn foreign_source() -> Vec<u8> {
    let mut source: Value = serde_json::from_slice(HELLO).unwrap();
    source["imports"]
        .as_array_mut()
        .unwrap()
        .push(json!({"uri":"urn:example:foreign"}));
    source["schemas"]["example::note"] = json!({"dataType":"String"});
    let nodes = source["data"].as_array_mut().unwrap();
    nodes.push(json!({"path":"/cad/d1/e1","attributes":{"example::note":"authored fragment"}}));
    nodes.push(json!({"path":"foreign","attributes":{"example::counter":9007199254740993_u64},"children":{"drawing":"/cad/d1"}}));
    serde_json::to_vec_pretty(&source).unwrap()
}

#[test]
fn source_snapshot_retains_fragments_foreign_content_and_policy() {
    let bytes = foreign_source();
    for policy in [
        IfccadCompositionPolicy::LaterWins,
        IfccadCompositionPolicy::RejectConflicts,
    ] {
        let loaded = load_ifccad_bytes(
            &bytes,
            IfccadReadOptions {
                composition_policy: policy,
            },
        )
        .unwrap();
        let source = loaded.graph();
        assert_eq!(source.source_bytes(), bytes);
        assert_eq!(source.composition_policy(), policy);
        let original: Value = serde_json::from_slice(source.source_bytes()).unwrap();
        let count = |root: &Value| {
            root["data"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|n| n["path"] == "/cad/d1/e1")
                .count()
        };
        assert_eq!(count(&original), 2);
        assert_eq!(count(source.composed_ifcx()), 1);
        let nodes = source.composed_ifcx()["data"].as_array().unwrap();
        let line = nodes.iter().find(|n| n["path"] == "/cad/d1/e1").unwrap();
        assert_eq!(line["attributes"]["example::note"], "authored fragment");
        let foreign = nodes.iter().find(|n| n["path"] == "foreign").unwrap();
        assert_eq!(
            foreign["attributes"]["example::counter"].as_u64(),
            Some(9007199254740993)
        );
        assert_eq!(foreign["children"]["drawing"], "/cad/d1");
        assert_eq!(
            source.composed_ifcx()["imports"].as_array().unwrap().len(),
            2
        );
        assert_eq!(
            source.composed_ifcx()["schemas"]["example::note"]["dataType"],
            "String"
        );
        assert_eq!(loaded.graph().composed_ifcx(), source.composed_ifcx());
    }
}

#[test]
fn extracted_document_edits_do_not_change_source_snapshot() {
    let bytes = foreign_source();
    let loaded = load_ifccad_bytes(&bytes, Default::default()).unwrap();
    let expected = loaded.document().clone();
    let (source, mut document) = loaded.into_parts();
    let before = source.composed_ifcx().clone();
    let entity = document
        .model
        .entities
        .iter_mut()
        .find(|e| e.id == 1)
        .unwrap();
    let IfccadEntityKind::LineSegment { end, .. } = &mut entity.kind else {
        panic!("fixture line")
    };
    end[0] += 10.;
    let next = document.id_counters.next_entity_id;
    assert_eq!(document.id_counters.allocate_entity_id().unwrap(), next);
    assert_ne!(document, expected);
    assert_eq!(source.composed_ifcx(), &before);
    assert_eq!(source.source_bytes(), bytes);
    assert_eq!(
        load_ifccad_bytes(&bytes, Default::default())
            .unwrap()
            .into_document(),
        expected
    );
}

fn paper_document() -> IfccadDocument {
    load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-paper-layouts.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document()
}

#[test]
fn invalid_typed_documents_fail_without_encoding() {
    let base = paper_document();
    let mut cases = Vec::new();
    let mut add = |name, edit: fn(&mut IfccadDocument)| {
        let mut document = base.clone();
        edit(&mut document);
        cases.push((name, document));
    };
    add("header", |d| d.header.author.clear());
    add("unit", |d| d.length_unit = "unknown".into());
    add("layer identity", |d| d.layers.push(d.layers[0].clone()));
    add("layout identity", |d| d.paper_layouts[0].id = d.model.id);
    add("block identity", |d| d.blocks.push(d.blocks[0].clone()));
    add("entity identity", |d| {
        d.model.entities.push(d.model.entities[0].clone())
    });
    add("entity in different owners", |d| {
        d.blocks[0].entities.push(d.model.entities[0].clone())
    });
    add("missing layer", |d| d.model.entities[0].layer_id = 900);
    add("missing layer pattern", |d| {
        d.layers[0].appearance.line_pattern = IfccadLinePatternId(900)
    });
    add("missing entity pattern", |d| {
        d.model.entities[0].appearance.line_pattern = IfccadMode::Explicit(IfccadLinePatternId(900))
    });
    add("missing block", |d| {
        d.model.entities[0].kind = IfccadEntityKind::BlockInstance {
            definition_id: 900,
            transform: transform(),
        }
    });
    add("block cycle", |d| {
        d.blocks[0].entities[0].kind = IfccadEntityKind::BlockInstance {
            definition_id: d.blocks[0].id,
            transform: transform(),
        }
    });
    add("nonfinite line", |d| {
        d.model.entities[0].kind = IfccadEntityKind::LineSegment {
            start: [f64::NAN, 0., 0.],
            end: [1., 0., 0.],
        }
    });
    add("circle radius", |d| {
        d.model.entities[0].kind = IfccadEntityKind::Circle {
            radius: 0.,
            placement: placement(),
        }
    });
    add("polyline vertices", |d| {
        d.model.entities[0].kind = {
            let vertices: Vec<[f64; 2]> = vec![[0., 0.]];
            IfccadEntityKind::PlanarPolyline {
                bulges: vec![0.; vertices.len()],
                vertices,
                closed: false,
                placement: placement(),
                line_pattern_generation: IfccadLinePatternGeneration::PerSegment,
            }
        }
    });
    add("placement", |d| {
        let mut p = placement();
        p.y_axis = p.x_axis;
        d.model.entities[0].kind = IfccadEntityKind::Circle {
            radius: 2.,
            placement: p,
        };
    });
    add("block scale", |d| {
        let mut t = transform();
        t.scale[0] = 0.;
        d.model.entities[0].kind = IfccadEntityKind::BlockInstance {
            definition_id: d.blocks[0].id,
            transform: t,
        };
    });
    add("block rotation", |d| {
        let mut t = transform();
        t.rotation = f64::INFINITY;
        d.model.entities[0].kind = IfccadEntityKind::BlockInstance {
            definition_id: d.blocks[0].id,
            transform: t,
        };
    });
    add("entity appearance", |d| {
        d.model.entities[0].appearance.opacity = IfccadMode::Explicit(2.)
    });
    add("layer appearance", |d| {
        d.layers[0].appearance.color = "red".into()
    });
    add("layer name", |d| d.layers[0].name.clear());
    add("paper dimensions", |d| {
        d.paper_layouts[0].settings.media.as_mut().unwrap().width = -1.
    });
    add("paper unit", |d| {
        d.paper_layouts[0].settings.media.as_mut().unwrap().unit =
            ocdraw::ifccad::IfccadMediaUnit::Physical(
                ocdraw::geometry_kernel::CoordinateLengthUnit::Unitless,
            )
    });
    add("paper name", |d| d.paper_layouts[0].name = " ".into());
    add("block unit", |d| {
        d.blocks[0].insertion_unit = "unknown".into()
    });
    add("block base", |d| d.blocks[0].base_point[2] = f64::INFINITY);
    add("block name", |d| d.blocks[0].name.clear());
    add("watermark", |d| d.id_counters.next_entity_id = 0);
    add("drawing pattern scale", |d| d.line_pattern_scale = 0.);
    add("entity pattern scale", |d| {
        d.model.entities[0].line_pattern_scale = f64::NAN
    });
    add("pattern definition", |d| {
        d.line_patterns[0].pattern = vec![f64::INFINITY]
    });
    for (name, document) in cases {
        let report = validate_ifccad_document(&document).expect_err(name);
        assert!(!report.errors.is_empty(), "{name} needs diagnostics");
        assert!(
            encode_ifccad_document(&document).is_err(),
            "encoder accepted {name}"
        );
    }
}

fn placement() -> IfccadPlacement {
    IfccadPlacement {
        origin: [0.; 3],
        x_axis: [1., 0., 0.],
        y_axis: [0., 1., 0.],
    }
}
fn transform() -> IfccadBlockTransform {
    IfccadBlockTransform {
        placement: placement(),
        rotation: 0.,
        scale: [1.; 3],
    }
}

#[test]
fn typed_document_encoding_keeps_history_and_supported_semantics() {
    for fixture in [
        HELLO,
        include_bytes!("../examples/ifccad/hello-line-patterns.ifcx"),
        include_bytes!("../examples/ifccad/hello-nested-blocks.ifcx"),
        include_bytes!("../examples/ifccad/hello-paper-layouts.ifcx"),
    ] {
        let mut document = load_ifccad_bytes(fixture, Default::default())
            .unwrap()
            .into_document();
        document.id_counters.next_entity_id = 9007199254740993;
        document.model.entities[0].id = document.id_counters.allocate_entity_id().unwrap();
        document.model.entities.reverse();
        document.layers[0].name = "Renamed".into();
        validate_ifccad_document(&document).unwrap();
        let bytes = encode_ifccad_document(&document).unwrap();
        assert_eq!(bytes, encode_ifccad_document(&document).unwrap());
        assert_eq!(
            load_ifccad_bytes(bytes.bytes(), Default::default())
                .unwrap()
                .document(),
            &document
        );
        assert!(String::from_utf8((bytes).into_bytes())
            .unwrap()
            .contains("9007199254740994"));
    }
    let mut empty = paper_document();
    empty.model.entities.clear();
    empty.paper_layouts.clear();
    empty.blocks.clear();
    empty.layers.clear();
    empty.line_patterns.clear();
    empty.id_counters.next_entity_id = u64::MAX;
    empty.id_counters.next_block_id = u64::MAX;
    validate_ifccad_document(&empty).unwrap();
    let bytes = encode_ifccad_document(&empty).unwrap();
    assert_eq!(
        load_ifccad_bytes(bytes.bytes(), Default::default())
            .unwrap()
            .document(),
        &empty
    );
}

#[test]
fn cad_profile_encoding_does_not_claim_source_graph_writeback() {
    let bytes = foreign_source();
    let (source, mut document) = load_ifccad_bytes(&bytes, Default::default())
        .unwrap()
        .into_parts();
    document.layers[0].name = "Edited".into();
    let encoded = encode_ifccad_document(&document).unwrap();
    let rebuilt = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    assert_eq!(rebuilt.document(), &document);
    assert!(!rebuilt.graph().composed_ifcx()["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n["path"] == "foreign"));
    assert_eq!(
        rebuilt.graph().composed_ifcx()["imports"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(source.source_bytes(), bytes);
}
