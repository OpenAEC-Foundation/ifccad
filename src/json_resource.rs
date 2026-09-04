use std::path::{Path, PathBuf};

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
    pub(crate) fn new(
        uri: String,
        path: PathBuf,
        bytes: Vec<u8>,
        value: serde_json::Value,
    ) -> Self {
        Self {
            uri,
            path,
            bytes,
            value,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn uri(&self) -> &str {
        &self.uri
    }

    #[allow(dead_code)]
    pub(crate) fn path(&self) -> &Path {
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
