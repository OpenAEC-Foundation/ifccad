#[test]
fn ocdraw_hatch_inspection_exposes_loops_sources_and_limit() {
    let out = viewer::inspect_drawing_bytes(
        "hatch.ocdraw.json",
        include_bytes!("../../../conformance/next/ocdraw/valid/hatch-solid.ocdraw.json"),
    );
    assert_eq!(out["presentation"]["hatchEntityCount"], 1);
    let h = out["presentation"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["geometry"]["type"] == "hatch")
        .unwrap();
    assert_eq!(h["geometry"]["areaRule"], "normal");
    assert_eq!(h["geometry"]["joinTolerance"], 1e-9);
    assert_eq!(h["geometry"]["loops"].as_array().unwrap().len(), 2);
    assert!(h["geometry"]["loops"]
        .as_array()
        .unwrap()
        .iter()
        .any(|l| l["sourceEntityId"].is_string()));
    assert_eq!(h["geometry"]["fillEvaluation"], "unassessed");
}
#[test]
fn ifccad_hatch_inspection_and_both_export_routes_are_available() {
    let bytes = include_bytes!("../../../conformance/next/ifccad/valid/hatch-solid.ifcx");
    let out = viewer::inspect_drawing_bytes("hatch.ifcx", bytes);
    assert_eq!(out["presentation"]["hatchEntityCount"], 1);
    assert_eq!(out["presentation"]["hatches"].as_array().unwrap().len(), 1);
    for format in ["dxf", "dwg"] {
        let out = viewer::export_drawing_bytes("hatch.ifcx", bytes, format, "AC1032");
        assert!(out["failure"].is_null(), "{out}");
    }
}
