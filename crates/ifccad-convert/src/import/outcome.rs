use super::{ImportDiagnostic, ImportEntityMapping};
use cadcodec::CadDocument;

/// Result of importing an IFCCAD drawing into a cadcodec document.
///
/// Diagnostics and source-to-target entity mappings are retained alongside
/// the document so callers can inspect conversion fidelity.
pub struct ImportOutcome {
    document: CadDocument,
    diagnostics: Vec<ImportDiagnostic>,
    entity_mapping: ImportEntityMapping,
}

impl ImportOutcome {
    pub(crate) fn new(
        document: CadDocument,
        diagnostics: Vec<ImportDiagnostic>,
        entity_mapping: ImportEntityMapping,
    ) -> Self {
        Self {
            document,
            diagnostics,
            entity_mapping,
        }
    }

    /// Borrows the produced CAD document.
    pub fn document(&self) -> &CadDocument {
        &self.document
    }

    /// Returns non-fatal diagnostics collected during import.
    pub fn diagnostics(&self) -> &[ImportDiagnostic] {
        &self.diagnostics
    }

    /// Maps IFCDR entity identifiers to their cadcodec handles.
    pub fn entity_mapping(&self) -> &ImportEntityMapping {
        &self.entity_mapping
    }

    /// Consumes the outcome and returns only the CAD document.
    pub fn into_document(self) -> CadDocument {
        self.document
    }

    /// Consumes the outcome and returns all three result components.
    pub fn into_parts(self) -> (CadDocument, Vec<ImportDiagnostic>, ImportEntityMapping) {
        (self.document, self.diagnostics, self.entity_mapping)
    }
}

#[cfg(test)]
mod tests {
    use super::ImportOutcome;
    use crate::ImportEntityMapping;
    use cadcodec::CadDocument;

    #[test]
    fn outcome_borrows_and_transfers_all_conversion_results() {
        let outcome = ImportOutcome::new(
            CadDocument::new(),
            Vec::new(),
            ImportEntityMapping::default(),
        );

        assert_eq!(outcome.document().entities().count(), 0);
        assert!(outcome.diagnostics().is_empty());
        assert!(outcome.entity_mapping().is_empty());

        let (document, diagnostics, mapping) = outcome.into_parts();
        assert_eq!(document.entities().count(), 0);
        assert!(diagnostics.is_empty());
        assert!(mapping.is_empty());

        let document = ImportOutcome::new(
            CadDocument::new(),
            Vec::new(),
            ImportEntityMapping::default(),
        )
        .into_document();
        assert_eq!(document.entities().count(), 0);
    }
}
