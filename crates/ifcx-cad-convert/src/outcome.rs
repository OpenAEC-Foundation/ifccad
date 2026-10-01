use cadcodec::{CadDocument, Handle};
use ocdraw::ifcx_cad::{IfcxCadHeader, ValidatedIfcxCad};
use std::collections::BTreeMap;

/// Explicit target identity and provenance, supplied by the caller.
#[derive(Clone, Debug)]
pub struct IfcxCadTargetMetadata {
    pub header: IfcxCadHeader,
    pub drawing_id: u64,
}

/// Located unsupported content or a documented source recovery.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfcxCadDiagnostic {
    pub code: &'static str,
    pub location: String,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum IfcxCadConversionError {
    #[error("invalid CAD structure: {0}")]
    InvalidStructure(String),
    #[error("unsupported conversion content: {0:?}")]
    Unsupported(Vec<IfcxCadDiagnostic>),
    #[error("IFCX-CAD validation failed: {0}")]
    CoreValidation(String),
    #[error("CAD construction failed: {0}")]
    CadConstruction(String),
}

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
pub struct CadToIfcxCadOutcome {
    pub(crate) validated: ValidatedIfcxCad,
    pub(crate) bytes: Vec<u8>,
    pub(crate) diagnostics: Vec<IfcxCadDiagnostic>,
    pub(crate) mappings: IfcxCadMappings,
}
impl CadToIfcxCadOutcome {
    pub fn validated_ifcx(&self) -> &ValidatedIfcxCad {
        &self.validated
    }
    pub fn ifcx_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn diagnostics(&self) -> &[IfcxCadDiagnostic] {
        &self.diagnostics
    }
    pub fn mappings(&self) -> &IfcxCadMappings {
        &self.mappings
    }
}
pub(crate) fn diagnostic(
    code: &'static str,
    location: impl Into<String>,
    message: impl Into<String>,
) -> IfcxCadDiagnostic {
    IfcxCadDiagnostic {
        code,
        location: location.into(),
        message: message.into(),
    }
}
pub(crate) const UNITS: [&str; 25] = [
    "unitless",
    "in",
    "ft",
    "mi",
    "mm",
    "cm",
    "m",
    "km",
    "microin",
    "mil",
    "yd",
    "angstrom",
    "nm",
    "um",
    "dm",
    "dam",
    "hm",
    "Gm",
    "au",
    "ly",
    "pc",
    "usSurveyFoot",
    "usSurveyInch",
    "usSurveyYard",
    "usSurveyMile",
];
pub(crate) fn unit_code(unit: &str) -> i16 {
    UNITS
        .iter()
        .position(|u| *u == unit)
        .expect("validated unit") as i16
}
