#[path = "../../../tests/support/cad_dimension_repair.rs"]
mod repair;
use ocdraw::{dimension::*, ocdraw::*};
use ocdraw_convert::*;
use opencadcodec::{DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType};
use std::io::Cursor;

#[test]
fn repair_profiles_survive_strict_native_readback_and_cad_return() {
    let cad = repair::drawing();
    let original = cad.text_styles.get("Standard").unwrap().height;
    let native = cad_document_to_ocdraw_document(&cad, Default::default()).unwrap();
    assert_eq!(native.document().dimension_entities.len(), 2);
    assert_eq!(cad.text_styles.get("Standard").unwrap().height, original);
    let bytes = encode_ocdraw_document(native.document()).unwrap();
    let loaded = load_ocdraw_bytes(bytes.bytes()).unwrap();
    validate_ocdraw_document(loaded.document()).unwrap();
    let target = ocdraw_document_to_cad_document(loaded.document(), Default::default()).unwrap();
    assert_eq!(
        target
            .document()
            .entities()
            .filter(|e| matches!(e, EntityType::Dimension(_)))
            .count(),
        2,
        "{:?}",
        target.diagnostics()
    );
    for format in [0, 1, 2] {
        let returned = match format {
            0 => DxfReader::from_reader(Cursor::new(
                DxfWriter::new(target.document()).write_to_vec().unwrap(),
            ))
            .unwrap()
            .read()
            .unwrap(),
            1 => DxfReader::from_reader(Cursor::new(
                DxfWriter::new_binary(target.document())
                    .write_to_vec()
                    .unwrap(),
            ))
            .unwrap()
            .read()
            .unwrap(),
            _ => DwgReader::from_stream(Cursor::new(
                DwgWriter::write_to_vec(target.document()).unwrap(),
            ))
            .read()
            .unwrap(),
        };
        let read = cad_document_to_ocdraw_document(&returned, Default::default()).unwrap();
        assert_eq!(
            read.document().dimension_entities.len(),
            2,
            "{:?}",
            read.diagnostics()
        );
        assert!(read.document().dimension_entities.iter().any(|d| d
            .overrides
            .numbers
            .primary
            .fraction_layout
            == Some(DimensionFractionLayout::Diagonal)));
    }
}

#[test]
fn conflicting_fixed_height_is_diagnosed() {
    let cad = repair::drawing();
    let native = cad_document_to_ocdraw_document(&cad, Default::default()).unwrap();
    let mut doc = native.document().clone();
    doc.dimension_styles[0].settings.text.height = 51.0;
    let target = ocdraw_document_to_cad_document(&doc, Default::default()).unwrap();
    assert_eq!(
        target
            .document()
            .entities()
            .filter(|e| matches!(e, EntityType::Dimension(_)))
            .count(),
        0
    );
    assert!(target
        .diagnostics()
        .iter()
        .any(|d| d.location.contains("fixedHeight")));
}

#[test]
fn unused_dxf_style_with_zero_separator_passes_strict_native_readback() {
    let mut cad = opencadcodec::CadDocument::new();
    cad.dim_styles.get_mut("Standard").unwrap().dimdsep = 0;
    let bytes = DxfWriter::new(&cad).write_to_vec().unwrap();
    let cad = DxfReader::from_reader(Cursor::new(bytes))
        .unwrap()
        .read()
        .unwrap();
    assert_eq!(cad.dim_styles.get("Standard").unwrap().dimdsep, 0);
    let native = cad_document_to_ocdraw_document(&cad, Default::default()).unwrap();
    let bytes = encode_ocdraw_document(native.document()).unwrap();
    let loaded = load_ocdraw_bytes(bytes.bytes()).unwrap();
    validate_ocdraw_document(loaded.document()).unwrap();
    assert_eq!(loaded.document().dimension_entities.len(), 0);
    assert!(!loaded.document().dimension_styles.is_empty());
    assert!(loaded.document().dimension_styles.iter().all(|s| s
        .settings
        .numbers
        .primary
        .decimal_separator
        == '.'));
}
