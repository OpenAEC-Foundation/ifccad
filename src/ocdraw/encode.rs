use super::{
    DrawingBuildError, DrawingDiagnostic, DrawingLoadStatus, EncodedDrawing, OcdrawDocument,
    OcdrawValidationError,
};

/// Distinguishes invalid logical input from encoding and output-readback failures.
#[derive(Debug, thiserror::Error)]
pub enum OcdrawEncodeError {
    #[error(transparent)]
    InvalidDocument(#[from] OcdrawValidationError),
    #[error("drawing serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("encoded drawing failed strict production readback ({status:?})")]
    Readback {
        status: DrawingLoadStatus,
        diagnostics: Vec<DrawingDiagnostic>,
    },
}
impl From<OcdrawEncodeError> for DrawingBuildError {
    fn from(error: OcdrawEncodeError) -> Self {
        match error {
            OcdrawEncodeError::Serialization(e) => Self::Serialization(e),
            OcdrawEncodeError::InvalidDocument(e) => {
                Self::Invalid(format!("{:?}", e.diagnostics()))
            }
            OcdrawEncodeError::Readback {
                status,
                diagnostics,
            } => Self::Invalid(format!("{status:?}: {diagnostics:?}")),
        }
    }
}

/// Encodes validated content without changing identities, order or supplied bounds.
/// Output is checked by the production reader before it is returned.
pub fn encode_document(doc: &OcdrawDocument) -> Result<EncodedDrawing, OcdrawEncodeError> {
    super::validate_document(doc)?;
    let bytes = super::codec::json::encode_document_bytes(doc)?;
    let read = super::load_drawing_bytes(&bytes);
    if read.status() != DrawingLoadStatus::Valid {
        return Err(OcdrawEncodeError::Readback {
            status: read.status(),
            diagnostics: read.diagnostics().to_vec(),
        });
    }
    Ok(EncodedDrawing::from_bytes(bytes))
}
