#[test]
fn inspector_exposes_decoded_geometry_without_frontend_column_decoding() {
    let bytes =
        include_bytes!("../../../conformance/next/ocdraw/valid/named-line-patterns.ocdraw.json");
    let result = viewer::inspect_drawing_bytes("example.ocdraw.json", bytes);
    assert_eq!(result["validation"]["strictAvailable"], true);
    let entities = result["presentation"]["entities"]
        .as_array()
        .expect("logical entities");
    assert_eq!(entities.len(), 2);
    assert_eq!(entities[0]["geometry"]["vertices"][0][0], -1.25);
    assert_eq!(entities[0]["geometry"]["type"], "planarPolyline");
    assert_eq!(entities[1]["geometry"]["vertices"][0][2], -3.75);
}

#[test]
fn authored_drawing_view_state_is_exposed_with_the_actual_encoding_key() {
    let result = viewer::inspect_drawing_bytes(
        "state.ocdraw.json",
        include_bytes!("../../../examples/ocdraw/state-and-storage.ocdraw.json"),
    );
    assert_eq!(
        result["presentation"]["viewState"]["activeModelWindowId"],
        7
    );
}
