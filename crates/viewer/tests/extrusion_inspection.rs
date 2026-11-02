#[path = "../../../examples/support/extrusion_gallery.rs"]
mod gallery;
#[test]
fn ordinary_and_saved_graphics_expose_signed_sweeps_without_a_render_claim() {
    let bytes = ifccad::encode_ifccad_document(&gallery::document()).unwrap();
    let result = viewer::inspect_ifccad_bytes("gallery.ifcx", bytes.bytes());
    let rows = result["presentation"]["extrusions"].as_array().unwrap();
    assert!(rows.len() >= 18);
    assert!(rows.iter().any(|v| v["thickness"] == -3.0));
    assert!(rows
        .iter()
        .any(|v| v["direction"] == serde_json::json!([0.0, -0.6, 0.8])));
    assert!(rows
        .iter()
        .any(|v| v["path"].as_str().unwrap().contains("/graphics/")));
    for row in rows {
        assert_eq!(row["displayEvaluation"], "unassessed");
        assert_eq!(row["unit"], "mm");
    }
}
