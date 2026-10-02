use crate::IfcxCadDiagnostic;
use ocdraw::ifcx_cad::{IfcxCadDocument, ValidatedIfcxCad};
use opencadcodec::{CadDocument, Handle};
use std::collections::BTreeMap;
/// A single identity domain. Numeric IDs never pass through floating point.
#[derive(Clone, Debug, Default)]
pub struct IfcxCadIdentityMap(BTreeMap<u64, Handle>);
impl IfcxCadIdentityMap {
    pub fn cad_handle(&self, id: u64) -> Option<Handle> {
        self.0.get(&id).copied()
    }
    pub fn ifcx_id(&self, handle: Handle) -> Option<u64> {
        self.0
            .iter()
            .find_map(|(&id, &h)| (h == handle).then_some(id))
    }
    pub fn iter(&self) -> impl Iterator<Item = (u64, Handle)> + '_ {
        self.0.iter().map(|(&id, &h)| (id, h))
    }
    pub(crate) fn insert(&mut self, id: u64, handle: Handle) {
        self.0.insert(id, handle);
    }
}

#[derive(Clone, Debug, Default)]
pub struct IfcxCadMappings {
    pub line_patterns: IfcxCadIdentityMap,
    pub layouts: IfcxCadIdentityMap,
    pub layers: IfcxCadIdentityMap,
    pub blocks: IfcxCadIdentityMap,
    pub entities: IfcxCadIdentityMap,
}

pub struct IfcxCadToCadOutcome {
    pub(crate) document: CadDocument,
    pub(crate) diagnostics: Vec<IfcxCadDiagnostic>,
    pub(crate) mappings: IfcxCadMappings,
}
impl IfcxCadToCadOutcome {
    pub fn document(&self) -> &CadDocument {
        &self.document
    }
    pub fn into_document(self) -> CadDocument {
        self.document
    }
    pub fn diagnostics(&self) -> &[IfcxCadDiagnostic] {
        &self.diagnostics
    }
    pub fn mappings(&self) -> &IfcxCadMappings {
        &self.mappings
    }
}
/// A validated logical CAD projection, before IFCX encoding.
pub struct CadToIfcxCadDocumentOutcome {
    pub(crate) document: IfcxCadDocument,
    pub(crate) diagnostics: Vec<IfcxCadDiagnostic>,
    pub(crate) mappings: IfcxCadMappings,
}
impl CadToIfcxCadDocumentOutcome {
    pub fn document(&self) -> &IfcxCadDocument {
        &self.document
    }
    pub fn into_document(self) -> IfcxCadDocument {
        self.document
    }
    pub fn diagnostics(&self) -> &[IfcxCadDiagnostic] {
        &self.diagnostics
    }
    pub fn mappings(&self) -> &IfcxCadMappings {
        &self.mappings
    }
}

pub struct CadToEncodedIfcxCadOutcome {
    pub(crate) validated: ValidatedIfcxCad,
    pub(crate) encoded: ocdraw::ifcx_cad::EncodedIfcxCad,
    pub(crate) diagnostics: Vec<IfcxCadDiagnostic>,
    pub(crate) mappings: IfcxCadMappings,
}
impl CadToEncodedIfcxCadOutcome {
    pub fn validated_source(&self) -> &ValidatedIfcxCad {
        &self.validated
    }
    pub fn encoded(&self) -> &ocdraw::ifcx_cad::EncodedIfcxCad {
        &self.encoded
    }
    pub fn into_encoded(self) -> ocdraw::ifcx_cad::EncodedIfcxCad {
        self.encoded
    }
    pub fn diagnostics(&self) -> &[IfcxCadDiagnostic] {
        &self.diagnostics
    }
    pub fn mappings(&self) -> &IfcxCadMappings {
        &self.mappings
    }
}
