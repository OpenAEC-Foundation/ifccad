use super::*;

/// Validated experimental Zstd output; not a general IFCX serialization.
#[derive(Debug, PartialEq, Eq)]
pub struct EncodedIfccadCborZstd {
    bytes: Vec<u8>,
}
impl EncodedIfccadCborZstd {
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
impl AsRef<[u8]> for EncodedIfccadCborZstd {
    fn as_ref(&self) -> &[u8] {
        self.bytes()
    }
}
#[derive(Debug, thiserror::Error)]
pub enum IfccadCborZstdOpenError {
    #[error("could not read IFCCAD-CBOR Zstd: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Read(#[from] IfccadCborZstdReadError),
}
pub fn load_ifccad_cbor_zstd_file(
    path: impl AsRef<Path>,
    options: IfccadCborZstdReadOptions,
) -> Result<ValidatedIfccadCbor, IfccadCborZstdOpenError> {
    use std::io::Read;
    options.validate()?;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(options.limits.max_compressed_bytes + 1)
        .read_to_end(&mut bytes)?;
    Ok(load_ifccad_cbor_zstd_bytes(&bytes, options)?)
}
