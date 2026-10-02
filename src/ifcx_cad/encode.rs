use super::*;

#[derive(Debug, thiserror::Error)]
pub enum IfcxCadEncodeError {
    #[error(transparent)]
    InvalidDocument(#[from] IfcxCadReport),
    #[error("IFCX serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("strict IFCX readback failed: {0}")]
    Readback(IfcxCadReadError),
    #[error("strict IFCX readback changed CAD semantics")]
    SemanticMismatch(IfcxCadReport),
}
/// Encode a fresh CAD-profile file, without updating an original source graph.
pub fn encode_ifcx_cad_document(
    document: &IfcxCadDocument,
) -> Result<EncodedIfcxCad, IfcxCadEncodeError> {
    validate_ifcx_cad_document(document)?;
    let bytes = super::codec::ifcx_json::encode_bytes(document)?;
    let loaded = load_ifcx_cad_bytes(&bytes, IfcxCadReadOptions::default())
        .map_err(IfcxCadEncodeError::Readback)?;
    let mut expected = document.clone();
    expected.layers.sort_by_key(|layer| layer.id.to_string());
    expected.blocks.sort_by_key(|block| block.id.to_string());
    expected.line_patterns.sort_by_key(|p| p.id.0.to_string());
    expected.paper_layouts.sort_by_key(|layout| layout.id);
    if loaded.document() != &expected {
        return Err(IfcxCadEncodeError::SemanticMismatch(IfcxCadReport::one(
            "strict IFCX readback changed CAD semantics",
        )));
    }
    Ok(EncodedIfcxCad::from_bytes(bytes))
}

impl IfcxCadEncodeError {
    pub fn report(&self) -> Option<&IfcxCadReport> {
        match self {
            Self::InvalidDocument(report) | Self::SemanticMismatch(report) => Some(report),
            Self::Readback(error) => Some(error.report()),
            Self::Serialization(_) => None,
        }
    }
}
