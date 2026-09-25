use ifccad_convert::cadcodec::{CadDocument, DwgWriter, DxfWriter, EntityType, Line};
use ifccad_viewer::{export_package_files, inspect_cad_bytes, inspect_package_files};
use std::{collections::BTreeMap, fs, path::Path};

#[test]
fn in_memory_package_inspection_has_a_strict_report_and_presentation() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../format-explorer/examples/blocks-demo");
    let files = BTreeMap::from([
        (
            "package.ifcx.json".to_owned(),
            fs::read(root.join("package.ifcx.json")).unwrap(),
        ),
        (
            "resources/drawing.ifcdr.json".to_owned(),
            fs::read(root.join("resources/drawing.ifcdr.json")).unwrap(),
        ),
    ]);
    let report = inspect_package_files("blocks-demo", &files);
    assert_eq!(report["validation"]["strictAvailable"], true);
    assert!(report["failure"].is_null());
    assert_eq!(
        report["presentation"]["documents"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn cad_bytes_convert_to_a_strict_in_memory_package() {
    let mut document = CadDocument::new();
    document
        .add_entity(EntityType::Line(Line::from_coords(0., 0., 0., 1., 2., 0.)))
        .unwrap();
    for format in ["dxf", "dwg"] {
        let bytes = if format == "dxf" {
            DxfWriter::new(&document).write_to_vec().unwrap()
        } else {
            DwgWriter::write_to_vec(&document).unwrap()
        };
        let result = inspect_cad_bytes("example", format, &bytes, "2026-09-25T12:00:00Z");
        assert_eq!(
            result["validation"]["strictAvailable"], true,
            "{format}: {result}"
        );
        assert!(result["failure"].is_null(), "{format}: {result}");
        assert_eq!(
            result["conversion"]["entities"].as_array().unwrap().len(),
            1
        );
        assert!(
            result["presentation"]["documents"]
                .as_array()
                .unwrap()
                .len()
                >= 2
        );
    }
}

#[test]
fn in_memory_package_exports_checked_cad_bytes() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../format-explorer/examples/blocks-demo");
    let files = BTreeMap::from([
        (
            "package.ifcx.json".to_owned(),
            fs::read(root.join("package.ifcx.json")).unwrap(),
        ),
        (
            "resources/drawing.ifcdr.json".to_owned(),
            fs::read(root.join("resources/drawing.ifcdr.json")).unwrap(),
        ),
    ]);
    let result = export_package_files("blocks-demo", &files, "drawing-0", "dxf", "AC1032");
    assert!(result["failure"].is_null(), "{result}");
    assert_eq!(result["export"]["fileCheck"]["readable"], true);
    assert_eq!(result["export"]["effectiveVersion"], "AC1032");
    assert!(result["export"]["download"]["byteLength"].as_u64().unwrap() > 0);
}
