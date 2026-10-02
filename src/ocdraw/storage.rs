use super::{load_ocdraw_bytes, OcdrawReadError, ValidatedOcdraw};
use std::path::Path;
#[derive(Debug, thiserror::Error)]
pub enum OcdrawWriteError {
    #[error("could not write drawing: {0}")]
    Io(#[from] std::io::Error),
}

/// Validated encoded bytes. Equality compares bytes, not logical drawing semantics.
#[derive(Debug, PartialEq, Eq)]
pub struct EncodedOcdraw {
    bytes: Vec<u8>,
}

impl EncodedOcdraw {
    pub(crate) fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn write_file(&self, path: impl AsRef<Path>) -> Result<(), OcdrawWriteError> {
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
pub enum OcdrawOpenError {
    #[error("could not read drawing: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Read(#[from] OcdrawReadError),
}
pub fn load_ocdraw_file(path: impl AsRef<Path>) -> Result<ValidatedOcdraw, OcdrawOpenError> {
    Ok(load_ocdraw_bytes(&std::fs::read(path)?)?)
}

impl AsRef<[u8]> for EncodedOcdraw {
    fn as_ref(&self) -> &[u8] {
        self.bytes()
    }
}
