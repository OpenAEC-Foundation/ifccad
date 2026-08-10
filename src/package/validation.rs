use super::discovery::discover_resources;
use super::loader::{DirectoryPackageLoader, PackageLoadLimits};
use super::{PackageOpenError, PackageValidationReport};
use std::path::Path;

// The conformance composer will become this internal orchestrator's production caller.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn validate_directory_package(
    root: impl AsRef<Path>,
) -> Result<PackageValidationReport, PackageOpenError> {
    let mut loader = DirectoryPackageLoader::open(root, PackageLoadLimits::default())?;
    let Some(entrypoint) = loader.load_entrypoint()? else {
        return Ok(loader.into_report());
    };

    let discovery = discover_resources(&entrypoint.value);
    let mut references = discovery.references;
    references.sort_by(|left, right| {
        (left.uri.as_str(), left.kind, left.location.as_str()).cmp(&(
            right.uri.as_str(),
            right.kind,
            right.location.as_str(),
        ))
    });
    for reference in &references {
        loader.load_json_resource(&reference.uri, Some(&reference.location))?;
    }

    let mut diagnostics = loader.into_report().into_diagnostics();
    diagnostics.extend(discovery.diagnostics);
    Ok(PackageValidationReport::from_diagnostics(diagnostics))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conformance::bundled_conformance_root;
    use crate::package::codes::{
        IFCCAD_PACKAGE_ENTRYPOINT_INVALID, IFCCAD_PACKAGE_JSON_INVALID,
        IFCCAD_PACKAGE_RESOURCE_MISSING,
    };
    use crate::package::DIRECTORY_PACKAGE_ENTRYPOINT;
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

    #[test]
    fn validates_a_directory_from_ifcx_discovered_resources() {
        let root = TestDirectory::new("discovered-resources");
        fs::write(
            root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
            br#"{
              "data": [{
                "type": "openaec:DrawingGeometryRepresentation",
                "attributes": {
                  "geometry": { "uri": "custom-name.ifcdr.json" }
                }
              }]
            }"#,
        )
        .expect("write entrypoint");
        fs::write(root.path().join("custom-name.ifcdr.json"), b"{}")
            .expect("write discovered resource");

        let report = validate_directory_package(root.path()).expect("inspect directory package");

        assert!(report.is_valid());
        assert!(report.is_empty());
    }

    #[test]
    fn reports_a_resource_missing_at_its_ifcx_declaration() {
        let root = TestDirectory::new("missing-discovered-resource");
        fs::write(
            root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT),
            br#"{
              "data": [{
                "type": "openaec:DrawingGeometryRepresentation",
                "attributes": {
                  "geometry": { "uri": "custom-name.ifcdr.json" }
                }
              }]
            }"#,
        )
        .expect("write entrypoint");

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
        assert_eq!(
            codes,
            [
                IFCCAD_PACKAGE_ENTRYPOINT_INVALID,
                IFCCAD_PACKAGE_JSON_INVALID
            ]
        );
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
    fn validates_all_committed_loadable_directory_packages() {
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
            let report =
                validate_directory_package(root).expect("inspect committed directory package");
            assert!(report.is_empty(), "case {case}: {:?}", report.diagnostics());
        }
    }

    #[test]
    fn discovers_the_committed_missing_resource_case() {
        let root = bundled_conformance_root()
            .join("packages")
            .join("invalid")
            .join("package-missing-resource");
        let report = validate_directory_package(root).expect("inspect missing-resource case");

        assert_eq!(report.len(), 1);
        assert_eq!(
            report.diagnostics()[0].code,
            IFCCAD_PACKAGE_RESOURCE_MISSING
        );
    }
}
