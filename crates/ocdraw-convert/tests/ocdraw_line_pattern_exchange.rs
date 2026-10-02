use ocdraw::ocdraw::*;
use ocdraw_convert::opencadcodec::{DwgReader, DwgWriter, DxfReader, DxfWriter};
use ocdraw_convert::{
    cad_document_to_encoded_ocdraw, ocdraw_source_to_cad_document, CadToOcdrawOptions,
    OcdrawToCadOptions,
};
use std::io::Cursor;

fn exchange(dwg: bool) {
    let mut builder =
        OcdrawBuilder::new(OcdrawBuildOptions::new("pattern-exchange", "mm")).unwrap();
    let dash = builder
        .add_line_pattern(LinePatternDefinition {
            name: "Custom".into(),
            description: Some("Pattern".into()),
            pattern: vec![0.5, -0.25, 0.0, -0.25],
        })
        .unwrap();
    let fallback = builder
        .add_line_pattern(LinePatternDefinition {
            name: "GAS_LEIDING".into(),
            description: Some("Gasleiding".into()),
            pattern: vec![],
        })
        .unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(255, 255, 255),
            dash,
        ))
        .unwrap();
    builder.set_line_pattern_scale(2.0).unwrap();
    let mut line = LineDefinition::new(layer, [0.0; 3], [10.0, 0.0, 0.0]);
    line.appearance.line_pattern = AppearanceSelection::Explicit(fallback);
    line.appearance.line_pattern_scale = 0.5;
    builder.add_line(line).unwrap();
    let mut poly = PlanarPolylineDefinition::new(
        layer,
        vec![[0.0, 1.0, 0.0], [10.0, 1.0, 0.0], [10.0, 2.0, 0.0]],
        false,
    );
    poly.line_pattern_generation = LinePatternGeneration::Continuous;
    builder.add_planar_polyline(poly).unwrap();
    let mut spatial =
        SpatialPolylineDefinition::new(layer, vec![[0.0, 0.0, 0.0], [1.0, 2.0, 3.0]], false);
    spatial.line_pattern_generation = LinePatternGeneration::Continuous;
    builder.add_spatial_polyline(spatial).unwrap();
    let original = builder.finish().unwrap();
    let loaded = load_ocdraw_bytes(original.bytes());
    let target =
        ocdraw_source_to_cad_document(loaded.as_ref().ok().unwrap(), OcdrawToCadOptions::default())
            .unwrap();
    assert!(target.document().entities().any(|e| matches!(
        e, ocdraw_convert::opencadcodec::EntityType::Polyline3D(p) if p.flags.linetype_continuous
    )));
    let decoded = if dwg {
        DwgReader::from_stream(Cursor::new(
            DwgWriter::write_to_vec(target.document()).unwrap(),
        ))
        .read()
        .unwrap()
    } else {
        DxfReader::from_reader(Cursor::new(
            DxfWriter::new(target.document()).write_to_vec().unwrap(),
        ))
        .unwrap()
        .read()
        .unwrap()
    };
    let native = cad_document_to_encoded_ocdraw(&decoded, CadToOcdrawOptions::default()).unwrap();
    assert_eq!(
        native.entity_mapping().len(),
        3,
        "{:?}",
        native.diagnostics()
    );
    let read = load_ocdraw_bytes(native.encoded().bytes());
    let drawing = read.as_ref().ok().unwrap();
    let dash = drawing
        .line_patterns()
        .iter()
        .find(|p| p.name == "Custom")
        .unwrap();
    assert_eq!(dash.pattern, vec![0.5, -0.25, 0.0, -0.25]);
    let fallback = drawing
        .line_patterns()
        .iter()
        .find(|p| p.name == "GAS_LEIDING")
        .unwrap();
    assert!(fallback.pattern.is_empty());
    assert_eq!(drawing.typed_layers()[0].line_pattern_id, dash.id);
    assert_eq!(drawing.line_pattern_scale(), 2.0);
    let line = drawing
        .geometric_entities()
        .iter()
        .find(|e| matches!(e.geometry(), DrawingGeometry::Line { .. }))
        .unwrap();
    assert_eq!(
        line.appearance().line_pattern,
        AppearanceSelection::Explicit(fallback.id)
    );
    assert_eq!(line.appearance().line_pattern_scale, 0.5);
    let poly = drawing
        .geometric_entities()
        .iter()
        .find(|e| matches!(e.geometry(), DrawingGeometry::PlanarPolyline { .. }))
        .unwrap();
    assert!(matches!(
        poly.geometry(),
        DrawingGeometry::PlanarPolyline {
            line_pattern_generation: LinePatternGeneration::Continuous,
            ..
        }
    ));
    // The pinned DWG codec writes only the closed bit for Polyline3D;
    // direct CAD and DXF retain the continuous-generation flag.
    let expected_spatial = if dwg {
        LinePatternGeneration::PerSegment
    } else {
        LinePatternGeneration::Continuous
    };
    assert!(drawing.geometric_entities().iter().any(|e| matches!(
        e.geometry(),
        DrawingGeometry::SpatialPolyline {
            line_pattern_generation,
            ..
        } if *line_pattern_generation == expected_spatial
    )));
    // CAD requires its Continuous scaffold even when the native table omits it.
    assert!(drawing
        .line_patterns()
        .iter()
        .any(|p| p.name.eq_ignore_ascii_case("Continuous") && p.pattern.is_empty()));
}

#[test]
fn dxf_patterns_exchange() {
    exchange(false)
}
#[test]
fn dwg_patterns_exchange() {
    exchange(true)
}
