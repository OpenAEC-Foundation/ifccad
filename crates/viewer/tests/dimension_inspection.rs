#[test]
fn independent_ifccad_inspector_keeps_full_width_source_paths() {
    let out = viewer::inspect_ifccad_bytes(
        "dims.ifcx",
        include_bytes!("../../../conformance/next/ifccad/valid/dimension-linear-aligned.ifcx"),
    );
    let d = &out["presentation"]["dimensions"][0];
    assert_eq!(d["geometry"]["rawMeasurement"]["value"], 4000.0);
    assert!(d["geometry"]["associations"][0]["source"]
        .as_str()
        .unwrap()
        .ends_with("e9007199254740993"));
    assert!(d["geometry"]["label"]["fragments"].is_array());
    assert_eq!(d["geometry"]["displayEvaluation"], "unassessed");
}

#[test]
fn maintained_dimension_examples_strictly_load_with_model_paper_and_block_content() {
    let ifccad = ifccad::load_ifccad_bytes(
        include_bytes!("../../../examples/ifccad/dimensions.ifcx"),
        Default::default(),
    )
    .unwrap();
    let doc = ifccad.document();
    ifccad::validate_ifccad_document(doc).unwrap();
    assert_eq!(doc.paper_layouts.len(), 1);
    assert_eq!(doc.blocks.len(), 1);
    assert_eq!(doc.model.entities.len(), 4);
    assert_eq!(doc.paper_layouts[0].entities.len(), 2);
    assert_eq!(doc.blocks[0].entities.len(), 2);
}
