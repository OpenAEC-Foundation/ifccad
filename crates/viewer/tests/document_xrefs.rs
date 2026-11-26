use base64::{engine::general_purpose::STANDARD, Engine};
use ifccad::*;
const DRAWING: &[u8] = include_bytes!("../../../conformance/next/ifccad/valid-xref-relative.ifcx");

#[test]
fn native_inspection_exposes_authored_sources_without_resolving_files() {
    let result = viewer::inspect_ifccad_bytes("Tekeningen/hoofd.ifcx", DRAWING);
    assert!(result["failure"].is_null(), "{result}");
    assert_eq!(result["presentation"]["externalSources"][0]["id"], "1");
    assert_eq!(
        result["presentation"]["externalSources"][0]["location"]["uri"],
        "../Referenties/grondplan.dwg"
    );
    assert_eq!(
        result["presentation"]["externalSources"][0]["status"],
        "notChecked"
    );
    assert_eq!(result["presentation"]["xrefs"][0]["scale"], 500.);
    assert_eq!(result["presentation"]["xrefs"][0]["sourceId"], "1");
    let d = load_ifccad_bytes(DRAWING, Default::default()).unwrap();
    let cbor = encode_ifccad_cbor_document(d.document()).unwrap();
    let binary = viewer::inspect_ifccad_bytes("Tekeningen/hoofd.ifccad.cbor", cbor.bytes());
    assert_eq!(
        result["presentation"]["externalSources"],
        binary["presentation"]["externalSources"]
    );
}

#[test]
fn active_document_route_returns_physical_external_reference() {
    let exported = viewer::document_export_ifccad("hoofd.ifcx", DRAWING, "dwg", "AC1032", "{}");
    assert!(exported["failure"].is_null(), "{exported}");
    let base64 = exported["export"]["download"]["base64"].as_str().unwrap();
    let bytes = STANDARD.decode(base64).unwrap();
    let returned =
        viewer::document_cad_to_ifccad("hoofd.dwg", "dwg", &bytes, "2026-11-19T00:00:00Z", "{}");
    assert!(returned["failure"].is_null(), "{returned}");
    assert_eq!(
        returned["presentation"]["externalSources"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn source_formats_open_natively_but_refuse_cad_delivery() {
    for format in ["dxf", "ifccad"] {
        let mut graph: serde_json::Value = serde_json::from_slice(DRAWING).unwrap();
        let source = graph["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find_map(|node| node["attributes"].get_mut("ifccad::externalSource"))
            .unwrap();
        source["source"]["format"] = format.into();
        // The declared format must control delivery despite a DWG locator.
        if format == "ifccad" {
            source["source"]
                .as_object_mut()
                .unwrap()
                .remove("fingerprintGuid");
            source["source"]["basePoint"] = serde_json::json!([0., 0., 0.]);
        }
        let bytes = serde_json::to_vec(&graph).unwrap();
        let opened = viewer::document_open_ifccad("main.ifccad", &bytes);
        assert!(opened["failure"].is_null(), "{opened}");
        assert!(opened["preview"]["failure"].is_null(), "{opened}");
        assert_eq!(
            opened["presentation"]["externalSources"][0]["source"]["format"],
            format
        );
        for cad in ["dwg", "dxf"] {
            let exported =
                viewer::document_export_ifccad("main.ifccad", &bytes, cad, "AC1032", "{}");
            assert!(!exported["failure"].is_null(), "{exported}");
            assert!(
                exported["failure"]
                    .to_string()
                    .contains("convert the external source to DWG"),
                "{exported}"
            );
            assert!(exported["export"]["download"].is_null(), "{exported}");
        }
    }
}
