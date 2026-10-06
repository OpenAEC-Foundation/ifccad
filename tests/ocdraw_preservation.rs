//! Generic preservation tests deliberately have no CAD runtime dependency.
use ocdraw::ocdraw::*;

fn document() -> OcdrawDocument {
    OcdrawBuilder::new(OcdrawBuildOptions::new("generic", "mm"))
        .unwrap()
        .build_document()
        .unwrap()
}

fn source(id: &str) -> OcdrawPreservationSource {
    OcdrawPreservationSource {
        id: id.into(),
        provider: "test-provider".into(),
        provider_revision: "test-revision".into(),
        origin: OcdrawPreservationOrigin::CadDocument,
        source_version: None,
    }
}

fn record(id: u64) -> OcdrawPreservationRecord {
    OcdrawPreservationRecord {
        id: OcdrawPreservationRecordId(id),
        source_id: "source-a".into(),
        source_key: "abc".into(),
        category: OcdrawPreservationCategory::Entity,
        role: OcdrawPreservationRole::Complete,
        representation: OcdrawPreservationRepresentation::CodecTyped,
        subject: None,
        dependency_coverage: OcdrawPreservationDependencyCoverage::Unknown,
        bindings: vec![],
        conditions: vec![],
        payload: OcdrawPreservationPayload {
            schema: "unknown.payload".into(),
            version: 7,
            kind: OcdrawPreservationPayloadKind::AdapterSnapshot,
            bytes: vec![0, 255, 8],
        },
    }
}

fn preservation(records: Vec<OcdrawPreservationRecord>) -> OcdrawPreservation {
    OcdrawPreservation {
        version: 1,
        next_record_id: records.iter().map(|r| r.id.0).max().unwrap_or(0) + 1,
        sources: vec![source("source-a")],
        records,
    }
}

fn has_code(doc: &OcdrawDocument, code: &str) -> bool {
    validate_ocdraw_document(doc)
        .unwrap_err()
        .diagnostics()
        .iter()
        .any(|d| d.code == code)
}

#[test]
fn detached_records_and_missing_soft_targets_remain_storage_valid() {
    let mut doc = document();
    let mut r = record(1);
    r.subject = Some(OcdrawPreservationTarget::Entity(998));
    r.bindings.push(OcdrawPreservationBinding {
        slot: "missing-layer".into(),
        source_key: "xyz".into(),
        target: OcdrawPreservationTarget::Layer(99),
    });
    r.conditions.push(OcdrawPreservationCondition {
        target: OcdrawPreservationTarget::Record(OcdrawPreservationRecordId(999)),
        predicate: "unknown.predicate".into(),
        version: 9,
        baseline: vec![0, 255],
    });
    doc.preservation = Some(preservation(vec![r]));
    validate_ocdraw_document(&doc).unwrap();
    doc.preservation.as_mut().unwrap().records[0].role = OcdrawPreservationRole::Supplement;
    validate_ocdraw_document(&doc).unwrap();
}

#[test]
fn source_namespaces_are_independent_but_source_links_are_hard() {
    let mut doc = document();
    let mut p = preservation(vec![record(1), record(2)]);
    p.sources.push(source("source-b"));
    p.records[1].source_id = "source-b".into();
    doc.preservation = Some(p);
    validate_ocdraw_document(&doc).unwrap();
    doc.preservation.as_mut().unwrap().records[0].source_id = "missing".into();
    assert!(has_code(&doc, "PRESERVATION_SOURCE"));
    doc.preservation.as_mut().unwrap().records[0].source_id = "source-a".into();
    doc.preservation
        .as_mut()
        .unwrap()
        .sources
        .push(source("source-a"));
    assert!(has_code(&doc, "PRESERVATION_SOURCE"));
}

#[test]
fn record_watermark_allocation_is_checked_and_retained_after_deletion() {
    let mut p = preservation(vec![]);
    assert_eq!(
        p.allocate_record_id().unwrap(),
        OcdrawPreservationRecordId(1)
    );
    p.records.push(record(1));
    p.records.clear();
    assert_eq!(
        p.allocate_record_id().unwrap(),
        OcdrawPreservationRecordId(2)
    );
    assert_eq!(p.next_record_id, 3);
    p.next_record_id = u64::MAX;
    assert_eq!(
        p.allocate_record_id(),
        Err(OcdrawPreservationAllocationError::IdExhausted)
    );
    assert_eq!(p.next_record_id, u64::MAX);
    let mut doc = document();
    doc.preservation = Some(p);
    validate_ocdraw_document(&doc).unwrap();
}

