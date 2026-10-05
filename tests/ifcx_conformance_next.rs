use ocdraw::ifcx_cad::{encode_ifcx_cad_document, load_ifcx_cad_bytes};
#[test]
fn candidate_ifcx_viewport_contract_uses_production_reader() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("conformance/next/ifcx-native-cad");
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../conformance/next/ifcx-native-cad/cases.json"
    ))
    .unwrap();
    for case in cases {
        let name = case["file"].as_str().unwrap();
        let bytes = std::fs::read(root.join(name)).unwrap();
        let result = load_ifcx_cad_bytes(&bytes, Default::default());
        assert_eq!(
            result.is_ok(),
            case["valid"].as_bool().unwrap(),
            "{name}: {result:?}"
        );
        if let Ok(valid) = result {
            let encoded = encode_ifcx_cad_document(valid.document()).unwrap();
            assert_eq!(
                load_ifcx_cad_bytes(encoded.bytes(), Default::default())
                    .unwrap()
                    .document(),
                valid.document()
            );
        }
    }
}
