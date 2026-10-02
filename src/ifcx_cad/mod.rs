//! Experimental IFCX-native CAD profile, independent of standalone OCDraw.
//!
//! The formats share geometric validation types and the OCDraw unit registry.
//! IFCX composition, nodes, schema imports and CAD attributes stay in this module.

mod allocation;
mod model;
mod parse;
mod patterns;
mod validate;
mod wire;
mod write;

pub(crate) const PROFILE_URI: &str = "urn:example:ifccad:0.1.0";

pub use allocation::{IfcxCadIdAllocationError, IfcxCadIdCounters, IfcxCadIdDomain};
pub use model::*;
pub use patterns::validate_ifcx_cad_line_patterns;
pub use write::write_native_cad_ifcx;

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
    validate::validate(parse::compose(bytes, policy)?)
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
