#[test]
fn ifccad_hatch_inspection_and_both_export_routes_are_available() {
    let bytes = include_bytes!("../../../conformance/next/ifccad/valid/hatch-solid.ifcx");
    let out = viewer::inspect_ifccad_bytes("hatch.ifcx", bytes);
    assert_eq!(out["presentation"]["hatchEntityCount"], 1);
    assert_eq!(out["presentation"]["hatches"].as_array().unwrap().len(), 1);
    for format in ["dxf", "dwg"] {
        let out = viewer::document_export_ifccad("hatch.ifcx", bytes, format, "AC1032", "{}");
        assert!(out["failure"].is_null(), "{out}");
    }
}
