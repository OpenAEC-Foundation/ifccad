/// Acceptance of diagnosed semantic losses. Structural and numeric failures
/// remain errors under both policies.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IfcxCadLossPolicy {
    #[default]
    Allow,
    Reject,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CadToIfcxCadOptions {
    pub loss_policy: IfcxCadLossPolicy,
}

/// Loss acceptance for conversion from an IFCX-CAD source or document.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IfcxCadToCadOptions {
    pub loss_policy: IfcxCadLossPolicy,
}
/// Explicit target identity and provenance, supplied by the caller.
#[derive(Clone, Debug)]
pub struct IfcxCadTargetMetadata {
    pub header: IfcxCadHeader,
    pub drawing_id: u64,
}

use ocdraw::ifcx_cad::IfcxCadHeader;
