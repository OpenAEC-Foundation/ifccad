mod common;
#[path = "../../../tests/support/cad_dimension_repair.rs"]
mod repair;
use ifccad_convert::*;
use ocdraw::{dimension::*, ifccad::*};
use opencadcodec::{DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType};
use std::io::Cursor;

fn count(doc: &IfccadDocument) -> usize {
    doc.model
        .entities
        .iter()
        .filter(|e| {
            matches!(
                e.as_native().map(|e| &e.kind),
                Some(IfccadEntityKind::Dimension(_))
            )
        })
        .count()
}
#[test]
fn repair_profiles_survive_strict_native_readback_and_cad_return() {
    let cad = repair::drawing();
    let original = cad.text_styles.get("Standard").unwrap().height;
    let native =
        cad_document_to_ifccad_document(&cad, common::metadata(), Default::default()).unwrap();
    assert_eq!(count(native.document()), 2);
    assert_eq!(cad.text_styles.get("Standard").unwrap().height, original);
    let bytes = encode_ifccad_document(native.document()).unwrap();
    let loaded = load_ifccad_bytes(bytes.bytes(), Default::default()).unwrap();
    validate_ifccad_document(loaded.document()).unwrap();
    let target = ifccad_source_to_cad_document(&loaded, Default::default()).unwrap();
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
        let read =
            cad_document_to_ifccad_document(&returned, common::metadata(), Default::default())
                .unwrap();
        assert_eq!(count(read.document()), 2, "{:?}", read.diagnostics());
        assert!(read.document().model.entities.iter().any(|e|matches!(e.as_native().map(|e|&e.kind),Some(IfccadEntityKind::Dimension(d)) if d.overrides.numbers.primary.fraction_layout==Some(DimensionFractionLayout::Diagonal))));
    }
}

#[test]
fn omitted_dimension_has_entity_loss_action() {
    let mut cad = repair::drawing();
    cad.dim_styles.get_mut("Standard").unwrap().annotative = true;
    let out =
        cad_document_to_ifccad_document(&cad, common::metadata(), Default::default()).unwrap();
    assert_eq!(count(out.document()), 0);
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.location.starts_with("entity/")
            && d.code == "dimension-metadata"
            && d.action == IfccadDiagnosticAction::Omitted));
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
    let native =
        cad_document_to_ifccad_document(&cad, common::metadata(), Default::default()).unwrap();
    let bytes = encode_ifccad_document(native.document()).unwrap();
    let loaded = load_ifccad_bytes(bytes.bytes(), Default::default()).unwrap();
    validate_ifccad_document(loaded.document()).unwrap();
    assert_eq!(count(loaded.document()), 0);
    assert!(!loaded.document().dimension_styles.is_empty());
    assert!(loaded.document().dimension_styles.iter().all(|s| s
        .settings
        .numbers
        .primary
        .decimal_separator
        == '.'));
}
