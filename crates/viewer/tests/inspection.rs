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

#[test]
fn text_inspection_exposes_authored_content_style_and_unverified_estimated_bounds() {
    use ocdraw::{ocdraw::*, text::*};
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("inspected-text", "mm")).unwrap();
    let pattern = b.ensure_continuous_line_pattern().unwrap();
    let layer = b
        .add_layer(LayerDefinition::new(
            "Text",
            RgbColor::new(255, 255, 255),
            pattern,
        ))
        .unwrap();
    let style = b
        .add_text_style(TextStyleDefinition {
            name: "Requested".into(),
            properties: TextStyleProperties::new(FontRequest::family("Face")),
        })
        .unwrap();
    b.add_text(TextEntityDefinition::new(
        layer,
        style,
        2.,
        vec![TextRun {
            text: "Literal \\P é🙂".into(),
            ..Default::default()
        }],
    ))
    .unwrap();
    let bytes = b.finish().unwrap();
    let result = viewer::inspect_drawing_bytes("text.ocdraw.json", bytes.bytes());
    assert_eq!(result["validation"]["strictAvailable"], true);
    assert_eq!(
        result["presentation"]["entities"][0]["geometry"]["type"],
        "text"
    );
    assert_eq!(result["presentation"]["entities"][0]["styleId"], style.0);
    assert_eq!(
        result["presentation"]["entities"][0]["geometry"]["content"][0]["text"],
        "Literal \\P é🙂"
    );
    assert_eq!(
        result["presentation"]["textStyles"][0]["font"]["family"],
        "Face"
    );
    assert_eq!(
        result["presentation"]["boundsCompleteness"][0]["quality"],
        "estimated"
    );
    assert_eq!(
        result["presentation"]["boundsCompleteness"][0]["enclosureVerified"],
        false
    );
    assert_eq!(
        result["presentation"]["boundsCompleteness"][0]["safeForNegativeQuery"],
        false
    );
}
