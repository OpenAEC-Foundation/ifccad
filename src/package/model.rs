use super::diagnostic::PackageValidationReport;
use super::discovery::ResourceDeclaration;
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
    pub(crate) fn uri(&self) -> &str {
        &self.uri
    }

    pub(crate) fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn value(&self) -> &serde_json::Value {
        &self.value
    }
}

#[derive(Debug)]
pub(crate) struct LoadedIfccadPackage {
    pub(crate) entrypoint: LoadedJsonResource,
    pub(crate) declarations: Vec<ResourceDeclaration>,
    pub(crate) resources: BTreeMap<String, Arc<LoadedJsonResource>>,
    pub(crate) node_indices_by_path: BTreeMap<String, usize>,
}

#[derive(Debug)]
pub(crate) struct PackageLoadOutcome {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) package: Option<LoadedIfccadPackage>,
    pub(crate) report: PackageValidationReport,
}
