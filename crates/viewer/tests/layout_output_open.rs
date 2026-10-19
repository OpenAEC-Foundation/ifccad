use viewer::inspect_ifccad_bytes;
#[test]
fn medium_only_is_visible_without_invented_plot_configuration() {
    let i = inspect_ifccad_bytes(
        "medium.ifcx",
        include_bytes!("../../../conformance/next/ifccad/valid/layout-medium-only.ifcx"),
    );
    assert!(i["failure"].is_null());
    let paper = i["presentation"]["layouts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["attributes"]["ifccad::layout"]["kind"] == "Paper")
        .unwrap();
    assert_eq!(paper["attributes"]["ifccad::layout"]["media"]["width"], 420);
    assert!(paper["attributes"]["ifccad::layout"]
        .get("plotSettings")
        .is_none());
    assert!(paper["attributes"]["ifccad::layout"]
        .get("lengthUnit")
        .is_none());
}
