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

#[path = "../../../tests/support/ifccad_dimension.rs"]
mod support;

#[test]
fn remaining_kinds_inspect_through_both_candidate_readers_with_full_width_ids() {
    use ifccad::*;
    use serde_json::json;
    let cases = [
        json!({"kind":"radius","center":[0.,0.,7.],"curvePoint":[3.,4.,9.],"leaderLength":0.}),
        json!({"kind":"diameter","firstDiameterPoint":[-3.,-4.,7.],"secondDiameterPoint":[3.,4.,9.],"leaderLength":0.}),
        json!({"kind":"angularTwoLines","line1Start":[0.,0.,7.],"line1End":[1.,0.,8.],"line2Start":[0.,0.,9.],"line2End":[0.,1.,10.],"ray1Sign":"positive","ray2Sign":"positive","traversal":"counterclockwise","dimensionArcPoint":[5.,5.]}),
        json!({"kind":"angularThreePoints","vertex":[0.,0.,7.],"firstPoint":[1.,0.,8.],"secondPoint":[0.,1.,9.],"traversal":"clockwise","dimensionArcPoint":[5.,5.]}),
        json!({"kind":"ordinate","origin":[10.,20.,3.],"featurePoint":[7.,24.,8.],"axisAngle":0.,"leaderEnd":[20.,40.],"valueMode":"signed"}),
        json!({"kind":"arcLength","center":[0.,0.,7.],"radius":100.,"startParameter":0.,"sweep":{"kind":"full","traversal":"clockwise"},"dimensionArcPoint":[120.,0.]}),
        json!({"kind":"joggedRadius","center":[0.,0.,7.],"curvePoint":[3.,4.,9.],"displayCenter":[100.,200.],"jogPoint":[50.,50.],"jogAngle":0.5}),
    ];
    for case in cases {
        let mut drawing = support::drawing();
        let IfccadEntityKind::Dimension(d) =
            &mut drawing.model.entities[0].as_native_mut().unwrap().kind
        else {
            panic!()
        };
        d.definition = serde_json::from_value(case.clone()).unwrap();
        let json = encode_ifccad_document(&drawing).unwrap();
        let cbor = encode_ifccad_cbor_document(&drawing).unwrap();
        for (name, bytes) in [
            ("candidate.ifcx", json.bytes()),
            ("candidate.ifccad.cbor", cbor.bytes()),
        ] {
            let inspected = viewer::inspect_ifccad_bytes(name, bytes);
            let dimension = &inspected["presentation"]["dimensions"][0];
            assert_eq!(dimension["entityId"], support::DIMENSION.to_string());
            assert_eq!(
                dimension["geometry"]["definition"], case,
                "{name}: {inspected}"
            );
            assert_eq!(dimension["geometry"]["graphics"]["state"], "absent");
            assert_eq!(dimension["geometry"]["label"]["state"], "visible");
            assert_eq!(dimension["geometry"]["associationUpdate"], "external");
        }
    }
}
