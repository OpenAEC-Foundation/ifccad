use ocdraw::dimension::{DimensionLabel, DimensionLabelPart};
use ocdraw::ocdraw::*;
#[test]
fn inspector_measures_stored_definition_and_shows_equal_override() {
    let read = load_ocdraw_bytes(include_bytes!(
        "../../../conformance/next/ocdraw/valid/dimension-linear-aligned.ocdraw.json"
    ))
    .unwrap();
    let mut doc = read.document().clone();
    let id = doc.dimension_entities[0].associations[1].source_entity_id;
    let source = doc
        .geometric_entities
        .iter_mut()
        .find(|e| e.id == id)
        .unwrap();
    let DrawingGeometry::Line { end, .. } = &mut source.geometry else {
        panic!("line")
    };
    end[0] = 4500.0;
    let height = doc.dimension_styles[0].settings.text.height;
    doc.dimension_entities[0].overrides.text.height = Some(height);
    let settings = &mut doc.dimension_styles[0].settings.numbers.primary;
    settings.measurement_factor = 1.0 / 16000.0;
    settings.format = ocdraw::dimension::DimensionDistanceFormat::Fractional;
    settings.fraction_layout = ocdraw::dimension::DimensionFractionLayout::Diagonal;
    settings.fraction_scale = 0.75;
    recompute_ocdraw_document_bounds(&mut doc).unwrap();
    let encoded = encode_ocdraw_document(&doc).unwrap();
    let out = viewer::inspect_drawing_bytes("dims.ocdraw.json", encoded.bytes());
    let d = out["presentation"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["geometry"]["type"] == "dimension")
        .unwrap();
    assert_eq!(d["geometry"]["rawMeasurement"]["value"], 4000.0);
    assert!(d["geometry"]["label"]["fragments"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["kind"] == "fraction" && p["layout"] == "diagonal" && p["textScale"] == 0.75));
    assert_eq!(d["geometry"]["overrides"]["text"]["height"], height);
    assert_eq!(d["geometry"]["displayEvaluation"], "unassessed");
    assert_eq!(d["geometry"]["associations"][1]["selector"]["kind"], "end");
    assert!(
        !out["presentation"]["boundsCompleteness"][0]["safeForNegativeQuery"]
            .as_bool()
            .unwrap()
    );
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
fn inspector_distinguishes_suppressed_and_literal_labels() {
    let read = load_ocdraw_bytes(include_bytes!(
        "../../../conformance/next/ocdraw/valid/dimension-linear-aligned.ocdraw.json"
    ))
    .unwrap();
    let mut doc = read.document().clone();
    for label in [
        DimensionLabel::Suppressed,
        DimensionLabel::Template(vec![DimensionLabelPart::Literal(vec![
            ocdraw::text::TextRun {
                text: "Literal <> %<field>%".into(),
                ..Default::default()
            },
        ])]),
    ] {
        doc.dimension_entities[0].label = label.clone();
        let bytes = encode_ocdraw_document(&doc).unwrap();
        let out = viewer::inspect_drawing_bytes("dims.ocdraw.json", bytes.bytes());
        let id = doc.dimension_entities[0].id.to_string();
        let d = out["presentation"]["entities"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["id"].as_str() == Some(id.as_str()))
            .unwrap();
        if matches!(label, DimensionLabel::Suppressed) {
            assert_eq!(d["geometry"]["label"]["state"], "suppressed");
            assert!(d["geometry"]["label"]["runs"]
                .as_array()
                .unwrap()
                .is_empty());
        } else {
            assert_eq!(
                d["geometry"]["label"]["runs"][0]["text"],
                "Literal <> %<field>%"
            );
        }
    }
}

#[test]
fn maintained_dimension_examples_strictly_load_with_model_paper_and_block_content() {
    let draw = load_ocdraw_bytes(include_bytes!(
        "../../../examples/ocdraw/dimensions.ocdraw.json"
    ))
    .unwrap();
    validate_ocdraw_document(draw.document()).unwrap();
    assert_eq!(draw.dimension_entities().len(), 5);
    assert_eq!(draw.document().layouts.len(), 2);
    assert_eq!(draw.document().block_definitions.len(), 1);
    assert!(draw.document().dimension_styles[0]
        .type_overrides
        .contains_key(&ocdraw::dimension::DimensionKind::Aligned));
    let ifccad = ocdraw::ifccad::load_ifccad_bytes(
        include_bytes!("../../../examples/ifccad/dimensions.ifcx"),
        Default::default(),
    )
    .unwrap();
    let doc = ifccad.document();
    ocdraw::ifccad::validate_ifccad_document(doc).unwrap();
    assert_eq!(doc.paper_layouts.len(), 1);
    assert_eq!(doc.blocks.len(), 1);
    assert_eq!(doc.model.entities.len(), 4);
    assert_eq!(doc.paper_layouts[0].entities.len(), 2);
    assert_eq!(doc.blocks[0].entities.len(), 2);
}
