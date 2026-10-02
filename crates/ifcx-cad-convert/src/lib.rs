//! Experimental direct conversion of a bounded IFCX-CAD subset.
//!
//! Default conversion allows partial output with located loss diagnostics.
//! Explicit [`IfcxCadLossPolicy::Reject`] refuses diagnosed semantic losses.
//! Structural and numeric failures remain errors under both policies.
//! Core owns IFCX composition and shared logical CAD validation. Direct document
//! routes avoid an IFCX byte bridge; loaded-source conversion retains graph loss
//! and numeric projection checks.
//!
//! ```
//! use ifcx_cad_convert::{cad_document_to_ifcx_cad_document,
//!     ifcx_cad_document_to_cad_document, IfcxCadTargetMetadata,
//!     IfcxCadConversionError};
//! use ocdraw::ifcx_cad::encode_ifcx_cad_document;
//! # fn convert(source: &opencadcodec::CadDocument, metadata: IfcxCadTargetMetadata)
//! # -> Result<(), IfcxCadConversionError> {
//! let logical = cad_document_to_ifcx_cad_document(source, metadata, Default::default())?;
//! let cad = ifcx_cad_document_to_cad_document(logical.document(), Default::default())?;
//! let bytes = encode_ifcx_cad_document(logical.document())
//!     .map_err(|e| IfcxCadConversionError::CoreValidation(format!("{e:?}")))?;
//! # let _ = (cad, bytes);
//! # Ok(())
//! # }
//! ```
//!
//! ```
//! use ocdraw::ifcx_cad::ValidatedIfcxCad;
//! use ifcx_cad_convert::{ifcx_cad_source_to_cad_document, cad_document_to_encoded_ifcx_cad,
//!     IfcxCadTargetMetadata, IfcxCadConversionError};
//! # fn convert(source: &ValidatedIfcxCad, metadata: IfcxCadTargetMetadata)
//! # -> Result<Vec<u8>, IfcxCadConversionError> {
//! let cad = ifcx_cad_source_to_cad_document(source, Default::default())?;
//! let back = cad_document_to_encoded_ifcx_cad(cad.document(), metadata, Default::default())?;
//! // Emitted bytes have passed the production IFCX-CAD reader.
//! Ok(back.encoded().bytes().to_vec())
//! # }
//! ```
mod appearance;
mod blocks;
mod diagnostics;
mod from_cad;
mod geometry;
mod loss;
mod options;
mod outcome;
mod patterns;
mod source;
mod to_cad;
mod units;
pub use diagnostics::{IfcxCadConversionError, IfcxCadDiagnostic, IfcxCadDiagnosticAction};
pub use from_cad::{cad_document_to_encoded_ifcx_cad, cad_document_to_ifcx_cad_document};
pub use opencadcodec;
pub use options::*;
pub use outcome::*;
pub use to_cad::{ifcx_cad_document_to_cad_document, ifcx_cad_source_to_cad_document};
