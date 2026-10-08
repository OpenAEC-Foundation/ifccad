use super::*;

/// The phase that prevented strict IFCCAD encoding, retaining its original cause.
#[derive(Debug, thiserror::Error)]
pub enum IfccadEncodeError {
    #[error("{0}")]
    InvalidDocument(#[from] IfccadReport),
    #[error("IFCX serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("strict IFCX readback failed: {0}")]
    Readback(#[source] IfccadReadError),
    #[error("strict IFCX readback changed CAD semantics")]
    SemanticMismatch(#[source] IfccadReport),
}
/// Encode a fresh CAD-profile file, without updating an original source graph.
///
/// Failures retain the phase and typed cause instead of flattening diagnostics.
///
/// ```
/// use ocdraw::ifccad::{encode_ifccad_document, IfccadDocument, IfccadEncodeError};
/// # fn inspect(document: &IfccadDocument) -> Result<(), IfccadEncodeError> {
/// match encode_ifccad_document(document) {
///     Ok(encoded) => { let _ = encoded.bytes(); }
///     Err(IfccadEncodeError::InvalidDocument(report)) => {
///         for message in &report.errors { eprintln!("{message}"); }
///     }
///     Err(error) => return Err(error),
/// }
/// # Ok(())
/// # }
/// ```
pub fn encode_ifccad_document(
    document: &IfccadDocument,
) -> Result<EncodedIfccad, IfccadEncodeError> {
    validate_ifccad_document(document)?;
    let bytes = super::codec::json::encode_bytes(document)?;
    verify_readback(&bytes, document)?;
    Ok(EncodedIfccad::from_bytes(bytes))
}

fn verify_readback(bytes: &[u8], document: &IfccadDocument) -> Result<(), IfccadEncodeError> {
    let loaded = load_ifccad_bytes(bytes, IfccadReadOptions::default())
        .map_err(IfccadEncodeError::Readback)?;
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
        if let Some(entity) = entity.as_native_mut() {
            if let IfccadEntityKind::Viewport(v) = &mut entity.kind {
                v.frozen_layers.sort_unstable();
            }
        }
    }
    if let Some(p) = &mut expected.preservation {
        p.records.sort_by_key(|r| r.id.0);
    }
    expected.layers.sort_by_key(|layer| layer.id.to_string());
    expected.blocks.sort_by_key(|block| block.id.to_string());
    expected.line_patterns.sort_by_key(|p| p.id.0.to_string());
    expected
        .paper_layouts
        .sort_by_key(|layout| layout.tab_index);
    if loaded.document() != &expected {
        return Err(IfccadEncodeError::SemanticMismatch(IfccadReport::one(
            "strict IFCX readback changed CAD semantics",
        )));
    }
    Ok(())
}

impl IfccadEncodeError {
    pub fn report(&self) -> Option<&IfccadReport> {
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

    const HELLO: &[u8] = include_bytes!("../../examples/ifccad/hello-cad.ifcx");

    #[test]
    fn readback_failure_retains_reader_report_and_source() {
        let document = load_ifccad_bytes(HELLO, Default::default())
            .unwrap()
            .into_document();
        let malformed = b"{";
        let expected = load_ifccad_bytes(malformed, Default::default()).unwrap_err();
        let error = verify_readback(malformed, &document).unwrap_err();
        let IfccadEncodeError::Readback(read_error) = &error else {
            panic!("expected production-reader failure");
        };
        assert_eq!(read_error.report(), expected.report());
        assert_eq!(error.report(), Some(expected.report()));
        let source = error
            .source()
            .unwrap()
            .downcast_ref::<IfccadReadError>()
            .unwrap();
        assert_eq!(
            source.source().unwrap().downcast_ref::<IfccadReport>(),
            Some(expected.report())
        );
    }

    #[test]
    fn semantic_mismatch_is_distinct_from_readback() {
        let mut document = load_ifccad_bytes(HELLO, Default::default())
            .unwrap()
            .into_document();
        document.header.author = "Different author".into();
        let error = verify_readback(HELLO, &document).unwrap_err();
        let IfccadEncodeError::SemanticMismatch(report) = &error else {
            panic!("valid output with different semantics is not a reader failure");
        };
        assert_eq!(
            report.errors,
            ["strict IFCX readback changed CAD semantics"]
        );
        assert_eq!(error.report(), Some(report));
        assert_eq!(
            error.source().unwrap().downcast_ref::<IfccadReport>(),
            Some(report)
        );
    }

    #[test]
    fn report_messages_survive_wrapping_and_display() {
        let report = IfccadReport {
            errors: vec!["first failure".into(), "second failure".into()],
        };
        let display = report.to_string();
        let error = IfccadEncodeError::InvalidDocument(report);
        let source = error
            .source()
            .unwrap()
            .downcast_ref::<IfccadReport>()
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
        let error = IfccadEncodeError::Serialization(source);
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
