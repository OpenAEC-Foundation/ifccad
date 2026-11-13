use ifccad::*;
pub fn base() -> IfccadDocument {
    load_ifccad_bytes(
        include_bytes!("../../tests/fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document()
}
pub fn with_opaque() -> IfccadDocument {
    let mut d = base();
    let id = d.id_counters.allocate_entity_id().unwrap();
    let record_id = d.id_counters.allocate_preservation_record_id().unwrap();
    d.model
        .entities
        .push(IfccadEntity::Opaque(IfccadOpaqueEntity {
            id,
            preservation_record_id: record_id,
            layer_id: None,
            appearance: None,
            visible: true,
        }));
    d.preservation = Some(IfccadPreservation {
        version: 2,
        sources: vec![IfccadPreservationSource {
            id: "source".into(),
            provider: "unknown".into(),
            provider_revision: "future".into(),
            origin: IfccadPreservationOrigin::CadDocument,
            source_version: None,
        }],
        records: vec![IfccadPreservationRecord {
            id: record_id,
            source_id: "source".into(),
            source_key: "ab".into(),
            category: IfccadPreservationCategory::Entity,
            role: IfccadPreservationRole::Complete,
            representation: IfccadPreservationRepresentation::CodecTyped,
            subject: Some(IfccadPreservationTarget::Entity(id)),
            dependency_coverage: IfccadPreservationDependencyCoverage::Unknown,
            bindings: vec![],
            conditions: vec![],
            storage_backing: None,
            payload: Some(IfccadPreservationPayload {
                schema: "future.spline".into(),
                version: 19,
                kind: IfccadPreservationPayloadKind::AdapterSnapshot,
                bytes: vec![0xff, 0, 3],
            }),
        }],
    });
    d
}

#[allow(dead_code)]
pub fn with_shared_selection() -> IfccadDocument {
    let mut drawing = with_opaque();
    let second_entity = drawing.id_counters.allocate_entity_id().unwrap();
    let second_record = drawing
        .id_counters
        .allocate_preservation_record_id()
        .unwrap();
    let backing_id = drawing
        .id_counters
        .allocate_preservation_record_id()
        .unwrap();
    drawing
        .model
        .entities
        .push(IfccadEntity::Opaque(IfccadOpaqueEntity {
            id: second_entity,
            preservation_record_id: second_record,
            layer_id: None,
            appearance: None,
            visible: true,
        }));
    let preservation = drawing.preservation.as_mut().unwrap();
    let first = &mut preservation.records[0];
    let payload = first.payload.take();
    first.storage_backing = Some(backing_id);
    let mut second = first.clone();
    second.id = second_record;
    second.source_key = "b".into();
    second.subject = Some(IfccadPreservationTarget::Entity(second_entity));
    second.bindings.push(IfccadPreservationBinding {
        slot: "root-linetype".into(),
        source_key: "source-pattern".into(),
        target: IfccadPreservationTarget::LinePattern(IfccadLinePatternId(250)),
    });
    let backing = IfccadPreservationRecord {
        id: backing_id,
        source_id: first.source_id.clone(),
        source_key: "package".into(),
        category: IfccadPreservationCategory::Shared,
        role: IfccadPreservationRole::Shared,
        representation: first.representation,
        subject: None,
        dependency_coverage: first.dependency_coverage,
        bindings: vec![IfccadPreservationBinding {
            slot: "soft-cycle".into(),
            source_key: first.source_key.clone(),
            target: IfccadPreservationTarget::Record(first.id),
        }],
        conditions: vec![],
        payload,
        storage_backing: None,
    };
    preservation.records.extend([second, backing]);
    drawing.id_counters.next_line_pattern_id = 251;
    drawing
}
