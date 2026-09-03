use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct EncodedIfccadPackage {
    files: BTreeMap<String, Vec<u8>>,
}

impl EncodedIfccadPackage {
    pub(crate) fn new(files: impl IntoIterator<Item = (String, Vec<u8>)>) -> Self {
        Self {
            files: files.into_iter().collect(),
        }
    }

    pub fn files(&self) -> impl ExactSizeIterator<Item = (&str, &[u8])> {
        self.files
            .iter()
            .map(|(path, bytes)| (path.as_str(), bytes.as_slice()))
    }

    pub fn file(&self, path: &str) -> Option<&[u8]> {
        self.files.get(path).map(Vec::as_slice)
    }
}
