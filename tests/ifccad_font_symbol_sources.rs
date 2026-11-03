use ifccad::line_pattern::*;
use ifccad::*;

#[test]
fn shx_wire_closure_and_internal_refs_remain_strict() {
    let wire = include_str!("../conformance/next/ifccad/valid/font-symbol-source-requests.ifcx");
    let duplicate = wire.replacen(
        "\"cadFileName\":",
        "\"cadFileName\": \"ignored.shx\", \"cadFileName\":",
        1,
    );
    assert!(load_ifccad_bytes(duplicate.as_bytes(), Default::default()).is_err());
    let mut document = load_ifccad_bytes(wire.as_bytes(), Default::default())
        .unwrap()
        .into_document();
    document.id_counters.next_line_pattern_id = 2;
    document.line_patterns.push(IfccadLinePattern {
        id: IfccadLinePatternId(1),
        name: "Reference check".into(),
        description: None,
        pattern: vec![LinePatternElement {
            length: 10.,
            marker: Some(IfccadLinePatternMarker::Symbol {
                symbol_source_id: IfccadSymbolSourceId(0),
                shape_number: 42,
                transform: Default::default(),
            }),
        }],
    });
    assert!(validate_ifccad_document(&document).is_ok());
    document
        .symbol_sources
        .retain(|s| s.id != IfccadSymbolSourceId(0));
    assert!(validate_ifccad_document(&document).is_err());
}

#[test]
fn composition_replaces_whole_source_without_rebasing() {
    let mut graph: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../conformance/next/ifccad/valid/font-symbol-source-requests.ifcx"
    ))
    .unwrap();
    graph["data"].as_array_mut().unwrap().push(serde_json::json!({
        "path":"/cad/d1/symbolSource/0", "attributes": {
            "ifccad::symbolSource":{"kind":"shx","fileName":"missing.shx","cadFileName":"../other/missing.shx"}
        }
    }));
    let bytes = serde_json::to_vec(&graph).unwrap();
    let loaded = load_ifccad_bytes(&bytes, Default::default()).unwrap();
    assert_eq!(
        loaded.document().symbol_sources[0]
            .source
            .effective_cad_file_name(),
        "../other/missing.shx"
    );
    assert_eq!(
        loaded.document().symbol_sources[1]
            .source
            .effective_cad_file_name(),
        "missing.shx"
    );
}

#[test]
fn native_requests_roundtrip_with_two_annotation_scales() {
    let original = load_ifccad_bytes(
        include_bytes!("../conformance/next/ifccad/valid/font-symbol-source-requests.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    assert!(original.annotation_scales.len() >= 2);
    assert_eq!(original.symbol_sources.len(), 2);
    assert_eq!(original.symbol_sources[1].id.0, 9007199254740993);
    let json = encode_ifccad_document(&original).unwrap();
    assert_eq!(
        load_ifccad_bytes(json.bytes(), Default::default())
            .unwrap()
            .document(),
        &original
    );
    let cbor = encode_ifccad_cbor_document(&original).unwrap();
    assert_eq!(
        load_ifccad_cbor_bytes(cbor.bytes(), Default::default())
            .unwrap()
            .document(),
        &original
    );
    #[cfg(feature = "ifccad-cbor-zstd")]
    {
        let zstd = encode_ifccad_cbor_zstd_document(&original, Default::default()).unwrap();
        assert_eq!(
            load_ifccad_cbor_zstd_bytes(zstd.bytes(), Default::default())
                .unwrap()
                .document(),
            &original
        );
    }
}

#[test]
fn native_requests_roundtrip_without_loading_external_files() {
    let mut document = load_ifccad_bytes(
        include_bytes!("fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    document.symbol_sources.push(IfccadSymbolSource {
        id: IfccadSymbolSourceId(0),
        source: SymbolSource::Shx {
            file_name: "missing.shx".into(),
            cad_file_name: Some(r"..\project\missing.shx".into()),
        },
    });
    let json = encode_ifccad_document(&document).unwrap();
    let reopened = load_ifccad_bytes(json.bytes(), Default::default()).unwrap();
    assert_eq!(reopened.document().symbol_sources, document.symbol_sources);
    let cbor = encode_ifccad_cbor_document(&document).unwrap();
    let reopened = load_ifccad_cbor_bytes(cbor.bytes(), Default::default()).unwrap();
    assert_eq!(reopened.document().symbol_sources, document.symbol_sources);
}

#[test]
fn shx_cad_request_basename_is_platform_independent() {
    for request in [
        "project.shx",
        r"C:project.shx",
        r"..\library\project.shx",
        r"C:\library\project.shx",
        r"\\server\share\project.shx",
        "/library/project.shx",
    ] {
        assert_eq!(
            shx_file_name_from_cad_request(request).unwrap(),
            "project.shx"
        );
    }
    for request in ["", "bad\0.shx", "library/", "library/.", r"library\.."] {
        assert!(
            shx_file_name_from_cad_request(request).is_err(),
            "{request:?}"
        );
    }
}

#[test]
fn shx_request_must_match_declared_basename() {
    let source = SymbolSource::Shx {
        file_name: "project.shx".into(),
        cad_file_name: Some(r"..\library\project.shx".into()),
    };
    assert!(validate_symbol_source(&source).is_ok());
    assert_eq!(source.effective_cad_file_name(), r"..\library\project.shx");
    for request in ["other.shx", "Project.shx", "", "library/", "bad\0.shx"] {
        let source = SymbolSource::Shx {
            file_name: "project.shx".into(),
            cad_file_name: Some(request.into()),
        };
        assert!(validate_symbol_source(&source).is_err(), "{request:?}");
    }
}
