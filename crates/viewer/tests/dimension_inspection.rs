#[test]
fn stored_picture_inspection_keeps_origin_empty_state_and_child_paths() {
    let out = viewer::inspect_ifccad_bytes(
        "graphics.ifcx",
        include_bytes!("../../../conformance/next/ifccad/valid/dimension-graphics.ifcx"),
    );
    let dims = out["presentation"]["dimensions"].as_array().unwrap();
    let stored = &dims[0]["geometry"]["graphics"];
    assert_eq!(stored["state"], "stored");
    assert_eq!(stored["origin"], "sourceSnapshot");
    assert_eq!(stored["partCount"], 2);
    assert!(stored["path"].as_str().unwrap().ends_with("/graphics"));
    assert_eq!(dims[1]["geometry"]["graphics"]["origin"], "generated");
    assert_eq!(dims[1]["geometry"]["graphics"]["partCount"], 0);
    assert_eq!(dims[0]["geometry"]["displayEvaluation"], "unassessed");
}

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
fn dimension_inspector_exposes_spatial_references_and_planar_line() {
    let out = viewer::inspect_ifccad_bytes(
        "spatial.ifcx",
        include_bytes!("../../../conformance/next/ifccad/valid/dimension-spatial-references.ifcx"),
    );
    let def = &out["presentation"]["dimensions"][0]["geometry"]["definition"];
    assert_eq!(def["firstPoint"].as_array().unwrap().len(), 3);
    assert_eq!(def["firstPoint"][2], 544.);
    assert_eq!(def["secondPoint"][2], 1632.);
    assert_eq!(def["dimensionLinePoint"].as_array().unwrap().len(), 2);
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
