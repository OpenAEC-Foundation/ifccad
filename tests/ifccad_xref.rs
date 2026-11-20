use ifccad::*;

fn document() -> IfccadDocument {
    load_ifccad_bytes(
        include_bytes!("fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document()
}

fn xref_document() -> IfccadDocument {
    let mut d = document();
    d.external_sources.push(IfccadExternalSource {
        id: d.id_counters.allocate_external_source_id().unwrap(),
        name: "grondplan".into(),
        location: IfccadExternalSourceLocation::UriReference {
            uri: "../Referenties/grondplan.dwg".into(),
        },
        source: IfccadExternalSourceContent::Dwg {
            base_point: [10., 20., 0.],
            insertion_unit: Some("m".into()),
            fingerprint_guid: Some("e5b4a9af-cf20-4fc9-a30f-ffb20fd987e1".into()),
        },
    });
    let mut entity = d.model.entities[0].as_native().unwrap().clone();
    entity.id = d.id_counters.allocate_entity_id().unwrap();
    entity.kind = IfccadEntityKind::Xref(IfccadXref {
        source_id: d.external_sources[0].id,
        placement: IfccadPlacement {
            origin: [10000., 5000., 0.],
            x_axis: [0., 1., 0.],
            y_axis: [-1., 0., 0.],
        },
        scale: 500.,
    });
    d.model.entities.push(IfccadEntity::Native(entity));
    d.model.bounds = None;
    d.model.bounds_quality = None;
    d
}

#[test]
fn xref_placement_is_explicit_and_source_optional_on_disk() {
    let d = xref_document();
    validate_ifccad_document(&d).unwrap();
    let IfccadEntityKind::Xref(x) = &d.model.entities.last().unwrap().as_native().unwrap().kind
    else {
        panic!()
    };
    let point = |p: [f64; 3]| -> [f64; 3] {
        std::array::from_fn(|i| {
            x.placement.origin[i]
                + x.scale
                    * (x.placement.x_axis[i] * (p[0] - 10.) + x.placement.y_axis[i] * (p[1] - 20.))
        })
    };
    assert_eq!(point([10., 20., 0.]), [10000., 5000., 0.]);
    assert_eq!(point([12., 20., 0.]), [10000., 6000., 0.]);
    for scale in [0., -1., f64::INFINITY, f64::NAN] {
        let mut bad = d.clone();
        let IfccadEntityKind::Xref(x) = &mut bad
            .model
            .entities
            .last_mut()
            .unwrap()
            .as_native_mut()
            .unwrap()
            .kind
        else {
            panic!()
        };
        x.scale = scale;
        assert!(validate_ifccad_document(&bad).is_err());
    }
    let mut bad = d.clone();
    bad.external_sources.clear();
    assert!(validate_ifccad_document(&bad).is_err());
}

#[test]
fn unresolved_xref_forbids_complete_bounds() {
    let mut d = xref_document();
    recompute_ifccad_document_bounds(&mut d).unwrap();
    assert!(d.model.bounds.is_none());
    d.model.bounds = Some(IfccadBounds3d {
        min: [-1e6; 3],
        max: [1e6; 3],
    });
    d.model.bounds_quality = Some(IfccadBoundsQuality::Enclosing);
    assert!(validate_ifccad_document(&d).is_err());
}

#[test]
fn source_and_xref_wire_closure_and_owner_are_strict() {
    let d = xref_document();
    let encoded = encode_ifccad_document(&d).unwrap();
    let graph: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    let index = graph["data"]
        .as_array()
        .unwrap()
        .iter()
        .position(|n| n["attributes"].get("ifccad::externalSource").is_some())
        .unwrap();
    for (field, value) in [
        ("/location/fileName", serde_json::json!("mixed.dwg")),
        ("/location/uri", serde_json::json!("bad%XX")),
        ("/source/fingerprintGuid", serde_json::json!("bad-guid")),
        ("/source/insertionUnit", serde_json::Value::Null),
        ("/extra", serde_json::json!(true)),
    ] {
        let mut bad = graph.clone();
        let source = &mut bad["data"][index]["attributes"]["ifccad::externalSource"];
        let parts: Vec<_> = field.trim_start_matches('/').split('/').collect();
        if parts.len() == 1 {
            source[parts[0]] = value;
        } else {
            source[parts[0]][parts[1]] = value;
        }
        assert!(
            load_ifccad_bytes(&serde_json::to_vec(&bad).unwrap(), Default::default()).is_err(),
            "{field}"
        );
    }
    let mut bad = d.clone();
    let x = bad.model.entities.pop().unwrap();
    bad.paper_layouts = document().paper_layouts;
    // The fixture has no Paper; a block is another disallowed owner.
    bad.blocks.push(IfccadBlockDefinition {
        annotation: Default::default(),
        description: String::new(),
        anonymous: false,
        explodable: true,
        uniform_scaling: false,
        bounds_quality: None,
        id: bad.id_counters.allocate_block_id().unwrap(),
        name: "local".into(),
        base_point: [0.; 3],
        insertion_unit: "mm".into(),
        bounds: None,
        entities: vec![x],
    });
    assert!(validate_ifccad_document(&bad)
        .unwrap_err()
        .to_string()
        .contains("IFCCAD-XREF-002"));
}

#[test]
fn native_and_cbor_keep_xref_with_missing_source() {
    let mut d = xref_document();
    let mut unused = d.external_sources[0].clone();
    d.id_counters.next_external_source_id = 9_007_199_254_740_993;
    unused.id = d.id_counters.allocate_external_source_id().unwrap();
    unused.name = "unused".into();
    unused.location = IfccadExternalSourceLocation::SearchFileName {
        file_name: "missing.dwg".into(),
    };
    d.external_sources.push(unused);
    let json = encode_ifccad_document(&d).unwrap();
    assert_eq!(
        load_ifccad_bytes(json.bytes(), Default::default())
            .unwrap()
            .document(),
        &d
    );
    let cbor = encode_ifccad_cbor_document(&d).unwrap();
    assert_eq!(
        load_ifccad_cbor_bytes(cbor.bytes(), Default::default())
            .unwrap()
            .document(),
        &d
    );
}
