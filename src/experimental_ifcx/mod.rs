//! Experimental IFCX-native CAD profile. This module does not alter the
//! released IFCCAD package formats.

mod appearance;
mod model;
mod parse;
mod validate;
mod write;

pub(crate) const PROFILE_URI: &str = "urn:example:ifccad:experimental-cad:0.1.0";

pub use model::*;
pub use write::write_native_cad_ifcx;

/// Load and strictly validate the experimental IFCX CAD profile.
pub fn read_native_cad_ifcx(bytes: &[u8]) -> Result<ValidatedIfcxCad, IfcxCadReport> {
    validate::validate(parse::compose(bytes)?)
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
