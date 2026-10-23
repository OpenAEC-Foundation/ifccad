use base64::{engine::general_purpose::STANDARD, Engine};
use ifccad_convert::opencadcodec::{DxfReader, EntityType};
use std::io::Cursor;
const INPUT: &[u8] =
    include_bytes!("../../../conformance/next/ifccad/valid/polyline-defaults-overrides.ifcx");
#[test]
fn defaulted_inspection_distinguishes_inherited_zero_and_equal() {
    let v = viewer::inspect_ifccad_bytes("width.ifcx", INPUT);
    assert!(v["presentation"]["polylineWidthExportPreflight"].is_array());
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
fn dwg_reject_refuses_before_download() {
    let v = viewer::export_ifccad_bytes_with_options(
        "width.ifcx",
        INPUT,
        "dwg",
        "AC1032",
        r#"{"lossPolicy":"reject"}"#,
    );
    assert!(!v["failure"].is_null(), "{v}");
    assert!(v["export"]["download"].is_null());
    assert!(v.to_string().contains("polyline-width-dependency"));
}
#[test]
fn classic_dxf_export_selects_preserving_writer() {
    let v = viewer::export_ifccad_bytes("width.ifcx", INPUT, "dxf", "AC1032");
    assert!(v["failure"].is_null(), "{v}");
    let bytes = STANDARD
        .decode(v["export"]["download"]["base64"].as_str().unwrap())
        .unwrap();
    let c = DxfReader::from_reader(Cursor::new(bytes))
        .unwrap()
        .read()
        .unwrap();
    let p = c
        .entities()
        .find_map(|e| {
            if let EntityType::Polyline2D(p) = e {
                Some(p)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!([p.start_width, p.end_width], [0.2, 0.4]);
    assert!(p.vertices[0].start_width.is_none());
    assert_eq!(p.vertices[1].start_width, Some(0.));
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
