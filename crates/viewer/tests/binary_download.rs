use base64::{engine::general_purpose::STANDARD, Engine};
use ifccad::{load_ifccad_bytes, load_ifccad_cbor_bytes};

#[test]
fn binary_downloads_use_the_production_writers_and_keep_document_values() {
    let input = include_bytes!("../../../tests/fixtures/ifccad/hello-hatch-gradient.ifcx");
    let original = load_ifccad_bytes(input, Default::default()).unwrap();
    for encoding in ["cbor", "cbor-zstd"] {
        if encoding == "cbor-zstd" && !cfg!(feature = "ifccad-cbor-zstd") {
            continue;
        }
        let output = viewer::export_ifccad_binary_bytes("drawing.ifcx.json", input, encoding);
        assert!(output["failure"].is_null(), "{output}");
        let file = &output["export"]["download"];
        let bytes = STANDARD.decode(file["base64"].as_str().unwrap()).unwrap();
        assert_eq!(file["byteLength"], bytes.len());
        let extension = if encoding == "cbor" {
            "ifccad.cbor"
        } else {
            "ifccad.cbor.zst"
        };
        assert_eq!(file["fileName"], format!("drawing.{extension}"));
        let loaded = if encoding == "cbor" {
            load_ifccad_cbor_bytes(&bytes, Default::default()).unwrap()
        } else {
            #[cfg(feature = "ifccad-cbor-zstd")]
            {
                ifccad::load_ifccad_cbor_zstd_bytes(&bytes, Default::default()).unwrap()
            }
            #[cfg(not(feature = "ifccad-cbor-zstd"))]
            {
                continue;
            }
        };
        assert_eq!(loaded.document(), original.document());
    }
}

#[test]
fn binary_download_rejects_invalid_source_and_unknown_encoding() {
    for (input, encoding) in [
        (b"{}".as_slice(), "cbor"),
        (
            include_bytes!("../../../examples/ifccad/overview.ifcx").as_slice(),
            "zip",
        ),
    ] {
        let output = viewer::export_ifccad_binary_bytes("drawing.ifcx", input, encoding);
        assert!(output["failure"].is_object());
        assert!(output["export"]["download"].is_null());
    }
}
