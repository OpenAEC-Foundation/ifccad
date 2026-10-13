use cad_dimension::*;
use ocdraw::dimension::*;
use opencadcodec::{
    objects::{AssocDimensionAssociation, AssocDimensionReference},
    CadDocument, EntityType, Handle, Line, Vector3,
};

#[test]
fn start_reference_keeps_relation_despite_stale_dimension_definition() {
    let mut doc = CadDocument::new();
    let h = doc
        .add_entity(EntityType::Line(Line::from_points(
            Vector3::new(4500.0, 0.0, 0.0),
            Vector3::new(9000.0, 0.0, 0.0),
        )))
        .unwrap();
    let mut a = AssocDimensionAssociation {
        associativity: 1,
        dimension: Handle::new(90),
        ..Default::default()
    };
    a.references[0].push(AssocDimensionReference {
        class_name: "AcDbOsnapPointRef".into(),
        osnap_type: 13,
        xrefs: vec![h],
        osnap_point: Vector3::new(4000.0, 0.0, 0.0),
        ..Default::default()
    });
    let refs = prepare_point_references(&a, &doc).unwrap();
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].source, h);
    assert_eq!(refs[0].role, DimensionRole::FirstPoint);
    assert_eq!(refs[0].selector, DimensionPointSelector::Start);
}

#[test]
fn ambiguous_endpoint_and_transspace_are_not_guessed() {
    let mut doc = CadDocument::new();
    let h = doc
        .add_entity(EntityType::Line(Line::from_points(
            Vector3::ZERO,
            Vector3::ZERO,
        )))
        .unwrap();
    let mut a = AssocDimensionAssociation {
        associativity: 1,
        ..Default::default()
    };
    a.references[0].push(AssocDimensionReference {
        class_name: "AcDbOsnapPointRef".into(),
        osnap_type: 1,
        xrefs: vec![h],
        ..Default::default()
    });
    assert!(prepare_point_references(&a, &doc).is_err());
    a.trans_space = true;
    assert!(prepare_point_references(&a, &doc).is_err());
}

#[test]
fn emitted_circle_center_snap_is_wcs_instead_of_ocs() {
    let mut doc = CadDocument::new();
    let mut circle = opencadcodec::Circle::new();
    circle.center = Vector3::new(2.0, 7.0, 3.0);
    circle.normal = Vector3::new(1.0, 0.0, 0.0);
    circle.radius = 1.0;
    let source = doc.add_entity(EntityType::Circle(circle)).unwrap();
    let dimension = doc
        .add_entity(EntityType::Dimension(
            opencadcodec::entities::Dimension::Aligned(Default::default()),
        ))
        .unwrap();
    let a = encode_point_references(
        dimension,
        &[CadDimensionPointReference {
            role: DimensionRole::FirstPoint,
            source,
            selector: DimensionPointSelector::Center,
        }],
        &doc,
    )
    .unwrap();
    assert_eq!(a.references[0][0].osnap_point, Vector3::new(3.0, 2.0, 7.0));
}
