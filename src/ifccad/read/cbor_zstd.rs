use super::*;
use std::io::{Cursor, Read};

/// Resource bounds for the experimental, single-frame Zstd transport.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IfccadCborZstdReadLimits {
    pub max_compressed_bytes: u64,
    /// Maximum permitted frame window in log2 bytes (10..=31).
    pub max_window_log: u32,
}
impl Default for IfccadCborZstdReadLimits {
    fn default() -> Self {
        Self {
            max_compressed_bytes: 512 * 1024 * 1024,
            max_window_log: 27,
        }
    }
}
/// The contained CBOR limit also bounds decompressed bytes before parsing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IfccadCborZstdReadOptions {
    pub limits: IfccadCborZstdReadLimits,
    pub cbor: IfccadCborReadOptions,
}
impl IfccadCborZstdReadOptions {
    pub(crate) fn validate(&self) -> Result<(), IfccadCborZstdReadError> {
        for limit in [
            self.limits.max_compressed_bytes,
            self.cbor.limits.max_input_bytes,
        ] {
            if limit
                .checked_add(1)
                .and_then(|n| usize::try_from(n).ok())
                .is_none()
            {
                return Err(IfccadCborZstdReadError::InvalidOptions {
                    message: "byte limit cannot be represented with a detection byte".into(),
                });
            }
        }
        if !(10..=31).contains(&self.limits.max_window_log) {
            return Err(IfccadCborZstdReadError::InvalidOptions {
                message: "window log must be in 10..=31".into(),
            });
        }
        Ok(())
    }
}
#[derive(Debug, thiserror::Error)]
pub enum IfccadCborZstdReadError {
    #[error("invalid IFCCAD-CBOR Zstd options: {message}")]
    InvalidOptions { message: String },
    #[error("malformed IFCCAD-CBOR Zstd transport: {message}")]
    Malformed { message: String },
    #[error("IFCCAD-CBOR Zstd resource limit {limit}: {observed} exceeds {allowed}")]
    ResourceLimit {
        limit: &'static str,
        allowed: u64,
        observed: u64,
    },
    #[error("IFCCAD-CBOR Zstd decompression failed: {0}")]
    Decompression(#[source] std::io::Error),
    #[error("{0}")]
    Cbor(#[from] IfccadCborReadError),
}
impl IfccadCborZstdReadError {
    pub fn report(&self) -> Option<&IfccadReport> {
        match self {
            Self::Cbor(error) => error.report(),
            _ => None,
        }
    }
}
fn malformed(message: &str) -> IfccadCborZstdReadError {
    IfccadCborZstdReadError::Malformed {
        message: message.into(),
    }
}
fn bound(limit: &'static str, observed: u64, allowed: u64) -> Result<(), IfccadCborZstdReadError> {
    if observed > allowed {
        return Err(IfccadCborZstdReadError::ResourceLimit {
            limit,
            allowed,
            observed,
        });
    }
    Ok(())
}
/// Load exactly one standard, checksummed, dictionary-free Zstd frame.
///
/// Its payload must be ordinary IFCCAD CBOR. No filename inference, nested
/// decompression, concatenated frames or skippable frames are accepted.
pub fn load_ifccad_cbor_zstd_bytes(
    bytes: &[u8],
    options: IfccadCborZstdReadOptions,
) -> Result<ValidatedIfccadCbor, IfccadCborZstdReadError> {
    options.validate()?;
    bound(
        "compressed_bytes",
        bytes.len() as u64,
        options.limits.max_compressed_bytes,
    )?;
    if bytes.len() < 5 || bytes[..4] != [0x28, 0xB5, 0x2F, 0xFD] {
        return Err(malformed("expected a standard Zstd frame header"));
    }
    let descriptor = bytes[4];
    if descriptor & 4 == 0 {
        return Err(malformed("frame checksum is required"));
    }
    if descriptor & 3 != 0 {
        return Err(malformed("dictionary ID fields are not supported"));
    }
    if descriptor & 8 != 0 {
        return Err(malformed("reserved frame-header bit is set"));
    }
    let content_size = zstd::zstd_safe::get_frame_content_size(bytes)
        .map_err(|_| malformed("invalid or truncated frame header"))?;
    let decoded_limit = options.cbor.limits.max_input_bytes;
    if let Some(size) = content_size {
        bound("decompressed_bytes", size, decoded_limit)?;
    }
    let window = if descriptor & 0x20 != 0 {
        content_size.ok_or_else(|| malformed("single-segment frame lacks content size"))?
    } else {
        let value = *bytes
            .get(5)
            .ok_or_else(|| malformed("missing window descriptor"))?;
        let base = 1u64 << (10 + (value >> 3));
        base + (base / 8) * u64::from(value & 7)
    };
    bound(
        "window_bytes",
        window,
        1u64 << options.limits.max_window_log,
    )?;
    let mut decoder = zstd::stream::read::Decoder::with_buffer(Cursor::new(bytes))
        .map_err(IfccadCborZstdReadError::Decompression)?
        .single_frame();
    decoder
        .window_log_max(options.limits.max_window_log)
        .map_err(IfccadCborZstdReadError::Decompression)?;
    let mut decoded = Vec::new();
    (&mut decoder)
        .take(decoded_limit + 1)
        .read_to_end(&mut decoded)
        .map_err(IfccadCborZstdReadError::Decompression)?;
    bound("decompressed_bytes", decoded.len() as u64, decoded_limit)?;
    if decoder.finish().position() != bytes.len() as u64 {
        return Err(malformed(
            "trailing bytes or multiple frames are not supported",
        ));
    }
    Ok(load_ifccad_cbor_bytes(&decoded, options.cbor)?)
}
