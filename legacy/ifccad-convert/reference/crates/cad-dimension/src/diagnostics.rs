#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CadDimensionPreparationError {
    #[error("unsupported Dimension {field}: {detail}")]
    Unsupported { field: String, detail: String },
    #[error("invalid Dimension {field}: {detail}")]
    Invalid { field: String, detail: String },
    #[error("numerical Dimension failure {field}: {detail}")]
    Numeric { field: String, detail: String },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CadDimensionLoss {
    pub field: String,
    pub detail: String,
}
pub(crate) fn unsupported(field: &str, detail: &str) -> CadDimensionPreparationError {
    CadDimensionPreparationError::Unsupported {
        field: field.into(),
        detail: detail.into(),
    }
}
pub(crate) fn invalid(field: &str, detail: &str) -> CadDimensionPreparationError {
    CadDimensionPreparationError::Invalid {
        field: field.into(),
        detail: detail.into(),
    }
}
pub(crate) fn numeric(field: &str, detail: &str) -> CadDimensionPreparationError {
    CadDimensionPreparationError::Numeric {
        field: field.into(),
        detail: detail.into(),
    }
}
