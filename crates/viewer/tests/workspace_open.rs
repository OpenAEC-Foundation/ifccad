use base64::{engine::general_purpose::STANDARD, Engine};
use viewer::{export_drawing_bytes, inspect_drawing_bytes};

#[test]
fn ifccad_workspace_records_and_source_bytes_keep_full_width_identity() {
    const BYTES: &[u8] = include_bytes!("../../../examples/ifccad/workspace-state.ifcx");
    let opened = inspect_drawing_bytes("workspace.ifcx", BYTES);
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
    let exported = export_drawing_bytes("workspace.ifcx", BYTES, "ifccad", "AC1032");
    assert!(exported["failure"].is_null(), "{exported}");
    assert_eq!(
        STANDARD
            .decode(exported["export"]["download"]["base64"].as_str().unwrap())
            .unwrap(),
        BYTES
    );
}

#[test]
fn ocdraw_workspace_inspection_keeps_unspecified_choices_and_dormant_values() {
    let opened = inspect_drawing_bytes(
        "workspace.ocdraw.json",
        include_bytes!("../../../examples/ocdraw/workspace-state.ocdraw.json"),
    );
    assert_eq!(opened["validation"]["status"], "valid");
    let p = &opened["presentation"];
    assert!(p["viewState"]["activeModelWindowId"].is_null());
    assert!(p["paperCanvases"][0]["activeContext"].is_null());
    assert!(p["paperCanvases"][0]["currentUcs"].is_null());
    assert_eq!(p["paperCanvases"][0]["frame"]["center"]["z"], 3.);
    assert_eq!(p["paperCanvases"][0]["snap"]["spacing"]["x"], 0.);
    assert_eq!(p["viewportWorkspaces"][0]["useStoredUcs"], false);
}
