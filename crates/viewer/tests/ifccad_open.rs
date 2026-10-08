use base64::{engine::general_purpose::STANDARD, Engine};
use ocdraw::ifccad::load_ifccad_bytes;
use viewer::{export_drawing_bytes, inspect_drawing_bytes};

const HELLO: &[u8] = include_bytes!("../../../examples/ifccad/hello-line-patterns.ifcx");

#[test]
fn ifccad_viewer_selection_identifies_entities_in_written_cad() {
    use ocdraw_convert::opencadcodec::{DwgReader, DxfReader, Handle};
    use std::io::Cursor;
    for format in ["dxf", "dwg"] {
        let out = export_drawing_bytes("patterns.ifcx", HELLO, format, "AC1032");
        assert!(out["failure"].is_null(), "{out}");
        let selection = &out["export"]["viewerSelection"];
        assert_eq!(selection["format"], "ifccad");
        let entries = selection["entities"].as_array().expect("selection entries");
        assert!(!entries.is_empty());
        let bytes = STANDARD
            .decode(out["export"]["download"]["base64"].as_str().unwrap())
            .unwrap();
        let cad = if format == "dxf" {
            DxfReader::from_reader(Cursor::new(bytes))
                .unwrap()
                .read()
                .unwrap()
        } else {
            DwgReader::from_stream(Cursor::new(bytes)).read().unwrap()
        };
        for entry in entries {
            assert!(entry["path"].as_str().unwrap().starts_with("/cad/d1/e"));
            assert_eq!(entry["layout"], "Model");
            let handle =
                Handle::new(u64::from_str_radix(entry["handle"].as_str().unwrap(), 16).unwrap());
            let entity = cad.get_entity(handle).expect("mapped written entity");
            assert_eq!(
                entity.common().owner_handle,
                cad.header.model_space_block_handle
            );
        }
    }
}

#[test]
fn viewer_selection_preserves_large_native_identities_as_paths() {
    use ocdraw::ifccad::encode_ifccad_document;
    let mut drawing = load_ifccad_bytes(HELLO, Default::default())
        .unwrap()
        .into_document();
    drawing.drawing_id = 9_007_199_254_740_995;
    drawing.model.entities[0].as_native_mut().unwrap().id = 9_007_199_254_740_993;
    drawing.id_counters.next_entity_id = 9_007_199_254_740_994;
    let bytes = encode_ifccad_document(&drawing).unwrap();
    let output = export_drawing_bytes("large.ifcx", bytes.bytes(), "dxf", "AC1032");
    assert!(output["failure"].is_null(), "{output}");
    assert!(output["export"]["viewerSelection"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["path"] == "/cad/d9007199254740995/e9007199254740993"));
}

#[test]
fn ifccad_geometry_and_bounds_are_inspectable() {
    let bytes = include_bytes!("../../../examples/ifccad/hello-geometry.ifcx");
    let opened = inspect_drawing_bytes("geometry.ifcx", bytes);
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
    for format in ["dxf", "dwg"] {
        let out = export_drawing_bytes("geometry.ifcx", bytes, format, "AC1032");
        assert!(out["failure"].is_null(), "{out}");
        assert!(out["export"]["geometryAssessment"]["domains"].is_array());
        assert_eq!(out["export"]["fileCheck"]["ifccadStrictReadback"], true);
        assert!(out["export"]["fileCheck"]["geometryAssessment"]["domains"].is_array());
    }
}