#[test]
fn record_ids_are_exact_and_watermarks_must_exceed_them() {
    let mut doc = document();
    doc.preservation = Some(preservation(vec![record(9_007_199_254_740_993)]));
    validate_ocdraw_document(&doc).unwrap();
    doc.preservation.as_mut().unwrap().next_record_id = 9_007_199_254_740_993;
    assert!(has_code(&doc, "PRESERVATION_ID"));
    doc.preservation.as_mut().unwrap().next_record_id += 1;
    doc.preservation
        .as_mut()
        .unwrap()
        .records
        .push(record(9_007_199_254_740_993));
    assert!(has_code(&doc, "PRESERVATION_ID"));
    doc.preservation.as_mut().unwrap().records = vec![record(0)];
    assert!(has_code(&doc, "PRESERVATION_ID"));
}

#[test]
fn record_cycles_do_not_make_storage_invalid() {
    let mut doc = document();
    let mut records = vec![record(1), record(2)];
    for (r, target) in records.iter_mut().zip([2, 1]) {
        r.bindings.push(OcdrawPreservationBinding {
            slot: "other".into(),
            source_key: "other-key".into(),
            target: OcdrawPreservationTarget::Record(OcdrawPreservationRecordId(target)),
        });
    }
    doc.preservation = Some(preservation(records));
    validate_ocdraw_document(&doc).unwrap();
}

#[test]
fn envelope_fields_and_slots_are_validated_without_inspecting_provider_bytes() {
    let mut doc = document();
    let mut p = preservation(vec![record(1)]);
    p.records[0].bindings = vec![
        OcdrawPreservationBinding {
            slot: "same".into(),
            source_key: "a".into(),
            target: OcdrawPreservationTarget::Drawing,
        },
        OcdrawPreservationBinding {
            slot: "same".into(),
            source_key: "b".into(),
            target: OcdrawPreservationTarget::Drawing,
        },
    ];
    doc.preservation = Some(p);
    assert!(has_code(&doc, "PRESERVATION_FIELD"));
    doc.preservation.as_mut().unwrap().records[0]
        .bindings
        .clear();
    doc.preservation.as_mut().unwrap().records[0]
        .payload
        .version = 0;
    assert!(has_code(&doc, "PRESERVATION_FIELD"));
    doc.preservation.as_mut().unwrap().records[0]
        .payload
        .version = 7;
    doc.preservation.as_mut().unwrap().version = 2;
    assert!(has_code(&doc, "PRESERVATION_VERSION"));
}

fn live_document() -> OcdrawDocument {
    let mut doc = document();
    let mut r = record(1);
    r.subject = Some(OcdrawPreservationTarget::Entity(1));
    doc.preservation = Some(preservation(vec![r]));
    doc.opaque_entities.push(DrawingOpaqueEntity {
        id: 1,
        preservation_record_id: OcdrawPreservationRecordId(1),
        layer_id: None,
        appearance: None,
        visible: true,
    });
    doc.next_entity_id = 2;
    doc.scopes[0].entities.push(1);
    doc
}

#[test]
fn opaque_entities_participate_in_identity_and_ownership_without_invented_properties() {
    let mut doc = live_document();
    // Full null-bounds acceptance belongs to the geometry-completeness task.
    if let Err(error) = validate_ocdraw_document(&doc) {
        assert!(
            error.diagnostics().iter().all(|d| d.code == "SCOPE_BOUNDS"),
            "{:?}",
            error.diagnostics()
        );
    }
    doc.scopes[0].entities.clear();
    assert!(has_code(&doc, "ENTITY_OWNERSHIP"));
    doc.scopes[0].entities = vec![1, 1];
    assert!(has_code(&doc, "ENTITY_OWNERSHIP"));
    doc.scopes[0].entities = vec![1];
    doc.next_entity_id = 1;
    assert!(has_code(&doc, "ID_WATERMARK"));
    doc.next_entity_id = 2;
    doc.opaque_entities.push(doc.opaque_entities[0].clone());
    assert!(has_code(&doc, "ENTITY_ID"));
}

#[test]
fn live_opaque_records_and_present_native_properties_are_hard_links() {
    let mut doc = live_document();
    doc.preservation.as_mut().unwrap().records[0].subject =
        Some(OcdrawPreservationTarget::Layer(1));
    assert!(has_code(&doc, "PRESERVATION_LINK"));
    doc.preservation.as_mut().unwrap().records[0].subject =
        Some(OcdrawPreservationTarget::Entity(1));
    doc.preservation.as_mut().unwrap().records[0].role = OcdrawPreservationRole::Supplement;
    assert!(has_code(&doc, "PRESERVATION_LINK"));
    doc.preservation.as_mut().unwrap().records[0].role = OcdrawPreservationRole::Complete;
    doc.opaque_entities[0].layer_id = Some(0);
    assert!(has_code(&doc, "ENTITY_REF"));
    doc.opaque_entities[0].layer_id = None;
    doc.opaque_entities[0].appearance = Some(EntityAppearance {
        opacity: AppearanceSelection::Explicit(f64::NAN),
        ..EntityAppearance::default()
    });
    assert!(has_code(&doc, "APPEARANCE_VALUE"));
    doc.preservation = None;
    assert!(has_code(&doc, "PRESERVATION_LINK"));
}

