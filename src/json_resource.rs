use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct LoadedJsonResource {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) uri: String,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) path: Option<PathBuf>,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) bytes: Option<Vec<u8>>,
    pub(crate) value: serde_json::Value,
}

impl LoadedJsonResource {
    pub(crate) fn inline(document_uri: String, value: serde_json::Value) -> Self {
        Self {
            uri: document_uri,
            path: None,
            bytes: None,
            value,
        }
    }
    pub(crate) fn new(
        uri: String,
        path: PathBuf,
        bytes: Vec<u8>,
        value: serde_json::Value,
    ) -> Self {
        Self {
            uri,
            path: Some(path),
            bytes: Some(bytes),
            value,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn uri(&self) -> &str {
        &self.uri
    }

    #[allow(dead_code)]
    pub(crate) fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub(crate) fn bytes(&self) -> Option<&[u8]> {
        self.bytes.as_deref()
    }

    pub(crate) fn value(&self) -> &serde_json::Value {
        &self.value
    }

    #[cfg(test)]
    pub(crate) fn with_test_value(&self, value: serde_json::Value) -> Self {
        Self {
            uri: self.uri.clone(),
            path: self.path.clone(),
            bytes: self
                .bytes
                .as_ref()
                .map(|_| serde_json::to_vec(&value).expect("serialize test JSON")),
            value,
        }
    }
}
