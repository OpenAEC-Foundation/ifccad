use super::*;
use std::path::Path;

/// Validated encoded bytes. Equality compares bytes, not logical drawing semantics.
#[derive(Debug, PartialEq, Eq)]
pub struct EncodedIfccad {
    bytes: Vec<u8>,
}
impl EncodedIfccad {
    pub(crate) fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
    pub fn write_file(&self, path: impl AsRef<Path>) -> Result<(), IfccadWriteError> {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        file.write_all(&self.bytes)?;
        Ok(())
    }
}
#[derive(Debug, thiserror::Error)]
pub enum IfccadOpenError {
    #[error("could not read IFCCAD: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Read(#[from] IfccadReadError),
}
#[derive(Debug, thiserror::Error)]
pub enum IfccadWriteError {
    #[error("could not write IFCCAD: {0}")]
    Io(#[from] std::io::Error),
}
pub fn load_ifccad_file(
    path: impl AsRef<Path>,
    options: IfccadReadOptions,
) -> Result<ValidatedIfccad, IfccadOpenError> {
    Ok(load_ifccad_bytes(&std::fs::read(path)?, options)?)
}

impl AsRef<[u8]> for EncodedIfccad {
    fn as_ref(&self) -> &[u8] {
        self.bytes()
    }
}
