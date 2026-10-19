use ocdraw::ifccad::*;
fn encoded() -> Vec<u8> {
    let d = load_ifccad_bytes(
        include_bytes!("fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap();
    encode_ifccad_cbor_document(d.document())
        .unwrap()
        .into_bytes()
}
#[test]
fn all_proper_prefixes_and_trailing_bytes_are_rejected() {
    let bytes = encoded();
    for n in 0..bytes.len() {
        assert!(
            load_ifccad_cbor_bytes(&bytes[..n], Default::default()).is_err(),
            "prefix {n}"
        );
    }
    let mut tail = bytes;
    tail.push(0);
    assert!(load_ifccad_cbor_bytes(&tail, Default::default()).is_err());
}
#[test]
fn limits_apply_at_the_boundary_before_allocation() {
    let bytes = encoded();
    let mut limits = IfccadCborReadLimits {
        max_input_bytes: bytes.len() as u64,
        ..Default::default()
    };
    assert!(load_ifccad_cbor_bytes(&bytes, IfccadCborReadOptions { limits }).is_ok());
    limits.max_input_bytes -= 1;
    assert!(matches!(
        load_ifccad_cbor_bytes(&bytes, IfccadCborReadOptions { limits }),
        Err(IfccadCborReadError::ResourceLimit { .. })
    ));
    for options in [
        IfccadCborReadLimits {
            max_depth: 1,
            ..Default::default()
        },
        IfccadCborReadLimits {
            max_total_items: 2,
            ..Default::default()
        },
        IfccadCborReadLimits {
            max_container_entries: 2,
            ..Default::default()
        },
    ] {
        assert!(matches!(
            load_ifccad_cbor_bytes(&bytes, IfccadCborReadOptions { limits: options }),
            Err(IfccadCborReadError::ResourceLimit { .. })
        ));
    }
    assert!(load_ifccad_cbor_bytes(
        &[0x9b, 255, 255, 255, 255, 255, 255, 255, 255],
        Default::default()
    )
    .is_err());
}
#[test]
fn forbidden_tokens_unknown_keys_and_duplicates_are_hard_errors() {
    for bytes in [
        &[0xf6][..],
        &[0xf7],
        &[0xc0, 0],
        &[0x9f, 0xff],
        &[0xfa, 0, 0, 0, 0],
        &[0xf9, 0, 0],
        &[0xfb, 0x7f, 0xf0, 0, 0, 0, 0, 0, 0],
        &[0x61, 0xff],
    ] {
        assert!(
            matches!(
                load_ifccad_cbor_bytes(bytes, Default::default()),
                Err(IfccadCborReadError::Malformed { .. })
            ),
            "{bytes:?}"
        );
    }
    // Root duplicate/unknown numeric key. Duplicate values do not make it legal.
    for bytes in [
        &b"\xa4\x00\x6bIFCCAD-CBOR\x01\x01\x02\xa0\x01\x01"[..],
        &b"\xa4\x00\x6bIFCCAD-CBOR\x01\x01\x02\xa0\x03\x00"[..],
    ] {
        assert!(matches!(
            load_ifccad_cbor_bytes(bytes, Default::default()),
            Err(IfccadCborReadError::Malformed { .. })
        ));
    }
}
