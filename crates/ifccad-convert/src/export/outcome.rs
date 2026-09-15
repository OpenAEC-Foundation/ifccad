use super::{ExportDiagnostic, ExportEntityMapping};
use ifccad::package::EncodedPackage;

pub struct ExportOutcome {
    geometry_assessment: crate::ConversionGeometryAssessment,
    transfer_assessment: crate::TransferAssessment,
    package: EncodedPackage,
    diagnostics: Vec<ExportDiagnostic>,
    entity_mapping: ExportEntityMapping,
}

impl ExportOutcome {
    pub(crate) fn with_geometry_assessment(
        mut self,
        value: crate::ConversionGeometryAssessment,
    ) -> Self {
        self.geometry_assessment = value;
        self
    }
    pub fn geometry_assessment(&self) -> &crate::ConversionGeometryAssessment {
        &self.geometry_assessment
    }
    pub fn into_all_parts(self) -> ExportOutcomeParts {
        ExportOutcomeParts {
            package: self.package,
            diagnostics: self.diagnostics,
            entity_mapping: self.entity_mapping,
            transfer_assessment: self.transfer_assessment,
            geometry_assessment: self.geometry_assessment,
        }
    }

    pub(crate) fn new(
        package: EncodedPackage,
        diagnostics: Vec<ExportDiagnostic>,
        entity_mapping: ExportEntityMapping,
    ) -> Self {
        Self {
            geometry_assessment: crate::ConversionGeometryAssessment::new(
                crate::ConversionGeometryTolerance::default(),
                ifccad::ifcdr::IfcdrLengthUnit::Unitless,
            )
            .unwrap(),
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

pub struct ExportOutcomeParts {
    pub package: EncodedPackage,
    pub diagnostics: Vec<ExportDiagnostic>,
    pub entity_mapping: ExportEntityMapping,
    pub transfer_assessment: crate::TransferAssessment,
    pub geometry_assessment: crate::ConversionGeometryAssessment,
}
