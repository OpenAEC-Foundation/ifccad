use super::*;

/// Resource limits for untrusted binary input; not drawing-semantic maxima.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IfccadCborReadLimits {
    pub max_input_bytes: u64,
    pub max_depth: u32,
    pub max_container_entries: u64,
    pub max_total_items: u64,
}
impl Default for IfccadCborReadLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 512 * 1024 * 1024,
            max_depth: 128,
            max_container_entries: 16_000_000,
            max_total_items: 32_000_000,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IfccadCborReadOptions {
    pub limits: IfccadCborReadLimits,
}

/// Physical, version, resource and logical failures remain distinguishable.
#[derive(Clone, Debug, thiserror::Error)]
pub enum IfccadCborReadError {
    #[error("IFCCAD-CBOR malformed at {location}, byte {offset}: {message}")]
    Malformed {
        location: String,
        offset: usize,
        message: String,
    },
    #[error("IFCCAD-CBOR mapping revision {found} is unsupported (supported: {supported})")]
    UnsupportedRevision { found: u32, supported: u32 },
    #[error("IFCCAD-CBOR resource limit {limit} at {location}: {observed} exceeds {allowed}")]
    ResourceLimit {
        location: String,
        limit: &'static str,
        allowed: u64,
        observed: u64,
    },
    #[error("{0}")]
    InvalidDocument(#[source] IfccadReport),
}
impl IfccadCborReadError {
    pub fn report(&self) -> Option<&IfccadReport> {
        if let Self::InvalidDocument(r) = self {
            Some(r)
        } else {
            None
        }
    }
}

/// A validated binary CAD document, without an invented IFCX source graph.
#[derive(Clone, Debug)]
pub struct ValidatedIfccadCbor {
    document: IfccadDocument,
}
impl ValidatedIfccadCbor {
    pub fn document(&self) -> &IfccadDocument {
        &self.document
    }
    pub fn into_document(self) -> IfccadDocument {
        self.document
    }
}
pub fn load_ifccad_cbor_bytes(
    bytes: &[u8],
    options: IfccadCborReadOptions,
) -> Result<ValidatedIfccadCbor, IfccadCborReadError> {
    let document = crate::ifccad::codec::cbor::decode_bytes(bytes, options)?;
    validate_ifccad_document(&document).map_err(IfccadCborReadError::InvalidDocument)?;
    Ok(ValidatedIfccadCbor { document })
}
