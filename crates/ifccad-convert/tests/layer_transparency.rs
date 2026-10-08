mod common;
use common::*;
use opencadcodec::{CadDocument, DwgReader, DwgWriter, SemanticPartV1, Transparency};
use std::io::Cursor;

fn transparent_dwg() -> CadDocument {
    let mut doc = cad();
    doc.layers.get_mut("0").unwrap().transparency = Transparency::Explicit(128);
    DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(&doc).unwrap()))
        .read()
        .unwrap()
}

#[test]
fn dwg_layer_transparency_duplicate_survives_strict_native_roundtrip() {
    let doc = transparent_dwg();
    let mut records = 0;
    doc.semantic_inventory_v1().visit(|part| {
        if let SemanticPartV1::NonEntityExtendedData { application, .. } = part {
            assert_eq!(application.unwrap().name, "AcCmTransparency");
            records += 1;
        }
    });
    assert_eq!(records, 1);
    let native = from_cad(&doc, metadata()).unwrap();
    let loaded =
        ocdraw::ifccad::load_ifccad_bytes(native.encoded().bytes(), Default::default()).unwrap();
    let restored = to_cad(&loaded).unwrap();
    assert_eq!(
        restored.document().layers.get("0").unwrap().transparency,
        Transparency::Explicit(128)
    );
}

#[test]
fn disagreeing_layer_transparency_duplicate_remains_rejected() {
    let mut doc = transparent_dwg();
    doc.layers.get_mut("0").unwrap().transparency = Transparency::Explicit(204);
    assert!(from_cad(&doc, metadata()).is_err());
    let allow =
        ifccad_convert::cad_document_to_encoded_ifccad(&doc, metadata(), Default::default())
            .unwrap();
    assert!(allow
        .diagnostics()
        .iter()
        .any(|d| d.code == "xdata" && d.is_loss()));
}

#[test]
fn layer_bylayer_default_is_opaque_and_keeps_text_under_reject() {
    let mut doc = CadDocument::new();
    let layer = doc.layers.get_mut("0").unwrap();
    layer.color = opencadcodec::Color::from_rgb(17, 146, 238);
    layer.line_weight = opencadcodec::LineWeight::Value(25);
    layer.transparency = Transparency::ByLayer;
    let mut text = opencadcodec::Text::with_value("Default opacity", opencadcodec::Vector3::ZERO);
    text.common.line_weight = opencadcodec::LineWeight::ByLayer;
    doc.add_entity(opencadcodec::EntityType::Text(text))
        .unwrap();
    for dwg in [false, true] {
        let source = if dwg {
            DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(&doc).unwrap()))
                .read()
                .unwrap()
        } else {
            opencadcodec::DxfReader::from_reader(Cursor::new(
                opencadcodec::DxfWriter::new(&doc).write_to_vec().unwrap(),
            ))
            .unwrap()
            .read()
            .unwrap()
        };
        let native = from_cad(&source, metadata()).unwrap();
        assert_eq!(
            native.validated_source().document().layers[0]
                .appearance
                .opacity,
            1.
        );
        assert_eq!(native.validated_source().document().model.entities.len(), 1);
    }
}
