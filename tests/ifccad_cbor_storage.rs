use ocdraw::ifccad::*;
#[test]
fn file_limits_error_phases_and_create_new_are_preserved() {
    let root = std::env::temp_dir().join(format!(
        "ifccad-cbor-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    let file = root.join("drawing.ifccad.cbor");
    let d = load_ifccad_bytes(
        include_bytes!("fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap();
    let out = encode_ifccad_cbor_document(d.document()).unwrap();
    out.write_file(&file).unwrap();
    assert_eq!(
        load_ifccad_cbor_file(&file, Default::default())
            .unwrap()
            .document(),
        d.document()
    );
    assert!(out.write_file(&file).is_err());
    assert_eq!(std::fs::read(&file).unwrap(), out.bytes());
    let options = IfccadCborReadOptions {
        limits: IfccadCborReadLimits {
            max_input_bytes: 4,
            ..Default::default()
        },
    };
    assert!(matches!(
        load_ifccad_cbor_file(&file, options),
        Err(IfccadCborOpenError::Read(
            IfccadCborReadError::ResourceLimit { .. }
        ))
    ));
    assert!(matches!(
        load_ifccad_cbor_file(root.join("missing"), Default::default()),
        Err(IfccadCborOpenError::Io(_))
    ));
    std::fs::write(&file, [0]).unwrap();
    assert!(matches!(
        load_ifccad_cbor_file(&file, Default::default()),
        Err(IfccadCborOpenError::Read(
            IfccadCborReadError::Malformed { .. }
        ))
    ));
    std::fs::remove_file(file).unwrap();
    std::fs::remove_dir(root).unwrap();
}