#[test]
fn unknown_provider_bytes_survive_production_readback_without_source() {
    let mut doc = live_document();
    let record = &mut doc.preservation.as_mut().unwrap().records[0];
    record.payload.bytes = vec![0, 0xff, 0xfe, 1];
    record.conditions.push(OcdrawPreservationCondition {
        target: OcdrawPreservationTarget::Entity(999),
        predicate: "unknown".into(),
        version: 123,
        baseline: vec![0, 0xff, 0xfd],
    });
    let expected = doc.preservation.clone();
    let encoded = encode_ocdraw_document(&doc).unwrap();
    drop(doc);
    let loaded = load_ocdraw_bytes(encoded.bytes()).unwrap();
    assert_eq!(loaded.preservation(), expected.as_ref());
    assert_eq!(loaded.owner_scope_id(1), Some(0));
    assert_eq!(loaded.opaque_entities().len(), 1);
    let mut edited = loaded.into_document();
    edited.opaque_entities[0].visible = false;
    let reloaded = load_ocdraw_bytes(encode_ocdraw_document(&edited).unwrap().bytes()).unwrap();
    assert_eq!(reloaded.preservation(), expected.as_ref());
    assert!(!reloaded.opaque_entities()[0].visible);
}

#[test]
fn opaque_stream_nullable_properties_and_exact_record_ids_survive_json() {
    let mut doc = live_document();
    doc.preservation.as_mut().unwrap().records[0].id =
        OcdrawPreservationRecordId(9_007_199_254_740_993);
    doc.preservation.as_mut().unwrap().next_record_id = 9_007_199_254_740_994;
    doc.opaque_entities[0].preservation_record_id =
        OcdrawPreservationRecordId(9_007_199_254_740_993);
    doc.opaque_entities[0].appearance = Some(EntityAppearance::default());
    let encoded = encode_ocdraw_document(&doc).unwrap();
    let loaded = load_ocdraw_bytes(encoded.bytes()).unwrap();
    assert_eq!(loaded.preservation(), doc.preservation.as_ref());
    assert_eq!(loaded.opaque_entities(), doc.opaque_entities);
    let value = loaded.as_value();
    assert_eq!(value["streams"]["opaqueEntityStream"]["id"][0], 1);
    assert_eq!(
        value["streams"]["opaqueEntityStream"]["layerId"][0],
        serde_json::Value::Null
    );
    assert_eq!(
        value["streams"]["opaqueEntityStream"]["appearance"][0]["opacity"]["mode"],
        "ByLayer"
    );
    assert!(value["streams"]["opaqueEntityStream"]
        .get("owner")
        .is_none());
}

#[test]
fn preservation_physical_rejects_noncanonical_bytes_and_malformed_columns() {
    let encoded = encode_ocdraw_document(&live_document()).unwrap();
    let original: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    for bytes in ["AP8I\n", "AP8I=", "-P8I", "AB==", "AA", "A==="] {
        let mut value = original.clone();
        value["preservation"]["records"][0]["payload"]["bytes"] = bytes.into();
        assert!(
            load_ocdraw_bytes(&serde_json::to_vec(&value).unwrap()).is_err(),
            "{bytes}"
        );
    }
    for field in [
        "id",
        "preservationRecordId",
        "visible",
        "layerId",
        "appearance",
    ] {
        let mut value = original.clone();
        value["streams"]["opaqueEntityStream"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            load_ocdraw_bytes(&serde_json::to_vec(&value).unwrap()).is_err(),
            "{field}"
        );
    }
    let mut value = original.clone();
    value["streams"]["opaqueEntityStream"]["count"] = u64::MAX.into();
    assert!(load_ocdraw_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut value = original.clone();
    value["preservation"]["records"][0]["id"] = serde_json::json!(1.0);
    assert!(load_ocdraw_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut value = original.clone();
    value["streams"]["opaqueEntityStream"]["invented"] = serde_json::json!([null]);
    assert!(load_ocdraw_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut value = original.clone();
    value["preservation"]["version"] = 2.into();
    assert_eq!(
        load_ocdraw_bytes(&serde_json::to_vec(&value).unwrap())
            .unwrap_err()
            .status(),
        OcdrawReadStatus::UnsupportedVersion
    );
    let mut value = original;
    value["preservation"]["version"] = 0.into();
    assert_eq!(
        load_ocdraw_bytes(&serde_json::to_vec(&value).unwrap())
            .unwrap_err()
            .status(),
        OcdrawReadStatus::Invalid
    );
}
