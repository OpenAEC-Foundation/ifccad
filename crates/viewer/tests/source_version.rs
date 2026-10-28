use opencaddocument::{Document, Dwg, Dxf, DxfVersion, Format};

#[test]
fn shared_route_reports_the_version_read_from_real_cad_files() {
    let mut cad = Document::new();
    cad.version = DxfVersion::AC1027;
    for (format, bytes) in [
        ("dxf", Dxf.write(&cad).unwrap().0),
        ("dwg", Dwg.write(&cad).unwrap().0),
    ] {
        {
            let opened = viewer::document_cad_to_ifccad(
                "source",
                format,
                &bytes,
                "2026-10-07T09:00:00Z",
                "{}",
            );
            assert!(opened["failure"].is_null(), "{opened}");
            assert_eq!(opened["validation"]["strictAvailable"], true);
            assert_eq!(opened["reader"]["version"], "AC1027");
            assert_eq!(opened["source"]["format"], format);
        }
    }
}
