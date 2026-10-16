mod common;
use ifccad_convert::*;
use ocdraw::{dimension::*, ifccad::*};
use opencadcodec::{
    entities::{Dimension, DimensionLinear},
    CadDocument, EntityType, Vector3,
};
#[test]
fn independent_dimension_ids_paths_and_measurement_pass_strict_readback() {
    let mut cad = CadDocument::new();
    let mut d = DimensionLinear::new(Vector3::ZERO, Vector3::new(3.0, 4.0, 0.0));
    d.base.actual_measurement = 999.0;
    cad.add_entity(EntityType::Dimension(Box::new(Dimension::Linear(d))))
        .unwrap();
    let out =
        cad_document_to_ifccad_document(&cad, common::metadata(), Default::default()).unwrap();
    let doc = out.document();
    assert_eq!(doc.dimension_styles.len(), 1);
    let IfccadEntityKind::Dimension(d) = &doc.model.entities[0].as_native().unwrap().kind else {
        panic!("Dimension")
    };
    assert_eq!(
        evaluate_dimension(&d.definition).unwrap(),
        DimensionMeasurement::Distance(3.0)
    );
    assert!(d.style_id.0 > 0);
    assert_eq!(doc.drawing_id, 7);
    let encoded = encode_ifccad_document(doc).unwrap();
    let read = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    validate_ifccad_document(read.document()).unwrap();
    assert_eq!(read.document().dimension_styles, doc.dimension_styles);
}
#[test]
fn independent_numeric_excess_is_fatal_under_both_policies() {
    let mut cad = CadDocument::new();
    cad.add_entity(EntityType::Dimension(Box::new(Dimension::Linear(
        DimensionLinear::new(Vector3::ZERO, Vector3::new(3.0, 4.0, 0.1)),
    ))))
    .unwrap();
    for loss_policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        assert!(matches!(
            cad_document_to_ifccad_document(
                &cad,
                common::metadata(),
                CadToIfccadOptions {
                    loss_policy,
                    ..Default::default()
                }
            ),
            Err(IfccadConversionError::Geometry(_))
        ));
    }
}

#[test]
fn independent_forward_relation_keeps_stale_measurement_and_unused_style() {
    use opencadcodec::{objects::*, Line};
    let mut cad = CadDocument::new();
    let dimension = cad
        .add_entity(EntityType::Dimension(Box::new(Dimension::Linear(
            DimensionLinear::new(Vector3::ZERO, Vector3::new(4000.0, 0.0, 0.0)),
        ))))
        .unwrap();
    let source = cad
        .add_entity(EntityType::Line(Line::from_coords(
            4500.0, 0.0, 0.0, 9000.0, 0.0, 0.0,
        )))
        .unwrap();
    let mut a = AssocDimensionAssociation {
        associativity: 1,
        dimension,
        ..Default::default()
    };
    a.references[0].push(AssocDimensionReference {
        class_name: "AcDbOsnapPointRef".into(),
        osnap_type: 13,
        xrefs: vec![source],
        osnap_point: Vector3::ZERO,
        ..Default::default()
    });
    let mut o = AssociativeObject::new("DIMASSOC", "AcDbDimAssoc");
    o.handle = cad.allocate_handle();
    o.owner = dimension;
    o.data = AssociativeData::DimensionAssociation(a);
    cad.objects.insert(o.handle, ObjectType::Associative(o));
    let mut unused = opencadcodec::tables::DimStyle::new("Unused");
    unused.handle = cad.allocate_handle();
    unused.dimlfac = -2.0;
    cad.dim_styles.add(unused).unwrap();
    let result =
        cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
    let doc = result.validated_source().document();
    let e = doc.model.entities[0].as_native().unwrap();
    let IfccadEntityKind::Dimension(d) = &e.kind else {
        panic!("Dimension")
    };
    assert_eq!(
        evaluate_dimension(&d.definition).unwrap(),
        DimensionMeasurement::Distance(4000.0)
    );
    assert_eq!(
        d.associations[0].source_entity_id,
        result.mappings().entities.ifccad_id(source).unwrap()
    );
    assert_eq!(result.mappings().entities.cad_handle(e.id), Some(dimension));
    assert!(doc.dimension_styles.iter().any(|s| s.name == "Unused"
        && s.settings.numbers.primary.application == DimensionFactorApplication::PaperOnly));
}
