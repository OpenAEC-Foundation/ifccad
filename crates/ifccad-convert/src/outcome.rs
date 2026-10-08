use crate::IfccadDiagnostic;
use ocdraw::ifccad::{IfccadDocument, ValidatedIfccad};
use opencadcodec::{CadDocument, Handle};
use std::collections::BTreeMap;
/// A single identity domain. Numeric IDs never pass through floating point.
#[derive(Clone, Debug, Default)]
pub struct IfccadIdentityMap(BTreeMap<u64, Handle>);
impl IfccadIdentityMap {
    pub fn cad_handle(&self, id: u64) -> Option<Handle> {
        self.0.get(&id).copied()
    }
    pub fn ifccad_id(&self, handle: Handle) -> Option<u64> {
        self.0
            .iter()
            .find_map(|(&id, &h)| (h == handle).then_some(id))
    }
    pub fn iter(&self) -> impl Iterator<Item = (u64, Handle)> + '_ {
        self.0.iter().map(|(&id, &h)| (id, h))
    }
    pub(crate) fn remove(&mut self, id: u64) {
        self.0.remove(&id);
    }
    pub(crate) fn insert(&mut self, id: u64, handle: Handle) {
        self.0.insert(id, handle);
    }
}

#[derive(Clone, Debug, Default)]
pub struct IfccadMappings {
    pub text_styles: IfccadIdentityMap,
    pub ucss: IfccadIdentityMap,
    pub model_windows: IfccadIdentityMap,
    pub line_patterns: IfccadIdentityMap,
    pub layouts: IfccadIdentityMap,
    pub layers: IfccadIdentityMap,
    pub blocks: IfccadIdentityMap,
    pub entities: IfccadIdentityMap,
}

pub struct IfccadToCadOutcome {
    pub(crate) text: crate::IfccadTextAssessment,
    pub(crate) document: CadDocument,
    pub(crate) diagnostics: Vec<IfccadDiagnostic>,
    pub(crate) mappings: IfccadMappings,
    pub(crate) geometry: crate::IfccadGeometryAssessment,
    pub(crate) preservation: crate::IfccadPreservationReport,
}
impl IfccadToCadOutcome {
    pub fn text_assessment(&self) -> &crate::IfccadTextAssessment {
        &self.text
    }
    pub fn preservation_report(&self) -> &crate::IfccadPreservationReport {
        &self.preservation
    }
    pub fn geometry_assessment(&self) -> &crate::IfccadGeometryAssessment {
        &self.geometry
    }
    pub fn document(&self) -> &CadDocument {
        &self.document
    }
    pub fn into_document(self) -> CadDocument {
        self.document
    }
    pub fn diagnostics(&self) -> &[IfccadDiagnostic] {
        &self.diagnostics
    }
    pub fn mappings(&self) -> &IfccadMappings {
        &self.mappings
    }
}
/// A validated logical CAD projection, before IFCX encoding.
pub struct CadToIfccadDocumentOutcome {
    pub(crate) text: crate::IfccadTextAssessment,
    pub(crate) document: IfccadDocument,
    pub(crate) diagnostics: Vec<IfccadDiagnostic>,
    pub(crate) mappings: IfccadMappings,
    pub(crate) geometry: crate::IfccadGeometryAssessment,
    pub(crate) preservation: crate::IfccadPreservationReport,
}
impl CadToIfccadDocumentOutcome {
    pub fn text_assessment(&self) -> &crate::IfccadTextAssessment {
        &self.text
    }
    pub fn preservation_report(&self) -> &crate::IfccadPreservationReport {
        &self.preservation
    }
    pub fn geometry_assessment(&self) -> &crate::IfccadGeometryAssessment {
        &self.geometry
    }
    pub fn document(&self) -> &IfccadDocument {
        &self.document
    }
    pub fn into_document(self) -> IfccadDocument {
        self.document
    }
    pub fn diagnostics(&self) -> &[IfccadDiagnostic] {
        &self.diagnostics
    }
    pub fn mappings(&self) -> &IfccadMappings {
        &self.mappings
    }
}

pub struct CadToEncodedIfccadOutcome {
    pub(crate) text: crate::IfccadTextAssessment,
    pub(crate) validated: ValidatedIfccad,
    pub(crate) encoded: ocdraw::ifccad::EncodedIfccad,
    pub(crate) diagnostics: Vec<IfccadDiagnostic>,
    pub(crate) mappings: IfccadMappings,
    pub(crate) geometry: crate::IfccadGeometryAssessment,
    pub(crate) preservation: crate::IfccadPreservationReport,
}
impl CadToEncodedIfccadOutcome {
    pub fn text_assessment(&self) -> &crate::IfccadTextAssessment {
        &self.text
    }
    pub fn preservation_report(&self) -> &crate::IfccadPreservationReport {
        &self.preservation
    }
    pub fn geometry_assessment(&self) -> &crate::IfccadGeometryAssessment {
        &self.geometry
    }
    pub fn validated_source(&self) -> &ValidatedIfccad {
        &self.validated
    }
    pub fn encoded(&self) -> &ocdraw::ifccad::EncodedIfccad {
        &self.encoded
    }
    pub fn into_encoded(self) -> ocdraw::ifccad::EncodedIfccad {
        self.encoded
    }
    pub fn diagnostics(&self) -> &[IfccadDiagnostic] {
        &self.diagnostics
    }
    pub fn mappings(&self) -> &IfccadMappings {
        &self.mappings
    }
}
