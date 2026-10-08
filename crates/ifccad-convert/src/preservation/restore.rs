use super::*;
use cad_preservation::{
    decode_spline_snapshot, CODEC_REVISION, LEGACY_CODEC_REVISION, PREVIOUS_CODEC_REVISION,
    SPLINE_PAYLOAD_VERSION,
};
use ocdraw::ifccad::*;
use opencadcodec::entities::Spline;

pub(crate) fn restore_spline(
    record: &IfccadPreservationRecord,
    entity: &IfccadOpaqueEntity,
    drawing: &IfccadDocument,
) -> Result<Spline, IfccadPreservationReason> {
    use IfccadPreservationReason::*;
    let p = drawing.preservation.as_ref().ok_or(MissingDependency)?;
    let source = p
        .sources
        .iter()
        .find(|s| s.id == record.source_id)
        .ok_or(MissingDependency)?;
    if record.payload.schema != "openaec.opencadcodec.spline"
        || !matches!(record.payload.version, 1 | 2)
        || record.payload.kind != IfccadPreservationPayloadKind::AdapterSnapshot
        || record.representation != IfccadPreservationRepresentation::CodecTyped
    {
        return Err(UnsupportedPayload);
    }
    let expected_version = match source.provider_revision.as_str() {
        CODEC_REVISION | PREVIOUS_CODEC_REVISION => SPLINE_PAYLOAD_VERSION,
        LEGACY_CODEC_REVISION => 1,
        _ => return Err(UnsupportedPayload),
    };
    if source.provider != "opencadcodec" || record.payload.version != expected_version {
        return Err(UnsupportedPayload);
    }
    super::conditions::evaluate(record, entity, drawing)?;
    let snapshot = decode_spline_snapshot(&record.payload.bytes).map_err(|e| match e {
        cad_preservation::CadSplineSnapshotError::UnsupportedRevision => UnsupportedPayload,
        _ => MalformedPayload,
    })?;
    if snapshot.codec_revision() != source.provider_revision {
        return Err(MalformedPayload);
    }
    if record.source_key != format!("{:x}", snapshot.source_handle()) {
        return Err(MalformedPayload);
    }
    let spline = snapshot.to_source();
    if super::conditions::unsupported_common_context(&spline) {
        return Err(UnsupportedContext);
    }
    super::references::qualify_references(&spline, record, entity, drawing)?;
    Ok(spline)
}

pub(crate) fn report_restore(
    report: &mut IfccadPreservationReport,
    drawing: &IfccadDocument,
    record: &IfccadPreservationRecord,
    entity: Option<u64>,
    reason: Option<IfccadPreservationReason>,
) {
    report.entries.push(IfccadPreservationReportEntry {
        record_id: Some(record.id),
        entity_id: entity,
        source_id: record.source_id.clone(),
        source_key: record.source_key.clone(),
        schema: record.payload.schema.clone(),
        version: record.payload.version,
        phase: IfccadPreservationPhase::Restoration,
        result: if reason.is_some() {
            IfccadPreservationResult::NotRestored
        } else {
            IfccadPreservationResult::RestoredTyped
        },
        reason,
        location: format!("/cad/d{}/preservation/r{}", drawing.drawing_id, record.id.0),
        message: if let Some(r) = reason {
            format!("preserved source restoration unavailable: {r:?}")
        } else {
            "typed source restored with current native common properties".into()
        },
    });
}
