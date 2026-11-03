#[path = "../../../examples/support/font_symbol_gallery.rs"]
mod gallery;

#[test]
fn saved_dimension_context_mtext_requests_are_inspected() {
    use serde_json::{json, Value};
    let mut graph: Value = serde_json::from_slice(include_bytes!(
        "../../../conformance/next/ifccad/valid/annotation-dimension-linear-aligned.ifcx"
    ))
    .unwrap();
    let gallery: Value = serde_json::from_slice(&gallery::build().unwrap()).unwrap();
    let text = gallery["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["path"] == "/cad/d1/e300")
        .unwrap();
    let mut part = json!({"ifccad::dimensionGraphicsPart":{},
        "ifccad::mText":text["attributes"]["ifccad::mText"],
        "ifccad::geom::placement":text["attributes"]["ifccad::geom::placement"]});
    part["ifccad::mText"]["style"] = json!("/cad/d1/textStyle/0");
    part["ifccad::mText"]["characterFormat"]["font"] = json!({"family":"Saved context family"});
    let dimension = graph["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"].get("ifccad::annotation").is_some())
        .unwrap();
    let owner = dimension["path"].as_str().unwrap().to_owned();
    dimension["attributes"]["ifccad::annotation"]["contexts"][0]["presentation"]["graphics"]
        ["parts"] = json!([part]);
    let result = viewer::inspect_ifccad_bytes("context.ifcx", &serde_json::to_vec(&graph).unwrap());
    assert!(result["failure"].is_null(), "{}", result["failure"]);
    let requests = result["presentation"]["fontSymbolRequests"]
        .as_array()
        .unwrap();
    let request = requests
        .iter()
        .find(|r| r["requestedName"] == "Saved context family")
        .expect("saved context request");
    assert_eq!(request["requestPath"], format!("{owner}#/attributes/ifccad::annotation/contexts/0/presentation/graphics/parts/0/ifccad::mText/characterFormat/font/family"));
    assert_eq!(request["status"], "notChecked");
}

#[test]
fn requests_include_unused_styles_and_all_mtext_scopes() {
    let bytes = gallery::build().unwrap();
    let result = viewer::inspect_ifccad_bytes("gallery.ifcx", &bytes);
    assert!(result["failure"].is_null(), "{}", result["failure"]);
    let requests = result["presentation"]["fontSymbolRequests"]
        .as_array()
        .unwrap();
    assert_eq!(
        requests.iter().filter(|r| r["fontRole"] == "shx").count(),
        6
    );
    assert!(requests.iter().any(|r| r["requestedName"] == "Consolas"));
    for scope in [
        "/characterFormat/font/",
        "/content/1/characterFormat/font/",
        "/inlines/1/characterFormat/font/",
        "/inlines/2/characterFormat/font/",
    ] {
        assert!(
            requests
                .iter()
                .any(|r| r["requestPath"].as_str().unwrap().contains(scope)),
            "{scope}"
        );
    }
    assert!(requests.iter().all(|r| r["status"] == "notChecked"));
}

#[test]
fn high_source_request_paths_stay_exact_and_unchecked() {
    let result = viewer::inspect_ifccad_bytes(
        "sources.ifcx",
        include_bytes!("../../../conformance/next/ifccad/valid/font-symbol-source-requests.ifcx"),
    );
    assert!(result["failure"].is_null());
    let requests = result["presentation"]["fontSymbolRequests"]
        .as_array()
        .expect("request inventory");
    let high = requests
        .iter()
        .find(|r| {
            r["requestPath"]
                .as_str()
                .unwrap()
                .starts_with("/cad/d1/symbolSource/9007199254740993#")
        })
        .unwrap();
    assert_eq!(high["fontRole"], "shx");
    assert_eq!(high["requestedName"], "missing.shx");
    assert_eq!(high["status"], "notChecked");
    assert!(requests
        .iter()
        .any(|r| r["requestedName"] == "..\\project\\missing.shx"));
}
