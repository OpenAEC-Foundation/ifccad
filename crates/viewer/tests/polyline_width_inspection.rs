const INPUT: &[u8] =
    include_bytes!("../../../conformance/next/ifccad/valid/polyline-defaults-overrides.ifcx");

#[test]
fn defaulted_inspection_distinguishes_inherited_zero_and_equal() {
    let v = viewer::inspect_ifccad_bytes("width.ifcx", INPUT);
    assert!(v["presentation"]["polylineWidthExportPreflight"].is_null());
    let authored = v["presentation"]["polylineWidths"].as_array().unwrap();
    let row = authored
        .iter()
        .find(|v| v["widths"]["kind"] == "defaulted")
        .unwrap();
    assert!(row["widths"]["overrides"][0].get("start").is_none());
    assert_eq!(row["widths"]["overrides"][1]["start"], 0.);
    assert_eq!(row["effectiveWidths"][0], serde_json::json!([0.2, 0.4]));
}
#[test]
fn wide_bounds_do_not_become_safe_culling_when_partial() {
    let out = viewer::inspect_ifccad_bytes("widths.ifcx", INPUT);
    let rows = out["presentation"]["boundsCompleteness"]
        .as_array()
        .unwrap();
    let partial: Vec<_> = rows
        .iter()
        .filter(|r| {
            r["polylineWidthReasons"]
                .as_array()
                .is_some_and(|v| !v.is_empty())
        })
        .collect();
    assert!(!partial.is_empty());
    for row in partial {
        assert_eq!(row["complete"], false);
        assert_eq!(row["safeForNegativeQuery"], false);
        assert_eq!(row["enclosureVerified"], false);
    }
}
