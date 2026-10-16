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
            pattern: vec![0.5, -0.25, 0.0, -0.25]
                .into_iter()
                .map(ocdraw::line_pattern::LinePatternElement::simple)
                .collect(),
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
    assert_eq!(
        dash.pattern.iter().map(|e| e.length).collect::<Vec<_>>(),
        vec![0.5, -0.25, 0.0, -0.25]
    );
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

use opencadcodec::{
    tables::{LineTypeComplexContent, LineTypeComplexData, LineTypeElement, TextStyle},
    CadDocument, Handle, LineType,
};
fn source() -> CadDocument {
    let mut doc = CadDocument::new();
    let text = doc.text_styles.get("Standard").unwrap().handle;
    for handle in [900, 901] {
        let mut shape = TextStyle::new("");
        shape.handle = Handle::new(handle);
        shape.is_shape_file = true;
        shape.font_file = "custom.shx".into();
        doc.text_styles.add_allow_duplicate(shape);
    }
    let mut pattern = LineType::new("Mixed");
    pattern.description = "GAS symbols".into();
    pattern.elements = vec![
        LineTypeElement {
            length: 2.,
            complex: Some(LineTypeComplexData {
                content: LineTypeComplexContent::Text {
                    text: "GAS β".into(),
                },
                style_handle: text,
                scale: 0.1,
                rotation: 0.25,
                offset: [-1., 2.],
                ..Default::default()
            }),
        },
        LineTypeElement {
            length: 0.,
            complex: Some(LineTypeComplexData {
                source_flags: 0,
                content: LineTypeComplexContent::Shape { shape_number: 42 },
                style_handle: Handle::new(900),
                scale: 0.1,
                rotation: -0.5,
                absolute_rotation: true,
                offset: [-1., 2.],
            }),
        },
        LineTypeElement::space(2.),
    ];
    pattern.pattern_length = 4.;
    pattern.handle = doc.allocate_handle();
    doc.line_types.add(pattern).unwrap();
    doc
}

fn complex_exchange(dwg: bool) {
    let mut source = source();
    let marker = source.line_types.get("Mixed").unwrap().elements[0]
        .complex
        .clone();
    source.line_types.get_mut("Mixed").unwrap().elements[2].complex = marker;
    let out = cad_document_to_encoded_ocdraw(&source, Default::default()).unwrap();
    let read = load_ocdraw_bytes(out.encoded().bytes()).unwrap();
    let target = ocdraw_source_to_cad_document(&read, Default::default()).unwrap();

    let decoded = if dwg {
        opencadcodec::DwgReader::from_stream(std::io::Cursor::new(
            opencadcodec::DwgWriter::write_to_vec(target.document()).unwrap(),
        ))
        .read()
        .unwrap()
    } else {
        opencadcodec::DxfReader::from_reader(std::io::Cursor::new(
            opencadcodec::DxfWriter::new(target.document())
                .write_to_vec()
                .unwrap(),
        ))
        .unwrap()
        .read()
        .unwrap()
    };
    let row = decoded.line_types.get("Mixed").unwrap();
    assert_eq!(row.elements.len(), 3);
    assert_eq!(row.description, "GAS symbols");
    for (original, actual) in source
        .line_types
        .get("Mixed")
        .unwrap()
        .elements
        .iter()
        .zip(&row.elements)
    {
        assert_eq!(original.length, actual.length);
        match (&original.complex, &actual.complex) {
            (Some(a), Some(b)) => {
                assert_eq!(a.content, b.content);
                assert_eq!(a.scale, b.scale);
                assert_eq!(a.rotation, b.rotation);
                assert_eq!(a.absolute_rotation, b.absolute_rotation);
                assert_eq!(a.offset, b.offset);
                let style = decoded
                    .text_styles
                    .iter()
                    .find(|s| s.handle == b.style_handle)
                    .unwrap();
                assert_eq!(style.is_shape_file, b.is_shape());
                if b.is_shape() {
                    assert_eq!(style.font_file, "custom.shx");
                }
            }
            (None, None) => {}
            _ => panic!("complex marker disappeared"),
        }
    }
    assert_eq!(
        decoded
            .text_styles
            .iter()
            .filter(|s| s.is_shape_file)
            .count(),
        2
    );
    let native = cad_document_to_encoded_ocdraw(&decoded, Default::default()).unwrap();
    let read = load_ocdraw_bytes(native.encoded().bytes()).unwrap();
    assert_eq!(read.symbol_sources().len(), 2);
    assert_eq!(
        read.line_patterns()
            .iter()
            .find(|p| p.name == "Mixed")
            .unwrap()
            .pattern
            .iter()
            .filter(|e| e.marker.is_some())
            .count(),
        3
    );
}

#[test]
fn mixed_text_shape_dxf_roundtrip() {
    complex_exchange(false)
}
#[test]
fn mixed_text_shape_dwg_roundtrip() {
    complex_exchange(true)
}
#[test]
fn literal_dxf_marker_flags_and_degrees() {
    let decoded = opencadcodec::DxfReader::from_reader(std::io::Cursor::new(include_bytes!(
        "../../../tests/fixtures/line-patterns/complex-literal-ac1032.dxf"
    )))
    .unwrap()
    .read()
    .unwrap();
    let row = decoded.line_types.get("LiteralMixed").unwrap();
    assert_eq!(row.elements.len(), 3);
    let text = row.elements[0].complex.as_ref().unwrap();
    let shape = row.elements[1].complex.as_ref().unwrap();
    assert!(matches!(&text.content,LineTypeComplexContent::Text{text} if text=="GAS"));
    assert!(matches!(
        shape.content,
        LineTypeComplexContent::Shape { shape_number: 42 }
    ));
    assert!(!text.absolute_rotation);
    assert!(shape.absolute_rotation);
    assert_eq!(text.rotation, 0.25);
    assert_eq!(shape.rotation, -0.5);
    for marker in [text, shape] {
        assert_eq!(marker.scale, 0.1);
        assert_eq!(marker.offset, [-1., 2.]);
        let style = decoded
            .text_styles
            .iter()
            .find(|s| s.handle == marker.style_handle)
            .unwrap();
        assert_eq!(style.is_shape_file, marker.is_shape());
    }
}
