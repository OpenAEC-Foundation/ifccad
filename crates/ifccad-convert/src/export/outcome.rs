use super::{ExportDiagnostic, ExportEntityMapping};
use ifccad::package::EncodedPackage;

pub struct ExportOutcome {
    package: EncodedPackage,
    diagnostics: Vec<ExportDiagnostic>,
    entity_mapping: ExportEntityMapping,
}

impl ExportOutcome {
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
