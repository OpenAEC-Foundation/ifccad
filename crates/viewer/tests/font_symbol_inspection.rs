#[path = "../../../examples/support/font_symbol_gallery.rs"]
mod gallery;

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
