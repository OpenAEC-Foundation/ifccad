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
