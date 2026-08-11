use super::discovery::{discover_resources, ResourceKind};
use super::loader::{DirectoryPackageLoader, PackageLoadLimits};
use super::model::{LoadedIfccadPackage, PackageLoadOutcome};
use super::schema::{validate_ifcpr, validate_ifcx};
use super::{
    PackageDiagnostic, PackageDiagnosticContextValue, PackageDiagnosticSeverity, PackageOpenError,
    PackageValidationReport,
};
use crate::package::codes::IFCCAD_PACKAGE_CHECKSUM_MISMATCH;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

// The conformance composer will become this internal orchestrator's production caller.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn load_directory_package(
    root: impl AsRef<Path>,
) -> Result<PackageLoadOutcome, PackageOpenError> {
    let mut loader = DirectoryPackageLoader::open(root, PackageLoadLimits::default())?;
    let Some(entrypoint) = loader.load_entrypoint()? else {
        return Ok(PackageLoadOutcome {
            package: None,
            report: loader.into_report(),
        });
    };

    let discovery = discover_resources(&entrypoint.value);
    let mut declarations = discovery.declarations;
    declarations.sort_by(|left, right| {
        (left.uri.as_str(), left.kind, left.uri_location.as_str()).cmp(&(
            right.uri.as_str(),
            right.kind,
            right.uri_location.as_str(),
        ))
    });
    let mut attempted_uris = BTreeSet::new();
    let mut resources = BTreeMap::new();
    for declaration in &declarations {
        if !attempted_uris.insert(declaration.uri.as_str()) {
            continue;
        }
        if let Some(resource) =
            loader.load_json_resource(&declaration.uri, Some(&declaration.uri_location))?
        {
            resources.insert(declaration.uri.clone(), resource);
        }
    }

    let mut diagnostics = loader.into_report().into_diagnostics();
    diagnostics.extend(discovery.diagnostics);
    let package = LoadedIfccadPackage {
        entrypoint,
        declarations,
        resources,
    };
    diagnostics.extend(validate_ifcx(&package.entrypoint.value));
    let mut validated_ifcpr_uris = BTreeSet::new();
    for declaration in &package.declarations {
        if declaration.kind != ResourceKind::Ifcpr
            || !validated_ifcpr_uris.insert(declaration.uri.as_str())
        {
            continue;
        }
        if let Some(resource) = package.resources.get(&declaration.uri) {
            diagnostics.extend(validate_ifcpr(&declaration.uri, &resource.value));
        }
    }
    diagnostics.extend(verify_resource_checksums(&package));

    Ok(PackageLoadOutcome {
        package: Some(package),
        report: PackageValidationReport::from_diagnostics(diagnostics),
    })
}

fn verify_resource_checksums(package: &LoadedIfccadPackage) -> Vec<PackageDiagnostic> {
    package
        .declarations
        .iter()
        .filter_map(|declaration| {
            let expected = declaration.checksum.as_deref()?;
            if !is_sha256_checksum(expected) {
                return None;
            }
            let resource = package.resources.get(&declaration.uri)?;
            let actual = sha256_checksum(&resource.bytes);
            if actual == expected {
                return None;
            }
            Some(PackageDiagnostic {
                code: IFCCAD_PACKAGE_CHECKSUM_MISMATCH.to_owned(),
                severity: PackageDiagnosticSeverity::Error,
                resource_uri: Some(declaration.uri.clone()),
                location: Some(declaration.checksum_location.clone()),
                context: BTreeMap::from([
                    (
                        "actualChecksum".to_owned(),
                        PackageDiagnosticContextValue::String(actual),
                    ),
                    (
                        "expectedChecksum".to_owned(),
                        PackageDiagnosticContextValue::String(expected.to_owned()),
                    ),
                ]),
                message: format!(
                    "resource checksum does not match the exact bytes for {:?}",
                    declaration.uri
                ),
            })
        })
        .collect()
}

