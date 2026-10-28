use std::{path::PathBuf, process::Command};
fn cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_viewer"))
        .args(args)
        .output()
        .unwrap()
}
fn result(output: &std::process::Output) -> serde_json::Value {
    String::from_utf8(output.stdout.clone())
        .unwrap()
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .find(|v| v["type"] == "result")
        .unwrap()["result"]
        .clone()
}
#[test]
fn cli_uses_shared_conversion_and_rejects_unavailable_capture() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/open-cubic.dxf");
    let file = path.to_str().unwrap();
    let opened = cli(&["cad", file]);
    assert!(opened.status.success());
    assert_eq!(result(&opened)["presentation"]["format"], "ifccad");
    let selected = cli(&[
        "cad",
        file,
        "--drawing-format",
        "ifccad",
        "--preserve-splines",
    ]);
    assert!(selected.status.success());
    assert_eq!(
        result(&selected)["failure"]["code"],
        "DOCUMENT_OPTION_UNAVAILABLE"
    );
    assert!(result(&selected)["export"]["download"].is_null());
    let exported = cli(&["export-cad", file, "dwg", "--drawing-format", "ifccad"]);
    assert!(exported.status.success());
    let output = result(&exported);
    assert!(output["failure"].is_null(), "{}", output["failure"]);
    assert_eq!(output["export"]["fileCheck"]["strictNativeReadback"], true);
    for args in [
        vec!["cad", file, "--drawing-format"],
        vec!["cad", file, "--drawing-format", "future"],
    ] {
        assert!(!cli(&args).status.success());
    }
}
