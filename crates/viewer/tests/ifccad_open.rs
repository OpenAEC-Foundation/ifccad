use base64::{engine::general_purpose::STANDARD, Engine};
use ifccad::load_ifccad_bytes;
use viewer::inspect_ifccad_bytes;

fn export(name: &str, bytes: &[u8], format: &str, version: &str) -> serde_json::Value {
    viewer::document_export_ifccad(name, bytes, format, version, "{}")
}

#[test]
fn native_ifccad_text_and_independent_bounds_evidence_are_inspectable() {
    let opened = inspect_ifccad_bytes(
        "text.ifcx",
        include_bytes!("../../../examples/ifccad/text.ifcx"),
    );
    assert_eq!(opened["validation"]["status"], "valid");
    assert_eq!(opened["presentation"]["textEntityCount"], 3);
    assert_eq!(opened["presentation"]["mTextEntityCount"], 3);
    assert_eq!(
        opened["presentation"]["textStyles"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let scope = opened["presentation"]["boundsCompleteness"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["scopePath"] == "/cad/d1/layout/1")
        .unwrap();
    assert_eq!(scope["quality"], "estimated");
    assert_eq!(scope["enclosureVerified"], false);
    assert_eq!(scope["safeForNegativeQuery"], false);
}

const HELLO: &[u8] = include_bytes!("../../../tests/fixtures/ifccad/hello-line-patterns.ifcx");

#[test]
fn ifccad_geometry_and_bounds_are_inspectable() {
    let bytes = include_bytes!("../../../tests/fixtures/ifccad/hello-geometry.ifcx");
    let opened = inspect_ifccad_bytes("geometry.ifcx", bytes);
    assert_eq!(opened["validation"]["status"], "valid");
    assert!(opened["presentation"]["layouts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n["attributes"]["ifccad::layout"]["bounds"]["max"]
            == serde_json::json!([10000, 10000, 10000])));
    for kind in [
        "point",
        "arc",
        "ellipse",
        "ellipseArc",
        "planarPolyline",
        "spatialPolyline",
    ] {
        assert!(opened["presentation"]["entities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n["attributes"]
                .get(format!("ifccad::geom::{kind}"))
                .is_some()));
    }
}

#[test]
fn ifccad_native_download_preserves_large_counter_bytes() {
    use ifccad::{encode_ifccad_document, IfccadIdCounters};
    let mut document = load_ifccad_bytes(HELLO, Default::default())
        .unwrap()
        .document()
        .clone();
    document.id_counters = IfccadIdCounters {
        next_dimension_style_id: 1,
        next_symbol_source_id: 1,
        next_text_style_id: 1,
        next_preservation_record_id: 1,
        next_ucs_id: 1,
        next_model_window_id: 1,

        next_entity_id: 9_007_199_254_740_993,
        next_layer_id: 9_223_372_036_854_775_809,
        next_layout_id: u64::MAX,
        next_block_id: u64::MAX,
        next_line_pattern_id: u64::MAX,
    };
    let bytes = encode_ifccad_document(&document).unwrap();
    let inspected = inspect_ifccad_bytes("large.ifcx", bytes.bytes());
    assert_eq!(inspected["validation"]["strictAvailable"], true);
    let exported = export("large.ifcx", bytes.bytes(), "ifccad", "AC1032");
    let returned = STANDARD
        .decode(exported["export"]["download"]["base64"].as_str().unwrap())
        .unwrap();
    assert_eq!(returned, bytes.bytes());
    assert_eq!(
        load_ifccad_bytes(&returned, Default::default())
            .unwrap()
            .document()
            .id_counters,
        document.id_counters
    );
}

#[test]
fn missing_counter_never_yields_native_or_cad_export_bytes() {
    let mut graph: serde_json::Value = serde_json::from_slice(HELLO).unwrap();
    graph["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|node| node["path"] == "/cad/d1")
        .unwrap()["attributes"]["ifccad::drawing"]
        .as_object_mut()
        .unwrap()
        .remove("nextEntityId");
    let bytes = serde_json::to_vec(&graph).unwrap();
    for format in ["ifccad", "dxf", "dwg"] {
        let result = export("legacy.ifcx", &bytes, format, "AC1032");
        assert_eq!(result["validation"]["strictAvailable"], false);
        assert!(result["export"]["download"].is_null());
    }
}

#[test]
fn ifccad_uses_its_own_reader_and_shows_composed_nodes() {
    let result = inspect_ifccad_bytes("hello.ifcx", HELLO);
    assert_eq!(result["failure"], serde_json::Value::Null);
    assert_eq!(result["source"]["format"], "ifccad");
    assert_eq!(result["validation"]["strictAvailable"], true);
    assert_eq!(result["presentation"]["format"], "ifccad");
    assert_eq!(
        result["presentation"]["linePatterns"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert!(result["presentation"]["graph"]["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|n| n["attributes"]["ifccad::linePattern"]["name"] == "DashDot"));
}

#[test]
fn ifccad_download_retains_original_fragments_and_foreign_information() {
    let mut graph: serde_json::Value = serde_json::from_slice(HELLO).unwrap();
    graph["data"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "path":"/cad/d1/e1", "attributes":{"example::note":"separate fragment"}
        }));
    graph["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["path"] == "/cad/d1")
        .unwrap()["attributes"]["ifccad::drawing"]["nextEntityId"] =
        serde_json::json!(9007199254740993_u64);
    graph["data"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"path":"/foreign","attributes":{"example::value":42}}));
    let bytes = serde_json::to_vec_pretty(&graph).unwrap();
    let result = export("hello.ifcx.json", &bytes, "ifccad", "AC1032");
    assert_eq!(result["failure"], serde_json::Value::Null);
    assert_eq!(
        STANDARD
            .decode(result["export"]["download"]["base64"].as_str().unwrap())
            .unwrap(),
        bytes
    );
    let loaded = load_ifccad_bytes(&bytes, Default::default()).unwrap();
    assert_eq!(loaded.graph().source_bytes(), bytes);
    let nodes = result["presentation"]["graph"]["data"].as_array().unwrap();
    assert_eq!(
        nodes.iter().filter(|n| n["path"] == "/cad/d1/e1").count(),
        1
    );
    assert!(nodes.iter().any(|n| n["path"] == "/foreign"));
}

#[test]
fn invalid_ifcx_never_yields_export_bytes() {
    for format in ["ifccad", "dxf", "dwg"] {
        let result = export("invalid.ifcx", b"{}", format, "AC1032");
        assert_eq!(result["validation"]["strictAvailable"], false);
        assert!(result["export"]["download"].is_null());
    }
}
