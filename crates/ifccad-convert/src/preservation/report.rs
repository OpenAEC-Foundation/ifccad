use ocdraw::ifccad::IfccadPreservationRecordId;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadPreservationPhase {
    Capture,
    NativeRepresentation,
    Restoration,
    CadExchange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadPreservationResult {
    CapturedTyped,
    NativePropertyUnrepresented,
    RestorationUnavailable,
    RestoredTyped,
    NotRestored,
    StorageSupplementOmitted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadPreservationReason {
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
pub struct IfccadPreservationReportEntry {
    pub record_id: Option<IfccadPreservationRecordId>,
    pub entity_id: Option<u64>,
    pub source_id: String,
    pub source_key: String,
    pub schema: String,
    pub version: u32,
    pub phase: IfccadPreservationPhase,
    pub result: IfccadPreservationResult,
    pub reason: Option<IfccadPreservationReason>,
    pub location: String,
    pub message: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct IfccadPreservationReport {
    pub(crate) entries: Vec<IfccadPreservationReportEntry>,
}
impl IfccadPreservationReport {
    pub fn entries(&self) -> &[IfccadPreservationReportEntry] {
        &self.entries
    }
}

pub(crate) fn spline_entry(
    record_id: IfccadPreservationRecordId,
    source_key: String,
    result: IfccadPreservationResult,
    location: String,
    message: impl Into<String>,
) -> IfccadPreservationReportEntry {
    let phase = match result {
        IfccadPreservationResult::CapturedTyped
        | IfccadPreservationResult::StorageSupplementOmitted => IfccadPreservationPhase::Capture,
        IfccadPreservationResult::NativePropertyUnrepresented => {
            IfccadPreservationPhase::NativeRepresentation
        }
        _ => IfccadPreservationPhase::Restoration,
    };
    IfccadPreservationReportEntry {
        record_id: Some(record_id),
        entity_id: None,
        source_id: "cad-source-1".into(),
        source_key,
        schema: "openaec.opencadcodec.spline".into(),
        version: cad_preservation::SPLINE_PAYLOAD_VERSION,
        phase,
        result,
        reason: None,
        location,
        message: message.into(),
    }
}
