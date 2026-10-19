#![cfg(feature = "ifccad-cbor-zstd")]
use ifccad::*;

fn document() -> IfccadDocument {
    load_ifccad_bytes(
        include_bytes!("fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document()
}

#[test]
fn optional_zstd_wraps_exact_standard_cbor_and_defaults_to_level_three() {
    let d = document();
    let raw = encode_ifccad_cbor_document(&d).unwrap();
    assert_eq!(IfccadCborZstdEncodeOptions::default().compression_level, 3);
    let encoded = encode_ifccad_cbor_zstd_document(&d, Default::default()).unwrap();
    assert_eq!(&encoded.bytes()[..4], &[0x28, 0xB5, 0x2F, 0xFD]);
    assert_ne!(
        encoded.bytes()[4] & 4,
        0,
        "writer includes the frame checksum"
    );
    assert_eq!(
        zstd::stream::decode_all(encoded.bytes()).unwrap(),
        raw.bytes()
    );
    assert_eq!(
        load_ifccad_cbor_zstd_bytes(encoded.bytes(), Default::default())
            .unwrap()
            .document(),
        &d
    );
    assert!(load_ifccad_cbor_bytes(encoded.bytes(), Default::default()).is_err());
    for compression_level in [1, 3, 9] {
        let out =
            encode_ifccad_cbor_zstd_document(&d, IfccadCborZstdEncodeOptions { compression_level })
                .unwrap();
        assert_eq!(zstd::stream::decode_all(out.bytes()).unwrap(), raw.bytes());
    }
    for compression_level in [-1, 0, 23] {
        assert!(matches!(
            encode_ifccad_cbor_zstd_document(&d, IfccadCborZstdEncodeOptions { compression_level }),
            Err(IfccadCborZstdEncodeError::InvalidOptions { .. })
        ));
    }
}

fn frame(raw: &[u8], with_size: bool) -> Vec<u8> {
    use std::io::Write;
    let mut encoder = zstd::stream::write::Encoder::new(Vec::new(), 3).unwrap();
    encoder.include_checksum(true).unwrap();
    encoder.include_contentsize(with_size).unwrap();
    if with_size {
        encoder
            .set_pledged_src_size(Some(raw.len() as u64))
            .unwrap();
    }
    encoder.write_all(raw).unwrap();
    encoder.finish().unwrap()
}

#[test]
fn transport_rejects_corruption_truncation_extra_frames_and_wrong_header() {
    let raw = encode_ifccad_cbor_document(&document()).unwrap();
    let original = frame(raw.bytes(), true);
    assert!(load_ifccad_cbor_zstd_bytes(&frame(raw.bytes(), false), Default::default()).is_ok());
    for cut in [0, 4, 5, original.len() - 1] {
        assert!(load_ifccad_cbor_zstd_bytes(&original[..cut], Default::default()).is_err());
    }
    let mut bad = original.clone();
    let last = bad.len() - 1;
    bad[last] ^= 0x80;
    assert!(matches!(
        load_ifccad_cbor_zstd_bytes(&bad, Default::default()),
        Err(IfccadCborZstdReadError::Decompression(_))
    ));
    for suffix in [&[0u8][..], original.as_slice()] {
        let mut extra = original.clone();
        extra.extend_from_slice(suffix);
        assert!(matches!(
            load_ifccad_cbor_zstd_bytes(&extra, Default::default()),
            Err(IfccadCborZstdReadError::Malformed { .. })
        ));
    }
    for (index, value) in [
        (0, 0x50),
        (4, original[4] & !4),
        (4, original[4] | 1),
        (4, original[4] | 8),
    ] {
        let mut bad = original.clone();
        bad[index] = value;
        assert!(matches!(
            load_ifccad_cbor_zstd_bytes(&bad, Default::default()),
            Err(IfccadCborZstdReadError::Malformed { .. })
        ));
    }
    assert!(matches!(
        load_ifccad_cbor_zstd_bytes(&frame(&original, true), Default::default()),
        Err(IfccadCborZstdReadError::Cbor(
            IfccadCborReadError::Malformed { .. }
        ))
    ));
}

#[test]
fn all_candidate_cbor_contract_cases_keep_their_validity_inside_zstd() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("conformance/next/ifccad-cbor");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("cases.json")).unwrap()).unwrap();
    for case in manifest["cases"].as_array().unwrap() {
        let bytes = std::fs::read(root.join(case["file"].as_str().unwrap())).unwrap();
        let raw = load_ifccad_cbor_bytes(&bytes, Default::default());
        let wrapped = load_ifccad_cbor_zstd_bytes(&frame(&bytes, true), Default::default());
        match (raw, wrapped) {
            (Ok(a), Ok(b)) => {
                assert_eq!(a.document(), b.document());
                let encoded =
                    encode_ifccad_cbor_zstd_document(a.document(), Default::default()).unwrap();
                let repeated = encode_ifccad_cbor_document(
                    load_ifccad_cbor_zstd_bytes(encoded.bytes(), Default::default())
                        .unwrap()
                        .document(),
                )
                .unwrap();
                assert_eq!(
                    zstd::stream::decode_all(encoded.bytes()).unwrap(),
                    repeated.bytes()
                );
            }
            (Err(a), Err(IfccadCborZstdReadError::Cbor(b))) => {
                assert_eq!(a.to_string(), b.to_string())
            }
            (a, b) => panic!("changed case validity: {}: {a:?} / {b:?}", case["file"]),
        }
    }
}

