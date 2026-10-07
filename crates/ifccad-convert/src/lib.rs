//! Experimental direct conversion of a bounded IFCCAD subset.
//!
//! Default conversion allows partial output with located loss diagnostics.
//! Explicit [`IfccadLossPolicy::Reject`] refuses diagnosed semantic losses.
//! Structural and numeric failures remain errors under both policies.
//! Core owns IFCX composition and shared logical CAD validation. Direct document
//! routes avoid an IFCX byte bridge; loaded-source conversion retains graph loss
//! and numeric projection checks.
//!
//! Both directions expose the same hard geometric tolerance as OCDraw.
//! Defaults use one micrometre in known coordinate units and zero for unitless
//! domains. Each Paper layout resolves its own coordinate-unit limit.
//! Certified within-limit rounding is accepted even under semantic Reject.
//! Exceedance or incomplete proof returns no output.
//!
//! ```
//! use ifccad_convert::{IfccadToCadOptions, CadToIfccadOptions,
//!     IfccadGeometryTolerance, IfccadLossPolicy};
//! let known_units = IfccadToCadOptions::default();
//! let exact = IfccadToCadOptions {
//!     geometry_tolerance: IfccadGeometryTolerance::exact(),
//!     ..known_units
//! };
//! let unitless = CadToIfccadOptions {
//!     geometry_tolerance: IfccadGeometryTolerance::drawing_units(1e-6)?,
//!     loss_policy: IfccadLossPolicy::Reject,
//! };
//! # let _ = (exact, unitless);
//! # Ok::<(), ifccad_convert::IfccadToleranceError>(())
//! ```
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
mod diagnostics;
mod from_cad;
mod geometry_assessment;
mod geometry_context;
pub use geometry_assessment::*;
mod loss;
mod mapping;
mod options;
mod outcome;
mod source;
mod to_cad;
mod units;
pub use diagnostics::{IfccadConversionError, IfccadDiagnostic, IfccadDiagnosticAction};
pub use from_cad::{cad_document_to_encoded_ifccad, cad_document_to_ifccad_document};
pub use opencadcodec;
pub use options::*;
pub use outcome::*;
pub use to_cad::{ifccad_document_to_cad_document, ifccad_source_to_cad_document};
