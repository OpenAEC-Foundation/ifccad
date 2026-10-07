use super::*;
use ocdraw::ocdraw::*;
use opencadcodec::entities::Spline;

pub(crate) fn restore_spline(
    record: &OcdrawPreservationRecord,
    entity: &DrawingOpaqueEntity,
    drawing: &OcdrawDocument,
) -> Result<Spline, OcdrawPreservationReason> {
    use OcdrawPreservationReason::*;
    let p = drawing.preservation.as_ref().ok_or(MissingDependency)?;
    let source = p
        .sources
        .iter()
        .find(|s| s.id == record.source_id)
        .ok_or(MissingDependency)?;
    if record.payload.schema != "openaec.opencadcodec.spline"
        || !matches!(record.payload.version, 1 | 2)
        || record.payload.kind != OcdrawPreservationPayloadKind::AdapterSnapshot
        || record.representation != OcdrawPreservationRepresentation::CodecTyped
    {
        return Err(UnsupportedPayload);
    }
    let expected_version = match source.provider_revision.as_str() {
        CODEC_REVISION => SPLINE_PAYLOAD_VERSION,
        LEGACY_CODEC_REVISION => 1,
        _ => return Err(UnsupportedPayload),
    };
    if source.provider != "opencadcodec" || record.payload.version != expected_version {
        return Err(UnsupportedPayload);
    }
    evaluate_spline_conditions(record, entity, drawing)?;
    let snapshot = decode_spline_snapshot(&record.payload.bytes).map_err(|e| match e {
        OcdrawSplineSnapshotError::UnsupportedRevision => UnsupportedPayload,
        _ => MalformedPayload,
    })?;
    if snapshot.codec_revision != source.provider_revision {
        return Err(MalformedPayload);
    }
    if record.source_key != format!("{:x}", snapshot.source_handle.0) {
        return Err(MalformedPayload);
    }
    let spline = snapshot.to_source();
    if unsupported_common_context(&spline) {
        return Err(UnsupportedContext);
    }
    qualify_references(&spline, record, entity, drawing)?;
    Ok(spline)
}

pub(crate) fn report_restore(
    report: &mut OcdrawPreservationReport,
    record: &OcdrawPreservationRecord,
    entity: Option<u64>,
    reason: Option<OcdrawPreservationReason>,
) {
    report.entries.push(OcdrawPreservationReportEntry {
        record_id: Some(record.id),
        entity_id: entity,
        source_id: record.source_id.clone(),
        source_key: record.source_key.clone(),
        schema: record.payload.schema.clone(),
        version: record.payload.version,
        phase: OcdrawPreservationPhase::Restoration,
        result: if reason.is_some() {
            OcdrawPreservationResult::NotRestored
        } else {
            OcdrawPreservationResult::RestoredTyped
        },
        reason,
        location: format!("/preservation/records/{}", record.id.0),
        message: if let Some(r) = reason {
            format!("preserved source restoration unavailable: {r:?}")
        } else {
            "typed source restored with current native common properties".into()
        },
    });
}
