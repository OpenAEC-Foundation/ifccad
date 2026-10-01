//! Experimental direct conversion of a bounded IFCX-CAD subset.
//!
//! Unsupported semantic content is rejected; no partial drawing is returned.
//! The core reader owns IFCX composition and profile validation.
//!
//! ```
//! use ocdraw::ifcx_cad::ValidatedIfcxCad;
//! use ifcx_cad_convert::{ifcx_cad_to_cad_document, cad_document_to_ifcx_cad,
//!     IfcxCadTargetMetadata, IfcxCadConversionError};
//! # fn convert(source: &ValidatedIfcxCad, metadata: IfcxCadTargetMetadata)
//! # -> Result<Vec<u8>, IfcxCadConversionError> {
//! let cad = ifcx_cad_to_cad_document(source)?;
//! let back = cad_document_to_ifcx_cad(cad.document(), metadata)?;
//! // Emitted bytes have passed the production IFCX-CAD reader.
//! Ok(back.ifcx_bytes().to_vec())
//! # }
//! ```
mod appearance;
mod blocks;
mod from_cad;
mod geometry;
mod outcome;
mod source;
mod to_cad;
pub use from_cad::cad_document_to_ifcx_cad;
pub use outcome::*;
pub use to_cad::ifcx_cad_to_cad_document;
