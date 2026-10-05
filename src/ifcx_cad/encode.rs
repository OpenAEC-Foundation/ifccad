use super::*;

/// The phase that prevented strict IFCX-CAD encoding, retaining its original cause.
#[derive(Debug, thiserror::Error)]
pub enum IfcxCadEncodeError {
    #[error("{0}")]
    InvalidDocument(#[from] IfcxCadReport),
    #[error("IFCX serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("strict IFCX readback failed: {0}")]
    Readback(#[source] IfcxCadReadError),
    #[error("strict IFCX readback changed CAD semantics")]
    SemanticMismatch(#[source] IfcxCadReport),
}
/// Encode a fresh CAD-profile file, without updating an original source graph.
///
/// Failures retain the phase and typed cause instead of flattening diagnostics.
///
/// ```
/// use ocdraw::ifcx_cad::{encode_ifcx_cad_document, IfcxCadDocument, IfcxCadEncodeError};
/// # fn inspect(document: &IfcxCadDocument) -> Result<(), IfcxCadEncodeError> {
/// match encode_ifcx_cad_document(document) {
///     Ok(encoded) => { let _ = encoded.bytes(); }
///     Err(IfcxCadEncodeError::InvalidDocument(report)) => {
///         for message in &report.errors { eprintln!("{message}"); }
///     }
///     Err(error) => return Err(error),
/// }
/// # Ok(())
/// # }
/// ```
pub fn encode_ifcx_cad_document(
    document: &IfcxCadDocument,
) -> Result<EncodedIfcxCad, IfcxCadEncodeError> {
    validate_ifcx_cad_document(document)?;
    let bytes = super::codec::ifcx_json::encode_bytes(document)?;
    verify_readback(&bytes, document)?;
    Ok(EncodedIfcxCad::from_bytes(bytes))
}

fn verify_readback(bytes: &[u8], document: &IfcxCadDocument) -> Result<(), IfcxCadEncodeError> {
    let loaded = load_ifcx_cad_bytes(bytes, IfcxCadReadOptions::default())
        .map_err(IfcxCadEncodeError::Readback)?;
    let mut expected = document.clone();
    for entity in expected
        .model
        .entities
        .iter_mut()
        .chain(
            expected
                .paper_layouts
                .iter_mut()
                .flat_map(|p| &mut p.entities),
        )
        .chain(expected.blocks.iter_mut().flat_map(|b| &mut b.entities))
    {
        if let IfcxCadEntityKind::Viewport(v) = &mut entity.kind {
            v.frozen_layers.sort_unstable();
        }
    }
    expected.layers.sort_by_key(|layer| layer.id.to_string());
    expected.blocks.sort_by_key(|block| block.id.to_string());
    expected.line_patterns.sort_by_key(|p| p.id.0.to_string());
    expected
        .paper_layouts
        .sort_by_key(|layout| layout.tab_index);
    if loaded.document() != &expected {
        return Err(IfcxCadEncodeError::SemanticMismatch(IfcxCadReport::one(
            "strict IFCX readback changed CAD semantics",
        )));
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::error::Error;

    const HELLO: &[u8] = include_bytes!("../../examples/ifcx-native-cad/hello-cad.ifcx");

    #[test]
    fn readback_failure_retains_reader_report_and_source() {
        let document = load_ifcx_cad_bytes(HELLO, Default::default())
            .unwrap()
            .into_document();
        let malformed = b"{";
        let expected = load_ifcx_cad_bytes(malformed, Default::default()).unwrap_err();
        let error = verify_readback(malformed, &document).unwrap_err();
        let IfcxCadEncodeError::Readback(read_error) = &error else {
            panic!("expected production-reader failure");
        };
        assert_eq!(read_error.report(), expected.report());
        assert_eq!(error.report(), Some(expected.report()));
        let source = error
            .source()
            .unwrap()
            .downcast_ref::<IfcxCadReadError>()
            .unwrap();
        assert_eq!(
            source.source().unwrap().downcast_ref::<IfcxCadReport>(),
            Some(expected.report())
        );
    }

    #[test]
    fn semantic_mismatch_is_distinct_from_readback() {
        let mut document = load_ifcx_cad_bytes(HELLO, Default::default())
            .unwrap()
            .into_document();
        document.header.author = "Different author".into();
        let error = verify_readback(HELLO, &document).unwrap_err();
        let IfcxCadEncodeError::SemanticMismatch(report) = &error else {
            panic!("valid output with different semantics is not a reader failure");
        };
        assert_eq!(
            report.errors,
            ["strict IFCX readback changed CAD semantics"]
        );
        assert_eq!(error.report(), Some(report));
        assert_eq!(
            error.source().unwrap().downcast_ref::<IfcxCadReport>(),
            Some(report)
        );
    }

    #[test]
    fn report_messages_survive_wrapping_and_display() {
        let report = IfcxCadReport {
            errors: vec!["first failure".into(), "second failure".into()],
        };
        let display = report.to_string();
        let error = IfcxCadEncodeError::InvalidDocument(report);
        let source = error
            .source()
            .unwrap()
            .downcast_ref::<IfcxCadReport>()
            .unwrap();
        assert_eq!(source.errors, ["first failure", "second failure"]);
        assert_eq!(source.to_string(), display);
        assert_eq!(error.report(), Some(source));
    }

    #[test]
    fn serialization_source_is_preserved() {
        // Synthetic wrapping evidence; valid current documents need not fail serialization.
        let source = serde_json::from_slice::<Value>(b"{").unwrap_err();
        let location = (source.line(), source.column(), source.classify());
        let error = IfcxCadEncodeError::Serialization(source);
        let source = error
            .source()
            .unwrap()
            .downcast_ref::<serde_json::Error>()
            .unwrap();
        assert_eq!(
            (source.line(), source.column(), source.classify()),
            location
        );
        assert!(error.report().is_none());
    }
}
