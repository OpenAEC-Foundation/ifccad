mod common;
use ifccad_convert::*;
use opencadcodec::{
    entities::{Dimension, DimensionLinear},
    CadDocument, EntityType, Line, LwPolyline, Vector2, Vector3,
};

fn partial() -> CadToIfccadOptions {
    CadToIfccadOptions {
        error_policy: CadImportErrorPolicy::SkipInvalidEntities,
        ..Default::default()
    }
}
fn bad_dimension() -> EntityType {
    let mut d = DimensionLinear::new(Vector3::ZERO, Vector3::new(3., 4., 1.));
    d.base.text_user_positioned = true;
    d.base.text_middle_point = Vector3::new(0., 0., 1.);
    EntityType::Dimension(Box::new(Dimension::Linear(d)))
}
fn bad_polyline() -> EntityType {
    let mut p = LwPolyline::from_points(vec![
        Vector2::new(f64::MAX / 2., 0.),
        Vector2::new(-f64::MAX / 2., 0.),
    ]);
    p.vertices[0].bulge = 2.;
    EntityType::LwPolyline(p)
}
#[test]
fn partial_import_omits_independent_failures_and_strictly_reads_retained_geometry() {
    let mut cad = CadDocument::new();
    let bad = cad.add_entity(bad_dimension()).unwrap();
    let poly = cad.add_entity(bad_polyline()).unwrap();
    let good = cad
        .add_entity(EntityType::Line(Line::from_coords(0., 0., 2., 1., 2., 3.)))
        .unwrap();
    let before = serde_json::to_value(&cad).unwrap();
    assert!(cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).is_err());
    let out = cad_document_to_encoded_ifccad(&cad, common::metadata(), partial()).unwrap();
    let report = out.partial_open_report().unwrap();
    assert_eq!(report.errors.len(), 2);
    for h in [bad, poly] {
        assert!(report
            .entities
            .iter()
            .any(|e| e.source.handle == h.to_string()));
        assert!(out.mappings().entities.ifccad_id(h).is_none());
    }
    assert!(out.mappings().entities.ifccad_id(good).is_some());
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "partial-entity-skipped"));
    assert_eq!(serde_json::to_value(&cad).unwrap(), before);
    let loaded = ifccad::load_ifccad_bytes(out.encoded().bytes(), Default::default()).unwrap();
    let restored = ifccad_source_to_cad_document(&loaded, Default::default()).unwrap();
    assert!(restored
        .document()
        .entities()
        .any(|e| matches!(e,EntityType::Line(l) if l.start.z==2. && l.end.z==3.)));
}
#[test]
fn partial_import_handles_entity_scalar_failure_without_mutating_source() {
    let mut cad = CadDocument::new();
    let bad = cad
        .add_entity(EntityType::Line(Line::from_coords(0., 0., 0., 1., 0., 0.)))
        .unwrap();
    if let EntityType::Line(l) = cad.get_entity_mut(bad).unwrap() {
        l.end.x = f64::NAN;
    }
    cad.add_entity(EntityType::Line(Line::from_coords(0., 0., 0., 2., 0., 0.)))
        .unwrap();
    let out = cad_document_to_encoded_ifccad(&cad, common::metadata(), partial()).unwrap();
    assert_eq!(out.partial_open_report().unwrap().errors.len(), 1);
    assert!(out.mappings().entities.ifccad_id(bad).is_none());
    assert!(matches!(cad.get_entity(bad).unwrap(),EntityType::Line(l) if l.end.x.is_nan()));
}
#[test]
fn partial_import_does_not_waive_reject_or_fatal_source_structure() {
    let mut cad = CadDocument::new();
    cad.add_entity(bad_dimension()).unwrap();
    let mut options = partial();
    options.loss_policy = IfccadLossPolicy::Reject;
    assert!(matches!(
        cad_document_to_encoded_ifccad(&cad, common::metadata(), options),
        Err(IfccadConversionError::Unsupported(_))
    ));
    let h = cad
        .add_entity(EntityType::Line(Line::from_coords(0., 0., 0., 1., 0., 0.)))
        .unwrap();
    cad.get_entity_mut(h).unwrap().common_mut().owner_handle = opencadcodec::Handle::new(999999);
    assert!(matches!(
        cad_document_to_encoded_ifccad(&cad, common::metadata(), partial()),
        Err(IfccadConversionError::InvalidStructure(_))
    ));
}
#[test]
fn clean_partial_import_has_no_error_omissions_and_default_mode_has_no_partial_report() {
    let mut cad = CadDocument::new();
    cad.add_entity(EntityType::Line(Line::from_coords(0., 0., 0., 1., 0., 0.)))
        .unwrap();
    let out = cad_document_to_encoded_ifccad(&cad, common::metadata(), partial()).unwrap();
    assert!(out.partial_open_report().unwrap().errors.is_empty());
    assert!(
        cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default())
            .unwrap()
            .partial_open_report()
            .is_none()
    );
}
#[test]
fn shared_block_keeps_other_members_and_reports_affected_occurrences() {
    let mut cad = ifccad_document_to_cad_document(&common::nested([0.; 3]), Default::default())
        .unwrap()
        .into_document();
    let leaf = cad.block_records.get("Inner").unwrap().entity_handles[0];
    let mut bad = bad_dimension();
    *bad.common_mut() = cad.get_entity(leaf).unwrap().common().clone();
    *cad.get_entity_mut(leaf).unwrap() = bad;
    let out = cad_document_to_encoded_ifccad(&cad, common::metadata(), partial()).unwrap();
    assert!(out.mappings().entities.ifccad_id(leaf).is_none());
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "partial-block-content"));
    assert!(out.partial_open_report().unwrap().errors.iter().any(|e| e
        .source
        .as_ref()
        .unwrap()
        .handle
        == leaf.to_string()));
}
#[test]
fn unprovable_occurrence_is_omitted_without_waiving_local_or_outer_accuracy() {
    let mut cad = ifccad_document_to_cad_document(&common::nested([0.; 3]), Default::default())
        .unwrap()
        .into_document();
    let leaf = cad.block_records.get("Inner").unwrap().entity_handles[0];
    let mut ellipse = opencadcodec::Ellipse::new();
    ellipse.common = cad.get_entity(leaf).unwrap().common().clone();
    ellipse.major_axis = Vector3::new(3., 4., 0.);
    ellipse.minor_axis_ratio = 0.5;
    *cad.get_entity_mut(leaf).unwrap() = EntityType::Ellipse(ellipse);
    let roots = cad
        .block_records
        .get("*Model_Space")
        .unwrap()
        .entity_handles
        .clone();
    for h in &roots {
        if let Some(EntityType::Insert(i)) = cad.get_entity_mut(*h) {
            i.set_x_scale(1e15);
            i.set_y_scale(1e15);
            i.set_z_scale(1e15);
        }
    }
    let mut hatch = opencadcodec::entities::Hatch::solid();
    let mut path = opencadcodec::entities::hatch::BoundaryPath::new();
    path.add_edge(opencadcodec::entities::hatch::BoundaryEdge::CircularArc(
        opencadcodec::entities::hatch::CircularArcEdge {
            center: Vector2::new(0., 0.),
            radius: 1.,
            start_angle: 0.,
            end_angle: std::f64::consts::TAU,
            counter_clockwise: true,
        },
    ));
    path.boundary_handles = roots.clone();
    hatch.paths.push(path);
    let hatch_handle = cad.add_entity(EntityType::Hatch(Box::new(hatch))).unwrap();
    let out = cad_document_to_encoded_ifccad(&cad, common::metadata(), partial()).unwrap();
    assert!(out.mappings().entities.ifccad_id(leaf).is_some());
    for h in roots {
        assert!(out.mappings().entities.ifccad_id(h).is_none());
    }
    assert_eq!(
        out.partial_open_report()
            .unwrap()
            .errors
            .iter()
            .filter(|e| e.phase == "block-occurrence")
            .count(),
        2
    );
    assert!(out.mappings().entities.ifccad_id(hatch_handle).is_some());
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "partial-reference-group"));
}
