//! Experimental IFCX-native CAD profile, independent of standalone OCDraw.
//!
//! The formats share geometric validation types and the OCDraw unit registry.
//! IFCX composition, nodes, schema imports and CAD attributes stay in this module.
//!
//! The owned [`IfcxCadDocument`] is a CAD projection; [`LoadedIfcxGraph`] retains
//! its complete immutable source. Encoding produces a fresh CAD-profile file.
//!
//! ```
//! use ocdraw::ifcx_cad::{read_native_cad_ifcx, validate_ifcx_cad_document,
//!     encode_ifcx_cad_document};
//! # fn edit(bytes: &[u8]) -> Result<(), ocdraw::ifcx_cad::IfcxCadReport> {
//! let (source, mut document) = read_native_cad_ifcx(bytes)?.into_parts();
//! document.model.entities.reverse();
//! validate_ifcx_cad_document(&document)?;
//! let profile_bytes = encode_ifcx_cad_document(&document)?;
//! // Original download retains all source information; profile encoding is separate.
//! let original_bytes = source.source_bytes();
//! # let _ = (profile_bytes, original_bytes);
//! # Ok(())
//! # }
//! ```

mod allocation;
mod document_validation;
mod graph;
mod model;
mod parse;
mod patterns;
mod validate;
mod wire;
mod write;

pub(crate) const PROFILE_URI: &str = "urn:example:ifccad:0.1.0";

pub use allocation::{IfcxCadIdAllocationError, IfcxCadIdCounters, IfcxCadIdDomain};
pub use document_validation::validate_ifcx_cad_document;
pub use graph::LoadedIfcxGraph;
pub use model::*;
pub use patterns::validate_ifcx_cad_line_patterns;
pub use write::{encode_ifcx_cad_document, write_native_cad_ifcx};

/// How repeated IFCX node paths are composed before CAD validation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IfcxCompositionPolicy {
    /// Merge keyed node fields in file order; later values win.
    #[default]
    LaterWins,
    /// Reject a repeated field or key when its value differs.
    RejectConflicts,
}

/// Load and validate the experimental IFCX CAD profile using later-wins composition.
pub fn read_native_cad_ifcx(bytes: &[u8]) -> Result<ValidatedIfcxCad, IfcxCadReport> {
    read_native_cad_ifcx_with_policy(bytes, IfcxCompositionPolicy::LaterWins)
}

/// Load and validate the experimental IFCX CAD profile with an explicit composition policy.
/// CAD constraints are checked on the final composed nodes in either mode.
pub fn read_native_cad_ifcx_with_policy(
    bytes: &[u8],
    policy: IfcxCompositionPolicy,
) -> Result<ValidatedIfcxCad, IfcxCadReport> {
    let graph = LoadedIfcxGraph::load(bytes, policy)?;
    let document = validate::project(graph.composed_ifcx())?;
    Ok(ValidatedIfcxCad { graph, document })
}

/// Errors found while loading or writing the experimental profile.
#[derive(Clone, Debug, PartialEq, Eq)]
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
