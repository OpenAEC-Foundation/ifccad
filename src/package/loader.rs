use super::codes::{
    IFCCAD_PACKAGE_ENTRYPOINT_MISSING, IFCCAD_PACKAGE_JSON_INVALID, IFCCAD_PACKAGE_PATH_INVALID,
    IFCCAD_PACKAGE_RESOURCE_LIMIT_EXCEEDED, IFCCAD_PACKAGE_RESOURCE_MISSING,
    IFCCAD_PACKAGE_TOTAL_LIMIT_EXCEEDED,
};
use super::model::LoadedJsonResource;
use super::path::{PackagePathResolution, PackageRoot, ResolvePackagePathError};
use super::{
    PackageDiagnostic, PackageDiagnosticContextValue, PackageDiagnosticSeverity, PackageOpenError,
    PackageValidationReport, DIRECTORY_PACKAGE_ENTRYPOINT,
};
use std::cmp;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

const DEFAULT_MAX_RESOURCE_BYTES: u64 = 256 * 1024 * 1024;
const DEFAULT_MAX_TOTAL_BYTES: u64 = 1024 * 1024 * 1024;

#[derive(Debug, Clone, Copy)]
pub(crate) struct PackageLoadLimits {
    pub(crate) max_resource_bytes: u64,
    pub(crate) max_total_bytes: u64,
}

impl Default for PackageLoadLimits {
    fn default() -> Self {
        Self {
            max_resource_bytes: DEFAULT_MAX_RESOURCE_BYTES,
            max_total_bytes: DEFAULT_MAX_TOTAL_BYTES,
        }
    }
}

#[derive(Debug)]
pub(crate) struct DirectoryPackageLoader {
    root: PackageRoot,
    limits: PackageLoadLimits,
    loaded_bytes: u64,
    diagnostics: Vec<PackageDiagnostic>,
}

impl DirectoryPackageLoader {
    pub(crate) fn open(
        root: impl AsRef<Path>,
        limits: PackageLoadLimits,
    ) -> Result<Self, PackageOpenError> {
        Ok(Self {
            root: PackageRoot::open(root)?,
            limits,
            loaded_bytes: 0,
            diagnostics: Vec::new(),
        })
    }

    pub(crate) fn load_entrypoint(
        &mut self,
    ) -> Result<Option<LoadedJsonResource>, PackageOpenError> {
        self.load_json(
            DIRECTORY_PACKAGE_ENTRYPOINT,
            None,
            IFCCAD_PACKAGE_ENTRYPOINT_MISSING,
        )
    }

    pub(crate) fn load_json_resource(
        &mut self,
        uri: &str,
        declaration_location: Option<&str>,
    ) -> Result<Option<LoadedJsonResource>, PackageOpenError> {
        self.load_json(uri, declaration_location, IFCCAD_PACKAGE_RESOURCE_MISSING)
    }

    pub(crate) fn into_report(self) -> PackageValidationReport {
        PackageValidationReport::from_diagnostics(self.diagnostics)
    }

