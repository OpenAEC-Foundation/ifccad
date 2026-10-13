mod common;
use ifccad_convert::*;
use ocdraw::ifccad::*;
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
    let out =
        cad_document_to_ifccad_document(&source, common::metadata(), Default::default()).unwrap();
    let encoded = encode_ifccad_document(out.document()).unwrap();
    let read = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    let target = ifccad_document_to_cad_document(read.document(), Default::default()).unwrap();

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
    let native =
        cad_document_to_ifccad_document(&decoded, common::metadata(), Default::default()).unwrap();
    let encoded = encode_ifccad_document(native.document()).unwrap();
    let read = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    assert_eq!(read.document().symbol_sources.len(), 2);
    assert_eq!(
        read.document()
            .line_patterns
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
