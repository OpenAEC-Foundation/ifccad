use ifccad::package::{load_directory_package, load_package_files};
use std::{collections::BTreeMap, fs, path::Path};

#[test]
fn memory_and_directory_loaders_agree_on_a_valid_package() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("format-explorer/examples/blocks-demo");
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
    let disk = load_directory_package(&root).unwrap();
    let memory = load_package_files(&files);
    assert_eq!(
        disk.validated_package().is_some(),
        memory.validated_package().is_some()
    );
    assert!(memory.validated_package().is_some());
    assert_eq!(
        serde_json::to_value(disk.report()).unwrap(),
        serde_json::to_value(memory.report()).unwrap()
    );
}

#[test]
fn memory_loader_reports_missing_and_unsafe_referenced_resources() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("format-explorer/examples/blocks-demo");
    let original: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("package.ifcx.json")).unwrap()).unwrap();
    for (uri, code) in [
        (
            "resources/missing.ifcdr.json",
            "IFCCAD_PACKAGE_RESOURCE_MISSING",
        ),
        ("../drawing.ifcdr.json", "IFCCAD_PACKAGE_PATH_INVALID"),
    ] {
        let mut ifcx = original.clone();
        let representation = ifcx["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|node| node["type"] == "openaec:DrawingRepresentation")
            .unwrap();
        representation["attributes"]["resource"]["uri"] = serde_json::json!(uri);
        let files = BTreeMap::from([(
            "package.ifcx.json".to_owned(),
            serde_json::to_vec(&ifcx).unwrap(),
        )]);
        let loaded = load_package_files(&files);
        assert!(loaded.validated_package().is_none(), "{uri}");
        assert!(
            loaded
                .report()
                .diagnostics()
                .iter()
                .any(|item| item.code == code),
            "{uri}"
        );
    }
}
