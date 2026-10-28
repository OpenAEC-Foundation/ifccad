use base64::{engine::general_purpose::STANDARD, Engine};
use opencaddocument::{entities::Line, types::Vector3, Document, Dxf, EntityType, Format};
use serde_json::Value;

fn line() -> Vec<u8> {
    let mut drawing = Document::new();
    drawing
        .add_entity(EntityType::Line(Line::from_points(
            Vector3::new(1.0, 2.0, 0.0),
            Vector3::new(4.0, 5.0, 0.0),
        )))
        .unwrap();
    Dxf.write(&drawing).unwrap().0
}

#[test]
fn shared_route_strictly_reads_native_output_and_prepares_import_selection() {
    let result =
        viewer::document_cad_to_ifccad("line.dxf", "dxf", &line(), "2026-10-27T10:00:00Z", "{}");
    assert!(result["failure"].is_null(), "{result}");
    assert_eq!(result["conversion"]["backend"], "opencaddocument");
    assert_eq!(
        result["conversion"]["geometryAssessment"]["status"],
        "unassessed"
    );
    let bytes = STANDARD
        .decode(result["export"]["download"]["base64"].as_str().unwrap())
        .unwrap();
    let native = ifccad::load_ifccad_bytes(&bytes, Default::default()).unwrap();
    assert_eq!(native.document().model.entities.len(), 1);
    assert_eq!(
        result["preview"]["viewerSelection"]["entities"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(result["presentation"]["hatchExportPreflight"].is_null());
    let export = viewer::document_export_ifccad("line.ifcx", &bytes, "dxf", "AC1032", "{}");
    assert!(export["failure"].is_null(), "{export}");
    assert_eq!(export["export"]["fileCheck"]["cadReadback"], true);
    let output = STANDARD
        .decode(export["export"]["download"]["base64"].as_str().unwrap())
        .unwrap();
    let (cad, _) = Dxf.read(&output).unwrap();
    let actual = cad.model_space_entities().next().unwrap();
    let EntityType::Line(actual) = actual else {
        panic!("wrong semantic kind")
    };
    assert_eq!(actual.start, Vector3::new(1.0, 2.0, 0.0));
    assert_eq!(actual.end, Vector3::new(4.0, 5.0, 0.0));
}

#[test]
fn unsupported_guarantees_are_rejected_explicitly() {
    for options in [
        r#"{"partialOpen":true}"#,
        r#"{"preserveSplines":true}"#,
        r#"{"tolerance":{"mode":"exact"}}"#,
    ] {
        let result = viewer::document_cad_to_ifccad(
            "line.dxf",
            "dxf",
            &line(),
            "2026-10-27T10:00:00Z",
            options,
        );
        assert_eq!(result["failure"]["code"], "DOCUMENT_OPTION_UNAVAILABLE");
        assert!(result["export"]["download"].is_null());
    }
}

#[test]
fn native_open_keeps_validation_independent_of_host_projection() {
    let result: Value = viewer::document_open_ifccad(
        "line.ifcx",
        include_bytes!("../../../tests/fixtures/ifccad/hello-cad.ifcx"),
    );
    assert!(result["failure"].is_null(), "{result}");
    assert_eq!(result["validation"]["strictAvailable"], true);
    assert!(result["conversion"].is_null());
}

#[test]
fn reader_failure_is_never_reported_as_a_successful_read_or_native_conversion() {
    let result =
        viewer::document_cad_to_ifccad("broken.dwg", "dwg", b"bad", "2026-10-27T10:00:00Z", "{}");
    assert_eq!(result["failure"]["stage"], "reading");
    assert_eq!(result["reader"]["status"], "failed");
    assert!(result["conversion"].is_null());
    assert!(result["export"]["download"].is_null());
}

#[test]
fn unsupported_policy_marks_the_unperformed_read_explicitly() {
    let result = viewer::document_cad_to_ifccad(
        "line.dxf",
        "dxf",
        &line(),
        "2026-10-27T10:00:00Z",
        r#"{"lossPolicy":"reject"}"#,
    );
    assert_eq!(result["failure"]["code"], "DOCUMENT_OPTION_UNAVAILABLE");
    assert_eq!(result["reader"]["status"], "notRun");
}

#[test]
fn native_and_binary_inspection_do_not_run_reference_cad_preflights() {
    let bytes = include_bytes!("../../../tests/fixtures/ifccad/hello-cad.ifcx");
    let opened = viewer::document_open_ifccad("native.ifcx", bytes);
    let binary = viewer::export_ifccad_binary_bytes("native.ifcx", bytes, "cbor");
    let stored = STANDARD
        .decode(binary["export"]["download"]["base64"].as_str().unwrap())
        .unwrap();
    let decoded = viewer::document_open_ifccad("native.cbor", &stored);
    for result in [opened, binary, decoded] {
        assert!(result["failure"].is_null());
        assert_eq!(result["validation"]["strictAvailable"], true);
        for key in [
            "hatchExportPreflight",
            "linePatternExportPreflight",
            "polylineWidthExportPreflight",
        ] {
            assert!(result["presentation"][key].is_null());
        }
    }
}

#[test]
fn malformed_options_are_rejected_before_any_read() {
    for options in [
        r#"{"tolerance":{"mode":"custom","value":-1,"unit":"mm"}}"#,
        r#"{"tolerance":{"mode":"custom","value":1,"unit":"unknown"}}"#,
        r#"{"tolerance":{"mode":"custom","value":1,"unit":"drawing","coordinateFallback":1}}"#,
        r#"{"tolerance":{"mode":"custom","value":1,"unit":"mm","coordinateFallback":-1}}"#,
    ] {
        let output = viewer::document_cad_to_ifccad(
            "bad.dxf",
            "dxf",
            b"invalid",
            "2026-10-27T00:00:00Z",
            options,
        );
        assert_eq!(output["failure"]["code"], "INVALID_CONVERSION_OPTIONS");
        assert_eq!(output["reader"]["status"], "notRun");
        assert!(output["export"]["download"].is_null());
    }
}
