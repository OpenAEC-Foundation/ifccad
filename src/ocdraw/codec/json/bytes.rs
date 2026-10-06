//! The file codec's canonical byte transport, independent of provider payload schemas.
use base64::{engine::general_purpose::STANDARD, Engine};

pub(super) fn encode(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}

pub(super) fn decode(text: &str) -> Option<Vec<u8>> {
    // Derive the upper bound from actual input, never a supplied decoded count.
    if !text.len().is_multiple_of(4) {
        return None;
    }
    let bound = text.len().checked_div(4)?.checked_mul(3)?;
    let bytes = STANDARD.decode(text).ok()?;
    if bytes.len() > bound || STANDARD.encode(&bytes) != text {
        return None;
    }
    Some(bytes)
}
