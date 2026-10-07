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
fn cli_selects_direct_ifccad_and_preserves_legacy_default() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../ocdraw-convert/tests/fixtures/splines/open-cubic.dxf");
    let file = path.to_str().unwrap();
    let legacy = cli(&["cad", file]);
    assert!(legacy.status.success());
    assert_eq!(result(&legacy)["presentation"]["format"], "ocdraw");
    let selected = cli(&[
        "cad",
        file,
        "--drawing-format",
        "ifccad",
        "--preserve-splines",
    ]);
    assert!(selected.status.success());
    assert_eq!(result(&selected)["presentation"]["opaqueEntityCount"], 1);
    let exported = cli(&[
        "export-cad",
        file,
        "dwg",
        "--drawing-format",
        "ifccad",
        "--preserve-splines",
    ]);
    assert!(exported.status.success());
    let output = result(&exported);
    assert!(output["failure"].is_null(), "{}", output["failure"]);
    assert_eq!(output["export"]["fileCheck"]["opaqueEntityCount"], 1);
    for args in [
        vec!["cad", file, "--drawing-format"],
        vec!["cad", file, "--drawing-format", "future"],
    ] {
        assert!(!cli(&args).status.success());
    }
}
