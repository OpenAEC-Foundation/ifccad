use ifccad::conformance::{
    bundled_conformance_root, load_conformance_manifest, ConformanceError, ConformanceOperationName,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("ifccad-{name}-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&path).expect("create test directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write_manifest(root: &Path, entrypoint: &str) {
    let manifest = json!({
        "suiteVersion": "1.1.0",
        "cases": [{
            "caseId": "case.one",
            "category": "valid",
            "description": "case",
            "entrypoint": entrypoint,
            "operations": [{
                "name": "validatePackage",
                "expected": { "diagnostics": [] },
            }],
        }],
    });
    fs::write(root.join("manifest.json"), manifest.to_string()).expect("write manifest");
}

#[test]
fn loads_bundled_suite_in_manifest_order() {
    let manifest = load_conformance_manifest(bundled_conformance_root())
        .expect("load bundled conformance suite");
    assert_eq!(manifest.suite_version, "1.1.0");
    assert_eq!(manifest.cases.len(), 127);
    assert_eq!(
        manifest.cases.first().unwrap().case_id,
        "vector.canonicalization"
    );
    assert_eq!(
        manifest.cases.last().unwrap().case_id,
        "invalid.layout-paper-kind-mismatch"
    );
}

#[test]
fn candidate_package_drawings_use_the_current_ifcdr_version() {
    let root = bundled_conformance_root();
    let manifest = load_conformance_manifest(&root).expect("load bundled conformance suite");
    for case in manifest.cases {
        if !case
            .operations
            .iter()
            .any(|operation| operation.name == ConformanceOperationName::ValidatePackage)
            || matches!(
                case.case_id.as_str(),
                "unsupported.unsupported-ifcdr-version" | "unsupported.ifcdr-0.7.0"
            )
        {
            continue;
        }

        let package_dir = root
            .join(&case.entrypoint)
            .parent()
            .expect("package entrypoint parent")
            .to_path_buf();
        let index: Value = serde_json::from_slice(
            &fs::read(package_dir.join("package.json")).expect("package index"),
        )
        .expect("parse package index");
        let ifcx: Value = serde_json::from_slice(
            &fs::read(package_dir.join(index["ifcx"].as_str().expect("IFCX path")))
                .expect("IFCX document"),
        )
        .expect("parse IFCX document");

        for node in ifcx["data"].as_array().expect("IFCX graph") {
            let resource = &node["attributes"]["resource"];
            if resource["format"] != "openaec.ifcdr" {
                continue;
            }
            assert_eq!(
                resource["version"], "0.10.0",
                "{} has an obsolete drawing descriptor",
                case.case_id
            );
            let drawing = if let Some(content) = resource.get("content") {
                Some(content.clone())
            } else if let Some(uri) = resource["uri"].as_str() {
                let path = package_dir.join(uri);
                fs::read(path)
                    .ok()
                    .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
            } else {
                None
            };
            if let Some(version) = drawing
                .as_ref()
                .and_then(|value| value["header"]["version"].as_str())
                .filter(|_| case.case_id != "invalid.inline-unsupported-body")
            {
                assert_eq!(
                    version, "0.10.0",
                    "{} has an obsolete drawing header",
                    case.case_id
                );
            }
        }

        let mut directories = vec![package_dir];
        while let Some(directory) = directories.pop() {
            for entry in fs::read_dir(directory).expect("package directory") {
                let path = entry.expect("package entry").path();
                if path.is_dir() {
                    directories.push(path);
                } else if path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".ifcdr.json"))
                {
                    let drawing: Value =
                        serde_json::from_slice(&fs::read(&path).expect("IFCDR file"))
                            .expect("parse IFCDR file");
                    if drawing["header"]["format"] == "openaec.ifcdr" {
                        assert_eq!(
                            drawing["header"]["version"],
                            "0.10.0",
                            "{} has an obsolete IFCDR file: {}",
                            case.case_id,
                            path.display()
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn rejects_missing_entrypoint() {
    let root = TestDirectory::new("missing-entrypoint");
    write_manifest(root.path(), "packages/missing.json");
    assert!(matches!(
        load_conformance_manifest(root.path()),
        Err(ConformanceError::MissingEntrypoint { .. })
    ));
}

#[test]
fn rejects_unsafe_entrypoint_syntax() {
    for entrypoint in [
        "../outside.json",
        "/absolute.json",
        "C:/absolute.json",
        r"packages\windows.json",
        "packages//empty.json",
    ] {
        let root = TestDirectory::new("unsafe-entrypoint");
        write_manifest(root.path(), entrypoint);
        assert!(
            matches!(
                load_conformance_manifest(root.path()),
                Err(ConformanceError::UnsafeEntrypoint { .. })
            ),
            "entrypoint unexpectedly accepted: {entrypoint}"
        );
    }
}
