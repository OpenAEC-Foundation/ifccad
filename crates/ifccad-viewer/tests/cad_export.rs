use base64::{engine::general_purpose::STANDARD, Engine};
use ifccad_convert::cadcodec::{
    CadDocument, Circle, DwgReader, DxfReader, DxfWriter, EntityType, Line,
};
use ifccad_viewer::{export_cad, export_package, inspect_cad};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../conformance/next/packages")
        .join(name)
}

#[test]
fn both_downloads_read_back_and_validate_with_honest_coverage() {
    for format in ["dxf", "dwg"] {
        let root =
            std::env::temp_dir().join(format!("explorer-export-{}-{format}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let result = export_package(
            &fixture("valid/unrepresented-packed"),
            "drawing-main",
            format,
        );
        assert!(result["failure"].is_null(), "{result}");
        assert_eq!(result["export"]["assessment"]["coverage"], "Incomplete");
        assert_ne!(
            result["export"]["assessment"]["conclusion"],
            "NoLossDetected"
        );
        assert_eq!(result["export"]["preservationRestored"], false);
        let bytes = STANDARD
            .decode(result["export"]["download"]["base64"].as_str().unwrap())
            .unwrap();
        let file = root.join(format!("drawing.{format}"));
        std::fs::write(&file, bytes).unwrap();
        let doc = if format == "dxf" {
            DxfReader::from_file(&file).unwrap().read().unwrap()
        } else {
            DwgReader::from_file(&file).unwrap().read().unwrap()
        };
        assert_eq!(doc.entities().count(), 4);
        let lines: Vec<_> = doc
            .entities()
            .filter_map(|entity| match entity {
                EntityType::Line(line) => Some((
                    [line.start.x, line.start.y, line.start.z],
                    [line.end.x, line.end.y, line.end.z],
                )),
                _ => None,
            })
            .collect();
        assert_eq!(
            lines,
            vec![([0., 0., 0.], [5., 5., 0.]), ([10., 0., 0.], [10., 8., 0.])]
        );
        let polylines: Vec<_> = doc
            .entities()
            .filter_map(|entity| match entity {
                EntityType::LwPolyline(polyline) => Some((
                    polyline
                        .vertices
                        .iter()
                        .map(|v| [v.location.x, v.location.y])
                        .collect::<Vec<_>>(),
                    polyline.is_closed,
                )),
                _ => None,
            })
            .collect();
        assert_eq!(
            polylines[0],
            (vec![[0., 0.], [10., 0.], [10., 5.], [0., 5.]], true)
        );
        let loaded = inspect_cad(&file, &root.join("readback"));
        assert_eq!(loaded["validation"]["strictAvailable"], true, "{loaded}");
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn selected_drawing_and_failures_never_return_the_wrong_file() {
    let multi = fixture("valid/multi-drawing-projections");
    let result = export_package(&multi, "drawing-second", "dxf");
    assert!(result["failure"].is_null(), "{result}");
    assert_eq!(result["export"]["drawing"], "drawing-second");
    let paper = export_package(
        &fixture("valid/shared-layout-representation"),
        "drawing-main",
        "dxf",
    );
    assert!(paper["failure"].is_null(), "{paper}");
    assert_eq!(paper["export"]["drawing"], "drawing-main");
    for result in [
        export_package(&multi, "missing", "dxf"),
        export_package(&multi, "drawing-main", "exe"),
        export_package(
            &fixture("invalid/schema-wrong-scalar"),
            "drawing-main",
            "dwg",
        ),
    ] {
        assert!(!result["failure"].is_null(), "{result}");
        assert!(result["export"]["download"].is_null());
    }
}

#[test]
fn native_package_download_validates_all_drawings_without_cad_conversion() {
    let result = export_package(&fixture("valid/multi-drawing-projections"), "", "ifccad");
    assert!(result["failure"].is_null(), "{result}");
    assert_eq!(result["export"]["packageReady"], true);
    assert_eq!(result["export"]["drawingCount"], 2);
    assert!(result["export"]["assessment"].is_null());
    assert!(result["conversion"].is_null());
    let invalid = export_package(&fixture("invalid/schema-wrong-scalar"), "", "ifccad");
    assert!(!invalid["failure"].is_null());
    assert!(invalid["export"]["packageReady"].is_null());
}

#[test]
fn cad_source_export_uses_native_content_and_retains_opening_losses() {
    let root = std::env::temp_dir().join(format!("explorer-export-cad-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let mut doc = CadDocument::new();
    doc.add_entity(EntityType::Line(Line::from_coords(0., 0., 0., 1., 2., 0.)))
        .unwrap();
    doc.add_entity(EntityType::Circle(Circle::new())).unwrap();
    let source = root.join("source.dxf");
    std::fs::write(&source, DxfWriter::new(&doc).write_to_vec().unwrap()).unwrap();
    let result = export_cad(&source, &root.join("native"), "drawing-0", "dwg");
    assert!(result["failure"].is_null(), "{result}");
    assert_eq!(result["export"]["entityCount"], 1);
    assert!(result["conversion"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entity| entity["kind"] == "CIRCLE" && entity["disposition"] == "skipped"));
    let bytes = STANDARD
        .decode(result["export"]["download"]["base64"].as_str().unwrap())
        .unwrap();
    let restored = DwgReader::from_stream(std::io::Cursor::new(bytes))
        .read()
        .unwrap();
    assert_eq!(restored.entities().count(), 1);
    assert!(matches!(
        restored.entities().next().unwrap(),
        EntityType::Line(_)
    ));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn line_pattern_loss_remains_visible_in_export_diagnostics() {
    let root = std::env::temp_dir().join(format!("explorer-export-loss-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let source = fixture("valid/inline-both");
    let mut document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(source.join("package.ifcx.json")).unwrap()).unwrap();
    for node in document["data"].as_array_mut().unwrap() {
        if node["type"] == "openaec:Appearance" {
            node["attributes"]["linePattern"]["value"] = serde_json::json!("custom-pattern");
        }
    }
    std::fs::write(
        root.join("package.ifcx.json"),
        serde_json::to_vec(&document).unwrap(),
    )
    .unwrap();
    // Inline preservation still references its original blob; preserve those bytes.
    for entry in std::fs::read_dir(&source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() != "package.ifcx.json" && entry.file_type().unwrap().is_file() {
            std::fs::copy(entry.path(), root.join(entry.file_name())).unwrap();
        }
    }
    let result = export_package(&root, "drawing-main", "dxf");
    assert!(result["failure"].is_null(), "{result}");
    assert_eq!(result["export"]["assessment"]["conclusion"], "LossDetected");
    assert!(result["export"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["code"] == "LinePatternFallback"));
    std::fs::remove_dir_all(root).unwrap();
}
