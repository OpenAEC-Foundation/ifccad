use ifccad::*;
#[test]
fn opaque_identity_and_allocation_are_independent() {
    let entity = IfccadEntity::Opaque(IfccadOpaqueEntity {
        id: 9007199254740993,
        preservation_record_id: IfccadPreservationRecordId(7),
        layer_id: None,
        appearance: None,
        visible: true,
    });
    assert_eq!(entity.id(), 9007199254740993);
    assert!(entity.as_native().is_none());
    let mut ids = IfccadIdCounters {
        next_symbol_source_id: 1,
        next_preservation_record_id: u64::MAX,
        ..Default::default()
    };
    let before = ids;
    assert!(ids.allocate_preservation_record_id().is_err());
    assert_eq!(ids, before);
    ids.next_preservation_record_id = 9007199254740993;
    assert_eq!(
        ids.allocate_preservation_record_id().unwrap().0,
        9007199254740993
    );
}

#[path = "support/ifccad_preservation.rs"]
mod support;
use support::with_opaque;

#[test]
fn shared_selection_ifcx_roundtrip_and_structural_rejections() {
    let original = support::with_shared_selection();
    let output = encode_ifccad_document(&original).unwrap();
    let read = load_ifccad_bytes(output.bytes(), Default::default()).unwrap();
    assert_eq!(read.document(), &original);
    for mutation in 0..7 {
        let mut drawing = original.clone();
        let envelope = drawing.preservation.as_mut().unwrap();
        let records = &mut envelope.records;
        match mutation {
            0 => {
                records.pop();
            }
            1 => {
                records[0].storage_backing = Some(records[0].id);
            }
            2 => {
                records[2].role = IfccadPreservationRole::Complete;
            }
            3 => {
                records[2].storage_backing = Some(records[0].id);
            }
            4 => {
                records[0].payload = records[2].payload.clone();
            }
            5 => {
                records[0].storage_backing = None;
            }
            _ => {
                envelope.version = 1;
            }
        }
        assert!(
            encode_ifccad_document(&drawing).is_err(),
            "mutation {mutation}"
        );
    }
}

