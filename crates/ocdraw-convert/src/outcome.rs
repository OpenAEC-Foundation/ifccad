use crate::*;
use ocdraw::ocdraw::{EncodedOcdraw, OcdrawDocument};
use opencadcodec::{CadDocument, Handle};
use std::collections::BTreeMap;
pub struct CadToEncodedOcdrawOutcome {
    pub(crate) text: crate::OcdrawTextAssessment,
    pub(crate) text_styles: BTreeMap<Handle, ocdraw::ocdraw::OcdrawTextStyleId>,
    pub(crate) preservation: crate::OcdrawPreservationReport,
    pub(crate) encoded: EncodedOcdraw,
    pub(crate) diagnostics: Vec<CadToOcdrawDiagnostic>,
    pub(crate) entity_mapping: BTreeMap<Handle, u64>,
    pub(crate) geometry: crate::OcdrawGeometryAssessment,
}

impl CadToEncodedOcdrawOutcome {
    pub fn text_assessment(&self) -> &crate::OcdrawTextAssessment {
        &self.text
    }
    /// Maps allocated source STYLE handles; unallocated records remain named styles.
    pub fn text_style_mapping(&self) -> &BTreeMap<Handle, ocdraw::ocdraw::OcdrawTextStyleId> {
        &self.text_styles
    }
    pub fn preservation_report(&self) -> &crate::OcdrawPreservationReport {
        &self.preservation
    }
    pub fn encoded(&self) -> &EncodedOcdraw {
        &self.encoded
    }
    pub fn diagnostics(&self) -> &[CadToOcdrawDiagnostic] {
        &self.diagnostics
    }
    pub fn entity_mapping(&self) -> &BTreeMap<Handle, u64> {
        &self.entity_mapping
    }
    pub fn geometry_assessment(&self) -> &crate::OcdrawGeometryAssessment {
        &self.geometry
    }
    pub fn into_encoded(self) -> EncodedOcdraw {
        self.encoded
    }
}

pub struct CadToOcdrawDocumentOutcome {
    pub(crate) text: crate::OcdrawTextAssessment,
    pub(crate) text_styles: BTreeMap<Handle, ocdraw::ocdraw::OcdrawTextStyleId>,
    pub(crate) preservation: crate::OcdrawPreservationReport,
    pub(crate) document: OcdrawDocument,
    pub(crate) diagnostics: Vec<CadToOcdrawDiagnostic>,
    pub(crate) entity_mapping: BTreeMap<Handle, u64>,
    pub(crate) geometry: crate::OcdrawGeometryAssessment,
}

impl CadToOcdrawDocumentOutcome {
    pub fn text_assessment(&self) -> &crate::OcdrawTextAssessment {
        &self.text
    }
    /// Maps allocated source STYLE handles; unallocated records remain named styles.
    pub fn text_style_mapping(&self) -> &BTreeMap<Handle, ocdraw::ocdraw::OcdrawTextStyleId> {
        &self.text_styles
    }
    pub fn preservation_report(&self) -> &crate::OcdrawPreservationReport {
        &self.preservation
    }
    pub fn document(&self) -> &OcdrawDocument {
        &self.document
    }
    pub fn diagnostics(&self) -> &[CadToOcdrawDiagnostic] {
        &self.diagnostics
    }
    pub fn entity_mapping(&self) -> &BTreeMap<Handle, u64> {
        &self.entity_mapping
    }
    pub fn geometry_assessment(&self) -> &crate::OcdrawGeometryAssessment {
        &self.geometry
    }
    pub fn into_document(self) -> OcdrawDocument {
        self.document
    }
}

pub struct OcdrawToCadOutcome {
    pub(crate) text: crate::OcdrawTextAssessment,
    pub(crate) text_styles: BTreeMap<ocdraw::ocdraw::OcdrawTextStyleId, Handle>,
    pub(crate) preservation: crate::OcdrawPreservationReport,
    pub(crate) document: CadDocument,
    pub(crate) diagnostics: Vec<OcdrawToCadDiagnostic>,
    pub(crate) entity_mapping: BTreeMap<u64, Handle>,
    pub(crate) geometry: crate::OcdrawGeometryAssessment,
}

impl OcdrawToCadOutcome {
    pub fn text_assessment(&self) -> &crate::OcdrawTextAssessment {
        &self.text
    }
    pub fn text_style_mapping(&self) -> &BTreeMap<ocdraw::ocdraw::OcdrawTextStyleId, Handle> {
        &self.text_styles
    }
    pub fn preservation_report(&self) -> &crate::OcdrawPreservationReport {
        &self.preservation
    }
    pub fn document(&self) -> &CadDocument {
        &self.document
    }
    pub fn diagnostics(&self) -> &[OcdrawToCadDiagnostic] {
        &self.diagnostics
    }
    pub fn entity_mapping(&self) -> &BTreeMap<u64, Handle> {
        &self.entity_mapping
    }
    pub fn geometry_assessment(&self) -> &crate::OcdrawGeometryAssessment {
        &self.geometry
    }
    pub fn into_document(self) -> CadDocument {
        self.document
    }
}
