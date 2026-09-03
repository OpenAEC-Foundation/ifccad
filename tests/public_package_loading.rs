use ifccad::conformance::bundled_conformance_root;
use ifccad::package::{
    load_directory_package, PackageHeaderRef, PackageOpenError, PackageValidationReport,
    ValidatedIfccadPackage, DIRECTORY_PACKAGE_ENTRYPOINT,
};
use ifccad::PackageId;
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

fn copy_minimal_package(root: &Path) -> serde_json::Value {
    let source = minimal_package();
    for entry in fs::read_dir(&source).expect("read minimal package") {
        let entry = entry.expect("minimal package entry");
        if entry.file_type().expect("minimal entry type").is_file() {
            fs::copy(entry.path(), root.join(entry.file_name()))
                .expect("copy minimal package file");
        }
    }
    serde_json::from_slice(
        &fs::read(root.join(DIRECTORY_PACKAGE_ENTRYPOINT)).expect("read copied entrypoint"),
    )
    .expect("parse copied entrypoint")
}

fn assert_public_types(
    package: &ValidatedIfccadPackage,
    header: PackageHeaderRef<'_>,
    report: &PackageValidationReport,
) {
    let _: &PackageId = header.package_id();
    let _: &str = header.ifcx_version();
    let _: &str = header.data_version();
    let _: &str = header.author();
    let _: &str = header.timestamp();
    let _ = (package, report);
}

#[test]
fn package_identity_is_distinct_and_preserves_caller_text() {
    let id = PackageId::new("  package:building/main  ").expect("package ID");

    assert_eq!(id.as_str(), "  package:building/main  ");
    assert_eq!(id.to_string(), "  package:building/main  ");
    assert_eq!(PackageId::new(""), Err(ifccad::InvalidPackageId));
}

#[test]
fn valid_directory_exposes_a_strict_package_and_report() {
    let outcome = load_directory_package(minimal_package()).expect("open valid package");

    assert!(outcome.report().is_valid());
    let package = outcome
        .validated_package()
        .expect("valid package has strict proof");
    let header = package.header();
    assert_public_types(package, header, outcome.report());
    assert_eq!(
        header.package_id().as_str(),
        "ifccad/examples/golden-minimal/golden_minimal.ifcx.json"
    );
    assert_eq!(header.ifcx_version(), "ifcx_alpha");
    assert_eq!(header.data_version(), "0.5.0");
    assert_eq!(header.author(), "IFC-CAD prototype");
    assert_eq!(header.timestamp(), "2026-07-06T00:00:00Z");
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
fn incomplete_header_is_inspectable_without_a_strict_proof() {
    let root = TestDirectory::new("incomplete-header");
    let mut entrypoint = copy_minimal_package(root.path());
    entrypoint["header"].as_object_mut().unwrap().remove("id");
    fs::write(
        root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
        serde_json::to_vec(&entrypoint).expect("serialize incomplete entrypoint"),
    )
    .expect("write incomplete entrypoint");

    let outcome = load_directory_package(root.path()).expect("inspect incomplete header");

    assert!(outcome.validated_package().is_none());
    assert!(outcome.report().iter().any(|diagnostic| {
        diagnostic.code == "IFCCAD_PACKAGE_SCHEMA_INVALID"
            && diagnostic.location.as_deref() == Some("/header")
            && diagnostic.context.get("property")
                == Some(&ifccad::package::PackageDiagnosticContextValue::String(
                    "id".to_owned(),
                ))
    }));
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
