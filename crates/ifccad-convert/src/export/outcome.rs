use super::{ExportDiagnostic, ExportEntityMapping};
use ifccad::package::EncodedPackage;

pub struct ExportOutcome {
    transfer_assessment: crate::TransferAssessment,
    package: EncodedPackage,
    diagnostics: Vec<ExportDiagnostic>,
    entity_mapping: ExportEntityMapping,
}

impl ExportOutcome {
    pub(crate) fn new(
        package: EncodedPackage,
        diagnostics: Vec<ExportDiagnostic>,
        entity_mapping: ExportEntityMapping,
    ) -> Self {
        Self {
            transfer_assessment: crate::TransferAssessment::export(
                diagnostics.iter().any(ExportDiagnostic::is_loss),
            ),
            package,
            diagnostics,
            entity_mapping,
        }
    }

    /// Returns fidelity evidence within the pinned public CadDocument boundary.
    pub fn transfer_assessment(&self) -> &crate::TransferAssessment {
        &self.transfer_assessment
    }

    pub fn package(&self) -> &EncodedPackage {
        &self.package
    }

    pub fn diagnostics(&self) -> &[ExportDiagnostic] {
        &self.diagnostics
    }

    pub fn entity_mapping(&self) -> &ExportEntityMapping {
        &self.entity_mapping
    }

    pub fn into_package(self) -> EncodedPackage {
        self.package
    }

    pub fn into_parts(self) -> (EncodedPackage, Vec<ExportDiagnostic>, ExportEntityMapping) {
        (self.package, self.diagnostics, self.entity_mapping)
    }
}
