use crate::IfcxCadLossPolicy;
/// What happened to the located source information.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfcxCadDiagnosticAction {
    Omitted,
    Modified,
    /// A uniquely established structural cache repair, without semantic loss.
    Recovery,
}

/// Located unsupported content or a documented source recovery.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfcxCadDiagnostic {
    pub code: &'static str,
    pub location: String,
    pub message: String,
    pub action: IfcxCadDiagnosticAction,
}

impl IfcxCadDiagnostic {
    pub fn is_loss(&self) -> bool {
        self.action != IfcxCadDiagnosticAction::Recovery
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IfcxCadConversionError {
    #[error("invalid CAD structure: {0}")]
    InvalidStructure(String),
    #[error("unsupported conversion content: {0:?}")]
    Unsupported(Vec<IfcxCadDiagnostic>),
    #[error("IFCX-CAD validation failed: {0}")]
    CoreValidation(String),
    #[error("CAD construction failed: {0}")]
    CadConstruction(String),
}

pub(crate) fn diagnostic(
    code: &'static str,
    location: impl Into<String>,
    message: impl Into<String>,
) -> IfcxCadDiagnostic {
    IfcxCadDiagnostic {
        code,
        location: location.into(),
        message: message.into(),
        action: IfcxCadDiagnosticAction::Omitted,
    }
}

pub(crate) fn modification(
    code: &'static str,
    location: impl Into<String>,
    message: impl Into<String>,
) -> IfcxCadDiagnostic {
    let mut d = diagnostic(code, location, message);
    d.action = IfcxCadDiagnosticAction::Modified;
    d
}

pub(crate) fn enforce_policy(
    loss_policy: IfcxCadLossPolicy,
    issues: &[IfcxCadDiagnostic],
) -> Result<(), IfcxCadConversionError> {
    if issues
        .iter()
        .any(|d| matches!(d.code, "rounding" | "scale-clamped" | "precision"))
        || (loss_policy == IfcxCadLossPolicy::Reject
            && issues.iter().any(IfcxCadDiagnostic::is_loss))
    {
        return Err(IfcxCadConversionError::Unsupported(issues.to_vec()));
    }
    Ok(())
}
