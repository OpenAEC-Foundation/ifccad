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
        reference_type: IfccadXrefReferenceType::Attach,
    });
    d.model.entities.push(IfccadEntity::Native(entity));
    d.model.bounds = None;
    d.model.bounds_quality = None;
    d
}

#[test]
fn xref_reference_types_json_cbor_strict_readback() {
    let mut d = xref_document();
    let mut overlay = d
        .model
        .entities
        .last()
        .unwrap()
        .as_native()
        .unwrap()
        .clone();
    overlay.id = d.id_counters.allocate_entity_id().unwrap();
    let IfccadEntityKind::Xref(x) = &mut overlay.kind else {
        panic!()
    };
    x.reference_type = IfccadXrefReferenceType::Overlay;
    x.placement.origin[0] += 100.;
    d.model.entities.push(IfccadEntity::Native(overlay));
    validate_ifccad_document(&d).unwrap();
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
    let mut graph: serde_json::Value = serde_json::from_slice(json.bytes()).unwrap();
    let nodes = graph["data"].as_array_mut().unwrap();
    let attach = nodes
        .iter_mut()
        .find(|n| {
            n["attributes"]["ifccad::xref"].is_object()
                && n["attributes"]["ifccad::xref"]
                    .get("referenceType")
                    .is_none()
        })
        .unwrap();
    attach["attributes"]["ifccad::xref"]["referenceType"] = serde_json::json!("attach");
    assert_eq!(
        load_ifccad_bytes(&serde_json::to_vec(&graph).unwrap(), Default::default())
            .unwrap()
            .document(),
        &d
    );
    let mut misplaced = graph.clone();
    let source = misplaced["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"]["ifccad::externalSource"].is_object())
        .unwrap();
    source["attributes"]["ifccad::externalSource"]["referenceType"] = serde_json::json!("overlay");
    assert!(
        load_ifccad_bytes(&serde_json::to_vec(&misplaced).unwrap(), Default::default()).is_err()
    );
    for bad in [serde_json::Value::Null, serde_json::json!("other")] {
        let mut invalid = graph.clone();
        let node = invalid["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["attributes"]["ifccad::xref"].is_object())
            .unwrap();
        node["attributes"]["ifccad::xref"]["referenceType"] = bad;
        assert!(
            load_ifccad_bytes(&serde_json::to_vec(&invalid).unwrap(), Default::default()).is_err()
        );
    }
}
