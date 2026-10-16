use opencadcodec::{
    entities::{Dimension, DimensionAligned, DimensionLinear},
    xdata::{ExtendedDataRecord, XDataValue as X},
    CadDocument, EntityType, Vector3,
};

pub fn drawing() -> CadDocument {
    let mut doc = CadDocument::new();
    doc.text_styles.get_mut("Standard").unwrap().height = 50.0;
    let style = doc.dim_styles.get_mut("Standard").unwrap();
    style.dimtad = 1;
    style.dimtvp = 1.0;
    style.dimzin = 8;
    style.dimmzf = 100.0;
    style.dimscale = 2.0;
    let mut d = DimensionAligned::new(Vector3::ZERO, Vector3::new(48.25, 0.0, 0.0));
    d.definition_point = Vector3::new(0.0, 10.0, 0.0);
    d.base.definition_point = d.definition_point;
    d.base.common.extended_data.add_record(ExtendedDataRecord {
        application_name: "ACAD".into(),
        values: vec![
            X::String("DSTYLE".into()),
            X::ControlString("{".into()),
            X::Integer16(276),
            X::Integer16(1),
            X::Integer16(277),
            X::Integer16(4),
            X::Integer16(146),
            X::Real(0.75),
            X::Integer16(176),
            X::Integer16(1),
            X::ControlString("}".into()),
        ],
    });
    doc.add_entity(EntityType::Dimension(Dimension::Aligned(d)))
        .unwrap();
    let mut d = DimensionLinear::new(Vector3::ZERO, Vector3::new(4000.0, 0.0, 0.0));
    d.base.definition_point = Vector3::new(0.0, 300.0, 0.0);
    d.base.common.extended_data.add_record(ExtendedDataRecord {
        application_name: "ACAD".into(),
        values: vec![
            X::String("DSTYLE".into()),
            X::ControlString("{".into()),
            X::Integer16(49),
            X::Real(20.0),
            X::Integer16(290),
            X::Integer16(1),
            X::ControlString("}".into()),
        ],
    });
    doc.add_entity(EntityType::Dimension(Dimension::Linear(d)))
        .unwrap();
    doc
}
