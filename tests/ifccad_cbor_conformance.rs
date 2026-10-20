use ifccad::*;
#[test]
fn candidate_binary_contract_uses_production_reader() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("conformance/next/ifccad-cbor");
    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("cases.json")).expect("binary conformance manifest exists"),
    )
    .unwrap();
    assert_eq!(manifest["mappingRevision"], 1);
    for case in manifest["cases"].as_array().unwrap() {
        let name = case["file"].as_str().unwrap();
        let bytes = std::fs::read(root.join(name)).unwrap();
        assert!(root.join(case["diagnostic"].as_str().unwrap()).is_file());
        let result = load_ifccad_cbor_bytes(&bytes, Default::default());
        assert_eq!(
            result.is_ok(),
            case["valid"].as_bool().unwrap(),
            "{name}: {result:?}"
        );
        match result {
            Ok(loaded) => {
                let out = encode_ifccad_cbor_document(loaded.document()).unwrap();
                assert_eq!(
                    load_ifccad_cbor_bytes(out.bytes(), Default::default())
                        .unwrap()
                        .document(),
                    loaded.document()
                );
            }
            Err(e) => {
                let class = match e {
                    IfccadCborReadError::Malformed { .. } => "Malformed",
                    IfccadCborReadError::UnsupportedRevision { .. } => "UnsupportedRevision",
                    IfccadCborReadError::ResourceLimit { .. } => "ResourceLimit",
                    IfccadCborReadError::InvalidDocument(_) => "InvalidDocument",
                };
                assert_eq!(class, case["errorClass"], "{name}: {e}");
            }
        }
    }
}