    fn load_json(
        &mut self,
        uri: &str,
        declaration_location: Option<&str>,
        missing_code: &'static str,
    ) -> Result<Option<LoadedJsonResource>, PackageOpenError> {
        let path = match self.root.resolve(uri) {
            Ok(PackagePathResolution::Existing(path)) => path,
            Ok(PackagePathResolution::Missing(_)) => {
                self.push_missing(missing_code, uri, declaration_location, "file is missing");
                return Ok(None);
            }
            Err(ResolvePackagePathError::Unsafe(rejected)) => {
                let rejected_uri = rejected.uri;
                self.push_error(
                    IFCCAD_PACKAGE_PATH_INVALID,
                    None,
                    declaration_location,
                    context([(
                        "invalidUri",
                        PackageDiagnosticContextValue::String(rejected_uri.clone()),
                    )]),
                    format!("invalid package resource path {rejected_uri:?}"),
                );
                return Ok(None);
            }
            Err(ResolvePackagePathError::Io(error)) => return Err(error),
        };

        let metadata = match fs::metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                self.push_missing(missing_code, uri, declaration_location, "file is missing");
                return Ok(None);
            }
            Err(source) => {
                return Err(PackageOpenError::Io {
                    path: path.clone(),
                    source,
                });
            }
        };
        if !metadata.is_file() {
            self.push_missing(
                missing_code,
                uri,
                declaration_location,
                "path is not a regular file",
            );
            return Ok(None);
        }

        if metadata.len() > self.limits.max_resource_bytes {
            self.push_limit(
                IFCCAD_PACKAGE_RESOURCE_LIMIT_EXCEEDED,
                uri,
                declaration_location,
                self.limits.max_resource_bytes,
                metadata.len(),
            );
            return Ok(None);
        }
        let total_from_metadata = self.loaded_bytes.saturating_add(metadata.len());
        if total_from_metadata > self.limits.max_total_bytes {
            self.push_limit(
                IFCCAD_PACKAGE_TOTAL_LIMIT_EXCEEDED,
                uri,
                declaration_location,
                self.limits.max_total_bytes,
                total_from_metadata,
            );
            return Ok(None);
        }

        let remaining_total = self
            .limits
            .max_total_bytes
            .saturating_sub(self.loaded_bytes);
        let read_limit = cmp::min(self.limits.max_resource_bytes, remaining_total);
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                self.push_missing(missing_code, uri, declaration_location, "file is missing");
                return Ok(None);
            }
            Err(source) => {
                return Err(PackageOpenError::Io {
                    path: path.clone(),
                    source,
                });
            }
        };
        let mut bytes = Vec::new();
        file.take(read_limit.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|source| PackageOpenError::Io {
                path: path.clone(),
                source,
            })?;
        let observed_resource_bytes = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if observed_resource_bytes > self.limits.max_resource_bytes {
            self.push_limit(
                IFCCAD_PACKAGE_RESOURCE_LIMIT_EXCEEDED,
                uri,
                declaration_location,
                self.limits.max_resource_bytes,
                observed_resource_bytes,
            );
            return Ok(None);
        }
        let observed_total_bytes = self.loaded_bytes.saturating_add(observed_resource_bytes);
        if observed_total_bytes > self.limits.max_total_bytes {
            self.push_limit(
                IFCCAD_PACKAGE_TOTAL_LIMIT_EXCEEDED,
                uri,
                declaration_location,
                self.limits.max_total_bytes,
                observed_total_bytes,
            );
            return Ok(None);
        }
        self.loaded_bytes = observed_total_bytes;

        let value = match serde_json::from_slice(&bytes) {
            Ok(value) => value,
            Err(error) => {
                self.push_error(
                    IFCCAD_PACKAGE_JSON_INVALID,
                    Some(uri),
                    declaration_location,
                    context([
                        (
                            "column",
                            PackageDiagnosticContextValue::Number(
                                u64::try_from(error.column()).unwrap_or(u64::MAX).into(),
                            ),
                        ),
                        (
                            "line",
                            PackageDiagnosticContextValue::Number(
                                u64::try_from(error.line()).unwrap_or(u64::MAX).into(),
                            ),
                        ),
                    ]),
                    format!("invalid JSON in package resource {uri}"),
                );
                return Ok(None);
            }
        };

        Ok(Some(LoadedJsonResource {
            uri: uri.to_owned(),
            path,
            bytes,
            value,
        }))
    }

    fn push_missing(
        &mut self,
        code: &'static str,
        uri: &str,
        location: Option<&str>,
        reason: &str,
    ) {
        self.push_error(
            code,
            Some(uri),
            location,
            context([(
                "missingUri",
                PackageDiagnosticContextValue::String(uri.to_owned()),
            )]),
            format!("package resource {uri} cannot be loaded: {reason}"),
        );
    }

    fn push_limit(
        &mut self,
        code: &'static str,
        uri: &str,
        location: Option<&str>,
        limit: u64,
        observed_bytes: u64,
    ) {
        self.push_error(
            code,
            Some(uri),
            location,
            context([
                ("limit", PackageDiagnosticContextValue::Number(limit.into())),
                (
                    "observedBytes",
                    PackageDiagnosticContextValue::Number(observed_bytes.into()),
                ),
            ]),
            format!("package resource {uri} exceeds a byte limit"),
        );
    }

    fn push_error(
        &mut self,
        code: &'static str,
        resource_uri: Option<&str>,
        location: Option<&str>,
        context: BTreeMap<String, PackageDiagnosticContextValue>,
        message: impl Into<String>,
    ) {
        self.diagnostics.push(PackageDiagnostic {
            code: code.to_owned(),
            severity: PackageDiagnosticSeverity::Error,
            resource_uri: resource_uri.map(str::to_owned),
            location: location.map(str::to_owned),
            context,
            message: message.into(),
        });
    }
}

