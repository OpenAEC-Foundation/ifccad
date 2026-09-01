use super::diagnostic::PackageValidationReport;
use super::discovery::ResourceDeclaration;
use crate::ResourceId;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct LoadedJsonResource {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) uri: String,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) path: PathBuf,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) bytes: Vec<u8>,
    pub(crate) value: serde_json::Value,
}

impl LoadedJsonResource {
    #[allow(dead_code)]
    pub(crate) fn uri(&self) -> &str {
        &self.uri
    }

    #[allow(dead_code)]
    pub(crate) fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn value(&self) -> &serde_json::Value {
        &self.value
    }

    #[cfg(test)]
    pub(crate) fn with_test_value(&self, value: serde_json::Value) -> Self {
        Self {
            uri: self.uri.clone(),
            path: self.path.clone(),
            bytes: serde_json::to_vec(&value).expect("serialize test JSON"),
            value,
        }
    }
}

#[derive(Debug)]
pub(crate) struct LoadedIfccadPackage {
    pub(crate) entrypoint: LoadedJsonResource,
    pub(crate) declarations: Vec<ResourceDeclaration>,
    pub(crate) resources: BTreeMap<String, Arc<LoadedJsonResource>>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct PackageAnalysis {
    pub(crate) node_indices_by_path: BTreeMap<String, usize>,
    pub(crate) validated_ifcdr_resources:
        BTreeMap<ResourceId, Arc<crate::ifcdr::ValidatedIfcdrResource>>,
    pub(crate) bindings: super::bindings::PackageBindings,
}

/// Result of inspecting and validating an IFCCAD directory package.
#[derive(Debug)]
pub struct PackageLoadOutcome {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) package: Option<Arc<LoadedIfccadPackage>>,
    #[allow(dead_code)]
    pub(crate) analysis: Option<Arc<PackageAnalysis>>,
    #[allow(dead_code)]
    pub(crate) validated_package: Option<super::analysis::ValidatedIfccadPackage>,
    pub(crate) report: PackageValidationReport,
}

impl PackageLoadOutcome {
    /// Returns every diagnostic collected while inspecting the package.
    pub fn report(&self) -> &PackageValidationReport {
        &self.report
    }

    /// Returns the strict package proof when no error diagnostic was produced.
    pub fn validated_package(&self) -> Option<&super::analysis::ValidatedIfccadPackage> {
        self.validated_package.as_ref()
    }

    /// Consumes the outcome and returns its strict package proof, when available.
    pub fn into_validated_package(self) -> Option<super::analysis::ValidatedIfccadPackage> {
        self.validated_package
    }

    /// Consumes the outcome into its strict proof and complete diagnostic report.
    pub fn into_parts(
        self,
    ) -> (
        Option<super::analysis::ValidatedIfccadPackage>,
        PackageValidationReport,
    ) {
        (self.validated_package, self.report)
    }
}
