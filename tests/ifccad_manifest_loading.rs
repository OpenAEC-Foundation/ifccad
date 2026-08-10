use ifccad::conformance::{bundled_conformance_root, load_conformance_manifest, ConformanceError};
use serde_json::json;
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
        "suiteVersion": "1.0.0",
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
    assert_eq!(manifest.suite_version, "1.0.0");
    assert_eq!(manifest.cases.len(), 20);
    assert_eq!(
        manifest.cases.first().unwrap().case_id,
        "vector.canonicalization"
    );
    assert_eq!(
        manifest.cases.last().unwrap().case_id,
        "invalid.projection-resource-missing"
    );
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
