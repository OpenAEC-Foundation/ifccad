use std::process::Command;

#[test]
fn native_api_rejects_parked_input_and_advertises_only_ifccad() {
    let output = viewer::inspect_ifccad_bytes("old.ocdraw.json", b"{}");
    assert_eq!(output["failure"]["code"], "OCDRAW_PARKED");
    assert!(output.get("export").is_none());
    let capabilities = viewer::document_capabilities();
    assert!(capabilities.get("ocdraw").is_none());
    assert_eq!(capabilities["ifccad"]["coordinateToleranceDefault"], false);
}

#[test]
fn cli_rejects_explicit_legacy_choice_before_reading_source() {
    let output = Command::new(env!("CARGO_BIN_EXE_viewer"))
        .args(["cad", "does-not-exist.dxf", "--drawing-format", "ocdraw"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("OCDRAW_PARKED"));
}

#[test]
fn cli_defaults_to_ifccad_for_cad_input() {
    use opencaddocument::{Document, Dxf, Format};
    let directory = std::env::temp_dir().join(format!("ifccad-main-track-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("empty.dxf");
    std::fs::write(&path, Dxf.write(&Document::new()).unwrap().0).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_viewer"))
        .arg("cad")
        .arg(&path)
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(directory).unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let lines = String::from_utf8(output.stdout).unwrap();
    let result: serde_json::Value = lines
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .find(|value| value["type"] == "result")
        .unwrap()["result"]
        .clone();
    assert!(result["failure"].is_null(), "{result}");
    assert_eq!(result["export"]["download"]["format"], "ifccad");
    assert_eq!(result["validation"]["strictAvailable"], true);
}
