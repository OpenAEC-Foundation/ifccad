#[path = "support/ifccad_dimension_graphics.rs"]
mod support;
use ifccad::*;
fn field(bytes: &[u8], start: usize, key: u64) -> usize {
    let mut d = minicbor::Decoder::new(&bytes[start..]);
    let n = d.map().unwrap().unwrap();
    for _ in 0..n {
        let k = d.u64().unwrap();
        let pos = start + d.position();
        if k == key {
            return pos;
        }
        d.skip().unwrap();
    }
    panic!("missing {key}")
}
fn graphics_offset(b: &[u8]) -> usize {
    let doc = field(b, 0, 2);
    let model = field(b, doc, 5);
    let list = field(b, model, 5);
    let mut a = minicbor::Decoder::new(&b[list..]);
    a.array().unwrap();
    let e = list + a.position();
    let native = field(b, e, 1);
    let body = field(b, native, 3);
    let dim = field(b, body, 0);
    field(b, dim, 7)
}
fn replacement(b: &[u8], value: &[u8]) -> Vec<u8> {
    let start = graphics_offset(b);
    let mut d = minicbor::Decoder::new(&b[start..]);
    d.skip().unwrap();
    let mut out = b[..start].to_vec();
    out.extend(value);
    out.extend(&b[start + d.position()..]);
    out
}
#[test]
fn graphics_cbor_matches_ifcx_logical_value() {
    let d = support::drawing();
    let c = encode_ifccad_cbor_document(&d).unwrap();
    assert_eq!(
        load_ifccad_cbor_bytes(c.bytes(), Default::default())
            .unwrap()
            .document(),
        &d
    );
    let j = encode_ifccad_document(&d).unwrap();
    assert_eq!(
        load_ifccad_bytes(j.bytes(), Default::default())
            .unwrap()
            .document(),
        load_ifccad_cbor_bytes(c.bytes(), Default::default())
            .unwrap()
            .document()
    );
    let mut d = d;
    for origin in [
        IfccadDimensionGraphicsOrigin::Generated,
        IfccadDimensionGraphicsOrigin::SourceSnapshot,
    ] {
        support::dimension(&mut d).graphics = Some(IfccadDimensionGraphics {
            origin,
            parts: vec![],
        });
        let c = encode_ifccad_cbor_document(&d).unwrap();
        assert_eq!(
            load_ifccad_cbor_bytes(c.bytes(), Default::default())
                .unwrap()
                .document(),
            &d
        );
    }
}
#[test]
fn graphics_cbor_rejects_invalid_contextual_bodies() {
    let b = encode_ifccad_cbor_document(&support::drawing()).unwrap();
    let invalid: Vec<Vec<u8>> = vec![
        vec![0xf6],
        vec![0xa1, 1, 0x80],
        vec![0xa1, 0, 0],
        vec![0xa2, 0, 2, 1, 0x80],
        vec![0xa3, 0, 0, 0, 1, 1, 0x80],
        vec![0xa3, 0, 0, 1, 0x80, 2, 0],
        vec![0xa2, 0, 0, 1, 0x81, 0xa2, 1, 9, 2, 0xa0],
        vec![0xa2, 0, 0, 1, 0x81, 0xa2, 1, 13, 2, 0xa0],
        vec![0xa2, 0, 0, 1, 0x81, 0xa2, 1, 0, 2, 0xa0],
        vec![0xa2, 0, 0, 1, 0x81, 0xa2, 1, 0, 2, 0xf6],
    ];
    for (i, v) in invalid.iter().enumerate() {
        assert!(
            matches!(
                load_ifccad_cbor_bytes(&replacement(b.bytes(), v), Default::default()),
                Err(IfccadCborReadError::Malformed { .. })
            ),
            "invalid physical case {i}"
        );
    }
}
#[test]
fn graphics_cbor_defaults_preserve_float_bits() {
    let mut d = support::drawing();
    let p = &mut support::dimension(&mut d).graphics.as_mut().unwrap().parts[0];
    p.visible = false;
    p.layer_id = Some(0);
    p.appearance.opacity = IfccadMode::Explicit(-0.0);
    let b = encode_ifccad_cbor_document(&d).unwrap();
    let mut r = load_ifccad_cbor_bytes(b.bytes(), Default::default())
        .unwrap()
        .into_document();
    let p = &support::dimension(&mut r).graphics.as_ref().unwrap().parts[0];
    assert!(!p.visible);
    assert_eq!(p.layer_id, Some(0));
    let IfccadMode::Explicit(o) = p.appearance.opacity else {
        panic!()
    };
    assert_eq!(o.to_bits(), (-0.0f64).to_bits());
    let IfccadDimensionGraphicsKind::LineSegment { start, .. } = p.kind else {
        panic!()
    };
    assert_eq!(start[1].to_bits(), (-0.0f64).to_bits());
}
