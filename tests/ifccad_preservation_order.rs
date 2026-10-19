use ifccad::*;
use serde_json::Value;

#[path = "support/ifccad_preservation.rs"]
mod support;

fn archived_records(ids: &[u64]) -> IfccadDocument {
    let mut document = support::with_opaque();
    document.model.entities.retain(|e| e.as_opaque().is_none());
    let preservation = document.preservation.as_mut().unwrap();
    let template = preservation.records[0].clone();
    preservation.records = ids
        .iter()
        .map(|&id| {
            let mut record = template.clone();
            record.id = IfccadPreservationRecordId(id);
            record.subject = None;
            record.source_key = format!("source-{id}");
            record.payload.bytes = id.to_le_bytes().to_vec();
            record
        })
        .collect();
    document.id_counters.next_preservation_record_id = ids.iter().max().unwrap() + 1;
    document
}

// Assemble individually writable records so the reader regression does not
// depend on the multi-record encoder succeeding first.
fn wire_with_records(document: &IfccadDocument) -> Value {
    let mut single = document.clone();
    let records = &document.preservation.as_ref().unwrap().records;
    single.preservation.as_mut().unwrap().records = vec![records[0].clone()];
    let encoded = encode_ifccad_document(&single).unwrap();
    let mut wire: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    let nodes = wire["data"].as_array_mut().unwrap();
    nodes.retain(|node| {
        node["attributes"]
            .get("ifccad::preservationRecord")
            .is_none()
    });
    let collection = nodes
        .iter_mut()
        .find(|node| node["attributes"].get("ifccad::preservation").is_some())
        .unwrap();
    collection["children"] = serde_json::json!({});
    for record in records {
        let key = format!("r{}", record.id.0);
        collection["children"][&key] =
            format!("/cad/d{}/preservation/{key}", document.drawing_id).into();
    }
    for record in records {
        single.preservation.as_mut().unwrap().records = vec![record.clone()];
        let encoded = encode_ifccad_document(&single).unwrap();
        let fragment: Value = serde_json::from_slice(encoded.bytes()).unwrap();
        nodes.push(
            fragment["data"]
                .as_array()
                .unwrap()
                .iter()
                .find(|node| {
                    node["attributes"]
                        .get("ifccad::preservationRecord")
                        .is_some()
                })
                .unwrap()
                .clone(),
        );
    }
    wire
}

// Write reversed object members explicitly: serde_json::Map itself sorts keys.
fn reversed_members(value: &Value) -> String {
    match value {
        Value::Object(map) => format!(
            "{{{}}}",
            map.iter()
                .rev()
                .map(|(key, value)| format!(
                    "{}:{}",
                    serde_json::to_string(key).unwrap(),
                    reversed_members(value)
                ))
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(reversed_members)
                .collect::<Vec<_>>()
                .join(",")
        ),
        _ => serde_json::to_string(value).unwrap(),
    }
}

#[test]
fn reader_canonicalizes_mixed_length_record_ids_without_changing_records_or_graph() {
    let expected_ids = [1, 2, 9, 10, 11, 100, 9007199254740993];
    let document = archived_records(&expected_ids);
    let original = wire_with_records(&document);
    for permutation in 0..3 {
        let mut wire = original.clone();
        let nodes = wire["data"].as_array_mut().unwrap();
        match permutation {
            1 => nodes.reverse(),
            2 => nodes.rotate_left(3),
            _ => {}
        }
        let bytes = if permutation == 1 {
            reversed_members(&wire).into_bytes()
        } else {
            serde_json::to_vec(&wire).unwrap()
        };
        let loaded = load_ifccad_bytes(&bytes, Default::default()).unwrap();
        let records = &loaded.document().preservation.as_ref().unwrap().records;
        assert_eq!(
            records.iter().map(|r| r.id.0).collect::<Vec<_>>(),
            expected_ids
        );
        assert_eq!(loaded.document(), &document);
        assert_eq!(loaded.graph().source_bytes(), bytes);
        validate_ifccad_document(loaded.document()).unwrap();
        let encoded = encode_ifccad_document(loaded.document()).unwrap();
        let reopened = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
        assert_eq!(reopened.document(), &document);
    }
}

#[test]
fn strict_encode_read_reencode_handles_record_count_digit_boundaries() {
    for count in [9, 10, 11, 121] {
        let ids: Vec<_> = (1..=count).collect();
        let expected = archived_records(&ids);
        let mut input = expected.clone();
        input.preservation.as_mut().unwrap().records.reverse();
        let before = input.clone();
        let encoded = encode_ifccad_document(&input).unwrap();
        assert_eq!(
            input, before,
            "encoding must not reorder the caller's records"
        );
        let loaded = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
        assert_eq!(loaded.document(), &expected, "record count {count}");
        let encoded_again = encode_ifccad_document(loaded.document()).unwrap();
        let reopened = load_ifccad_bytes(encoded_again.bytes(), Default::default()).unwrap();
        assert_eq!(reopened.document(), &expected);
    }
}
