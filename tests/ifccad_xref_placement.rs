use ifccad::*;
use serde_json::{json, Value};

fn placed_source(scale: [f64; 3]) -> Value {
    let mut graph: Value =
        serde_json::from_slice(include_bytes!("fixtures/ifccad/xref-source-formats.ifcx")).unwrap();
    for node in graph["data"].as_array_mut().unwrap() {
        let Some(attrs) = node["attributes"].as_object_mut() else {
            continue;
        };
        if attrs.contains_key("ifccad::xref") {
            attrs.remove("ifccad::geom::placement");
            let xref = attrs["ifccad::xref"].as_object_mut().unwrap();
            xref.remove("scale");
            xref.insert("transform".into(), json!({"placement":{"origin":[10,20,30],"xAxis":[1,0,0],"yAxis":[0,0,-1]},"rotation":0.5,"scale":scale}));
        }
    }
    graph
}

#[test]
fn signed_nonuniform_oblique_xrefs_survive_strict_json_and_cbor() {
    for scale in [[-2., 3., 4.], [2., -3., 4.], [2., 3., -4.], [-2., -3., 4.]] {
        let input = serde_json::to_vec(&placed_source(scale)).unwrap();
        let loaded = load_ifccad_bytes(&input, Default::default());
        assert!(
            loaded.is_ok(),
            "new XREF transform rejected: {:?}",
            loaded.err()
        );
        let drawing = loaded.unwrap().into_document();
        validate_ifccad_document(&drawing).unwrap();
        let json = encode_ifccad_document(&drawing).unwrap();
        assert_eq!(
            load_ifccad_bytes(json.bytes(), Default::default())
                .unwrap()
                .document(),
            &drawing
        );
        let cbor = encode_ifccad_cbor_document(&drawing).unwrap();
        assert_eq!(
            load_ifccad_cbor_bytes(cbor.bytes(), Default::default())
                .unwrap()
                .document(),
            &drawing
        );
    }
}

fn strict_roundtrip(d: &IfccadDocument) {
    validate_ifccad_document(d).unwrap();
    let json = encode_ifccad_document(d).unwrap();
    assert_eq!(
        load_ifccad_bytes(json.bytes(), Default::default())
            .unwrap()
            .document(),
        d
    );
    let cbor = encode_ifccad_cbor_document(d).unwrap();
    assert_eq!(
        load_ifccad_cbor_bytes(cbor.bytes(), Default::default())
            .unwrap()
            .document(),
        d
    );
}

