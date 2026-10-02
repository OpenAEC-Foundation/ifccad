use base64::{engine::general_purpose::STANDARD, Engine};
use ocdraw::ifcx_cad::read_native_cad_ifcx;
use ocdraw_viewer::{export_drawing_bytes, inspect_drawing_bytes};

const HELLO: &[u8] = include_bytes!("../../../examples/ifcx-native-cad/hello-line-patterns.ifcx");

#[test]
fn ifcx_native_download_preserves_large_counter_bytes() {
    use ocdraw::ifcx_cad::{write_native_cad_ifcx, IfcxCadIdCounters};
    let mut document = read_native_cad_ifcx(HELLO).unwrap().document().clone();
    document.id_counters = IfcxCadIdCounters {
        next_entity_id: 9_007_199_254_740_993,
        next_layer_id: 9_223_372_036_854_775_809,
        next_layout_id: u64::MAX,
        next_block_id: u64::MAX,
        next_line_pattern_id: u64::MAX,
    };
    let bytes = write_native_cad_ifcx(&document).unwrap();
    let inspected = inspect_drawing_bytes("large.ifcx", &bytes);
    assert_eq!(inspected["validation"]["strictAvailable"], true);
    let exported = export_drawing_bytes("large.ifcx", &bytes, "ifcx", "AC1032");
    let returned = STANDARD
        .decode(exported["export"]["download"]["base64"].as_str().unwrap())
        .unwrap();
    assert_eq!(returned, bytes);
    assert_eq!(
        read_native_cad_ifcx(&returned)
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
    for format in ["ifcx", "dxf", "dwg"] {
        let result = export_drawing_bytes("legacy.ifcx", &bytes, format, "AC1032");
        assert_eq!(result["validation"]["strictAvailable"], false);
        assert!(result["export"]["download"].is_null());
    }
}

#[test]
fn ifcx_uses_its_own_reader_and_shows_composed_nodes() {
    let result = inspect_drawing_bytes("hello.ifcx", HELLO);
    assert_eq!(result["failure"], serde_json::Value::Null);
    assert_eq!(result["source"]["format"], "ifcx");
    assert_eq!(result["validation"]["strictAvailable"], true);
    assert_eq!(result["presentation"]["format"], "ifcx");
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
        .any(|n| n["path"] == "/cad/d1/linePattern/7"));
}

#[test]
fn ifcx_download_retains_original_fragments_and_foreign_information() {
    let mut graph: serde_json::Value = serde_json::from_slice(HELLO).unwrap();
    graph["data"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "path":"/cad/d1/e42", "attributes":{"example::note":"separate fragment"}
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
    let result = export_drawing_bytes("hello.ifcx.json", &bytes, "ifcx", "AC1032");
    assert_eq!(result["failure"], serde_json::Value::Null);
    assert_eq!(
        STANDARD
            .decode(result["export"]["download"]["base64"].as_str().unwrap())
            .unwrap(),
        bytes
    );
    let loaded = read_native_cad_ifcx(&bytes).unwrap();
    assert_eq!(loaded.graph().source_bytes(), bytes);
    let nodes = result["presentation"]["graph"]["data"].as_array().unwrap();
    assert_eq!(
        nodes.iter().filter(|n| n["path"] == "/cad/d1/e42").count(),
        1
    );
    assert!(nodes.iter().any(|n| n["path"] == "/foreign"));
    let dxf = export_drawing_bytes("hello.ifcx", &bytes, "dxf", "AC1032");
    assert_eq!(dxf["failure"], serde_json::Value::Null);
    assert!(dxf["export"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["code"] == "foreign-ifcx"));
}

#[test]
fn ifcx_exports_real_dxf_and_dwg_with_diagnostics_and_strict_native_readback() {
    for format in ["dxf", "dwg"] {
        let mut graph: serde_json::Value = serde_json::from_slice(HELLO).unwrap();
        if format == "dwg" {
            // The pinned codec does not support consistent nonzero BLOCK base markers.
            graph["data"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|n| n["path"] == "/cad/d1/block/1")
                .unwrap()["attributes"]["ifccad::blockDefinition"]["basePoint"] =
                serde_json::json!([0., 0., 0.]);
        }
        let result = export_drawing_bytes(
            "hello.ifcx",
            &serde_json::to_vec(&graph).unwrap(),
            format,
            "AC1032",
        );
        assert_eq!(result["failure"], serde_json::Value::Null, "{result}");
        assert_eq!(result["export"]["fileCheck"]["ifcxStrictReadback"], true);
        assert!(result["export"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == "appearance"));
        let bytes = STANDARD
            .decode(result["export"]["download"]["base64"].as_str().unwrap())
            .unwrap();
        let cad = if format == "dxf" {
            ocdraw_convert::cadcodec::DxfReader::from_reader(std::io::Cursor::new(bytes))
                .unwrap()
                .read()
                .unwrap()
        } else {
            ocdraw_convert::cadcodec::DwgReader::from_stream(std::io::Cursor::new(bytes))
                .read()
                .unwrap()
        };
        let metadata = ifcx_cad_convert::IfcxCadTargetMetadata {
            header: read_native_cad_ifcx(HELLO)
                .unwrap()
                .document()
                .header
                .clone(),
            drawing_id: 1,
        };
        let restored = ifcx_cad_convert::cad_document_to_ifcx_cad(&cad, metadata).unwrap();
        assert_eq!(restored.validated_ifcx().document().line_patterns.len(), 3);
        assert!(restored
            .validated_ifcx()
            .document()
            .line_patterns
            .iter()
            .any(|p| p.name == "DashDot" && p.pattern == [0.5, -0.25, 0., -0.25]));
    }
}

#[test]
fn dwg_nonzero_base_readback_failure_blocks_download_and_retains_diagnostics() {
    let result = export_drawing_bytes("hello.ifcx", HELLO, "dwg", "AC1032");
    assert_eq!(result["failure"]["stage"], "checking");
    assert!(result["failure"]["message"]
        .as_str()
        .unwrap()
        .contains("BLOCK marker"));
    assert!(result["export"]["download"].is_null());
    assert!(!result["export"]["diagnostics"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn invalid_ifcx_never_yields_export_bytes() {
    for format in ["ifcx", "dxf", "dwg"] {
        let result = export_drawing_bytes("invalid.ifcx", b"{}", format, "AC1032");
        assert_eq!(result["validation"]["strictAvailable"], false);
        assert!(result["export"]["download"].is_null());
    }
}

#[test]
fn cad_input_can_be_downloaded_and_reopened_as_ifcx_with_caller_timestamp() {
    let output = export_drawing_bytes("hello.ifcx", HELLO, "dxf", "AC1032");
    let bytes = STANDARD
        .decode(output["export"]["download"]["base64"].as_str().unwrap())
        .unwrap();
    let converted =
        ocdraw_viewer::inspect_cad_as_ifcx_bytes("back.dxf", "dxf", &bytes, "2026-10-01T00:00:00Z");
    assert_eq!(
        converted["validation"]["strictAvailable"], true,
        "{converted}"
    );
    assert_eq!(converted["presentation"]["format"], "ifcx");
    assert_eq!(converted["export"]["download"]["format"], "ifcx");
    let native = STANDARD
        .decode(converted["export"]["download"]["base64"].as_str().unwrap())
        .unwrap();
    let loaded = read_native_cad_ifcx(&native).unwrap();
    assert_eq!(loaded.document().header.timestamp, "2026-10-01T00:00:00Z");
    assert_eq!(loaded.document().model.entities.len(), 5);
}
