use super::*;
use std::io::Write;

/// Experimental writer settings; compression is only enabled by calling this route.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IfccadCborZstdEncodeOptions {
    /// Zstd levels 1..=22; the level is not stored in standard Zstd headers.
    pub compression_level: i32,
}
impl Default for IfccadCborZstdEncodeOptions {
    fn default() -> Self {
        Self {
            compression_level: 3,
        }
    }
}
#[derive(Debug, thiserror::Error)]
pub enum IfccadCborZstdEncodeError {
    #[error("invalid IFCCAD-CBOR Zstd writer options: {message}")]
    InvalidOptions { message: String },
    #[error("{0}")]
    Cbor(#[from] IfccadCborEncodeError),
    #[error("IFCCAD-CBOR Zstd compression failed: {0}")]
    Compression(#[source] std::io::Error),
    #[error("strict IFCCAD-CBOR Zstd readback failed: {0}")]
    Readback(#[source] IfccadCborZstdReadError),
    #[error("{0}")]
    SemanticMismatch(#[source] IfccadReport),
}
impl IfccadCborZstdEncodeError {
    pub fn report(&self) -> Option<&IfccadReport> {
        match self {
            Self::Cbor(error) => error.report(),
            Self::Readback(error) => error.report(),
            Self::SemanticMismatch(report) => Some(report),
            _ => None,
        }
    }
}
/// Encode ordinary CBOR, wrap it in Zstd, and verify strict production readback.
///
/// Enabled by the optional native `ifccad-cbor-zstd` feature. Raw CBOR remains
/// the default route. The standard Zstd header identifies the method, not the level.
///
/// ```
/// use ocdraw::ifccad::{encode_ifccad_cbor_zstd_document,
///     load_ifccad_cbor_zstd_bytes, IfccadDocument, IfccadCborZstdEncodeOptions};
/// # fn example(document: &IfccadDocument) -> Result<(), Box<dyn std::error::Error>> {
/// let encoded = encode_ifccad_cbor_zstd_document(document,
///     IfccadCborZstdEncodeOptions { compression_level: 9 })?;
/// let loaded = load_ifccad_cbor_zstd_bytes(encoded.bytes(), Default::default())?;
/// assert_eq!(loaded.document(), document);
/// # Ok(()) }
/// ```
pub fn encode_ifccad_cbor_zstd_document(
    document: &IfccadDocument,
    options: IfccadCborZstdEncodeOptions,
) -> Result<EncodedIfccadCborZstd, IfccadCborZstdEncodeError> {
    if !(1..=22).contains(&options.compression_level) {
        return Err(IfccadCborZstdEncodeError::InvalidOptions {
            message: "compression level must be in 1..=22".into(),
        });
    }
    let raw = super::cbor::encode_checked_buffer(document)?;
    let compress = || -> std::io::Result<Vec<u8>> {
        let mut encoder = zstd::stream::write::Encoder::new(Vec::new(), options.compression_level)?;
        encoder.include_checksum(true)?;
        encoder.include_contentsize(true)?;
        encoder.set_pledged_src_size(Some(raw.bytes.len() as u64))?;
        encoder.write_all(&raw.bytes)?;
        encoder.finish()
    };
    let bytes = compress().map_err(IfccadCborZstdEncodeError::Compression)?;
    let limits = IfccadCborZstdReadOptions {
        limits: IfccadCborZstdReadLimits {
            max_compressed_bytes: bytes.len() as u64,
            max_window_log: 31,
        },
        cbor: IfccadCborReadOptions {
            limits: raw.readback_limits,
        },
    };
    let loaded =
        load_ifccad_cbor_zstd_bytes(&bytes, limits).map_err(IfccadCborZstdEncodeError::Readback)?;
    if !crate::ifccad::codec::cbor::documents_exactly_equal(document, loaded.document()) {
        return Err(IfccadCborZstdEncodeError::SemanticMismatch(
            IfccadReport::one("strict Zstd readback changed document values or numeric bits"),
        ));
    }
    Ok(EncodedIfccadCborZstd::from_bytes(bytes))
}
