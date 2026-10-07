use ocdraw::ifccad::*;
pub fn base() -> IfccadDocument {
    load_ifccad_bytes(
        include_bytes!("../../examples/ifccad/hello-cad.ifcx"),
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
        version: 1,
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
            payload: IfccadPreservationPayload {
                schema: "future.spline".into(),
                version: 19,
                kind: IfccadPreservationPayloadKind::AdapterSnapshot,
                bytes: vec![0xff, 0, 3],
            },
        }],
    });
    d
}
