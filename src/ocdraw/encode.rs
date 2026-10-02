use super::{
    EncodedOcdraw, OcdrawBuildError, OcdrawDiagnostic, OcdrawDocument, OcdrawReadStatus,
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
        status: OcdrawReadStatus,
        diagnostics: Vec<OcdrawDiagnostic>,
    },
}
impl From<OcdrawEncodeError> for OcdrawBuildError {
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
pub fn encode_ocdraw_document(doc: &OcdrawDocument) -> Result<EncodedOcdraw, OcdrawEncodeError> {
    super::validate_ocdraw_document(doc)?;
    let bytes = super::codec::json::encode_document_bytes(doc)?;
    if let Err(error) = super::load_ocdraw_bytes(&bytes) {
        return Err(OcdrawEncodeError::Readback {
            status: error.status(),
            diagnostics: error.diagnostics().to_vec(),
        });
    }
    Ok(EncodedOcdraw::from_bytes(bytes))
}
