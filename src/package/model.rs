use super::diagnostic::PackageValidationReport;
use super::discovery::ResourceDeclaration;
use std::collections::BTreeMap;
use std::path::PathBuf;

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

#[derive(Debug)]
pub(crate) struct LoadedIfccadPackage {
    pub(crate) entrypoint: LoadedJsonResource,
    pub(crate) declarations: Vec<ResourceDeclaration>,
    pub(crate) resources: BTreeMap<String, LoadedJsonResource>,
}

#[derive(Debug)]
pub(crate) struct PackageLoadOutcome {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) package: Option<LoadedIfccadPackage>,
    pub(crate) report: PackageValidationReport,
}
