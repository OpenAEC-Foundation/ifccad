use super::*;
use std::path::Path;

/// Validated encoded bytes. Equality compares bytes, not logical drawing semantics.
#[derive(Debug, PartialEq, Eq)]
pub struct EncodedIfcxCad {
    bytes: Vec<u8>,
}
impl EncodedIfcxCad {
    pub(crate) fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
    pub fn write_file(&self, path: impl AsRef<Path>) -> Result<(), IfcxCadWriteError> {
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
pub enum IfcxCadOpenError {
    #[error("could not read IFCX-CAD: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Read(#[from] IfcxCadReadError),
}
#[derive(Debug, thiserror::Error)]
pub enum IfcxCadWriteError {
    #[error("could not write IFCX-CAD: {0}")]
    Io(#[from] std::io::Error),
}
pub fn load_ifcx_cad_file(
    path: impl AsRef<Path>,
    options: IfcxCadReadOptions,
) -> Result<ValidatedIfcxCad, IfcxCadOpenError> {
    Ok(load_ifcx_cad_bytes(&std::fs::read(path)?, options)?)
}

impl AsRef<[u8]> for EncodedIfcxCad {
    fn as_ref(&self) -> &[u8] {
        self.bytes()
    }
}
