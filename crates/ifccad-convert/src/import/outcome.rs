use super::{ImportDiagnostic, ImportEntityMapping};
use cadcodec::CadDocument;

/// Result of importing an IFCCAD drawing into a cadcodec document.
///
/// Diagnostics and source-to-target entity mappings are retained alongside
/// the document so callers can inspect conversion fidelity.
pub struct ImportOutcome {
    transfer_assessment: crate::TransferAssessment,
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
            transfer_assessment: crate::TransferAssessment::import(diagnostics.iter().any(
                |diagnostic| match diagnostic {
                    super::ImportDiagnostic::LinePatternFallback { .. }
                    | super::ImportDiagnostic::LineWeightRounded { .. } => true,
                },
            )),
            document,
            diagnostics,
            entity_mapping,
        }
    }

    /// Returns scoped fidelity evidence for this completed import.
    pub fn transfer_assessment(&self) -> &crate::TransferAssessment {
        &self.transfer_assessment
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
    fn fallback_and_rounding_keep_loss_and_incomplete_coverage() {
        for diagnostic in [
            crate::ImportDiagnostic::LinePatternFallback {
                requested: "Center".into(),
                applied: "Continuous".into(),
                count: 2,
            },
            crate::ImportDiagnostic::LineWeightRounded {
                requested_mm: 0.181,
                applied_mm: 0.18,
                count: 1,
            },
        ] {
            let outcome = ImportOutcome::new(
                CadDocument::new(),
                vec![diagnostic],
                ImportEntityMapping::default(),
            );
            assert_eq!(
                outcome.transfer_assessment().conclusion(),
                crate::TransferConclusion::LossDetected
            );
            assert_eq!(
                outcome.transfer_assessment().coverage(),
                crate::TransferCoverage::Incomplete
            );
            assert!(!outcome.transfer_assessment().limitations().is_empty());
        }
    }

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
