use base64::{engine::general_purpose::STANDARD, Engine};
#[test]
fn ifccad_capture_durable_open_and_actual_export_show_separate_evidence() {
    let options = r#"{"preserveSplines":true}"#;
    let opened = viewer::inspect_cad_as_ifccad_bytes_with_options(
        "spline.dxf",
        "dxf",
        include_bytes!("../../ocdraw-convert/tests/fixtures/splines/open-cubic.dxf"),
        "2026-10-07T00:00:00Z",
        options,
    );
    assert!(opened["failure"].is_null(), "{opened}");
    assert_eq!(opened["presentation"]["opaqueEntityCount"], 1);
    assert_eq!(
        opened["conversion"]["geometryAssessment"]["complete"],
        false
    );
    assert!(opened["conversion"]["preservation"]["entries"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["result"] == "capturedTyped"));
    let bytes = STANDARD
        .decode(opened["export"]["download"]["base64"].as_str().unwrap())
        .unwrap();
    let native = viewer::inspect_ifccad_bytes("saved.ifcx", &bytes);
    assert_eq!(native["presentation"]["opaqueEntityCount"], 1);
    for format in ["dxf", "dwg"] {
        let out = viewer::export_ifccad_bytes("saved.ifcx", &bytes, format, "AC1032");
        assert!(out["failure"].is_null(), "{}", out["failure"]);
        assert!(out["conversion"]["preservation"]["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["result"] == "restoredTyped"));
        assert_eq!(out["export"]["fileCheck"]["opaqueEntityCount"], 1);
    }
}

#[test]
fn fit_dxf_parameterization_loss_is_distinct_from_successful_typed_restore() {
    use ocdraw_convert::opencadcodec::{CadDocument, DwgWriter, EntityType, Vector3};
    let mut c = CadDocument::new();
    let mut s = ocdraw_convert::opencadcodec::entities::Spline::new();
    s.fit_points = vec![Vector3::new(0., 0., 0.), Vector3::new(3., 1., 0.)];
    s.begin_tangent = Vector3::new(1., 0., 0.);
    s.end_tangent = Vector3::new(1., 0., 0.);
    s.knot_parameterization = 2;
    s.dwg_flags1 = 9;
    c.add_entity(EntityType::Spline(s)).unwrap();
    let bytes = DwgWriter::write_to_vec(&c).unwrap();
    let out = viewer::inspect_cad_as_ifccad_bytes_with_options(
        "fit.dwg",
        "dwg",
        &bytes,
        "2026-10-07T00:00:00Z",
        r#"{"preserveSplines":true}"#,
    );
    assert!(out["failure"].is_null(), "{}", out["failure"]);
    let saved = STANDARD
        .decode(out["export"]["download"]["base64"].as_str().unwrap())
        .unwrap();
    let out = viewer::export_ifccad_bytes("saved.ifcx", &saved, "dxf", "AC1032");
    assert!(out["failure"].is_null(), "{}", out["failure"]);
    assert!(
        out["conversion"]["preservation"]["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["result"] == "restoredTyped"),
        "{}",
        out["failure"]
    );
    assert!(
        out["export"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["code"] == "TARGET_CODEC_SPLINE_PARAMETERIZATION_LOSS"),
        "{}",
        out["export"]["diagnostics"]
    );
}
