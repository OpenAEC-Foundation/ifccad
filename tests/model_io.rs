use ocdraw::ifccad::{
    encode_ifccad_document, load_ifccad_bytes, load_ifccad_file, IfccadOpenError, IfccadReadOptions,
};
use ocdraw::ocdraw::{
    load_ocdraw_bytes, load_ocdraw_file, OcdrawBuildOptions, OcdrawBuilder, OcdrawOpenError,
    OcdrawReadError,
};

#[test]
fn result_readers_retain_invalid_input_evidence() {
    let error = load_ocdraw_bytes(b"{}").unwrap_err();
    assert!(!error.diagnostics().is_empty());
    assert!(matches!(error, OcdrawReadError::Invalid { .. }));
    let error = load_ifccad_bytes(b"{}", IfccadReadOptions::default()).unwrap_err();
    assert!(!error.report().errors.is_empty());
}

#[test]
fn encoded_profile_and_original_source_have_independent_ownership() {
    let original = include_bytes!("../examples/ifccad/hello-cad.ifcx");
    let loaded = load_ifccad_bytes(original, IfccadReadOptions::default()).unwrap();
    let (source, mut document) = loaded.into_parts();
    document.header.author = "edited logical document".into();
    let encoded = encode_ifccad_document(&document).unwrap();
    let readback = load_ifccad_bytes(encoded.bytes(), IfccadReadOptions::default()).unwrap();
    assert_eq!(readback.document().header.author, document.header.author);
    assert_eq!(source.source_bytes(), original);
    assert_ne!(encoded.bytes(), source.source_bytes());
    assert_eq!(encoded.into_bytes(), readback.graph().source_bytes());
}

#[test]
fn file_storage_refuses_overwrite_and_distinguishes_read_errors() {
    let directory = std::env::temp_dir().join(format!("model-io-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let ocdraw_path = directory.join("drawing.ocdraw.json");
    let ifcx_path = directory.join("drawing.ifcx");
    let invalid = directory.join("invalid.json");
    let missing = directory.join("missing.json");
    let encoded = OcdrawBuilder::new(OcdrawBuildOptions::new("io", "mm"))
        .unwrap()
        .finish()
        .unwrap();
    encoded.write_file(&ocdraw_path).unwrap();
    assert!(encoded.write_file(&ocdraw_path).is_err());
    assert_eq!(load_ocdraw_file(&ocdraw_path).unwrap().drawing_id(), "io");
    let loaded = load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-cad.ifcx"),
        IfccadReadOptions::default(),
    )
    .unwrap();
    let encoded = encode_ifccad_document(loaded.document()).unwrap();
    encoded.write_file(&ifcx_path).unwrap();
    assert!(encoded.write_file(&ifcx_path).is_err());
    load_ifccad_file(&ifcx_path, IfccadReadOptions::default()).unwrap();
    std::fs::write(&invalid, b"{}").unwrap();
    assert!(matches!(
        load_ocdraw_file(&missing),
        Err(OcdrawOpenError::Io(_))
    ));
    assert!(matches!(
        load_ocdraw_file(&invalid),
        Err(OcdrawOpenError::Read(_))
    ));
    assert!(matches!(
        load_ifccad_file(&missing, IfccadReadOptions::default()),
        Err(IfccadOpenError::Io(_))
    ));
    assert!(matches!(
        load_ifccad_file(&invalid, IfccadReadOptions::default()),
        Err(IfccadOpenError::Read(_))
    ));
    for path in [ocdraw_path, ifcx_path, invalid] {
        std::fs::remove_file(path).unwrap();
    }
    std::fs::remove_dir(directory).unwrap();
}
