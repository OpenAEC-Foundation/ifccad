use super::discovery::{discover_resources, ResourceKind};
use super::loader::{DirectoryPackageLoader, PackageLoadLimits};
use super::model::{LoadedIfccadPackage, PackageLoadOutcome};
use super::schema::{validate_ifcpr, validate_ifcx};
use super::{PackageOpenError, PackageValidationReport};
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

    Ok(PackageLoadOutcome {
        package: Some(package),
        report: PackageValidationReport::from_diagnostics(diagnostics),
    })
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
        IFCCAD_PACKAGE_ENTRYPOINT_INVALID, IFCCAD_PACKAGE_JSON_INVALID,
        IFCCAD_PACKAGE_RESOURCE_MISSING, IFCCAD_PACKAGE_SCHEMA_INVALID,
    };
    use crate::package::{PackageDiagnosticContextValue, DIRECTORY_PACKAGE_ENTRYPOINT};
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
    fn validates_a_directory_from_ifcx_discovered_resources() {
        let root = TestDirectory::new("discovered-resources");
        write_geometry_entrypoint(root.path(), "custom-name.ifcdr.json", &valid_checksum());
        fs::write(root.path().join("custom-name.ifcdr.json"), b"{}")
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
