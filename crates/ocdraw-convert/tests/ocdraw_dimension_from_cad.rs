use ocdraw::{dimension::*, ocdraw::*};
use ocdraw_convert::*;
use opencadcodec::{
    entities::{Dimension, DimensionLinear},
    xdata::{ExtendedDataRecord, XDataValue as X},
    CadDocument, EntityType, Vector3,
};

pub fn source() -> CadDocument {
    let mut doc = CadDocument::new();
    let mut d = DimensionLinear::new(Vector3::new(10.0, 20.0, 0.0), Vector3::new(13.0, 24.0, 0.0));
    d.definition_point = Vector3::new(10.0, 30.0, 0.0);
    d.base.definition_point = d.definition_point;
    d.base.actual_measurement = 999.0;
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
    doc.add_entity(EntityType::Dimension(Dimension::Linear(d)))
        .unwrap();
    doc
}
#[test]
fn ordinary_dimensions_strictly_read_back_with_explicit_reset() {
    let cad = source();
    let result = cad_document_to_ocdraw_document(&cad, Default::default()).unwrap();
    let doc = result.document();
    assert_eq!(doc.dimension_entities.len(), 1);
    assert_eq!(doc.dimension_styles.len(), 1);
    let dimension = &doc.dimension_entities[0];
    assert_eq!(
        evaluate_dimension(&dimension.definition).unwrap(),
        DimensionMeasurement::Distance(3.0)
    );
    assert_eq!(
        dimension.overrides.dimension_line.suppress_first,
        Some(false)
    );
    let encoded = encode_ocdraw_document(doc).unwrap();
    let read = load_ocdraw_bytes(encoded.bytes()).unwrap();
    validate_ocdraw_document(read.document()).unwrap();
    assert_eq!(read.document().dimension_entities, doc.dimension_entities);
}
#[test]
fn invalid_dimension_input_is_fatal_even_with_loss_reporting() {
    let mut cad = source();
    let h = cad.entities().next().unwrap().common().handle;
    if let Some(EntityType::Dimension(d)) = cad.get_entity_mut(h) {
        d.base_mut().normal = Vector3::ZERO;
    }
    assert!(cad_document_to_ocdraw_document(&cad, Default::default()).is_err());
}

#[test]
fn out_of_plane_numeric_excess_is_fatal_under_both_policies() {
    let mut cad = source();
    let h = cad.entities().next().unwrap().common().handle;
    if let Some(EntityType::Dimension(Dimension::Linear(d))) = cad.get_entity_mut(h) {
        d.second_point.z = 0.1;
    }
    for loss_policy in [OcdrawLossPolicy::Allow, OcdrawLossPolicy::Reject] {
        assert!(matches!(
            cad_document_to_ocdraw_document(
                &cad,
                CadToOcdrawOptions {
                    loss_policy,
                    ..Default::default()
                }
            ),
            Err(CadToOcdrawError::Geometry(_))
        ));
    }
}

#[test]
fn forward_association_retains_stale_definition_and_unused_style() {
    use opencadcodec::{objects::*, Line};
    let mut cad = source();
    let dim = cad.entities().next().unwrap().common().handle;
    let line = cad
        .add_entity(EntityType::Line(Line::from_coords(
            4500.0, 0.0, 0.0, 9000.0, 0.0, 0.0,
        )))
        .unwrap();
    let mut a = AssocDimensionAssociation {
        associativity: 1,
        dimension: dim,
        ..Default::default()
    };
    a.references[0].push(AssocDimensionReference {
        class_name: "AcDbOsnapPointRef".into(),
        osnap_type: 13,
        xrefs: vec![line],
        osnap_point: Vector3::new(4000.0, 0.0, 0.0),
        ..Default::default()
    });
    let mut o = AssociativeObject::new("DIMASSOC", "AcDbDimAssoc");
    o.handle = cad.allocate_handle();
    o.owner = dim;
    o.data = AssociativeData::DimensionAssociation(a);
    cad.objects.insert(o.handle, ObjectType::Associative(o));
    let mut unused = opencadcodec::tables::DimStyle::new("Unused");
    unused.handle = cad.allocate_handle();
    unused.dimlfac = -2.0;
    cad.dim_styles.add(unused).unwrap();
    let result = cad_document_to_ocdraw_document(&cad, Default::default()).unwrap();
    let d = &result.document().dimension_entities[0];
    assert_eq!(
        evaluate_dimension(&d.definition).unwrap(),
        DimensionMeasurement::Distance(3.0)
    );
    assert_eq!(d.associations.len(), 1);
    assert_eq!(
        d.associations[0].source_entity_id,
        result.entity_mapping()[&line]
    );
    assert!(result
        .document()
        .dimension_styles
        .iter()
        .any(|s| s.name == "Unused"
            && s.settings.numbers.primary.application == DimensionFactorApplication::PaperOnly));
    assert_eq!(result.document().scopes[0].entities[0], d.id);
    assert_eq!(
        result.dimension_assessment().entries()[0].display_coverage,
        OcdrawDimensionDisplayCoverage::Unassessed
    );
    assert!(!result.geometry_assessment().is_complete());
}