#[test]
fn paper_and_reused_local_block_share_one_source_and_keep_bounds_unknown() {
    let mut d = load_ifccad_bytes(
        include_bytes!("fixtures/ifccad/hello-paper-layouts.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let source_d = load_ifccad_bytes(
        &serde_json::to_vec(&placed_source([-2., 3., 4.])).unwrap(),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let mut source = source_d.external_sources[0].clone();
    source.id = d.id_counters.allocate_external_source_id().unwrap();
    let source_id = source.id;
    d.external_sources.push(source);
    let mut template = d.model.entities[0].as_native().unwrap().clone();
    let IfccadEntityKind::Xref(mut xref) =
        source_d.model.entities[0].as_native().unwrap().kind.clone()
    else {
        panic!()
    };
    xref.source_id = source_id;
    template.kind = IfccadEntityKind::Xref(xref);
    let mut entity = || {
        let mut e = template.clone();
        e.id = d.id_counters.allocate_entity_id().unwrap();
        IfccadEntity::Native(e)
    };
    let block_xref = entity();
    let block_id = d.id_counters.allocate_block_id().unwrap();
    d.blocks.push(IfccadBlockDefinition {
        annotation: Default::default(),
        description: String::new(),
        anonymous: false,
        explodable: true,
        uniform_scaling: false,
        bounds_quality: None,
        id: block_id,
        name: "local-xref".into(),
        base_point: [1., 2., 3.],
        insertion_unit: d.length_unit.clone(),
        bounds: None,
        entities: vec![block_xref],
    });
    recompute_ifccad_document_bounds(&mut d).unwrap();
    assert!(
        d.model.bounds.is_some(),
        "unused external definition must not spoil Model bounds"
    );
    assert!(d.blocks.last().unwrap().bounds.is_none());
    for layout in &mut d.paper_layouts {
        let mut e = template.clone();
        e.id = d.id_counters.allocate_entity_id().unwrap();
        layout.entities = vec![IfccadEntity::Native(e)];
        layout.bounds = None;
        layout.bounds_quality = None;
    }
    assert_eq!(d.paper_layouts.len(), 2);
    for (origin, scale) in [
        ([100., 0., 0.], [2., 3., 4.]),
        ([0., 100., 0.], [-2., 3., 4.]),
    ] {
        let mut e = template.clone();
        e.id = d.id_counters.allocate_entity_id().unwrap();
        e.kind = IfccadEntityKind::BlockInstance {
            definition_id: block_id,
            attributes: vec![],
            annotation: None,
            transform: IfccadBlockTransform {
                placement: IfccadPlacement {
                    origin,
                    x_axis: [1., 0., 0.],
                    y_axis: [0., 1., 0.],
                },
                rotation: std::f64::consts::FRAC_PI_4,
                scale,
            },
        };
        d.model.entities.push(IfccadEntity::Native(e));
    }
    recompute_ifccad_document_bounds(&mut d).unwrap();
    assert!(d.model.bounds.is_none());
    assert!(d.paper_layouts.iter().all(|p| p.bounds.is_none()));
    let mut direct = template.clone();
    direct.id = d.id_counters.allocate_entity_id().unwrap();
    d.model.entities.push(IfccadEntity::Native(direct));
    recompute_ifccad_document_bounds(&mut d).unwrap();
    strict_roundtrip(&d);
    let graph: Value = serde_json::from_slice(encode_ifccad_document(&d).unwrap().bytes()).unwrap();
    let count = |kind: &str| {
        graph["data"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["attributes"].get(kind).is_some())
            .count()
    };
    assert_eq!(count("ifccad::externalSource"), 1);
    assert_eq!(count("ifccad::xref"), 4);
    assert_eq!(
        graph["data"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["attributes"]["ifccad::blockInstance"]["definition"]
                == format!("/cad/d{}/block/{block_id}", d.drawing_id))
            .count(),
        2
    );
}

#[test]
fn legacy_duplicate_and_invalid_transform_fields_are_rejected() {
    let base = placed_source([2., 3., 4.]);
    for mutation in [
        json!({"scale":3}),
        json!({"transform":null}),
        json!({"transform":{"placement":{"origin":[0,0,0],"xAxis":[1,0,0],"yAxis":[0,1,0]},"scale":[0,1,1]}}),
        json!({"transform":{"placement":{"origin":[0,0,0],"xAxis":[2,0,0],"yAxis":[0,1,0]}}}),
    ] {
        let mut v = base.clone();
        let n = v["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["attributes"].get("ifccad::xref").is_some())
            .unwrap();
        for (k, value) in mutation.as_object().unwrap() {
            n["attributes"]["ifccad::xref"][k] = value.clone();
        }
        assert!(load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).is_err());
    }
    let mut v = base.clone();
    let n = v["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"].get("ifccad::xref").is_some())
        .unwrap();
    n["attributes"]["ifccad::geom::placement"] =
        json!({"origin":[0,0,0],"xAxis":[1,0,0],"yAxis":[0,1,0]});
    assert!(load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).is_err());
}

#[test]
fn transform_defaults_and_native_nonfinite_values_follow_block_rules() {
    let mut graph = placed_source([1.; 3]);
    for node in graph["data"].as_array_mut().unwrap() {
        if let Some(x) = node["attributes"].get_mut("ifccad::xref") {
            x["transform"].as_object_mut().unwrap().remove("rotation");
            x["transform"].as_object_mut().unwrap().remove("scale");
        }
    }
    let d = load_ifccad_bytes(&serde_json::to_vec(&graph).unwrap(), Default::default())
        .unwrap()
        .into_document();
    strict_roundtrip(&d);
    for scale in [
        [f64::NAN, 1., 1.],
        [1., f64::INFINITY, 1.],
        [1., 0., 1.],
        [1., 1., -0.],
    ] {
        let mut bad = d.clone();
        let IfccadEntityKind::Xref(x) = &mut bad.model.entities[0].as_native_mut().unwrap().kind
        else {
            panic!()
        };
        x.transform.scale = scale;
        assert!(validate_ifccad_document(&bad).is_err());
        assert!(encode_ifccad_cbor_document(&bad).is_err());
    }
    for angle in [f64::NAN, f64::INFINITY] {
        let mut bad = d.clone();
        let IfccadEntityKind::Xref(x) = &mut bad.model.entities[0].as_native_mut().unwrap().kind
        else {
            panic!()
        };
        x.transform.rotation = angle;
        assert!(validate_ifccad_document(&bad).is_err());
    }
}