#[test]
fn strict_reader_checks_missing_preservation_target_watermark() {
    let encoded = encode_ifccad_document(&with_opaque()).unwrap();
    let mut wire: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    let nodes = wire["data"].as_array_mut().unwrap();
    let record = nodes
        .iter_mut()
        .find_map(|node| node["attributes"].get_mut("ifccad::preservationRecord"))
        .unwrap();
    record["bindings"] = serde_json::json!([{
        "slot": "root-linetype",
        "sourceKey": "source-pattern",
        "target": {"role": "linePattern", "path": "/cad/d1/linePattern/250"}
    }]);
    let drawing = nodes
        .iter_mut()
        .find_map(|node| node["attributes"].get_mut("ifccad::drawing"))
        .unwrap();
    drawing["nextLinePatternId"] = 250.into();
    let bytes = serde_json::to_vec(&wire).unwrap();
    assert!(load_ifccad_bytes(&bytes, Default::default()).is_err());
    let drawing = wire["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find_map(|node| node["attributes"].get_mut("ifccad::drawing"))
        .unwrap();
    drawing["nextLinePatternId"] = 251.into();
    let bytes = serde_json::to_vec(&wire).unwrap();
    let mut reopened = load_ifccad_bytes(&bytes, Default::default())
        .unwrap()
        .into_document();
    assert_eq!(
        reopened.id_counters.allocate_line_pattern_id().unwrap().0,
        251
    );
}

#[test]
fn protecting_typed_targets_is_domain_local_and_atomic() {
    use IfccadPreservationTarget as Target;
    let mut drawing = support::with_shared_selection();
    let preservation = drawing.preservation.as_mut().unwrap();
    let targets = [
        Target::Entity(201),
        Target::Layer(202),
        Target::LinePattern(IfccadLinePatternId(203)),
        Target::Layout(204),
        Target::BlockDefinition(205),
        Target::Record(IfccadPreservationRecordId(206)),
    ];
    for (index, target) in targets.into_iter().enumerate() {
        preservation.records[0]
            .conditions
            .push(IfccadPreservationCondition {
                target,
                predicate: format!("future-{index}"),
                version: 1,
                baseline: vec![],
            });
    }
    let mut counters = IfccadIdCounters::default();
    counters.protect_preservation_targets(preservation).unwrap();
    assert_eq!(counters.next_entity_id, 202);
    assert_eq!(counters.next_layer_id, 203);
    assert_eq!(counters.next_line_pattern_id, 251);
    assert_eq!(counters.next_layout_id, 205);
    assert_eq!(counters.next_block_id, 206);
    assert_eq!(counters.next_preservation_record_id, 207);
    assert_eq!(counters.next_text_style_id, 1);
    let before = counters;
    preservation.records[0].conditions[0].target = Target::Layer(10_000);
    preservation.records[0].conditions[1].target = Target::Entity(u64::MAX);
    assert!(counters.protect_preservation_targets(preservation).is_err());
    assert_eq!(counters, before);
}
#[test]
fn hard_record_subject_layer_and_source_links_are_checked() {
    let original = with_opaque();
    validate_ifccad_document(&original).unwrap();
    for mutation in 0..6 {
        let mut d = original.clone();
        match mutation {
            0 => d.preservation = None,
            1 => d.preservation.as_mut().unwrap().records[0].subject = None,
            2 => d.preservation.as_mut().unwrap().records[0].source_id = "absent".into(),
            3 => {
                d.model
                    .entities
                    .last_mut()
                    .unwrap()
                    .as_opaque_mut()
                    .unwrap()
                    .layer_id = Some(u64::MAX)
            }
            4 => {
                let r = d.preservation.as_ref().unwrap().records[0].clone();
                d.preservation.as_mut().unwrap().records.push(r);
            }
            _ => d
                .model
                .entities
                .push(d.model.entities.last().unwrap().clone()),
        }
        assert!(validate_ifccad_document(&d).is_err(), "mutation {mutation}");
    }
}
#[test]
fn unknown_provider_missing_soft_targets_and_record_cycles_remain_transportable() {
    let mut d = with_opaque();
    let r = &mut d.preservation.as_mut().unwrap().records[0];
    r.bindings = vec![
        IfccadPreservationBinding {
            slot: "future".into(),
            source_key: "cd".into(),
            target: IfccadPreservationTarget::Entity(250),
        },
        IfccadPreservationBinding {
            slot: "self".into(),
            source_key: "ab".into(),
            target: IfccadPreservationTarget::Record(r.id),
        },
    ];
    r.conditions.push(IfccadPreservationCondition {
        target: IfccadPreservationTarget::Record(r.id),
        predicate: "future".into(),
        version: 19,
        baseline: vec![0xff],
    });
    d.id_counters.next_entity_id = 251;
    validate_ifccad_document(&d).unwrap();
}

#[test]
fn strict_storage_retains_unknown_payload_bytes_and_reserved_record_counter() {
    let mut d = with_opaque();
    d.id_counters.next_preservation_record_id = 9007199254740994;
    let bytes = encode_ifccad_document(&d).unwrap();
    let reopened = load_ifccad_bytes(bytes.bytes(), Default::default())
        .unwrap()
        .into_document();
    assert_eq!(reopened, d);
    let mut wire: serde_json::Value = serde_json::from_slice(bytes.bytes()).unwrap();
    let record = wire["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"].get("ifccad::preservationRecord").is_some())
        .unwrap();
    record["attributes"]["ifccad::preservationRecord"]["payload"]["bytes"] = "/wAD=".into();
    assert!(load_ifccad_bytes(&serde_json::to_vec(&wire).unwrap(), Default::default()).is_err());
}

