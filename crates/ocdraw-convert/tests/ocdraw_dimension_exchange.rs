use ocdraw::{dimension::*, ocdraw::*};
use ocdraw_convert::*;
use opencadcodec::{
    entities::{Dimension, DimensionAligned},
    xdata::{ExtendedDataRecord, XDataValue as X},
    CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType, Vector3,
};
use std::io::Cursor;
#[test]
fn strict_native_readback_and_both_cad_codecs_retain_definition_and_reset() {
    let mut cad = CadDocument::new();
    let mut d = DimensionAligned::new(Vector3::new(3.0, 0.0, 0.0), Vector3::new(3.0, 4.0, 5.0));
    d.definition_point = Vector3::new(3.0, 8.0, 0.0);
    d.base.definition_point = d.definition_point;
    d.base.normal = Vector3::new(1.0, 0.0, 0.0);
    d.base.text_user_positioned = true;
    d.base.text_middle_point = Vector3::new(2.0, 7.0, 3.0);
    d.base.set_text_override(Some("distance <> mm".into()));
    d.base.common.extended_data.add_record(ExtendedDataRecord {
        application_name: "ACAD".into(),
        values: vec![
            X::String("DSTYLE".into()),
            X::ControlString("{".into()),
            X::Integer16(281),
            X::Integer16(0),
            X::ControlString("}".into()),
        ],
    });
    cad.add_entity(EntityType::Dimension(Dimension::Aligned(d)))
        .unwrap();
    let native = cad_document_to_ocdraw_document(&cad, Default::default()).unwrap();
    let encoded = encode_ocdraw_document(native.document()).unwrap();
    let read = load_ocdraw_bytes(encoded.bytes()).unwrap();
    validate_ocdraw_document(read.document()).unwrap();
    let target = ocdraw_document_to_cad_document(read.document(), Default::default()).unwrap();
    assert!(target
        .diagnostics()
        .iter()
        .any(|d| d.location.ends_with("pictureBlock")));
    for dwg in [false, true] {
        let returned = if dwg {
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
        let roundtrip = cad_document_to_ocdraw_document(&returned, Default::default()).unwrap();
        let dimension = &roundtrip.document().dimension_entities[0];
        let expected = &read.document().dimension_entities[0];
        assert_eq!(dimension.definition, expected.definition, "dwg={dwg}");
        assert_eq!(dimension.text_placement, expected.text_placement);
        assert_eq!(dimension.label, expected.label);
        assert_eq!(
            dimension.overrides.dimension_line.suppress_first,
            Some(false)
        );
        assert!(
            matches!(evaluate_dimension(&dimension.definition).unwrap(),DimensionMeasurement::Distance(n) if n>6.0&&n<7.0)
        );
    }
}
