use ifccad::*;
#[path = "support/ifccad_preservation.rs"]
mod fixtures;

fn field(bytes: &[u8], key: u64) -> &[u8] {
    let mut d = minicbor::Decoder::new(bytes);
    for _ in 0..d.map().unwrap().unwrap() {
        let k = d.u64().unwrap();
        let start = d.position();
        d.skip().unwrap();
        if k == key {
            return &bytes[start..d.position()];
        }
    }
    panic!("missing field {key}")
}
fn present_appearance() -> Vec<u8> {
    let mut d = fixtures::with_opaque();
    let layer = d.layers[0].id;
    let e = d
        .model
        .entities
        .last_mut()
        .unwrap()
        .as_opaque_mut()
        .unwrap();
    e.layer_id = Some(layer);
    e.appearance = Some(IfccadOpaqueAppearance {
        appearance: IfccadEntityAppearance {
            color: IfccadMode::ByLayer,
            opacity: IfccadMode::ByLayer,
            line_pattern: IfccadMode::ByLayer,
            line_weight: IfccadMode::ByLayer,
        },
        line_pattern_scale: 1.0,
    });
    encode_ifccad_cbor_document(&d).unwrap().into_bytes()
}
fn opaque_modes(bytes: &[u8]) -> &[u8] {
    let entities = field(field(field(bytes, 2), 5), 5);
    let mut d = minicbor::Decoder::new(entities);
    let n = d.array().unwrap().unwrap();
    for _ in 0..n - 1 {
        d.skip().unwrap();
    }
    let start = d.position();
    d.skip().unwrap();
    field(field(field(&entities[start..d.position()], 1), 3), 0)
}
#[test]
fn present_opaque_modes_are_written_explicitly() {
    let bytes = present_appearance();
    let modes = opaque_modes(&bytes);
    assert_eq!(minicbor::Decoder::new(modes).map().unwrap(), Some(4));
}
#[test]
fn present_opaque_modes_cannot_be_missing() {
    let mut bytes = present_appearance();
    let modes = opaque_modes(&bytes);
    let start = modes.as_ptr() as usize - bytes.as_ptr() as usize;
    let len = modes.len();
    bytes.splice(start..start + len, [0xa0]);
    assert!(matches!(
        load_ifccad_cbor_bytes(&bytes, Default::default()),
        Err(IfccadCborReadError::Malformed { .. })
    ));
}
#[test]
fn opaque_payload_and_absent_appearance_survive_exactly() {
    let d = fixtures::with_opaque();
    let encoded = encode_ifccad_cbor_document(&d).unwrap();
    let loaded = load_ifccad_cbor_bytes(encoded.bytes(), Default::default())
        .unwrap()
        .into_document();
    assert_eq!(loaded, d);
    assert_eq!(
        loaded.preservation.unwrap().records[0].payload.bytes,
        [0xff, 0, 3]
    );
    assert!(loaded
        .model
        .entities
        .last()
        .unwrap()
        .as_opaque()
        .unwrap()
        .appearance
        .is_none());
    // bstr(3), ff 00 03; provider bytes are never converted to Base64.
    assert!(encoded.bytes().windows(4).any(|b| b == [0x43, 0xff, 0, 3]));
}
#[test]
fn an_authored_empty_envelope_is_not_none() {
    let mut d = fixtures::base();
    d.preservation = Some(IfccadPreservation {
        version: 1,
        sources: vec![],
        records: vec![],
    });
    let encoded = encode_ifccad_cbor_document(&d).unwrap();
    let loaded = load_ifccad_cbor_bytes(encoded.bytes(), Default::default())
        .unwrap()
        .into_document();
    assert_eq!(loaded.preservation, d.preservation);
}

#[test]
fn numeric_record_order_is_preserved_in_cbor_and_normalized_only_by_json() {
    let mut d = fixtures::with_opaque();
    let p = d.preservation.as_mut().unwrap();
    let template = p.records[0].clone();
    for id in 2..=12 {
        let mut r = template.clone();
        r.id = IfccadPreservationRecordId(id);
        r.subject = None;
        r.source_key = format!("archive-{id}");
        p.records.push(r);
    }
    p.records.reverse();
    d.id_counters.next_preservation_record_id = 13;
    let before = d.clone();
    let out = encode_ifccad_cbor_document(&d).unwrap();
    let loaded = load_ifccad_cbor_bytes(out.bytes(), Default::default())
        .unwrap()
        .into_document();
    assert_eq!(loaded, before);
    assert_eq!(d, before);
    let json = encode_ifccad_document(&loaded).unwrap();
    let json_loaded = load_ifccad_bytes(json.bytes(), Default::default())
        .unwrap()
        .into_document();
    let ids: Vec<_> = json_loaded
        .preservation
        .as_ref()
        .unwrap()
        .records
        .iter()
        .map(|r| r.id.0)
        .collect();
    assert_eq!(ids, (1..=12).collect::<Vec<_>>());
    let second = encode_ifccad_cbor_document(&json_loaded).unwrap();
    assert_eq!(
        load_ifccad_cbor_bytes(second.bytes(), Default::default())
            .unwrap()
            .document(),
        &json_loaded
    );
}
