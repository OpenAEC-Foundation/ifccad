#![doc = include_str!("../README.md")]

mod appearance;
mod conversion;
mod diagnostic;
mod entity_mapping;
mod units;

pub use conversion::convert_drawing;
pub use diagnostic::{ConversionDiagnostic, ConversionError, LostTransparencyMode};
pub use entity_mapping::EntityMapping;

pub use cadcodec;

use cadcodec::CadDocument;

pub struct ConversionOutcome {
    document: CadDocument,
    diagnostics: Vec<ConversionDiagnostic>,
    entity_mapping: EntityMapping,
}

impl ConversionOutcome {
    pub(crate) fn new(
        document: CadDocument,
        diagnostics: Vec<ConversionDiagnostic>,
        entity_mapping: EntityMapping,
    ) -> Self {
        Self {
            document,
            diagnostics,
            entity_mapping,
        }
    }

    pub fn document(&self) -> &CadDocument {
        &self.document
    }

    pub fn diagnostics(&self) -> &[ConversionDiagnostic] {
        &self.diagnostics
    }

    pub fn entity_mapping(&self) -> &EntityMapping {
        &self.entity_mapping
    }

    pub fn into_document(self) -> CadDocument {
        self.document
    }

    pub fn into_parts(self) -> (CadDocument, Vec<ConversionDiagnostic>, EntityMapping) {
        (self.document, self.diagnostics, self.entity_mapping)
    }
}

#[cfg(test)]
mod tests {
    use super::{ConversionOutcome, EntityMapping};
    use cadcodec::CadDocument;

    #[test]
    fn outcome_borrows_and_transfers_all_conversion_results() {
        let outcome =
            ConversionOutcome::new(CadDocument::new(), Vec::new(), EntityMapping::default());

        assert_eq!(outcome.document().entities().count(), 0);
        assert!(outcome.diagnostics().is_empty());
        assert!(outcome.entity_mapping().is_empty());

        let (document, diagnostics, mapping) = outcome.into_parts();
        assert_eq!(document.entities().count(), 0);
        assert!(diagnostics.is_empty());
        assert!(mapping.is_empty());

        let document =
            ConversionOutcome::new(CadDocument::new(), Vec::new(), EntityMapping::default())
                .into_document();
        assert_eq!(document.entities().count(), 0);
    }
}
