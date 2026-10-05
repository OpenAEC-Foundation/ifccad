/// Acceptance of diagnosed semantic losses. Structural and numeric failures
/// remain errors under both policies.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IfccadLossPolicy {
    #[default]
    Allow,
    Reject,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CadToIfccadOptions {
    pub loss_policy: IfccadLossPolicy,
}

/// Loss acceptance for conversion from an IFCCAD source or document.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IfccadToCadOptions {
    pub loss_policy: IfccadLossPolicy,
}
/// Explicit target identity and provenance, supplied by the caller.
#[derive(Clone, Debug)]
pub struct IfccadTargetMetadata {
    pub header: IfccadHeader,
    pub drawing_id: u64,
}

use ocdraw::ifccad::IfccadHeader;
