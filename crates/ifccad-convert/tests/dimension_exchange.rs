mod common;
use ifccad_convert::*;
use ocdraw::{dimension::*, ifccad::*};
use opencadcodec::{DwgReader, DwgWriter, DxfReader, DxfWriter};
use std::io::Cursor;
#[test]
fn independent_native_paths_roundtrip_through_actual_dxf_and_ac1032_dwg() {
    let source = load_ifccad_bytes(
        include_bytes!("../../../conformance/next/ifccad/valid/dimension-linear-aligned.ifcx"),
        Default::default(),
    )
    .unwrap();
    validate_ifccad_document(source.document()).unwrap();
    let target = ifccad_source_to_cad_document(&source, Default::default()).unwrap();
    for dwg in [false, true] {
        let cad = if dwg {
            let bytes = DwgWriter::write_to_vec(target.document()).unwrap();
            assert_eq!(&bytes[..6], b"AC1032");
            DwgReader::from_stream(Cursor::new(bytes)).read().unwrap()
        } else {
            DxfReader::from_reader(Cursor::new(
                DxfWriter::new(target.document()).write_to_vec().unwrap(),
            ))
            .unwrap()
            .read()
            .unwrap()
        };
        let native =
            cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
        let doc = native.validated_source().document();
        let dims = doc
            .model
            .entities
            .iter()
            .filter_map(|e| {
                if let IfccadEntityKind::Dimension(d) = &e.as_native()?.kind {
                    Some(d)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(dims.len(), 2, "dwg={dwg}");
        assert!(dims
            .iter()
            .all(|d| evaluate_dimension(&d.definition).unwrap()
                == DimensionMeasurement::Distance(4000.0)));
        assert!(dims.iter().all(|d| d.associations.len() == 2));
        assert_eq!(native.dimension_assessment().entries().len(), 2);
        assert!(!native.geometry_assessment().is_complete());
    }
}
