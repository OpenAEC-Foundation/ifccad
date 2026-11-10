use ifccad::*;

fn document() -> IfccadDocument {
    load_ifccad_bytes(
        include_bytes!("fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document()
}

#[test]
fn unused_scales_and_allocation_history_survive_json_and_cbor() {
    let mut document = document();
    let first = document.id_counters.allocate_annotation_scale_id().unwrap();
    document.annotation_scales.push(IfccadAnnotationScale {
        id: first,
        name: "Saved designation".into(),
        paper_units: 2.0,
        drawing_units: 100.0,
        unit_scale: true,
    });
    let abandoned = document.id_counters.allocate_annotation_scale_id().unwrap();
    assert_eq!(abandoned.0, first.0 + 1);
    let high = (1u64 << 53) + 1;
    document.annotation_scales.push(IfccadAnnotationScale {
        id: IfccadAnnotationScaleId(high),
        name: "Same ratio, separate identity".into(),
        paper_units: 1.0,
        drawing_units: 50.0,
        unit_scale: false,
    });
    document.id_counters.next_annotation_scale_id = high + 7;
    validate_ifccad_document(&document).unwrap();
    let json = encode_ifccad_document(&document).unwrap();
    let cbor = encode_ifccad_cbor_document(&document).unwrap();
    let json_read = load_ifccad_bytes(json.bytes(), Default::default())
        .unwrap()
        .into_document();
    let cbor_read = load_ifccad_cbor_bytes(cbor.bytes(), Default::default())
        .unwrap()
        .into_document();
    for read in [json_read, cbor_read] {
        assert_eq!(read.annotation_scales, document.annotation_scales);
        assert_eq!(read.id_counters.next_annotation_scale_id, high + 7);
    }
    assert_eq!(
        cbor.bytes()[15],
        8,
        "mapping revision advances with the contract"
    );
    let wire: serde_json::Value = serde_json::from_slice(json.bytes()).unwrap();
    assert_eq!(wire["imports"][0]["uri"], "urn:example:ifccad:0.8.0");
}

#[test]
fn shared_document_validation_rejects_duplicate_scales_and_stale_watermarks() {
    let mut document = document();
    document.annotation_scales.push(IfccadAnnotationScale {
        id: IfccadAnnotationScaleId(1),
        name: "Authored".into(),
        paper_units: 1.0,
        drawing_units: 20.0,
        unit_scale: false,
    });
    assert!(validate_ifccad_document(&document).is_err());
    document.id_counters.next_annotation_scale_id = 2;
    validate_ifccad_document(&document).unwrap();
    document
        .annotation_scales
        .push(document.annotation_scales[0].clone());
    assert!(encode_ifccad_document(&document).is_err());
    assert!(encode_ifccad_cbor_document(&document).is_err());
}

#[test]
fn unused_extreme_scale_is_storable_without_computing_its_quotient() {
    let mut document = document();
    document.annotation_scales.push(IfccadAnnotationScale {
        id: IfccadAnnotationScaleId(1),
        name: "Finite components".into(),
        paper_units: f64::from_bits(1),
        drawing_units: f64::MAX,
        unit_scale: false,
    });
    document.id_counters.next_annotation_scale_id = 2;
    let json = encode_ifccad_document(&document).unwrap();
    let cbor = encode_ifccad_cbor_document(&document).unwrap();
    assert_eq!(
        load_ifccad_bytes(json.bytes(), Default::default())
            .unwrap()
            .document()
            .annotation_scales,
        document.annotation_scales
    );
    assert_eq!(
        load_ifccad_cbor_bytes(cbor.bytes(), Default::default())
            .unwrap()
            .document()
            .annotation_scales,
        document.annotation_scales
    );
}

#[test]
fn scale_nodes_reject_null_unknown_fields_and_unlisted_or_noncanonical_identity() {
    let mut document = document();
    document.annotation_scales.push(IfccadAnnotationScale {
        id: IfccadAnnotationScaleId(1),
        name: "Stored".into(),
        paper_units: 1.0,
        drawing_units: 20.0,
        unit_scale: false,
    });
    document.id_counters.next_annotation_scale_id = 2;
    let source: serde_json::Value =
        serde_json::from_slice(encode_ifccad_document(&document).unwrap().bytes()).unwrap();
    let index = source["data"]
        .as_array()
        .unwrap()
        .iter()
        .position(|node| node["attributes"].get("ifccad::annotationScale").is_some())
        .unwrap();
    for (name, value) in [
        ("unitScale", serde_json::Value::Null),
        ("unknown", serde_json::json!(true)),
        ("paperUnits", serde_json::json!(0.0)),
    ] {
        let mut invalid = source.clone();
        invalid["data"][index]["attributes"]["ifccad::annotationScale"][name] = value;
        assert!(
            load_ifccad_bytes(&serde_json::to_vec(&invalid).unwrap(), Default::default()).is_err(),
            "{name}"
        );
    }
    let mut unlisted = source.clone();
    unlisted["data"][0]["children"]
        .as_object_mut()
        .unwrap()
        .remove("annotationScale1");
    assert!(
        load_ifccad_bytes(&serde_json::to_vec(&unlisted).unwrap(), Default::default()).is_err()
    );
    let mut noncanonical = source.clone();
    let path = format!("/cad/d{}/annotationScale/01", document.drawing_id);
    noncanonical["data"][index]["path"] = serde_json::json!(path);
    noncanonical["data"][0]["children"]["annotationScale1"] = serde_json::json!(path);
    assert!(load_ifccad_bytes(
        &serde_json::to_vec(&noncanonical).unwrap(),
        Default::default()
    )
    .is_err());
}
