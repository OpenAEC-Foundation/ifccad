use ifccad_convert::cadcodec::{CadDocument, Circle, DwgWriter, DxfWriter, EntityType, Line};
use ifccad_viewer::inspect_cad;
#[test]
fn both_cad_readers_emit_validated_output_and_loss_evidence() {
    for format in ["dxf", "dwg"] {
        let root = std::env::temp_dir().join(format!(
            "ifccad-viewer-test-{}-{format}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let mut doc = CadDocument::new();
        doc.add_entity(EntityType::Line(Line::from_coords(0., 0., 0., 1., 2., 0.)))
            .unwrap();
        doc.add_entity(EntityType::Circle(Circle::new())).unwrap();
        let mut partial = Line::from_coords(0., 1., 0., 2., 3., 0.);
        partial.common.linetype_scale = 2.;
        doc.add_entity(EntityType::Line(partial)).unwrap();
        let bytes = if format == "dxf" {
            DxfWriter::new(&doc).write_to_vec().unwrap()
        } else {
            DwgWriter::write_to_vec(&doc).unwrap()
        };
        let input = root.join(format!("drawing.{format}"));
        std::fs::write(&input, bytes).unwrap();
        let r = inspect_cad(&input, &root.join("output"));
        assert_eq!(r["validation"]["strictAvailable"], true, "{r}");
        let rows = r["conversion"]["entities"].as_array().unwrap();
        assert!(
            rows.iter()
                .any(|e| e["kind"] == "CIRCLE" && e["disposition"] == "skipped"),
            "{r}"
        );
        assert_eq!(rows.iter().filter(|e| !e["target"].is_null()).count(), 2);
        assert!(rows.iter().any(|e| e["disposition"] == "partial"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
#[test]
fn unreadable_cad_has_no_score_or_graph() {
    let r = inspect_cad(
        std::path::Path::new("missing.dwg"),
        std::path::Path::new("unused-output"),
    );
    assert_eq!(r["reader"]["status"], "failed");
    assert!(r["conversion"].is_null());
    assert!(r["presentation"].is_null());
}