#[test]
fn compressed_output_and_window_limits_apply_before_unbounded_allocation() {
    let raw = encode_ifccad_cbor_document(&document()).unwrap();
    let encoded = frame(raw.bytes(), true);
    let mut options = IfccadCborZstdReadOptions::default();
    options.limits.max_compressed_bytes = encoded.len() as u64 - 1;
    assert!(matches!(
        load_ifccad_cbor_zstd_bytes(&encoded, options),
        Err(IfccadCborZstdReadError::ResourceLimit {
            limit: "compressed_bytes",
            ..
        })
    ));
    options = Default::default();
    options.cbor.limits.max_input_bytes = raw.bytes().len() as u64;
    assert!(load_ifccad_cbor_zstd_bytes(&encoded, options).is_ok());
    options.cbor.limits.max_input_bytes -= 1;
    for encoded in [encoded, frame(raw.bytes(), false)] {
        assert!(matches!(
            load_ifccad_cbor_zstd_bytes(&encoded, options),
            Err(IfccadCborZstdReadError::ResourceLimit {
                limit: "decompressed_bytes",
                ..
            })
        ));
    }
    let mut huge = vec![0x28, 0xB5, 0x2F, 0xFD, 0xE4];
    huge.extend_from_slice(&u64::MAX.to_le_bytes());
    huge.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0]);
    // u64::MAX is Zstd's unknown-size sentinel, not a valid enormous size.
    assert!(matches!(
        load_ifccad_cbor_zstd_bytes(&huge, Default::default()),
        Err(IfccadCborZstdReadError::Malformed { .. })
    ));
    huge[5..13].copy_from_slice(&(1u64 << 40).to_le_bytes());
    assert!(matches!(
        load_ifccad_cbor_zstd_bytes(&huge, Default::default()),
        Err(IfccadCborZstdReadError::ResourceLimit {
            limit: "decompressed_bytes",
            ..
        })
    ));
    let huge_window = [0x28, 0xB5, 0x2F, 0xFD, 4, 0xA8, 1, 0, 0, 0, 0, 0, 0];
    assert!(matches!(
        load_ifccad_cbor_zstd_bytes(&huge_window, Default::default()),
        Err(IfccadCborZstdReadError::ResourceLimit {
            limit: "window_bytes",
            ..
        })
    ));
    for max_window_log in [0, 9, 32, u32::MAX] {
        let mut options = IfccadCborZstdReadOptions::default();
        options.limits.max_window_log = max_window_log;
        assert!(matches!(
            load_ifccad_cbor_zstd_bytes(&[], options),
            Err(IfccadCborZstdReadError::InvalidOptions { .. })
        ));
    }
    let mut options = IfccadCborZstdReadOptions::default();
    options.cbor.limits.max_input_bytes = u64::MAX;
    assert!(matches!(
        load_ifccad_cbor_zstd_bytes(&[], options),
        Err(IfccadCborZstdReadError::InvalidOptions { .. })
    ));
}

#[test]
fn file_io_is_bounded_create_new_and_keeps_error_phases() {
    let root = std::env::temp_dir().join(format!(
        "ifccad-zstd-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    let file = root.join("drawing.ifccad.cbor.zst");
    let d = document();
    let out = encode_ifccad_cbor_zstd_document(&d, Default::default()).unwrap();
    out.write_file(&file).unwrap();
    assert_eq!(
        load_ifccad_cbor_zstd_file(&file, Default::default())
            .unwrap()
            .document(),
        &d
    );
    assert!(out.write_file(&file).is_err());
    assert_eq!(std::fs::read(&file).unwrap(), out.bytes());
    let mut options = IfccadCborZstdReadOptions::default();
    options.limits.max_compressed_bytes = 4;
    assert!(matches!(
        load_ifccad_cbor_zstd_file(&file, options),
        Err(IfccadCborZstdOpenError::Read(
            IfccadCborZstdReadError::ResourceLimit { .. }
        ))
    ));
    assert!(matches!(
        load_ifccad_cbor_zstd_file(root.join("missing"), Default::default()),
        Err(IfccadCborZstdOpenError::Io(_))
    ));
    std::fs::remove_file(file).unwrap();
    std::fs::remove_dir(root).unwrap();
}

#[test]
fn invalid_document_error_retains_its_original_report() {
    let mut d = document();
    d.layers.clear();
    let error = encode_ifccad_cbor_zstd_document(&d, Default::default()).unwrap_err();
    assert!(matches!(
        error,
        IfccadCborZstdEncodeError::Cbor(IfccadCborEncodeError::InvalidDocument(_))
    ));
    assert!(error.report().is_some());
}

#[test]
fn independent_transport_fixtures_obey_the_published_contract() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("conformance/next/ifccad-cbor-zstd");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("cases.json")).unwrap()).unwrap();
    for case in manifest["cases"].as_array().unwrap() {
        let bytes = std::fs::read(root.join(case["file"].as_str().unwrap())).unwrap();
        let result = load_ifccad_cbor_zstd_bytes(&bytes, Default::default());
        if case["valid"] == true {
            let loaded = result.unwrap();
            let raw = std::fs::read(root.join(case["payload"].as_str().unwrap())).unwrap();
            let expected = load_ifccad_cbor_bytes(&raw, Default::default()).unwrap();
            // Re-encoding compares float bits as well as the logical content.
            assert_eq!(
                encode_ifccad_cbor_document(loaded.document()).unwrap(),
                encode_ifccad_cbor_document(expected.document()).unwrap()
            );
        } else {
            let class = match result.unwrap_err() {
                IfccadCborZstdReadError::Malformed { .. } => "Malformed",
                IfccadCborZstdReadError::Decompression(_) => "Decompression",
                IfccadCborZstdReadError::Cbor(IfccadCborReadError::UnsupportedRevision {
                    ..
                }) => "Cbor.UnsupportedRevision",
                other => panic!("unexpected error for {}: {other:?}", case["file"]),
            };
            assert_eq!(case["errorClass"], class, "{}", case["file"]);
        }
    }
}
