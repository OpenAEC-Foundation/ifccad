//! Experimental IFCX-native CAD profile, independent of standalone OCDraw.
//!
//! The formats share geometric validation types and the OCDraw unit registry.
//! IFCX composition, nodes, schema imports and CAD attributes stay in this module.
//!
//! The owned [`IfcxCadDocument`] is a CAD projection; [`LoadedIfcxGraph`] retains
//! its complete immutable source. Encoding produces a fresh CAD-profile file.
//!
//! ```
//! use ocdraw::ifcx_cad::{load_ifcx_cad_bytes, validate_ifcx_cad_document,
//!     encode_ifcx_cad_document};
//! # fn edit(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
//! let (source, mut document) = load_ifcx_cad_bytes(bytes, Default::default())?.into_parts();
//! document.model.entities.reverse();
//! validate_ifcx_cad_document(&document)?;
//! let profile_bytes = encode_ifcx_cad_document(&document)?;
//! // Original download retains all source information; profile encoding is separate.
//! let original_bytes = source.source_bytes();
//! # let _ = (profile_bytes, original_bytes);
//! # Ok(())
//! # }
//! ```

mod codec;
mod encode;
mod logical;
mod read;
mod source;
mod storage;
pub(crate) const PROFILE_URI: &str = "urn:example:ifccad:0.1.0";
pub use encode::{encode_ifcx_cad_document, IfcxCadEncodeError};
pub use logical::*;
pub use read::{load_ifcx_cad_bytes, IfcxCadReadError, IfcxCadReadOptions, ValidatedIfcxCad};
pub use source::LoadedIfcxGraph;
pub use storage::{load_ifcx_cad_file, EncodedIfcxCad, IfcxCadOpenError, IfcxCadWriteError};
/// How repeated IFCX node paths are composed before CAD validation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IfcxCompositionPolicy {
    /// Merge keyed node fields in file order; later values win.
    #[default]
    LaterWins,
    /// Reject a repeated field or key when its value differs.
    RejectConflicts,
}

/// Errors found while loading or writing the experimental profile.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("IFCX-CAD validation failed: {errors:?}")]
pub struct IfcxCadReport {
    /// Human-readable, location-oriented diagnostics.
    pub errors: Vec<String>,
}

impl IfcxCadReport {
    pub(crate) fn one(message: impl Into<String>) -> Self {
        Self {
            errors: vec![message.into()],
        }
    }
}
