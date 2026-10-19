use super::*;

/// Validated CBOR output. Equality compares bytes, not semantic equivalence.
#[derive(Debug, PartialEq, Eq)]
pub struct EncodedIfccadCbor {
    bytes: Vec<u8>,
}
impl EncodedIfccadCbor {
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
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        f.write_all(&self.bytes)?;
        Ok(())
    }
}
impl AsRef<[u8]> for EncodedIfccadCbor {
    fn as_ref(&self) -> &[u8] {
        self.bytes()
    }
}
#[derive(Debug, thiserror::Error)]
pub enum IfccadCborOpenError {
    #[error("could not read IFCCAD-CBOR: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Read(#[from] IfccadCborReadError),
}
pub fn load_ifccad_cbor_file(
    path: impl AsRef<Path>,
    options: IfccadCborReadOptions,
) -> Result<ValidatedIfccadCbor, IfccadCborOpenError> {
    use std::io::Read;
    let limit = options.limits.max_input_bytes;
    let bounded = limit
        .checked_add(1)
        .filter(|v| usize::try_from(*v).is_ok())
        .ok_or_else(|| IfccadCborReadError::Malformed {
            location: "/options".into(),
            offset: 0,
            message: "input limit cannot be represented with a detection byte".into(),
        })?;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(bounded)
        .read_to_end(&mut bytes)?;
    Ok(load_ifccad_cbor_bytes(&bytes, options)?)
}
