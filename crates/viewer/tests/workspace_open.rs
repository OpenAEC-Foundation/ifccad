use base64::{engine::general_purpose::STANDARD, Engine};
use viewer::{export_ifccad_bytes, inspect_ifccad_bytes};

#[test]
fn ifccad_workspace_records_and_source_bytes_keep_full_width_identity() {
    const BYTES: &[u8] = include_bytes!("../../../tests/fixtures/ifccad/workspace-state.ifcx");
    let opened = inspect_ifccad_bytes("workspace.ifcx", BYTES);
    assert_eq!(opened["validation"]["status"], "valid");
    assert_eq!(
        opened["presentation"]["ucsDefinitions"][0]["path"],
        "/cad/d1/ucs/9007199254740993"
    );
    assert_eq!(
        opened["presentation"]["modelWindows"][0]["path"],
        "/cad/d1/modelWindow/9007199254740993"
    );
    let drawing = opened["presentation"]["graph"]["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["path"] == "/cad/d1")
        .unwrap();
    assert_eq!(
        drawing["attributes"]["ifccad::drawing"]["nextUcsId"].as_u64(),
        Some(9007199254740994)
    );
    let exported = export_ifccad_bytes("workspace.ifcx", BYTES, "ifccad", "AC1032");
    assert!(exported["failure"].is_null(), "{exported}");
    assert_eq!(
        STANDARD
            .decode(exported["export"]["download"]["base64"].as_str().unwrap())
            .unwrap(),
        BYTES
    );
}
