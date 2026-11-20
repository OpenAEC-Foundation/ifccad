use ifccad::*;
use serde_json::{json, Value};

fn source_graph(format: &str) -> Value {
    let mut graph: Value = serde_json::from_slice(include_bytes!(
        "../conformance/next/ifccad/valid-xref-relative.ifcx"
    ))
    .unwrap();
    let source = graph["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find_map(|node| node["attributes"].get_mut("ifccad::externalSource"))
        .unwrap();
    source["source"]["format"] = json!(format);
    source["location"]["uri"] = json!(format!("../Referenties/grondplan.{format}"));
    if format == "ifccad" {
        source["source"]
            .as_object_mut()
            .unwrap()
            .remove("fingerprintGuid");
        source["source"]["basePoint"] = json!([0., 0., 0.]);
    }
    graph
}

#[test]
fn dxf_and_ifccad_sources_survive_strict_json_and_cbor_readback() {
    for format in ["dxf", "ifccad"] {
        let bytes = serde_json::to_vec(&source_graph(format)).unwrap();
        let drawing = load_ifccad_bytes(&bytes, Default::default())
            .unwrap_or_else(|error| panic!("{format} source must be readable: {error}"))
            .into_document();
        validate_ifccad_document(&drawing).unwrap();
        assert_eq!(
            serde_json::to_value(&drawing.external_sources[0].source).unwrap()["format"],
            format
        );
        let json = encode_ifccad_document(&drawing).unwrap();
        let json_read = load_ifccad_bytes(json.bytes(), Default::default()).unwrap();
        let cbor = encode_ifccad_cbor_document(&drawing).unwrap();
        let cbor_read = load_ifccad_cbor_bytes(cbor.bytes(), Default::default()).unwrap();
        for read in [json_read.document(), cbor_read.document()] {
            assert_eq!(read.external_sources, drawing.external_sources);
            assert_eq!(read.model.entities, drawing.model.entities);
            assert_eq!(read.id_counters, drawing.id_counters);
        }
    }
}

#[test]
fn native_source_has_no_fingerprint_field_and_unknown_formats_are_rejected() {
    let mut graph = source_graph("ifccad");
    let source = graph["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find_map(|node| node["attributes"].get_mut("ifccad::externalSource"))
        .unwrap();
    source["source"]["fingerprintGuid"] = json!("e5b4a9af-cf20-4fc9-a30f-ffb20fd987e1");
    assert!(load_ifccad_bytes(&serde_json::to_vec(&graph).unwrap(), Default::default()).is_err());
    assert!(load_ifccad_bytes(
        &serde_json::to_vec(&source_graph("unknown")).unwrap(),
        Default::default()
    )
    .is_err());
}
