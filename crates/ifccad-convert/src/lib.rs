//! Experimental direct conversion of a bounded IFCCAD subset.
//!
//! Default conversion allows partial output with located loss diagnostics.
//! Explicit [`IfccadLossPolicy::Reject`] refuses diagnosed semantic losses.
//! Structural and numeric failures remain errors under both policies.
//! Core owns IFCX composition and shared logical CAD validation. Direct document
//! routes avoid an IFCX byte bridge; loaded-source conversion retains graph loss
//! and numeric projection checks.
//!
//! Both directions expose the same domain-qualified hard geometric tolerance.
//! Defaults use 1e-9 in each domain's own coordinates, including unitless
//! domains. Each Paper layout resolves its own coordinate-unit limit.
//! Certified within-limit rounding is accepted even under semantic Reject.
//! Normal import returns no output on exceedance or incomplete proof. Explicit
//! partial CAD import can omit a whole failed entity with located evidence;
//! every retained entity and occurrence keeps the same hard proof requirements.
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
//!     ..Default::default()
//! };
//! # let _ = (exact, unitless);
//! # Ok::<(), ifccad_convert::IfccadToleranceError>(())
//! ```
//!
//! ```
//! use ifccad_convert::{cad_document_to_ifccad_document,
//!     ifccad_document_to_cad_document, IfccadTargetMetadata,
//!     IfccadConversionError};
//! use ifccad::encode_ifccad_document;
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
//! use ifccad::ValidatedIfccad;
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
mod audit;
pub use audit::*;
mod diagnostics;
mod dimension_assessment;
mod filled_surface_assessment;
mod polyline_width_assessment;
pub use polyline_width_assessment::*;
mod polyline_width_target;
pub use polyline_width_target::*;
mod from_cad;
mod geometry_assessment;
pub use filled_surface_assessment::{IfccadFilledSurfaceAssessment, IfccadFilledSurfaceCoverage};
mod dimension_graphics_context;
mod geometry_context;
mod text_assessment;
pub use dimension_assessment::*;
pub use geometry_assessment::*;
pub use text_assessment::*;
mod loss;
mod mapping;
mod options;
mod preservation;
pub use preservation::*;
mod outcome;
mod partial;
pub use partial::CadPartialOpenReport;
mod source;
mod to_cad;
mod units;
pub use diagnostics::{IfccadConversionError, IfccadDiagnostic, IfccadDiagnosticAction};
pub use from_cad::{cad_document_to_encoded_ifccad, cad_document_to_ifccad_document};
pub use opencadcodec;
pub use options::*;
pub use outcome::*;
pub use to_cad::{ifccad_document_to_cad_document, ifccad_source_to_cad_document};
