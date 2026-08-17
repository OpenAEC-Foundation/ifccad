use ifccad::conformance::bundled_conformance_root;
use ifccad::package::{
    load_directory_package, PackageOpenError, PackageValidationReport, ValidatedIfccadPackage,
    DIRECTORY_PACKAGE_ENTRYPOINT,
};
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
        let path = std::env::temp_dir().join(format!(
            "ifccad-public-loading-{name}-{}-{nonce}",
            std::process::id()
        ));
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

fn minimal_package() -> PathBuf {
    bundled_conformance_root()
        .join("packages")
        .join("valid")
        .join("minimal-no-preservation")
}

fn assert_public_types(_package: &ValidatedIfccadPackage, _report: &PackageValidationReport) {}

#[test]
fn valid_directory_exposes_a_strict_package_and_report() {
    let outcome = load_directory_package(minimal_package()).expect("open valid package");

    assert!(outcome.report().is_valid());
    let package = outcome
        .validated_package()
        .expect("valid package has strict proof");
    assert_public_types(package, outcome.report());
    assert_eq!(DIRECTORY_PACKAGE_ENTRYPOINT, "package.ifcx.json");

    assert!(load_directory_package(minimal_package())
        .unwrap()
        .into_validated_package()
        .is_some());
    let (package, report) = load_directory_package(minimal_package())
        .unwrap()
        .into_parts();
    assert!(package.is_some());
    assert!(report.is_valid());
}

#[test]
fn schema_failure_is_an_inspectable_outcome_without_a_strict_package() {
    let root = TestDirectory::new("schema-error");
    fs::write(
        root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
        br#"{"data":[{"path":"layout","type":"openaec:DrawingLayout","attributes":{"name":"Model","kind":"sheet","scopeId":0},"children":{"Representation":"missing"}}]}"#,
    )
    .expect("write invalid entrypoint");

    let outcome = load_directory_package(root.path()).expect("inspect invalid package");

    assert!(!outcome.report().is_valid());
    assert!(outcome.validated_package().is_none());
    assert!(outcome
        .report()
        .iter()
        .any(|diagnostic| diagnostic.code == "IFCCAD_PACKAGE_SCHEMA_INVALID"));
}

#[test]
fn file_root_is_a_public_open_error() {
    let root = TestDirectory::new("file-root");
    let file = root.path().join("not-a-directory.ifccad");
    fs::write(&file, b"not a directory").expect("write test file");

    let error = load_directory_package(&file).expect_err("file root must fail");

    assert!(matches!(
        error,
        PackageOpenError::RootNotDirectory { path } if path == file
    ));
}
