#[test]
fn ifccad_hatch_inspection_and_both_export_routes_are_available() {
    let bytes = include_bytes!("../../../conformance/next/ifccad/valid/hatch-solid.ifcx");
    let out = viewer::inspect_ifccad_bytes("hatch.ifcx", bytes);
    assert_eq!(out["presentation"]["hatchEntityCount"], 1);
    assert_eq!(out["presentation"]["hatches"].as_array().unwrap().len(), 1);
    for format in ["dxf", "dwg"] {
        let out = viewer::export_ifccad_bytes("hatch.ifcx", bytes, format, "AC1032");
        assert!(out["failure"].is_null(), "{out}");
    }
}

#[test]
fn physical_pattern_description_loss_is_reported_for_both_routes_and_codecs() {
    {
        let (name, bytes) = (
            "pattern.ifcx",
            include_bytes!("../../../tests/fixtures/ifccad/hello-hatch-pattern.ifcx").as_slice(),
        );
        for format in ["dxf", "dwg"] {
            let out = viewer::export_ifccad_bytes(name, bytes, format, "AC1032");
            assert!(out["failure"].is_null(), "{out}");
            let losses = out["export"]["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|d| d["code"] == "TARGET_CODEC_HATCH_DESCRIPTION_LOSS")
                .collect::<Vec<_>>();
            assert_eq!(
                losses.len(),
                1,
                "{name}/{format}: missing description diagnostic"
            );
            assert_eq!(losses[0]["phase"], "cadExchange");
            assert_eq!(losses[0]["action"], "Modified");
        }
    }
}
