use ocdraw::ocdraw::OcdrawPreservationRecordId;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OcdrawPreservationPhase {
    Capture,
    NativeRepresentation,
    Restoration,
    CadExchange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OcdrawPreservationResult {
    CapturedTyped,
    NativePropertyUnrepresented,
    RestorationUnavailable,
    RestoredTyped,
    NotRestored,
    StorageSupplementOmitted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OcdrawPreservationReason {
    MissingDependency,
    ChangedDependency,
    UnsupportedPayload,
    UnsupportedPredicate,
    UnsupportedContext,
    MalformedPayload,
    UnresolvedReference,
    SourceCodecRejected,
    TargetCodecRejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OcdrawPreservationReportEntry {
    pub record_id: Option<OcdrawPreservationRecordId>,
    pub entity_id: Option<u64>,
    pub source_id: String,
    pub source_key: String,
    pub schema: String,
    pub version: u32,
    pub phase: OcdrawPreservationPhase,
    pub result: OcdrawPreservationResult,
    pub reason: Option<OcdrawPreservationReason>,
    pub location: String,
    pub message: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OcdrawPreservationReport {
    pub(crate) entries: Vec<OcdrawPreservationReportEntry>,
}
impl OcdrawPreservationReport {
    pub fn entries(&self) -> &[OcdrawPreservationReportEntry] {
        &self.entries
    }
}

pub(crate) fn spline_entry(
    record_id: OcdrawPreservationRecordId,
    source_key: String,
    result: OcdrawPreservationResult,
    location: String,
    message: impl Into<String>,
) -> OcdrawPreservationReportEntry {
    let phase = match result {
        OcdrawPreservationResult::CapturedTyped
        | OcdrawPreservationResult::StorageSupplementOmitted => OcdrawPreservationPhase::Capture,
        OcdrawPreservationResult::NativePropertyUnrepresented => {
            OcdrawPreservationPhase::NativeRepresentation
        }
        _ => OcdrawPreservationPhase::Restoration,
    };
    OcdrawPreservationReportEntry {
        record_id: Some(record_id),
        entity_id: None,
        source_id: "cad-source-1".into(),
        source_key,
        schema: "openaec.opencadcodec.spline".into(),
        version: 1,
        phase,
        result,
        reason: None,
        location,
        message: message.into(),
    }
}