#[test]
fn conversion_evidence_retains_domains_and_large_ids() {
    use ocdraw::ifccad::*;
    let mut d = load_ifccad_bytes(
        include_bytes!("../../../examples/ifccad/hello-viewports.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    d.length_unit = "mm".into();
    let config: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../conformance/next/ifccad/valid/layout-plot-inch.ifcx"
    ))
    .unwrap();
    let config =
        load_ifccad_bytes(&serde_json::to_vec(&config).unwrap(), Default::default()).unwrap();
    d.paper_layouts[0].settings = config.document().paper_layouts[0].settings.clone();
    let mut e = d.paper_layouts[0].entities[0].clone();
    e.as_native_mut().unwrap().id = 9_007_199_254_740_993;
    e.as_native_mut().unwrap().kind = IfccadEntityKind::PlanarPolyline {
        vertices: vec![[1., 0.], [2., 0.]],
        bulges: vec![0., 0.],
        closed: false,
        line_pattern_generation: IfccadLinePatternGeneration::PerSegment,
        placement: IfccadPlacement {
            origin: [1e20, 0., 0.],
            x_axis: [1., 0., 0.],
            y_axis: [0., 1., 0.],
        },
    };
    d.model.entities.push(e);
    d.id_counters.next_entity_id = 9_007_199_254_740_994;
    let bytes = encode_ifccad_document(&d).unwrap();
    let failed = export_drawing_bytes("precision.ifcx", bytes.bytes(), "dxf", "AC1032");
    assert_eq!(
        failed["failure"]["geometry"]["source"]["entityId"],
        "9007199254740993"
    );
    assert_eq!(failed["failure"]["geometry"]["domain"]["kind"], "Drawing");
    assert!(failed["failure"]["geometry"]["reason"].is_string());
    assert!(failed["export"]["download"].is_null());
    let IfccadEntityKind::PlanarPolyline { placement, .. } =
        &mut d.model.entities[0].as_native_mut().unwrap().kind
    else {
        unreachable!()
    };
    placement.origin = [1000000000000., 0., 0.];
    if let IfccadEntityKind::PlanarPolyline { vertices, .. } =
        &mut d.model.entities[0].as_native_mut().unwrap().kind
    {
        vertices[0][0] = 0.0001;
    }
    let bytes = encode_ifccad_document(&d).unwrap();
    let rejected = export_drawing_bytes("round.ifcx", bytes.bytes(), "dxf", "AC1032");
    assert_eq!(
        rejected["failure"]["geometry"]["reason"],
        "ProvenExceedance"
    );
    let out = viewer::export_drawing_bytes_with_options(
        "round.ifcx",
        bytes.bytes(),
        "dxf",
        "AC1032",
        r#"{"tolerance":{"mode":"custom","value":0.001,"unit":"mm"}}"#,
    );
    assert!(out["failure"].is_null(), "{out}");
    let assessment = &out["export"]["geometryAssessment"];
    assert_eq!(assessment["status"], "RoundedWithinTolerance");
    assert_eq!(
        assessment["domains"][0]["worstEntity"]["entityId"],
        "9007199254740993"
    );
    assert_eq!(assessment["domains"].as_array().unwrap().len(), 2);
    assert_eq!(assessment["domains"][0]["coordinateMeaning"]["unit"], "mm");
    assert_eq!(
        assessment["domains"][1]["coordinateMeaning"]["kind"],
        "PaperCoordinates"
    );
    assert_eq!(
        assessment["domains"][1]["coordinateMeaning"]["physicalOutputFactor"]["numerator"],
        "127"
    );
    assert!(
        assessment["domains"][0]["resolvedTolerance"]["upper"]
            .as_f64()
            .unwrap()
            > assessment["domains"][1]["resolvedTolerance"]["upper"]
                .as_f64()
                .unwrap()
    );
}

#[test]
fn ifccad_native_download_preserves_large_counter_bytes() {
    use ocdraw::ifccad::{encode_ifccad_document, IfccadIdCounters};
    let mut document = load_ifccad_bytes(HELLO, Default::default())
        .unwrap()
        .document()
        .clone();
    document.id_counters = IfccadIdCounters {
        next_preservation_record_id: 1,
        next_entity_id: 9_007_199_254_740_993,
        next_layer_id: 9_223_372_036_854_775_809,
        next_layout_id: u64::MAX,
        next_block_id: u64::MAX,
        next_line_pattern_id: u64::MAX,
    };
    let bytes = encode_ifccad_document(&document).unwrap();
    let inspected = inspect_drawing_bytes("large.ifcx", bytes.bytes());
    assert_eq!(inspected["validation"]["strictAvailable"], true);
    let exported = export_drawing_bytes("large.ifcx", bytes.bytes(), "ifccad", "AC1032");
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
        let result = export_drawing_bytes("legacy.ifcx", &bytes, format, "AC1032");
        assert_eq!(result["validation"]["strictAvailable"], false);
        assert!(result["export"]["download"].is_null());
    }
}

