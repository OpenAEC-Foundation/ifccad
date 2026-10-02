use ocdraw::ocdraw::{load_ocdraw_bytes, OcdrawReadStatus};
use serde_json::{json, Value};
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../conformance/next/ocdraw/valid/ordered-scopes.ocdraw.json"
    ))
    .unwrap()
}
#[test]
fn scope_lists_define_owner_and_order_independent_of_stream_row_and_id_order() {
    let value = fixture();
    let read = load_ocdraw_bytes(&serde_json::to_vec(&value).unwrap());
    assert_eq!(
        read.as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Valid,
        "{:?}",
        read.as_ref()
            .err()
            .map(|e| e.diagnostics())
            .unwrap_or_default()
    );
    let drawing = read.as_ref().ok().unwrap();
    assert_eq!(drawing.scopes()[0].entities, vec![4, 1, 6]);
    assert_eq!(drawing.scopes()[1].entities, vec![2, 7]);
    assert_eq!(drawing.scopes()[2].entities, vec![3]);
    assert_eq!(drawing.scopes()[3].entities, vec![5]);
    assert!(drawing.scopes()[4].entities.is_empty());
    assert_eq!(
        drawing
            .geometric_entities()
            .iter()
            .map(|e| e.id())
            .collect::<Vec<_>>(),
        vec![4, 1, 6, 2, 7, 3, 5]
    );
    for (owner, ids) in [
        (0, vec![4, 1, 6]),
        (1, vec![2, 7]),
        (2, vec![3]),
        (3, vec![5]),
    ] {
        for id in ids {
            assert_eq!(drawing.owner_scope_id(id), Some(owner));
        }
    }
    assert_eq!(drawing.owner_scope_id(99), None);
}
#[test]
fn moving_an_entity_changes_owner_without_changing_its_id_or_stream_record() {
    let mut value = fixture();
    let records = value["streams"].clone();
    value["scopes"][0]["entities"] = json!([4, 6]);
    value["scopes"][1]["entities"] = json!([1, 2, 7]);
    let read = load_ocdraw_bytes(&serde_json::to_vec(&value).unwrap());
    assert_eq!(
        read.as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Valid,
        "{:?}",
        read.as_ref()
            .err()
            .map(|e| e.diagnostics())
            .unwrap_or_default()
    );
    let drawing = read.as_ref().ok().unwrap();
    assert_eq!(drawing.owner_scope_id(1), Some(1));
    assert_eq!(drawing.scopes()[1].entities, vec![1, 2, 7]);
    assert_eq!(drawing.as_value()["streams"], records);
}
#[test]
fn invalid_membership_reports_the_exact_scope_entry() {
    for (ids, code, location) in [
        (vec![4, 99, 6], "ENTITY_REFERENCE", "/scopes/0/entities/1"),
        (vec![4, 1, 1, 6], "ENTITY_OWNERSHIP", "/scopes/0/entities/2"),
    ] {
        let mut value = fixture();
        value["scopes"][0]["entities"] = json!(ids);
        let read = load_ocdraw_bytes(&serde_json::to_vec(&value).unwrap());
        assert_eq!(
            read.as_ref()
                .map(|_| OcdrawReadStatus::Valid)
                .unwrap_or_else(|e| e.status()),
            OcdrawReadStatus::Invalid
        );
        assert!(
            read.as_ref()
                .err()
                .map(|e| e.diagnostics())
                .unwrap_or_default()
                .iter()
                .any(|d| d.code == code && d.location == location),
            "{:?}",
            read.as_ref()
                .err()
                .map(|e| e.diagnostics())
                .unwrap_or_default()
        );
    }
}
