mod common;
use ifccad::{text::resolve_paragraph_format, *};
use ifccad_convert::*;
use opencadcodec::{
    CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType, MText, Vector3,
};
use std::io::Cursor;

#[test]
fn finite_paragraph_overflow_survives_native_and_actual_cad_readback() {
    let mut source = CadDocument::new();
    source.text_styles.get_mut("Standard").unwrap().font_file = "txt.shx".into();
    for (index, markup) in [
        r"\pi-5,l5,t5;one two three\PX",
        r"\pi-6,l6,t6;one two three\PX",
        r"\pr6;one two three",
        r"\pi6;one two three",
    ]
    .into_iter()
    .enumerate()
    {
        let mut text = MText::new();
        text.value = markup.into();
        text.height = 2.0;
        text.rectangle_width = 10.0;
        text.insertion_point = Vector3::new(index as f64 * 40.0, 0.0, 0.0);
        source
            .add_entity(EntityType::MText(Box::new(text)))
            .unwrap();
    }
    let before = serde_json::to_value(&source).unwrap();
    let native =
        cad_document_to_encoded_ifccad(&source, common::metadata(), Default::default()).unwrap();
    let strict = load_ifccad_bytes(native.encoded().bytes(), Default::default()).unwrap();
    validate_ifccad_document(strict.document()).unwrap();
    assert_eq!(strict.document().model.entities.len(), 4);
    assert_eq!(serde_json::to_value(&source).unwrap(), before);
    let cbor = encode_ifccad_cbor_document(strict.document()).unwrap();
    assert_eq!(
        load_ifccad_cbor_bytes(cbor.bytes(), Default::default())
            .unwrap()
            .document(),
        strict.document()
    );
    let target = ifccad_source_to_cad_document(&strict, Default::default()).unwrap();
    for dwg in [false, true] {
        let bytes = if dwg {
            DwgWriter::write_to_vec(target.document()).unwrap()
        } else {
            DxfWriter::new(target.document()).write_to_vec().unwrap()
        };
        let read = if dwg {
            DwgReader::from_stream(Cursor::new(bytes)).read().unwrap()
        } else {
            DxfReader::from_reader(Cursor::new(bytes))
                .unwrap()
                .read()
                .unwrap()
        };
        let back =
            cad_document_to_encoded_ifccad(&read, common::metadata(), Default::default()).unwrap();
        let loaded = load_ifccad_bytes(back.encoded().bytes(), Default::default()).unwrap();
        validate_ifccad_document(loaded.document()).unwrap();
        assert_eq!(loaded.document().model.entities.len(), 4);
        for (expected, actual) in strict
            .document()
            .model
            .entities
            .iter()
            .zip(&loaded.document().model.entities)
        {
            let (IfccadEntityKind::MText(expected), IfccadEntityKind::MText(actual)) = (
                &expected.as_native().unwrap().kind,
                &actual.as_native().unwrap().kind,
            ) else {
                panic!("MText kind changed")
            };
            assert_eq!(actual.content, expected.content);
            assert_eq!(actual.height, expected.height);
            assert_eq!(actual.wrap_width, expected.wrap_width);
            assert_eq!(actual.placement, expected.placement);
            for (a, b) in expected.content.iter().zip(&actual.content) {
                assert_eq!(
                    resolve_paragraph_format(&expected.paragraph_format, &a.paragraph_format)
                        .unwrap(),
                    resolve_paragraph_format(&actual.paragraph_format, &b.paragraph_format)
                        .unwrap()
                );
            }
        }
    }
}
