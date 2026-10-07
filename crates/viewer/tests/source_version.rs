use ocdraw_convert::opencadcodec::{CadDocument, DwgWriter, DxfVersion, DxfWriter};

#[test]
fn both_routes_report_the_version_read_from_real_cad_files() {
    let mut cad = CadDocument::new();
    cad.version = DxfVersion::AC1027;
    for (format, bytes) in [
        ("dxf", DxfWriter::new(&cad).write_to_vec().unwrap()),
        ("dwg", DwgWriter::write_to_vec(&cad).unwrap()),
    ] {
        for opened in [
            viewer::inspect_cad_as_drawing_bytes("source", format, &bytes),
            viewer::inspect_cad_as_ifccad_bytes("source", format, &bytes, "2026-10-07T09:00:00Z"),
        ] {
            assert!(opened["failure"].is_null(), "{opened}");
            assert_eq!(opened["validation"]["strictAvailable"], true);
            assert_eq!(opened["reader"]["version"], "AC1027");
            assert_eq!(opened["source"]["format"], format);
        }
    }
}
