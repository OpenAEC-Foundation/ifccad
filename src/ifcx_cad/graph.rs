use super::{parse, IfcxCadReport, IfcxCompositionPolicy};
use serde_json::Value;

/// Immutable source context for the experimental CAD loader.
///
/// Contains the exact input bytes and the complete composed envelope, including
/// information outside the CAD profile. This is not a general IFCX conformance
/// guarantee. Editing an extracted CAD document does not update this snapshot.
#[derive(Clone, Debug)]
pub struct LoadedIfcxGraph {
    source: Vec<u8>,
    composed: Value,
    policy: IfcxCompositionPolicy,
}

impl LoadedIfcxGraph {
    pub(super) fn load(bytes: &[u8], policy: IfcxCompositionPolicy) -> Result<Self, IfcxCadReport> {
        Ok(Self {
            composed: parse::compose(bytes, policy)?,
            source: bytes.to_vec(),
            policy,
        })
    }

    /// Original input, including formatting, fragment order and numeric tokens.
    pub fn source_bytes(&self) -> &[u8] {
        &self.source
    }

    /// Complete envelope after the loader's fragment composition.
    pub fn composed_ifcx(&self) -> &Value {
        &self.composed
    }

    /// The policy used to compose this immutable source snapshot.
    pub fn composition_policy(&self) -> IfcxCompositionPolicy {
        self.policy
    }
}
