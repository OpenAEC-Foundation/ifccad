use ifccad::*;
#[test]
fn filled_inspector_keeps_kind_slots_and_source() {
    let src = include_bytes!("../../../conformance/next/ifccad/valid/filled-primitives.ifcx");
    let d = load_ifccad_bytes(src, Default::default()).unwrap();
    let cbor = encode_ifccad_cbor_document(d.document()).unwrap();
    for (name, bytes) in [
        ("filled.ifcx", src.as_slice()),
        ("filled.ifccad.cbor", cbor.bytes()),
    ] {
        let out = viewer::inspect_ifccad_bytes(name, bytes);
        assert!(out["failure"].is_null(), "{out}");
        let nodes = out["presentation"]["graph"]["data"].as_array().unwrap();
        for k in ["solid", "trace"] {
            let n = nodes
                .iter()
                .find(|n| n["attributes"][format!("ifccad::geom::{k}")].is_object())
                .unwrap();
            assert!(n["path"].as_str().unwrap().starts_with("/cad/d1/e"));
            assert_eq!(
                n["attributes"][format!("ifccad::geom::{k}")]["corners"]
                    .as_array()
                    .unwrap()
                    .len(),
                4
            );
        }
    }
}
#[test]
fn filled_inspection_never_promotes_positions_to_display() {
    let src = include_bytes!("../../../conformance/next/ifccad/valid/filled-primitives.ifcx");
    let out = viewer::export_ifccad_bytes("filled.ifcx", src, "dxf", "AC1032");
    assert!(out["failure"].is_null(), "{out}");
    let a = &out["export"]["geometryAssessment"];
    assert_eq!(a["complete"], false);
    let records = a["filledSurfaces"].as_array().unwrap();
    assert_eq!(records.len(), 32);
    assert!(records.iter().any(|r| r["surfaceCoverage"] == "Unassessed"));
    assert!(records.iter().any(|r| r["surfaceCoverage"] == "Proven"));
    for r in records {
        assert_eq!(r["positionScope"], "FourBaseAndFourTop");
        assert_eq!(r["displayEvaluation"], "unassessed");
        assert!(r["source"]["entityId"].is_string());
    }
}