#[test]
fn closed_wire_rejects_unknown_core_fields_null_optionals_and_wrong_domains() {
    let bytes = encode_ifccad_document(&with_opaque()).unwrap();
    let original: serde_json::Value = serde_json::from_slice(bytes.bytes()).unwrap();
    for mutation in 0..13 {
        let mut v = original.clone();
        let nodes = v["data"].as_array_mut().unwrap();
        match mutation {
            0 => {
                nodes
                    .iter_mut()
                    .find(|n| n["attributes"].get("ifccad::drawing").is_some())
                    .unwrap()["attributes"]["ifccad::drawing"]["unknown"] = true.into()
            }
            1 => {
                nodes
                    .iter_mut()
                    .find(|n| n["attributes"].get("ifccad::preservation").is_some())
                    .unwrap()["attributes"]["ifccad::preservation"]["sources"][0]["sourceVersion"] =
                    serde_json::Value::Null
            }
            2 => {
                nodes
                    .iter_mut()
                    .find(|n| n["attributes"].get("ifccad::preservationRecord").is_some())
                    .unwrap()["attributes"]["ifccad::preservationRecord"]["unknown"] = true.into()
            }
            3 => {
                nodes
                    .iter_mut()
                    .find(|n| n["attributes"].get("ifccad::opaqueEntity").is_some())
                    .unwrap()["attributes"]["ifccad::opaqueEntity"]["nativeLayer"] =
                    serde_json::Value::Null
            }
            4 => {
                nodes
                    .iter_mut()
                    .find(|n| n["attributes"].get("ifccad::opaqueEntity").is_some())
                    .unwrap()["attributes"]["ifccad::entity"] = serde_json::json!({})
            }
            5 => {
                nodes
                    .iter_mut()
                    .find(|n| n["attributes"].get("ifccad::preservationRecord").is_some())
                    .unwrap()["attributes"]["ifccad::preservationRecord"]["bindings"] = serde_json::json!([{ "slot":"x","sourceKey":"k","target":{"role":"layout","path":"/cad/d2/layout/1"}}])
            }
            6 => {
                nodes
                    .iter_mut()
                    .find(|n| n["attributes"].get("ifccad::preservationRecord").is_some())
                    .unwrap()["attributes"]["ifccad::preservationRecord"]["id"] = 1.into()
            }
            7 => {
                nodes
                    .iter_mut()
                    .find(|n| n["attributes"].get("ifccad::drawing").is_some())
                    .unwrap()["attributes"]["ifccad::drawing"]
                    .as_object_mut()
                    .unwrap()
                    .remove("nextPreservationRecordId");
            }
            8 => {
                nodes
                    .iter_mut()
                    .find(|n| n["attributes"].get("ifccad::drawing").is_some())
                    .unwrap()["attributes"]["ifccad::drawing"]["nextPreservationRecordId"] =
                    serde_json::json!(2.0)
            }
            9 => {
                nodes
                    .iter_mut()
                    .find(|n| n["attributes"].get("ifccad::preservationRecord").is_some())
                    .unwrap()["attributes"]["ifccad::preservationRecord"]["bindings"] = serde_json::json!([{ "slot":"x","sourceKey":"k","target":{"role":"entity","path":"/cad/d1/layer/1"}}])
            }
            10 => {
                nodes
                    .iter_mut()
                    .find(|n| n["attributes"].get("ifccad::preservationRecord").is_some())
                    .unwrap()["attributes"]["ifccad::preservationRecord"]["subject"] =
                    serde_json::Value::Null
            }
            11 => {
                nodes
                    .iter_mut()
                    .find(|n| n["attributes"].get("ifccad::opaqueEntity").is_some())
                    .unwrap()["attributes"]["ifccad::future"] = true.into()
            }
            _ => {
                nodes
                    .iter_mut()
                    .find(|n| n["attributes"].get("ifccad::preservationRecord").is_some())
                    .unwrap()["attributes"]["ifccad::preservationRecord"]
                    .as_object_mut()
                    .unwrap()
                    .remove("bindings");
            }
        }
        assert!(
            load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).is_err(),
            "mutation {mutation}"
        );
    }
}

#[test]
fn old_incomplete_inline_profile_is_rejected_and_deleted_records_keep_counter() {
    let d = support::base();
    let b = encode_ifccad_document(&d).unwrap();
    let mut v: serde_json::Value = serde_json::from_slice(b.bytes()).unwrap();
    let module: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/ifccad/ifccad-profile-0.10.0.ifcx")).unwrap();
    v["schemas"] = module["schemas"].clone();
    v["schemas"]["ifccad::drawing"]["value"]["objectRestrictions"]["values"]
        .as_object_mut()
        .unwrap()
        .remove("nextPreservationRecordId");
    assert!(load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).is_err());
    let mut d = with_opaque();
    let watermark = d.id_counters.next_preservation_record_id;
    d.model.entities.retain(|e| e.as_opaque().is_none());
    d.preservation = None;
    let b = encode_ifccad_document(&d).unwrap();
    let mut d = load_ifccad_bytes(b.bytes(), Default::default())
        .unwrap()
        .into_document();
    assert_eq!(
        d.id_counters.allocate_preservation_record_id().unwrap().0,
        watermark
    );
}

#[test]
fn record_zero_is_rejected_in_allocation_and_soft_target_domains() {
    let mut ids = IfccadIdCounters {
        next_symbol_source_id: 1,
        next_preservation_record_id: 0,
        ..Default::default()
    };
    let before = ids;
    assert!(ids.allocate_preservation_record_id().is_err());
    assert_eq!(ids, before);
    let mut d = with_opaque();
    d.preservation.as_mut().unwrap().records[0]
        .bindings
        .push(IfccadPreservationBinding {
            slot: "future".into(),
            source_key: "x".into(),
            target: IfccadPreservationTarget::Record(IfccadPreservationRecordId(0)),
        });
    assert!(validate_ifccad_document(&d).is_err());
}

#[test]
fn preservation_nodes_reject_known_cad_attributes_for_another_node_role() {
    let bytes = encode_ifccad_document(&with_opaque()).unwrap();
    let original: serde_json::Value = serde_json::from_slice(bytes.bytes()).unwrap();
    for role in ["ifccad::preservation", "ifccad::preservationRecord"] {
        let mut v = original.clone();
        let node = v["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["attributes"].get(role).is_some())
            .unwrap();
        node["attributes"]["ifccad::geom::circle"] = serde_json::json!({"radius":2.});
        assert!(
            load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).is_err(),
            "{role}"
        );
    }
}
