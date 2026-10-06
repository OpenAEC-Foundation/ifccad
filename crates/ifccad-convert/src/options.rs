/// Acceptance of diagnosed semantic losses. Structural and numeric failures
/// remain errors under both policies.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IfccadLossPolicy {
    #[default]
    Allow,
    Reject,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CadToIfccadOptions {
    pub loss_policy: IfccadLossPolicy,
    pub geometry_tolerance: crate::IfccadGeometryTolerance,
}

/// Loss acceptance for conversion from an IFCCAD source or document.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct IfccadToCadOptions {
    pub loss_policy: IfccadLossPolicy,
    pub geometry_tolerance: crate::IfccadGeometryTolerance,
}
/// Explicit target identity and provenance, supplied by the caller.
#[derive(Clone, Debug)]
pub struct IfccadTargetMetadata {
    pub header: IfccadHeader,
    pub drawing_id: u64,
}

use ocdraw::ifccad::IfccadHeader;