fn context<const N: usize>(
    entries: [(&str, PackageDiagnosticContextValue); N],
) -> BTreeMap<String, PackageDiagnosticContextValue> {
    entries
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conformance::bundled_conformance_root;
    use crate::package::codes::{
        IFCCAD_PACKAGE_ENTRYPOINT_MISSING, IFCCAD_PACKAGE_JSON_INVALID,
        IFCCAD_PACKAGE_PATH_INVALID, IFCCAD_PACKAGE_RESOURCE_LIMIT_EXCEEDED,
        IFCCAD_PACKAGE_RESOURCE_MISSING, IFCCAD_PACKAGE_TOTAL_LIMIT_EXCEEDED,
    };
    use crate::package::{
        PackageDiagnosticContextValue, PackageDiagnosticSeverity, DIRECTORY_PACKAGE_ENTRYPOINT,
    };
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
            let path = std::env::temp_dir().join(format!(
                "ifccad-package-loader-{name}-{}-{nonce}",
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

    fn limits(max_resource_bytes: u64, max_total_bytes: u64) -> PackageLoadLimits {
        PackageLoadLimits {
            max_resource_bytes,
            max_total_bytes,
        }
    }

    #[test]
    fn entrypoint_load_retains_exact_bytes_and_parsed_json() {
        let root = TestDirectory::new("exact-entrypoint");
        let original = b"{\r\n  \"data\": []\r\n}\r\n  ";
        fs::write(root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT), original)
            .expect("write entrypoint");
        let mut loader = DirectoryPackageLoader::open(root.path(), limits(1024, 2048))
            .expect("open package loader");

        let loaded = loader
            .load_entrypoint()
            .expect("read entrypoint")
            .expect("loaded entrypoint");

        assert_eq!(loaded.uri, DIRECTORY_PACKAGE_ENTRYPOINT);
        assert_eq!(loaded.bytes, original);
        assert_eq!(loaded.value, json!({ "data": [] }));
        assert_eq!(
            loaded.path,
            fs::canonicalize(root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT))
                .expect("canonical entrypoint")
        );
        assert!(loader.into_report().is_valid());
    }

    #[test]
    fn missing_entrypoint_is_a_diagnostic_not_an_open_error() {
        let root = TestDirectory::new("missing-entrypoint");
        let mut loader = DirectoryPackageLoader::open(root.path(), limits(1024, 2048))
            .expect("open package loader");

        assert!(loader
            .load_entrypoint()
            .expect("inspect missing entrypoint")
            .is_none());
        let report = loader.into_report();

        assert_eq!(report.len(), 1);
        let diagnostic = &report.diagnostics()[0];
        assert_eq!(diagnostic.code, IFCCAD_PACKAGE_ENTRYPOINT_MISSING);
        assert_eq!(diagnostic.severity, PackageDiagnosticSeverity::Error);
        assert_eq!(
            diagnostic.resource_uri.as_deref(),
            Some(DIRECTORY_PACKAGE_ENTRYPOINT)
        );
        assert_eq!(diagnostic.location, None);
        assert_eq!(
            diagnostic.context.get("missingUri"),
            Some(&PackageDiagnosticContextValue::String(
                DIRECTORY_PACKAGE_ENTRYPOINT.to_owned()
            ))
        );
    }

    #[test]
    fn malformed_entrypoint_json_is_a_diagnostic() {
        let root = TestDirectory::new("malformed-entrypoint");
        fs::write(root.path().join(DIRECTORY_PACKAGE_ENTRYPOINT), b"{")
            .expect("write malformed JSON");
        let mut loader = DirectoryPackageLoader::open(root.path(), limits(1024, 2048))
            .expect("open package loader");

        assert!(loader
            .load_entrypoint()
            .expect("inspect malformed entrypoint")
            .is_none());
        let report = loader.into_report();

        assert_eq!(report.len(), 1);
        let diagnostic = &report.diagnostics()[0];
        assert_eq!(diagnostic.code, IFCCAD_PACKAGE_JSON_INVALID);
        assert_eq!(diagnostic.severity, PackageDiagnosticSeverity::Error);
        assert_eq!(
            diagnostic.resource_uri.as_deref(),
            Some(DIRECTORY_PACKAGE_ENTRYPOINT)
        );
        assert_eq!(diagnostic.location, None);
        assert_eq!(
            diagnostic.context.get("line"),
            Some(&PackageDiagnosticContextValue::Number(1.into()))
        );
        assert_eq!(
            diagnostic.context.get("column"),
            Some(&PackageDiagnosticContextValue::Number(1.into()))
        );
    }

    #[test]
    fn unsafe_resource_uri_is_a_path_diagnostic() {
        let root = TestDirectory::new("unsafe-resource");
        let mut loader = DirectoryPackageLoader::open(root.path(), limits(1024, 2048))
            .expect("open package loader");
        let location = "/data/0/attributes/geometry/uri";

        assert!(loader
            .load_json_resource("../outside.json", Some(location))
            .expect("inspect unsafe resource")
            .is_none());
        let report = loader.into_report();

        assert_eq!(report.len(), 1);
        let diagnostic = &report.diagnostics()[0];
        assert_eq!(diagnostic.code, IFCCAD_PACKAGE_PATH_INVALID);
        assert_eq!(diagnostic.resource_uri, None);
        assert_eq!(diagnostic.location.as_deref(), Some(location));
        assert_eq!(
            diagnostic.context.get("invalidUri"),
            Some(&PackageDiagnosticContextValue::String(
                "../outside.json".to_owned()
            ))
        );
    }

    #[test]
    fn missing_referenced_resource_is_a_resource_diagnostic() {
        let root = TestDirectory::new("missing-resource");
        let mut loader = DirectoryPackageLoader::open(root.path(), limits(1024, 2048))
            .expect("open package loader");
        let location = "/data/0/attributes/geometry/uri";

        assert!(loader
            .load_json_resource("missing.json", Some(location))
            .expect("inspect missing resource")
            .is_none());
        let report = loader.into_report();

        assert_eq!(report.len(), 1);
        let diagnostic = &report.diagnostics()[0];
        assert_eq!(diagnostic.code, IFCCAD_PACKAGE_RESOURCE_MISSING);
        assert_eq!(diagnostic.resource_uri.as_deref(), Some("missing.json"));
        assert_eq!(diagnostic.location.as_deref(), Some(location));
        assert_eq!(
            diagnostic.context.get("missingUri"),
            Some(&PackageDiagnosticContextValue::String(
                "missing.json".to_owned()
            ))
        );
    }

    #[test]
    fn directory_is_not_loaded_as_a_json_resource() {
        let root = TestDirectory::new("directory-resource");
        fs::create_dir(root.path().join("drawing.json")).expect("create resource directory");
        let mut loader = DirectoryPackageLoader::open(root.path(), limits(1024, 2048))
            .expect("open package loader");

        assert!(loader
            .load_json_resource("drawing.json", None)
            .expect("inspect resource directory")
            .is_none());
        let report = loader.into_report();

        assert_eq!(report.len(), 1);
        assert_eq!(
            report.diagnostics()[0].code,
            IFCCAD_PACKAGE_RESOURCE_MISSING
        );
    }

    #[test]
    fn resource_limit_is_checked_with_a_bounded_read() {
        let root = TestDirectory::new("resource-limit");
        fs::write(root.path().join("exact.json"), b"{}").expect("write exact resource");
        fs::write(root.path().join("over.json"), b"[] ").expect("write oversized resource");
        let mut loader =
            DirectoryPackageLoader::open(root.path(), limits(2, 10)).expect("open package loader");

        assert!(loader
            .load_json_resource("exact.json", None)
            .expect("load exact resource")
            .is_some());
        assert!(loader
            .load_json_resource("over.json", None)
            .expect("inspect oversized resource")
            .is_none());
        let report = loader.into_report();

        assert_eq!(report.len(), 1);
        let diagnostic = &report.diagnostics()[0];
        assert_eq!(diagnostic.code, IFCCAD_PACKAGE_RESOURCE_LIMIT_EXCEEDED);
        assert_eq!(
            diagnostic.context.get("limit"),
            Some(&PackageDiagnosticContextValue::Number(2.into()))
        );
        assert_eq!(
            diagnostic.context.get("observedBytes"),
            Some(&PackageDiagnosticContextValue::Number(3.into()))
        );
    }

    #[test]
    fn aggregate_limit_counts_exact_bytes_across_successful_loads() {
        let root = TestDirectory::new("aggregate-limit");
        fs::write(root.path().join("first.json"), b"{}").expect("write first resource");
        fs::write(root.path().join("second.json"), b"[]").expect("write second resource");
        let mut loader =
            DirectoryPackageLoader::open(root.path(), limits(10, 3)).expect("open package loader");

        assert!(loader
            .load_json_resource("first.json", None)
            .expect("load first resource")
            .is_some());
        assert!(loader
            .load_json_resource("second.json", None)
            .expect("inspect second resource")
            .is_none());
        let report = loader.into_report();

        assert_eq!(report.len(), 1);
        let diagnostic = &report.diagnostics()[0];
        assert_eq!(diagnostic.code, IFCCAD_PACKAGE_TOTAL_LIMIT_EXCEEDED);
        assert_eq!(
            diagnostic.context.get("limit"),
            Some(&PackageDiagnosticContextValue::Number(3.into()))
        );
        assert_eq!(
            diagnostic.context.get("observedBytes"),
            Some(&PackageDiagnosticContextValue::Number(4.into()))
        );
    }

    #[test]
    fn opens_every_committed_valid_ifcx_entrypoint() {
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
            let mut loader = DirectoryPackageLoader::open(root, PackageLoadLimits::default())
                .expect("open package root");

            let entrypoint = loader
                .load_entrypoint()
                .expect("read package entrypoint")
                .expect("valid package entrypoint");

            assert!(entrypoint.value.get("data").is_some(), "case {case}");
            assert!(loader.into_report().is_valid(), "case {case}");
        }
    }
}
