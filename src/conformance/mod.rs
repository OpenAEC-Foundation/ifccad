//! Language-neutral IFCCAD conformance support.

mod canonicalization;
mod error;
mod fingerprints;
mod manifest;

use std::path::PathBuf;

pub use canonicalization::verify_canonicalization_vectors;
pub use error::ConformanceError;
pub use fingerprints::verify_fingerprint_vectors;
pub use manifest::{
    load_conformance_manifest, parse_conformance_manifest, ConformanceCase, ConformanceCategory,
    ConformanceManifest, ConformanceOperation, ConformanceOperationName, ExpectedOutcome,
};

/// Version of the conformance test collection bundled with this crate.
pub const BUNDLED_CONFORMANCE_VERSION: &str = "1.1.0";
const BUNDLED_CONFORMANCE_DIRECTORY: &str = "next";

/// Path to the conformance collection shipped in this crate's source package.
///
/// This helper supports conformance runners and tests that retain the crate
/// sources. It does not embed the collection in a consuming executable.
pub fn bundled_conformance_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("conformance")
        .join(BUNDLED_CONFORMANCE_DIRECTORY)
}
