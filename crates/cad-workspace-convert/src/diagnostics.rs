#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceFieldLoss {
    pub field: &'static str,
    pub message: String,
}
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("invalid CAD workspace field {field}: {message}")]
pub struct WorkspaceNumericError {
    pub field: &'static str,
    pub message: String,
}
impl From<ocdraw::workspace_kernel::WorkspaceScalarError> for WorkspaceNumericError {
    fn from(error: ocdraw::workspace_kernel::WorkspaceScalarError) -> Self {
        Self {
            field: error.field,
            message: format!("{:?}", error.reason),
        }
    }
}
pub(crate) fn finite(
    field: &'static str,
    values: impl IntoIterator<Item = f64>,
) -> Result<(), WorkspaceNumericError> {
    if values.into_iter().all(f64::is_finite) {
        Ok(())
    } else {
        Err(WorkspaceNumericError {
            field,
            message: "nonfinite scalar".into(),
        })
    }
}
pub(crate) fn loss(field: &'static str, message: impl Into<String>) -> WorkspaceFieldLoss {
    WorkspaceFieldLoss {
        field,
        message: message.into(),
    }
}
