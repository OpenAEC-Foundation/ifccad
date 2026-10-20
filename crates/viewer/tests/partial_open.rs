use ifccad_convert::opencadcodec::{
    entities::{Dimension, DimensionLinear},
    CadDocument, DxfWriter, EntityType, Line, Vector3,
};
#[test]
fn explorer_reports_normal_failure_then_explicit_partial_open_and_omissions() {
    let mut cad = CadDocument::new();
    let mut d = DimensionLinear::new(Vector3::ZERO, Vector3::new(3., 4., 1.));
    d.base.text_user_positioned = true;
    d.base.text_middle_point = Vector3::new(0., 0., 1.);
    let bad = cad
        .add_entity(EntityType::Dimension(Box::new(Dimension::Linear(d))))
        .unwrap();
    cad.add_entity(EntityType::Line(Line::from_coords(0., 0., 2., 1., 2., 3.)))
        .unwrap();
    let bytes = DxfWriter::new(&cad).write_to_vec().unwrap();
    let normal = viewer::inspect_cad_as_ifccad_bytes_with_options(
        "bad.dxf",
        "dxf",
        &bytes,
        "2026-10-20T00:00:00Z",
        "{}",
    );
    assert_eq!(normal["failure"]["code"], "IFCCAD_CONVERSION_FAILED");
    assert_eq!(normal["failure"]["partialOpenAvailable"], true);
    assert!(normal["export"]["download"].is_null());
    let partial = viewer::inspect_cad_as_ifccad_bytes_with_options(
        "bad.dxf",
        "dxf",
        &bytes,
        "2026-10-20T00:00:00Z",
        r#"{"partialOpen":true}"#,
    );
    assert!(partial["failure"].is_null(), "{partial}");
    assert_eq!(partial["validation"]["strictAvailable"], true);
    assert_eq!(partial["partialOpening"]["errorCount"], 1);
    assert!(partial["partialOpening"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["handle"] == bad.to_string()));
}
