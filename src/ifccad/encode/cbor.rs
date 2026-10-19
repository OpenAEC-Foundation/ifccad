use super::*;

#[derive(Debug, thiserror::Error)]
pub enum IfccadCborEncodeError {
    #[error("{0}")]
    InvalidDocument(#[source] IfccadReport),
    #[error("IFCCAD-CBOR encoding at {location}: {message}")]
    Mapping { location: String, message: String },
    #[error("strict CBOR readback failed: {0}")]
    Readback(#[source] IfccadCborReadError),
    #[error("{0}")]
    SemanticMismatch(#[source] IfccadReport),
}
impl IfccadCborEncodeError {
    pub fn report(&self) -> Option<&IfccadReport> {
        match self {
            Self::InvalidDocument(r) | Self::SemanticMismatch(r) => Some(r),
            Self::Readback(e) => e.report(),
            Self::Mapping { .. } => None,
        }
    }
}
/// Encode a fresh standalone binary CAD projection and verify production readback.
pub fn encode_ifccad_cbor_document(
    document: &IfccadDocument,
) -> Result<EncodedIfccadCbor, IfccadCborEncodeError> {
    let output = encode_checked_buffer(document)?;
    Ok(EncodedIfccadCbor::from_bytes(output.bytes))
}
pub(super) fn encode_checked_buffer(
    document: &IfccadDocument,
) -> Result<crate::ifccad::codec::cbor::EncodedCborBuffer, IfccadCborEncodeError> {
    validate_ifccad_document(document).map_err(IfccadCborEncodeError::InvalidDocument)?;
    let output = crate::ifccad::codec::cbor::encode_bytes(document)?;
    verify_readback(&output.bytes, output.readback_limits, document)?;
    Ok(output)
}

fn verify_readback(
    bytes: &[u8],
    limits: IfccadCborReadLimits,
    document: &IfccadDocument,
) -> Result<(), IfccadCborEncodeError> {
    let loaded = load_ifccad_cbor_bytes(bytes, IfccadCborReadOptions { limits })
        .map_err(IfccadCborEncodeError::Readback)?;
    if !crate::ifccad::codec::cbor::documents_exactly_equal(document, loaded.document()) {
        return Err(IfccadCborEncodeError::SemanticMismatch(IfccadReport::one(
            "strict CBOR readback changed document values or numeric bits",
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn base() -> IfccadDocument {
        load_ifccad_bytes(
            include_bytes!("../../../tests/fixtures/ifccad/hello-cad.ifcx"),
            Default::default(),
        )
        .unwrap()
        .into_document()
    }
    #[test]
    fn readback_errors_preserve_their_phase_and_source() {
        use std::error::Error;
        let d = base();
        let error = verify_readback(&[0], Default::default(), &d).unwrap_err();
        assert!(matches!(error, IfccadCborEncodeError::Readback(_)));
        assert!(error
            .source()
            .unwrap()
            .downcast_ref::<IfccadCborReadError>()
            .is_some());
        let out = crate::ifccad::codec::cbor::encode_bytes(&d).unwrap();
        let mut changed = d;
        changed.header.author.push('x');
        assert!(matches!(
            verify_readback(&out.bytes, out.readback_limits, &changed),
            Err(IfccadCborEncodeError::SemanticMismatch(_))
        ));
    }
    #[test]
    fn exact_comparison_detects_signed_zero_in_deep_dimension_state() {
        use crate::dimension::*;
        let mut d = load_ifccad_bytes(
            include_bytes!("../../../conformance/next/ifccad/valid/dimension-linear-aligned.ifcx"),
            Default::default(),
        )
        .unwrap()
        .into_document();
        for e in &mut d.model.entities {
            if let Some(IfccadNativeEntity {
                kind: IfccadEntityKind::Dimension(dim),
                ..
            }) = e.as_native_mut()
            {
                dim.overrides.numbers.primary.rounding_increment = Some(0.0);
            }
        }
        let mut b = d.clone();
        let mut changed = false;
        for e in &mut b.model.entities {
            if let Some(IfccadNativeEntity {
                kind: IfccadEntityKind::Dimension(dim),
                ..
            }) = e.as_native_mut()
            {
                dim.overrides.numbers.primary.rounding_increment = Some(-0.0);
                dim.label = DimensionLabel::Generated;
                changed = true;
            }
        }
        // Keep label state equal as well; only zero-sign must differ.
        for e in &mut d.model.entities {
            if let Some(IfccadNativeEntity {
                kind: IfccadEntityKind::Dimension(dim),
                ..
            }) = e.as_native_mut()
            {
                dim.label = DimensionLabel::Generated;
            }
        }
        assert!(changed);
        assert_eq!(d, b);
        assert!(!crate::ifccad::codec::cbor::documents_exactly_equal(&d, &b));
    }
}
