use ocdraw::ocdraw::*;
use ocdraw_convert::*;
use opencadcodec::{
    CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType, LineWeight, MText, Text,
    Vector3,
};
use std::io::Cursor;

#[test]
fn simple_text_and_mtext_survive_native_then_actual_dxf_and_dwg_exchange() {
    let mut source = CadDocument::new();
    source.header.insertion_units = 4;
    let mut text = Text::with_value("Literal %%uonderstreept%%u é🙂", Vector3::new(4., 5., 0.));
    text.height = 2.5;
    text.common.line_weight = LineWeight::ByLayer;
    source.add_entity(EntityType::Text(text)).unwrap();
    let mut mtext = MText::new();
    mtext.value = "Eerste\\PTweede é🙂".into();
    mtext.height = 2.5;
    mtext.insertion_point = Vector3::new(10., 20., 0.);
    mtext.common.line_weight = LineWeight::ByLayer;
    source.add_entity(EntityType::MText(mtext)).unwrap();
    let native = cad_document_to_ocdraw_document(&source, Default::default()).unwrap();
    let file = encode_ocdraw_document(native.document()).unwrap();
    let strict = load_ocdraw_bytes(file.bytes()).unwrap();
    let target = ocdraw_document_to_cad_document(strict.document(), Default::default()).unwrap();
    for dwg in [false, true] {
        let bytes = if dwg {
            DwgWriter::write_to_vec(target.document()).unwrap()
        } else {
            DxfWriter::new(target.document()).write_to_vec().unwrap()
        };
        let source = if dwg {
            DwgReader::from_stream(Cursor::new(bytes)).read().unwrap()
        } else {
            DxfReader::from_reader(Cursor::new(bytes))
                .unwrap()
                .read()
                .unwrap()
        };
        let result = cad_document_to_ocdraw_document(&source, Default::default()).unwrap();
        assert_eq!(result.document().text_entities.len(), 1, "DWG={dwg}");
        assert_eq!(result.document().mtext_entities.len(), 1, "DWG={dwg}");
        assert_eq!(
            result.document().text_entities[0].content,
            native.document().text_entities[0].content,
            "DWG={dwg}"
        );
        assert_eq!(
            result.document().mtext_entities[0].content,
            native.document().mtext_entities[0].content,
            "DWG={dwg}"
        );
        assert!(!result.geometry_assessment().is_complete());
        assert_eq!(result.text_assessment().entries().len(), 2);
        assert!(result
            .text_assessment()
            .entries()
            .iter()
            .all(|e| e.glyph_coverage == OcdrawTextGlyphCoverage::Unassessed));
        encode_ocdraw_document(result.document()).unwrap();
    }
}
