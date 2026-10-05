use viewer::inspect_drawing_bytes;

#[test]
fn dxf_export_preserves_active_viewport_clip() {
    let bytes =
        include_bytes!("../../../conformance/next/ocdraw/valid/viewport-circle-clip.ocdraw.json");
    let result = viewer::export_drawing_bytes("clipped.ocdraw.json", bytes, "dxf", "AC1032");
    assert!(result["failure"].is_null(), "{result}");
    let diagnostics = result["export"]["diagnostics"].as_array().unwrap();
    assert_eq!(
        diagnostics
            .iter()
            .filter(|d| d["code"] == "DXF_VIEWPORT_CLIP_LOSS")
            .count(),
        0
    );
    let written = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        result["export"]["download"]["base64"].as_str().unwrap(),
    )
    .unwrap();
    let cad = ocdraw_convert::opencadcodec::DxfReader::from_reader(std::io::Cursor::new(written))
        .unwrap()
        .read()
        .unwrap();
    let viewport = cad
        .entities()
        .find_map(|e| match e {
            ocdraw_convert::opencadcodec::EntityType::Viewport(v) if v.id != 1 => Some(v),
            _ => None,
        })
        .unwrap();
    assert_ne!(viewport.status.to_bits() & 0x10000, 0);
    assert!(matches!(
        cad.get_entity(viewport.clip_boundary_handle),
        Some(ocdraw_convert::opencadcodec::EntityType::Circle(_))
    ));
}

#[test]
fn dwg_export_reports_spatial_generation_loss() {
    use viewer::export_drawing_bytes;
    let bytes =
        include_bytes!("../../../conformance/next/ocdraw/valid/named-line-patterns.ocdraw.json");
    for format in ["dwg", "dxf"] {
        let result = export_drawing_bytes("patterns.ocdraw.json", bytes, format, "AC1032");
        assert!(result["failure"].is_null(), "{result}");
        let diagnostics = result["export"]["diagnostics"].as_array().unwrap();
        assert_eq!(
            diagnostics
                .iter()
                .filter(|d| d["code"] == "DWG_SPATIAL_PATTERN_GENERATION_LOSS")
                .count(),
            usize::from(format == "dwg")
        );
    }
}

#[test]
fn named_line_patterns_are_inspectable() {
    let bytes = include_bytes!("../../../conformance/next/ocdraw/valid/ordered-scopes.ocdraw.json");
    let result = inspect_drawing_bytes("patterns.ocdraw.json", bytes);
    assert_eq!(
        result["presentation"]["linePatterns"][0]["name"],
        "Continuous"
    );
    assert_eq!(
        result["presentation"]["linePatterns"][0]["pattern"],
        serde_json::json!([])
    );
    assert_eq!(result["presentation"]["linePatternScale"], 1.0);
}

#[test]
fn standalone_drawing_opens_without_package_files() {
    let bytes = include_bytes!("../../../conformance/next/ocdraw/valid/empty.ocdraw.json");
    let result = inspect_drawing_bytes("empty.ocdraw.json", bytes);
    assert_eq!(result["source"]["format"], "ocdraw");
    assert_eq!(result["validation"]["strictAvailable"], true);
    assert_eq!(result["presentation"]["drawingId"], "empty-drawing");
    assert_eq!(
        result["presentation"]["layers"].as_array().unwrap().len(),
        0
    );
    assert_eq!(
        result["presentation"]["layouts"].as_array().unwrap().len(),
        1
    );
}

#[test]
fn invalid_drawing_reports_direct_reader_diagnostics() {
    let bytes =
        include_bytes!("../../../conformance/next/ocdraw/invalid/duplicate-layer-id.ocdraw.json");
    let result = inspect_drawing_bytes("invalid.ocdraw.json", bytes);
    assert_eq!(result["validation"]["strictAvailable"], false);
    assert_eq!(result["validation"]["status"], "invalid");
    assert!(result["validation"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|diagnostic| diagnostic["code"] == "DUPLICATE_ID"));
}

#[test]
fn cad_bytes_convert_directly_to_a_reopenable_drawing() {
    use viewer::inspect_cad_as_drawing_bytes;
    let cad = ocdraw_convert::opencadcodec::CadDocument::new();
    let bytes = ocdraw_convert::opencadcodec::DxfWriter::new(&cad)
        .write_to_vec()
        .unwrap();
    let result = inspect_cad_as_drawing_bytes("blank.dxf", "dxf", &bytes);
    assert_eq!(result["validation"]["strictAvailable"], true);
    assert_eq!(result["export"]["format"], "ocdraw");
    assert!(result["export"]["download"]["base64"].as_str().is_some());
}

#[test]
fn standalone_drawing_exports_to_cad_without_package_lookup() {
    use viewer::export_drawing_bytes;
    let bytes = include_bytes!("../../../conformance/next/ocdraw/valid/empty.ocdraw.json");
    let result = export_drawing_bytes("empty.ocdraw.json", bytes, "dxf", "AC1032");
    assert_eq!(result["validation"]["strictAvailable"], true);
    assert_eq!(result["export"]["format"], "dxf");
    let encoded = result["export"]["download"]["base64"].as_str().unwrap();
    let written =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded).unwrap();
    let cad = ocdraw_convert::opencadcodec::DxfReader::from_reader(std::io::Cursor::new(written))
        .unwrap()
        .read()
        .unwrap();
    assert_eq!(cad.version.as_str(), "AC1032");
}
