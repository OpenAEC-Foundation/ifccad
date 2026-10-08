use super::*;
use crate::*;
use cad_preservation::{capture_spline, CODEC_REVISION, SPLINE_PAYLOAD_VERSION, SPLINE_SCHEMA};
use ocdraw::ifccad::*;
use opencadcodec::{entities::Spline, CadDocument, Color, EntityType, LineWeight};

#[derive(Default)]
pub(crate) struct Capture {
    pub enabled: bool,
    pub records: Vec<IfccadPreservationRecord>,
    pub report: IfccadPreservationReport,
}
impl Capture {
    pub fn entity(
        &mut self,
        s: &Spline,
        index: u64,
        source: &CadDocument,
        patterns: &crate::mapping::line_pattern::SourcePatterns,
        ids: &mut IfccadIdCounters,
        mappings: &mut IfccadMappings,
    ) -> Result<IfccadEntity, IfccadConversionError> {
        let id = ids.allocate_entity_id()?;
        let record_id = ids.allocate_preservation_record_id()?;
        mappings.entities.insert(id, s.common.handle);
        let record = self.record(
            s,
            index,
            record_id,
            Some(IfccadPreservationTarget::Entity(id)),
        )?;
        let layer = if let Some(handle) = s.common.layer_handle.filter(|h| !h.is_null()) {
            source
                .layers
                .iter()
                .find(|l| l.handle == handle && l.name.eq_ignore_ascii_case(&s.common.layer))
        } else {
            source.layers.get(&s.common.layer)
        };
        let layer_id = layer.and_then(|l| mappings.layers.ifccad_id(l.handle));
        let c = &s.common;
        // Exact common representation is all-or-nothing. Preserve unavailable
        // source fields in the byte snapshot, with no fabricated native defaults.
        let mut issues = Vec::new();
        let qualified_weight = match c.line_weight {
            LineWeight::ByLayer | LineWeight::ByBlock => true,
            LineWeight::Value(value) if value >= 0 => {
                cad_presentation_convert::lineweight_to_cad(f64::from(value) / 100.0)
                    .is_ok_and(|mapped| !mapped.changed)
            }
            _ => false,
        };
        let appearance = if matches!(
            c.color,
            Color::ByLayer | Color::ByBlock | Color::Rgb { .. } | Color::Index(1..=255)
        ) && qualified_weight
            && c.linetype_scale.is_finite()
            && c.linetype_scale > 0.
            && patterns.resolve(&c.linetype, c.linetype_handle).is_ok()
        {
            let appearance =
                crate::mapping::appearance::from_common(c, patterns, "spline", &mut issues);
            // Omitted attached context is retained by the snapshot; only value
            // approximation prevents exposing the native appearance bundle.
            appearance
                .filter(|_| {
                    !issues
                        .iter()
                        .any(|i| i.action == IfccadDiagnosticAction::Modified)
                })
                .map(|appearance| IfccadOpaqueAppearance {
                    appearance,
                    line_pattern_scale: c.linetype_scale,
                })
        } else {
            None
        };
        if appearance.is_none() || layer_id.is_none() {
            self.report.entries.push(spline_entry(
                record_id,
                record.source_key.clone(),
                IfccadPreservationResult::NativePropertyUnrepresented,
                format!("entity/{}", s.common.handle),
                "unavailable native common values remain in the source snapshot",
            ));
        }
        self.records.push(record);
        Ok(IfccadEntity::Opaque(IfccadOpaqueEntity {
            id,
            preservation_record_id: record_id,
            layer_id,
            appearance,
            visible: !c.invisible,
        }))
    }
    fn record(
        &mut self,
        s: &Spline,
        index: u64,
        id: IfccadPreservationRecordId,
        subject: Option<IfccadPreservationTarget>,
    ) -> Result<IfccadPreservationRecord, IfccadConversionError> {
        let key = format!("{:x}", s.common.handle);
        let bytes = capture_spline(s, index, CODEC_REVISION)?;
        self.report.entries.push(spline_entry(
            id,
            key.clone(),
            IfccadPreservationResult::CapturedTyped,
            format!("entity/{}", s.common.handle),
            "complete interpreted SPLINE source captured",
        ));
        if s.common.raw_record.is_some() {
            self.report.entries.push(spline_entry(
                id,
                key.clone(),
                IfccadPreservationResult::StorageSupplementOmitted,
                format!("entity/{}", s.common.handle),
                "raw source record is not replayed by typed preservation",
            ));
        }
        Ok(IfccadPreservationRecord {
            id,
            source_id: "cad-source-1".into(),
            source_key: key,
            category: IfccadPreservationCategory::Entity,
            role: IfccadPreservationRole::Complete,
            representation: IfccadPreservationRepresentation::CodecTyped,
            subject,
            dependency_coverage: IfccadPreservationDependencyCoverage::Unknown,
            bindings: vec![],
            conditions: vec![],
            payload: IfccadPreservationPayload {
                schema: SPLINE_SCHEMA.into(),
                version: SPLINE_PAYLOAD_VERSION,
                kind: IfccadPreservationPayloadKind::AdapterSnapshot,
                bytes,
            },
        })
    }
    pub fn finish(
        &mut self,
        source: &CadDocument,
        drawing: &mut IfccadDocument,
        mappings: &IfccadMappings,
    ) -> Result<(), IfccadConversionError> {
        if !self.enabled {
            return Ok(());
        }
        // Source membership has already been validated. Unsupported owner contexts
        // retain a detached snapshot while their existing placement loss remains.
        for block in source.block_records.iter() {
            for (index, handle) in block.entity_handles.iter().enumerate() {
                if mappings.entities.ifccad_id(*handle).is_some() {
                    continue;
                }
                if let Some(EntityType::Spline(s)) = source.get_entity(*handle) {
                    let id = drawing.id_counters.allocate_preservation_record_id()?;
                    let record = self.record(s, index as u64, id, None)?;
                    self.records.push(record);
                }
            }
        }
        for record in &mut self.records {
            if let Some(IfccadPreservationTarget::Entity(id)) = record.subject {
                let s =
                    cad_preservation::decode_spline_snapshot(&record.payload.bytes)?.to_source();
                record.conditions = super::conditions::build(drawing, id, record.id);
                super::references::bind(&s, record, drawing, source, mappings)?;
                let reason = if super::conditions::unsupported_common_context(&s) {
                    Some(IfccadPreservationReason::UnsupportedContext)
                } else if record.dependency_coverage
                    != IfccadPreservationDependencyCoverage::Qualified
                {
                    Some(IfccadPreservationReason::UnresolvedReference)
                } else {
                    None
                };
                if let Some(reason) = reason {
                    let mut entry = spline_entry(
                        record.id,
                        record.source_key.clone(),
                        IfccadPreservationResult::RestorationUnavailable,
                        format!("entity/{}", s.common.handle),
                        "source retained; initial restoration context is not qualified",
                    );
                    entry.entity_id = Some(id);
                    entry.reason = Some(reason);
                    self.report.entries.push(entry);
                }
            }
        }
        if !self.records.is_empty() {
            drawing.preservation = Some(IfccadPreservation {
                version: 1,
                sources: vec![IfccadPreservationSource {
                    id: "cad-source-1".into(),
                    provider: "opencadcodec".into(),
                    provider_revision: CODEC_REVISION.into(),
                    origin: IfccadPreservationOrigin::CadDocument,
                    source_version: source.dwg_source_version.map(|v| format!("{v:?}")),
                }],
                records: std::mem::take(&mut self.records),
            });
        }
        Ok(())
    }
}
