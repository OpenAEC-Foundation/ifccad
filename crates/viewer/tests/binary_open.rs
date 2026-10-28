use base64::{engine::general_purpose::STANDARD, Engine};
use ifccad::{encode_ifccad_cbor_document, load_ifccad_bytes, load_ifccad_cbor_bytes};

#[test]
fn binary_input_opens_and_can_be_exported_to_json_binary_and_cad() {
    let original = load_ifccad_bytes(
        include_bytes!("../../../examples/ifccad/overview.ifcx"),
        Default::default(),
    )
    .unwrap();
    let cbor = encode_ifccad_cbor_document(original.document())
        .unwrap()
        .into_bytes();
    let inputs = vec![("drawing.ifccad.cbor", cbor, "cbor")];
    #[cfg(feature = "ifccad-cbor-zstd")]
    let inputs = {
        let mut inputs = inputs;
        inputs.push((
            "drawing.ifccad.cbor.zst",
            ifccad::encode_ifccad_cbor_zstd_document(original.document(), Default::default())
                .unwrap()
                .into_bytes(),
            "cbor-zstd",
        ));
        inputs
    };
    for (name, bytes, encoding) in inputs {
        let opened = viewer::inspect_ifccad_bytes(name, &bytes);
        assert!(opened["failure"].is_null(), "{opened}");
        assert_eq!(opened["validation"]["strictAvailable"], true);
        assert_eq!(opened["source"]["encoding"], encoding);
        assert_eq!(opened["source"]["representation"], "decoded");
        let same = viewer::export_ifccad_binary_bytes(name, &bytes, encoding);
        assert_eq!(
            STANDARD
                .decode(same["export"]["download"]["base64"].as_str().unwrap())
                .unwrap(),
            bytes
        );
        let display = load_ifccad_bytes(
            opened["nativeSourceText"].as_str().unwrap().as_bytes(),
            Default::default(),
        )
        .unwrap();
        assert_eq!(display.document(), original.document());
        let json = export(name, &bytes, "ifccad", "AC1032");
        let file = &json["export"]["download"];
        assert_eq!(file["fileName"], "drawing.ifcx");
        let decoded = STANDARD.decode(file["base64"].as_str().unwrap()).unwrap();
        assert_eq!(
            load_ifccad_bytes(&decoded, Default::default())
                .unwrap()
                .document(),
            original.document()
        );
        let raw = viewer::export_ifccad_binary_bytes(name, &bytes, "cbor");
        let decoded = STANDARD
            .decode(raw["export"]["download"]["base64"].as_str().unwrap())
            .unwrap();
        assert_eq!(raw["export"]["download"]["fileName"], "drawing.ifccad.cbor");
        assert_eq!(
            load_ifccad_cbor_bytes(&decoded, Default::default())
                .unwrap()
                .document(),
            original.document()
        );
        if encoding == "cbor" {
            assert_eq!(decoded, bytes);
        }
        for format in ["dxf", "dwg"] {
            let output = export(name, &bytes, format, "AC1032");
            assert!(output["failure"].is_null(), "{output}");
            assert!(output["export"]["download"]["base64"].is_string());
            assert!(
                !output.to_string().contains("foreign-ifcx"),
                "binary input has no IFCX source audit"
            );
        }
    }
}

#[cfg(feature = "ifccad-cbor-zstd")]
#[test]
fn compressed_input_enforces_the_application_decompressed_limit_before_allocation() {
    let mut header = vec![0x28, 0xb5, 0x2f, 0xfd, 0xa4];
    header.extend_from_slice(&(64_u32 * 1024 * 1024 + 1).to_le_bytes());
    let output = viewer::inspect_ifccad_bytes("large.cbor.zst", &header);
    assert!(output["failure"]["message"]
        .as_str()
        .unwrap()
        .contains("decompressed_bytes"));
    assert_eq!(output["validation"]["strictAvailable"], false);
}

#[test]
fn binary_input_rejects_malformed_bytes_and_wrong_transport() {
    for (name, bytes) in [
        ("broken.ifccad.cbor", b"{}".as_slice()),
        ("broken.ifccad.cbor.zst", b"{}".as_slice()),
    ] {
        let result = viewer::inspect_ifccad_bytes(name, bytes);
        assert_eq!(result["validation"]["strictAvailable"], false);
        assert!(result["failure"].is_object());
        assert!(result["presentation"].is_null());
        assert!(result["nativeSourceText"].is_null());
    }
}

fn export(name: &str, bytes: &[u8], format: &str, version: &str) -> serde_json::Value {
    viewer::document_export_ifccad(name, bytes, format, version, "{}")
}