#[test]
fn ifccad_uses_its_own_reader_and_shows_composed_nodes() {
    let result = inspect_drawing_bytes("hello.ifcx", HELLO);
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
    let result = export_drawing_bytes("hello.ifcx.json", &bytes, "ifccad", "AC1032");
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
    let dxf = export_drawing_bytes("hello.ifcx", &bytes, "dxf", "AC1032");
    assert_eq!(dxf["failure"], serde_json::Value::Null);
    assert!(dxf["export"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["code"] == "foreign-ifcx"));
}

#[test]
fn ifccad_exports_real_dxf_and_dwg_with_diagnostics_and_strict_native_readback() {
    for format in ["dxf", "dwg"] {
        let result = export_drawing_bytes("hello.ifcx", HELLO, format, "AC1032");
        assert_eq!(result["failure"], serde_json::Value::Null, "{result}");
        assert_eq!(result["export"]["fileCheck"]["ifccadStrictReadback"], true);
        assert!(result["export"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == "appearance"));
        let bytes = STANDARD
            .decode(result["export"]["download"]["base64"].as_str().unwrap())
            .unwrap();
        let cad = if format == "dxf" {
            ocdraw_convert::opencadcodec::DxfReader::from_reader(std::io::Cursor::new(bytes))
                .unwrap()
                .read()
                .unwrap()
        } else {
            ocdraw_convert::opencadcodec::DwgReader::from_stream(std::io::Cursor::new(bytes))
                .read()
                .unwrap()
        };
        let metadata = ifccad_convert::IfccadTargetMetadata {
            header: load_ifccad_bytes(HELLO, Default::default())
                .unwrap()
                .document()
                .header
                .clone(),
            drawing_id: 1,
        };
        let restored =
            ifccad_convert::cad_document_to_encoded_ifccad(&cad, metadata, Default::default())
                .unwrap();
        assert_eq!(
            restored.validated_source().document().line_patterns.len(),
            3
        );
        assert!(restored
            .validated_source()
            .document()
            .line_patterns
            .iter()
            .any(|p| p.name == "DashDot" && p.pattern == [0.5, -0.25, 0., -0.25]));
    }
}

#[test]
fn dwg_nonzero_base_passes_readback_and_download_retains_diagnostics() {
    let result = export_drawing_bytes("hello.ifcx", HELLO, "dwg", "AC1032");
    assert!(result["failure"].is_null(), "{result}");
    assert_eq!(result["export"]["fileCheck"]["ifccadStrictReadback"], true);
    let bytes = STANDARD
        .decode(result["export"]["download"]["base64"].as_str().unwrap())
        .unwrap();
    let cad = ocdraw_convert::opencadcodec::DwgReader::from_stream(std::io::Cursor::new(bytes))
        .read()
        .unwrap();
    let original = load_ifccad_bytes(HELLO, Default::default()).unwrap();
    for block in &original.document().blocks {
        let record = cad.block_records.get(&block.name).unwrap();
        assert_eq!(
            record.base_point,
            ocdraw_convert::opencadcodec::Vector3::new(
                block.base_point[0],
                block.base_point[1],
                block.base_point[2]
            )
        );
    }
    assert!(!result["export"]["diagnostics"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn invalid_ifcx_never_yields_export_bytes() {
    for format in ["ifccad", "dxf", "dwg"] {
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
        viewer::inspect_cad_as_ifccad_bytes("back.dxf", "dxf", &bytes, "2026-10-01T00:00:00Z");
    assert_eq!(
        converted["validation"]["strictAvailable"], true,
        "{converted}"
    );
    assert_eq!(converted["presentation"]["format"], "ifccad");
    assert!(converted["conversion"]["geometryAssessment"]["domains"].is_array());
    assert_eq!(converted["export"]["download"]["format"], "ifccad");
    assert!(converted["export"]["download"]["fileName"]
        .as_str()
        .unwrap()
        .ends_with(".ifcx"));
    let native = STANDARD
        .decode(converted["export"]["download"]["base64"].as_str().unwrap())
        .unwrap();
    let loaded = load_ifccad_bytes(&native, Default::default()).unwrap();
    assert_eq!(loaded.document().header.timestamp, "2026-10-01T00:00:00Z");
    assert_eq!(loaded.document().model.entities.len(), 5);
}