fn is_sha256_checksum(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn sha256_checksum(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn validate_directory_package(
    root: impl AsRef<Path>,
) -> Result<PackageValidationReport, PackageOpenError> {
    Ok(load_directory_package(root)?.report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conformance::bundled_conformance_root;
    use crate::package::codes::{
        IFCCAD_PACKAGE_CHECKSUM_MISMATCH, IFCCAD_PACKAGE_ENTRYPOINT_INVALID,
        IFCCAD_PACKAGE_JSON_INVALID, IFCCAD_PACKAGE_RESOURCE_MISSING,
        IFCCAD_PACKAGE_SCHEMA_INVALID,
    };
    use crate::package::{PackageDiagnosticContextValue, DIRECTORY_PACKAGE_ENTRYPOINT};
    use sha2::{Digest, Sha256};
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
                "ifccad-package-validation-{name}-{}-{nonce}",
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

    fn valid_checksum() -> String {
        format!("sha256:{}", "a".repeat(64))
    }

    fn write_geometry_entrypoint(root: &Path, uri: &str, checksum: &str) {
        let entrypoint = serde_json::json!({
            "data": [{
                "path": "geometry",
                "type": "openaec:DrawingGeometryRepresentation",
                "attributes": {
                    "geometry": {
                        "format": "openaec.ifcdr",
                        "version": "0.5.0",
                        "uri": uri,
                        "checksum": checksum,
                        "role": "modelspace"
                    }
                }
            }]
        });
        fs::write(
            root.join(DIRECTORY_PACKAGE_ENTRYPOINT),
            serde_json::to_vec(&entrypoint).expect("serialize entrypoint"),
        )
        .expect("write entrypoint");
    }

    #[test]
    fn load_outcome_retains_entrypoint_resources_and_exact_bytes() {
        let root = TestDirectory::new("loaded-model");
        let entrypoint = br#"{"data":[{"path":"geometry","type":"openaec:DrawingGeometryRepresentation","attributes":{"geometry":{"format":"openaec.ifcdr","version":"0.5.0","uri":"drawing.ifcdr.json","checksum":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","role":"modelspace"}}}]}"#;
        let drawing = b"{\r\n  \"header\": {}\r\n}\r\n";
        fs::write(root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT), entrypoint)
            .expect("write entrypoint");
        fs::write(root.path().join("drawing.ifcdr.json"), drawing).expect("write drawing resource");

        let outcome = load_directory_package(root.path()).expect("load directory package");
        let package = outcome.package.expect("entrypoint produced package");

        assert_eq!(package.entrypoint.bytes, entrypoint);
        assert_eq!(package.entrypoint.value["data"][0]["path"], "geometry");
        assert_eq!(package.declarations.len(), 1);
        assert_eq!(package.declarations[0].uri, "drawing.ifcdr.json");
        assert_eq!(package.resources["drawing.ifcdr.json"].bytes, drawing);
    }

    #[test]
    fn missing_resource_keeps_partial_loaded_package() {
        let root = TestDirectory::new("partial-missing-resource");
        write_geometry_entrypoint(root.path(), "missing.ifcdr.json", &valid_checksum());

        let outcome = load_directory_package(root.path()).expect("load directory package");
        let package = outcome.package.expect("parsed entrypoint is retained");

        assert_eq!(package.declarations[0].uri, "missing.ifcdr.json");
        assert!(!package.resources.contains_key("missing.ifcdr.json"));
        assert_eq!(
            outcome.report.diagnostics()[0].code,
            IFCCAD_PACKAGE_RESOURCE_MISSING
        );
    }

    #[test]
    fn malformed_entrypoint_has_no_loaded_package() {
        let root = TestDirectory::new("malformed-entrypoint-outcome");
        fs::write(root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT), b"{")
            .expect("write malformed entrypoint");

        let outcome = load_directory_package(root.path()).expect("inspect directory package");

        assert!(outcome.package.is_none());
        assert_eq!(outcome.report.len(), 1);
        assert_eq!(
            outcome.report.diagnostics()[0].code,
            IFCCAD_PACKAGE_JSON_INVALID
        );
    }

    #[test]
    fn ifcx_schema_error_keeps_loaded_entrypoint() {
        let root = TestDirectory::new("ifcx-schema-partial");
        fs::write(
            root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
            br#"{"data":[{"path":"layout","type":"openaec:DrawingLayout","attributes":{"name":"Model","kind":"sheet","scopeId":0},"children":{"Representation":"geometry"}}]}"#,
        )
        .expect("write entrypoint");

        let outcome = load_directory_package(root.path()).expect("load directory package");

        assert!(outcome.package.is_some());
        assert!(outcome.report.iter().any(|item| {
            item.code == IFCCAD_PACKAGE_SCHEMA_INVALID
                && item.location.as_deref() == Some("/data/0/attributes/kind")
        }));
    }

    #[test]
    fn checksum_uses_exact_resource_bytes() {
        let root = TestDirectory::new("exact-checksum");
        let bytes = b"{\r\n  \"header\": {}\r\n}\r\n";
        let checksum = format!("sha256:{:x}", Sha256::digest(bytes));
        write_geometry_entrypoint(root.path(), "drawing.ifcdr.json", &checksum);
        fs::write(root.path().join("drawing.ifcdr.json"), bytes).expect("write drawing resource");

        let matching = load_directory_package(root.path()).expect("load matching package");
        assert!(!matching
            .report
            .iter()
            .any(|item| item.code == IFCCAD_PACKAGE_CHECKSUM_MISMATCH));

        fs::write(root.path().join("drawing.ifcdr.json"), b"{\"header\":{}}")
            .expect("change exact resource bytes");
        let changed = load_directory_package(root.path()).expect("load changed package");
        let diagnostic = changed
            .report
            .iter()
            .find(|item| item.code == IFCCAD_PACKAGE_CHECKSUM_MISMATCH)
            .expect("changed exact bytes must mismatch");

        assert_eq!(
            diagnostic.location.as_deref(),
            Some("/data/0/attributes/geometry/checksum")
        );
        assert_eq!(
            diagnostic.context.get("expectedChecksum"),
            Some(&PackageDiagnosticContextValue::String(checksum))
        );
        assert_eq!(
            diagnostic.context.get("actualChecksum"),
            Some(&PackageDiagnosticContextValue::String(format!(
                "sha256:{:x}",
                Sha256::digest(b"{\"header\":{}}")
            )))
        );
    }

    #[test]
    fn malformed_checksum_is_not_reported_as_a_mismatch() {
        let root = TestDirectory::new("malformed-checksum");
        write_geometry_entrypoint(root.path(), "drawing.ifcdr.json", "sha256:ABC");
        fs::write(root.path().join("drawing.ifcdr.json"), b"{}").expect("write drawing resource");

        let outcome = load_directory_package(root.path()).expect("load package");
        let codes: Vec<_> = outcome
            .report
            .iter()
            .map(|item| item.code.as_str())
            .collect();

        assert!(codes.contains(&IFCCAD_PACKAGE_SCHEMA_INVALID));
        assert!(!codes.contains(&IFCCAD_PACKAGE_CHECKSUM_MISMATCH));
    }

    #[test]
    fn validates_a_directory_from_ifcx_discovered_resources() {
        let root = TestDirectory::new("discovered-resources");
        let resource = b"{}";
        let checksum = format!("sha256:{:x}", Sha256::digest(resource));
        write_geometry_entrypoint(root.path(), "custom-name.ifcdr.json", &checksum);
        fs::write(root.path().join("custom-name.ifcdr.json"), resource)
            .expect("write discovered resource");

        let report = validate_directory_package(root.path()).expect("inspect directory package");

        assert!(report.is_valid());
        assert!(report.is_empty());
    }

    #[test]
    fn reports_a_resource_missing_at_its_ifcx_declaration() {
        let root = TestDirectory::new("missing-discovered-resource");
        write_geometry_entrypoint(root.path(), "custom-name.ifcdr.json", &valid_checksum());

        let report = validate_directory_package(root.path()).expect("inspect directory package");

        assert_eq!(report.len(), 1);
        let diagnostic = &report.diagnostics()[0];
        assert_eq!(diagnostic.code, IFCCAD_PACKAGE_RESOURCE_MISSING);
        assert_eq!(
            diagnostic.resource_uri.as_deref(),
            Some("custom-name.ifcdr.json")
        );
        assert_eq!(
            diagnostic.location.as_deref(),
            Some("/data/0/attributes/geometry/uri")
        );
    }

    #[test]
    fn keeps_discovery_diagnostics_with_loader_diagnostics() {
        let root = TestDirectory::new("combined-diagnostics");
        fs::write(
            root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
            br#"{
              "data": [
                {
                  "type": "openaec:DrawingGeometryRepresentation",
                  "attributes": { "geometry": { "url": "ignored.json" } }
                },
                {
                  "type": "openaec:PreservationRepresentation",
                  "attributes": {
                    "preservation": { "uri": "malformed.ifcpr.json" }
                  }
                }
              ]
            }"#,
        )
        .expect("write entrypoint");
        fs::write(root.path().join("malformed.ifcpr.json"), b"{")
            .expect("write malformed resource");

        let report = validate_directory_package(root.path()).expect("inspect directory package");

        let codes: Vec<_> = report.iter().map(|item| item.code.as_str()).collect();
        assert!(codes.contains(&IFCCAD_PACKAGE_ENTRYPOINT_INVALID));
        assert!(codes.contains(&IFCCAD_PACKAGE_JSON_INVALID));
        assert!(codes.contains(&IFCCAD_PACKAGE_SCHEMA_INVALID));
    }

    #[test]
    fn returns_entrypoint_loading_diagnostics_without_discovery() {
        let root = TestDirectory::new("malformed-entrypoint");
        fs::write(root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT), b"{")
            .expect("write malformed entrypoint");

        let report = validate_directory_package(root.path()).expect("inspect directory package");

        assert_eq!(report.len(), 1);
        assert_eq!(report.diagnostics()[0].code, IFCCAD_PACKAGE_JSON_INVALID);
    }

    #[test]
    fn loads_committed_bootstrap_packages_and_reports_legacy_links() {
        for case in [
            "minimal-no-preservation",
            "unrepresented-packed",
            "source-archive",
            "generated-snapshot",
            "multi-drawing-projections",
        ] {
            let root = bundled_conformance_root()
                .join("packages")
                .join("valid")
                .join(case);
            let outcome = load_directory_package(root).expect("inspect committed package");
            assert!(outcome.package.is_some(), "case {case}");

            if case == "minimal-no-preservation" {
                assert!(
                    outcome.report.is_empty(),
                    "case {case}: {:?}",
                    outcome.report.diagnostics()
                );
                continue;
            }

            let properties: Vec<_> = outcome
                .report
                .iter()
                .filter_map(|item| item.context.get("property"))
                .collect();
            assert!(properties.contains(&&PackageDiagnosticContextValue::String(
                "linkedDrawingResourceIds".to_owned()
            )));
            assert!(properties.contains(&&PackageDiagnosticContextValue::String(
                "linkedDrawingResourceUris".to_owned()
            )));
        }
    }

    #[test]
    fn discovers_the_committed_missing_resource_case() {
        let root = bundled_conformance_root()
            .join("packages")
            .join("invalid")
            .join("package-missing-resource");
        let outcome = load_directory_package(root).expect("inspect missing-resource case");

        assert!(outcome.package.is_some());
        assert!(outcome
            .report
            .iter()
            .any(|item| item.code == IFCCAD_PACKAGE_RESOURCE_MISSING));
    }
}
