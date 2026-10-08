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
    pub hatch_join_tolerance: ocdraw::geometry_kernel::hatch::HatchJoinToleranceRequest,
    pub preservation: IfccadPreservationCapture,
    pub loss_policy: IfccadLossPolicy,
    pub geometry_tolerance: crate::IfccadGeometryTolerance,
}

/// Loss acceptance for conversion from an IFCCAD source or document.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct IfccadToCadOptions {
    pub preservation: IfccadPreservationRestore,
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

/// Capture is explicitly enabled by callers; native conversion defaults stay unchanged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IfccadPreservationCapture {
    #[default]
    Disabled,
    SupportedTyped,
}
/// Evaluate stored conditions for every fresh export, or explicitly omit opaque contents.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IfccadPreservationRestore {
    #[default]
    RestoreSupported,
    Skip,
}
