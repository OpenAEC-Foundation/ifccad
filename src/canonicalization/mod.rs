//! Canonical byte representation for language-neutral IFCCAD values.

mod error;
mod fingerprint;
mod typed;
mod value;

pub use error::{CanonicalizationError, CanonicalizationErrorCode};
pub use fingerprint::{fingerprint, fingerprint_typed_value};
pub use typed::{canonicalize_typed_value, decode_typed_value};
pub use value::{canonicalize, CanonicalValue};
