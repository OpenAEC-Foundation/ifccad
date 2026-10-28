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
