//! Experimental direct conversion of a bounded IFCCAD subset.
//!
//! Default conversion allows partial output with located loss diagnostics.
//! Explicit [`IfccadLossPolicy::Reject`] refuses diagnosed semantic losses.
//! Structural and numeric failures remain errors under both policies.
//! Core owns IFCX composition and shared logical CAD validation. Direct document
//! routes avoid an IFCX byte bridge; loaded-source conversion retains graph loss
//! and numeric projection checks.
//!
//! ```
//! use ifccad_convert::{cad_document_to_ifccad_document,
//!     ifccad_document_to_cad_document, IfccadTargetMetadata,
//!     IfccadConversionError};
//! use ocdraw::ifccad::encode_ifccad_document;
//! # fn convert(source: &opencadcodec::CadDocument, metadata: IfccadTargetMetadata)
//! # -> Result<(), IfccadConversionError> {
//! let logical = cad_document_to_ifccad_document(source, metadata, Default::default())?;
//! let cad = ifccad_document_to_cad_document(logical.document(), Default::default())?;
//! let bytes = encode_ifccad_document(logical.document())
//!     .map_err(IfccadConversionError::CoreEncoding)?;
//! # let _ = (cad, bytes);
//! # Ok(())
//! # }
//! ```
//!
//! ```
//! use ocdraw::ifccad::ValidatedIfccad;
//! use ifccad_convert::{ifccad_source_to_cad_document, cad_document_to_encoded_ifccad,
//!     IfccadTargetMetadata, IfccadConversionError};
//! # fn convert(source: &ValidatedIfccad, metadata: IfccadTargetMetadata)
//! # -> Result<Vec<u8>, IfccadConversionError> {
//! let cad = ifccad_source_to_cad_document(source, Default::default())?;
//! let back = cad_document_to_encoded_ifccad(cad.document(), metadata, Default::default())?;
//! // Emitted bytes have passed the production IFCCAD reader.
//! Ok(back.encoded().bytes().to_vec())
//! # }
//! ```
mod appearance;
mod blocks;
mod diagnostics;
mod entity_owners;
mod from_cad;
mod geometry;
mod layouts;
mod loss;
mod options;
mod outcome;
mod patterns;
mod source;
mod to_cad;
mod units;
mod viewports;
pub use diagnostics::{IfccadConversionError, IfccadDiagnostic, IfccadDiagnosticAction};
pub use from_cad::{cad_document_to_encoded_ifccad, cad_document_to_ifccad_document};
pub use opencadcodec;
pub use options::*;
pub use outcome::*;
pub use to_cad::{ifccad_document_to_cad_document, ifccad_source_to_cad_document};
