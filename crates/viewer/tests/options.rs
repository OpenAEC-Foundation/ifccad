#[test]
fn custom_tolerance_is_checked_and_exposed_in_output() {
    let bytes =
        include_bytes!("../../../conformance/next/ocdraw/valid/placed-geometry.ocdraw.json");
    let invalid = viewer::export_drawing_bytes_with_options(
        "geometry.ocdraw.json",
        bytes,
        "dxf",
        "AC1032",
        r#"{"tolerance":{"mode":"custom","value":-1,"unit":"mm"}}"#,
    );
    assert_eq!(invalid["failure"]["code"], "INVALID_CONVERSION_OPTIONS");
    let output = viewer::export_drawing_bytes_with_options(
        "geometry.ocdraw.json",
        bytes,
        "dxf",
        "AC1032",
        r#"{"tolerance":{"mode":"custom","value":0.001,"unit":"mm"}}"#,
    );
    assert!(output["failure"].is_null(), "{output}");
    assert_eq!(output["export"]["options"]["tolerance"]["value"], 0.001);
    assert!(output["export"]["geometry"]["resolvedTolerance"].is_object());
}
#[test]
fn native_ifccad_download_is_independent_of_cad_tolerance() {
    let bytes = include_bytes!("../../../examples/ifccad/blocks-and-fragments.ifcx");
    let result = viewer::export_ifccad_bytes_with_options(
        "source.ifcx",
        bytes,
        "ifccad",
        "AC1032",
        r#"{"tolerance":{"mode":"exact"}}"#,
    );
    assert!(result["failure"].is_null(), "{result}");
    let returned = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        result["export"]["download"]["base64"].as_str().unwrap(),
    )
    .unwrap();
    assert_eq!(returned, bytes);
}

#[test]
fn cad_input_uses_the_selected_ifccad_tolerance_and_reports_domains() {
    let input = include_bytes!("../../../examples/ifccad/overview.ifcx");
    let settings = r#"{"tolerance":{"mode":"custom","value":0.005,"unit":"mm"}}"#;
    let exported =
        viewer::export_ifccad_bytes_with_options("source.ifcx", input, "dxf", "AC1032", settings);
    assert!(exported["failure"].is_null(), "{exported}");
    let bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        exported["export"]["download"]["base64"].as_str().unwrap(),
    )
    .unwrap();
    let imported = viewer::inspect_cad_as_ifccad_bytes_with_options(
        "roundtrip.dxf",
        "dxf",
        &bytes,
        "2026-10-06T00:00:00Z",
        settings,
    );
    assert!(imported["failure"].is_null(), "{imported}");
    assert_eq!(
        imported["conversion"]["options"]["tolerance"]["value"],
        0.005
    );
    assert!(imported["conversion"]["geometryAssessment"]["domains"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["coordinateUnit"] == "mm"));
}
