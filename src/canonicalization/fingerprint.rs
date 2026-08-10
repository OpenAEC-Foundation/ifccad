use super::{canonicalize, canonicalize_typed_value, CanonicalValue, CanonicalizationError};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// Fingerprint an IFCCAD value from its canonical UTF-8 representation.
pub fn fingerprint(value: &CanonicalValue) -> Result<String, CanonicalizationError> {
    Ok(fingerprint_bytes(&canonicalize(value)?))
}

/// Decode, canonicalize, and fingerprint an IFCCAD typed value.
pub fn fingerprint_typed_value(value: &Value) -> Result<String, CanonicalizationError> {
    Ok(fingerprint_bytes(&canonicalize_typed_value(value)?))
}

fn fingerprint_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("sha256:{digest:x}")
}
