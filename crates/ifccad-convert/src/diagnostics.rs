use crate::IfccadLossPolicy;
use ocdraw::ifccad::{IfccadEncodeError, IfccadIdAllocationError, IfccadReadError, IfccadReport};
/// What happened to the located source information.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadDiagnosticAction {
    Omitted,
    Modified,
    /// A uniquely established structural cache repair, without semantic loss.
    Recovery,
}

/// Located unsupported content or a documented source recovery.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfccadDiagnostic {
    pub code: &'static str,
    pub location: String,
    pub message: String,
    pub action: IfccadDiagnosticAction,
}

impl IfccadDiagnostic {
    pub fn is_loss(&self) -> bool {
        self.action != IfccadDiagnosticAction::Recovery
    }
}

/// Conversion failure retaining the core phase and original typed cause.
#[derive(Debug, thiserror::Error)]
pub enum IfccadConversionError {
    #[error("invalid CAD structure: {0}")]
    InvalidStructure(String),
    #[error("unsupported conversion content: {0:?}")]
    Unsupported(Vec<IfccadDiagnostic>),
    #[error("IFCCAD validation failed: {0}")]
    CoreValidation(#[source] IfccadReport),
    #[error("IFCCAD encoding failed: {0}")]
    CoreEncoding(#[from] IfccadEncodeError),
    #[error("IFCCAD readback failed: {0}")]
    CoreReadback(#[source] IfccadReadError),
    #[error("IFCCAD ID allocation failed: {0}")]
    IdAllocation(#[from] IfccadIdAllocationError),
    #[error("CAD construction failed: {0}")]
    CadConstruction(String),
}

pub(crate) fn diagnostic(
    code: &'static str,
    location: impl Into<String>,
    message: impl Into<String>,
) -> IfccadDiagnostic {
    IfccadDiagnostic {
        code,
        location: location.into(),
        message: message.into(),
        action: IfccadDiagnosticAction::Omitted,
    }
}

pub(crate) fn modification(
    code: &'static str,
    location: impl Into<String>,
    message: impl Into<String>,
) -> IfccadDiagnostic {
    let mut d = diagnostic(code, location, message);
    d.action = IfccadDiagnosticAction::Modified;
    d
}

pub(crate) fn enforce_policy(
    loss_policy: IfccadLossPolicy,
    issues: &[IfccadDiagnostic],
) -> Result<(), IfccadConversionError> {
    if issues
        .iter()
        .any(|d| matches!(d.code, "rounding" | "scale-clamped" | "precision"))
        || (loss_policy == IfccadLossPolicy::Reject && issues.iter().any(IfccadDiagnostic::is_loss))
    {
        return Err(IfccadConversionError::Unsupported(issues.to_vec()));
    }
    Ok(())
}
