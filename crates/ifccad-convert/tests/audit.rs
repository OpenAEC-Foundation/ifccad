mod common;
use ifccad_convert::*;
use opencadcodec::{
    entities::{Dimension, DimensionLinear},
    CadDocument, EntityType, Line, LwPolyline, Vector2, Vector3,
};

fn bad_polyline() -> EntityType {
    // Scalar-valid inputs whose major-arc enclosure exceeds finite range.
    // Coincident endpoints are supported and must no longer be the error fixture.
    let mut p = LwPolyline::from_points(vec![
        Vector2::new(f64::MAX / 2., 0.),
        Vector2::new(-f64::MAX / 2., 0.),
    ]);
    p.vertices[0].bulge = 2.;
    EntityType::LwPolyline(p)
}
fn bad_dimension() -> EntityType {
    let mut d = DimensionLinear::new(Vector3::ZERO, Vector3::new(3., 4., 1.));
    // XYZ references are supported; an explicit text anchor outside the plane
    // still cannot be represented and must exercise the hard numeric failure.
    d.base.text_user_positioned = true;
    d.base.text_middle_point = Vector3::new(0., 0., 1.);
    EntityType::Dimension(Box::new(Dimension::Linear(d)))
}

#[test]
fn collects_independent_failures_and_reaches_valid_later_entity_without_mutation() {
    // Catches fail-fast auditing, pre-exclusion and mutation of original membership.
    let mut cad = CadDocument::new();
    let p = cad.add_entity(bad_polyline()).unwrap();
    let d = cad.add_entity(bad_dimension()).unwrap();
    let line = cad
        .add_entity(EntityType::Line(Line::from_coords(0., 0., 0., 1., 2., 0.)))
        .unwrap();
    let before = serde_json::to_value(&cad).unwrap();
    let report = diagnose_cad_document(&cad, common::metadata(), Default::default());
    assert!(!report.conversion_ready);
    assert!(report.traversal_complete);
    assert_eq!(report.entity(p).unwrap().status, CadAuditStatus::Failed);
    assert_eq!(report.entity(d).unwrap().status, CadAuditStatus::Failed);
    assert_eq!(report.entity(line).unwrap().status, CadAuditStatus::Passed);
    assert_eq!(report.failures().count(), 2);
    let primitive = report
        .failures()
        .find(|f| f.source.as_ref().is_some_and(|s| s.handle == p.to_string()))
        .unwrap();
    assert_eq!(primitive.detail["preparationCause"], "InvalidGeometry");
    let dimension = report
        .failures()
        .find(|f| f.source.as_ref().is_some_and(|s| s.handle == d.to_string()))
        .unwrap();
    assert_eq!(dimension.detail["reason"], "ProvenExceedance");
    assert_eq!(dimension.detail["vertexIndex"], 3);
    assert_eq!(dimension.detail["deviation"]["lower"], 1.0);
    assert_eq!(serde_json::to_value(&cad).unwrap(), before);
    assert!(matches!(
        cad_document_to_ifccad_document(&cad, common::metadata(), Default::default()),
        Err(IfccadConversionError::Geometry(_))
    ));
    assert!(report
        .events
        .iter()
        .any(|e| e.phase == "finalization" && e.status == CadAuditStatus::Blocked));
}

#[test]
fn clean_audit_matches_successful_production_conversion() {
    let mut cad = CadDocument::new();
    let h = cad
        .add_entity(EntityType::Line(Line::from_coords(0., 0., 0., 1., 2., 0.)))
        .unwrap();
    let report = diagnose_cad_document(&cad, common::metadata(), Default::default());
    assert!(report.conversion_ready, "{:#?}", report.events);
    assert_eq!(report.entity(h).unwrap().status, CadAuditStatus::Passed);
    assert_eq!(report.failures().count(), 0);
    assert!(cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).is_ok());
}

#[test]
fn fatal_source_graph_marks_remaining_entities_blocked() {
    let mut cad = CadDocument::new();
    let h = cad
        .add_entity(EntityType::Line(Line::from_coords(0., 0., 0., 1., 2., 0.)))
        .unwrap();
    cad.get_entity_mut(h).unwrap().common_mut().owner_handle = opencadcodec::Handle::new(999999);
    let report = diagnose_cad_document(&cad, common::metadata(), Default::default());
    assert!(!report.conversion_ready);
    assert!(!report.traversal_complete);
    assert_eq!(report.entity(h).unwrap().status, CadAuditStatus::Blocked);
    assert!(report.failures().any(|f| f.phase == "source-context"));
}

#[test]
fn progress_reports_completed_entities_and_propagates_write_failure() {
    let mut cad = CadDocument::new();
    cad.add_entity(bad_polyline()).unwrap();
    cad.add_entity(bad_dimension()).unwrap();
    let mut seen = Vec::new();
    let report = diagnose_cad_document_with_progress(
        &cad,
        common::metadata(),
        Default::default(),
        &mut |event| {
            seen.push(event.clone());
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(
        seen.iter()
            .filter(|e| e.phase == "entity-preparation")
            .count(),
        2
    );
    assert_eq!(report.failures().count(), 2);
    let result = diagnose_cad_document_with_progress(
        &cad,
        common::metadata(),
        Default::default(),
        &mut |_| Err(std::io::Error::other("journal unavailable")),
    );
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("journal unavailable"));
}

#[test]
fn failed_nested_leaf_blocks_dependents_and_keeps_independent_root_assessed() {
    let mut native = common::nested([0.; 3]);
    native
        .model
        .entities
        .push(common::instance(100, 4, [0.; 3]));
    let mut cad = ifccad_document_to_cad_document(&native, Default::default())
        .unwrap()
        .into_document();
    let leaf = cad.block_records.get("Inner").unwrap().entity_handles[0];
    let common = cad.get_entity(leaf).unwrap().common().clone();
    let mut bad = bad_polyline();
    *bad.common_mut() = common;
    *cad.get_entity_mut(leaf).unwrap() = bad;
    let report = diagnose_cad_document(&cad, common::metadata(), Default::default());
    assert!(report.traversal_complete);
    assert_eq!(report.failures().count(), 1);
    let occurrences = report
        .events
        .iter()
        .filter(|e| e.phase == "block-occurrence")
        .collect::<Vec<_>>();
    assert_eq!(
        occurrences
            .iter()
            .filter(|e| e.status == CadAuditStatus::Blocked)
            .count(),
        3
    );
    assert_eq!(
        occurrences
            .iter()
            .filter(|e| e.status == CadAuditStatus::Passed)
            .count(),
        1
    );
}

#[test]
fn occurrence_errors_are_collected_per_root_after_successful_local_preparation() {
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
    for h in roots {
        if let Some(EntityType::Insert(i)) = cad.get_entity_mut(h) {
            i.set_x_scale(1e15);
            i.set_y_scale(1e15);
            i.set_z_scale(1e15);
        }
    }
    let report = diagnose_cad_document(&cad, common::metadata(), Default::default());
    assert_eq!(report.entity(leaf).unwrap().status, CadAuditStatus::Passed);
    assert_eq!(
        report
            .failures()
            .filter(|f| f.phase == "block-occurrence")
            .count(),
        2
    );
    assert!(!report.conversion_ready);
}

#[test]
fn paper_failure_retains_its_coordinate_domain_and_other_layout_is_checked() {
    let mut cad = CadDocument::new();
    cad.add_layout("PaperAudit").unwrap();
    let bad = cad
        .add_entity_to_layout(bad_dimension(), "PaperAudit")
        .unwrap();
    let good = cad
        .add_entity(EntityType::Line(Line::from_coords(0., 0., 0., 1., 2., 0.)))
        .unwrap();
    let report = diagnose_cad_document(&cad, common::metadata(), Default::default());
    assert_eq!(report.entity(good).unwrap().status, CadAuditStatus::Passed);
    let failure = report
        .failures()
        .find(|f| {
            f.source
                .as_ref()
                .is_some_and(|s| s.handle == bad.to_string())
        })
        .unwrap();
    assert!(failure.detail["domain"]
        .as_str()
        .unwrap()
        .starts_with("PaperLayout("));
    assert!(failure.detail["coordinateMeaning"]
        .as_str()
        .unwrap()
        .contains("PaperCoordinates"));
}

#[test]
fn reject_policy_reports_semantic_loss_while_allow_records_omission() {
    let mut cad = CadDocument::new();
    let h = cad
        .add_entity(EntityType::Ray(opencadcodec::entities::Ray::new(
            Vector3::ZERO,
            Vector3::new(1., 0., 0.),
        )))
        .unwrap();
    let allowed = diagnose_cad_document(&cad, common::metadata(), Default::default());
    assert_eq!(allowed.entity(h).unwrap().status, CadAuditStatus::Omitted);
    assert!(allowed.conversion_ready);
    assert!(!allowed.diagnostics.is_empty());
    let rejected = diagnose_cad_document(
        &cad,
        common::metadata(),
        CadToIfccadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        },
    );
    assert!(!rejected.conversion_ready);
    assert!(rejected.failures().any(|f| f.code == "Unsupported"));
    assert!(!rejected.diagnostics.is_empty());
}

#[test]
fn successful_entity_progress_is_flushed_before_a_later_failure() {
    // An interrupted owner must still expose completed successful checks.
    let mut cad = CadDocument::new();
    let good = cad
        .add_entity(EntityType::Line(Line::from_coords(0., 0., 0., 1., 2., 0.)))
        .unwrap();
    let bad = cad.add_entity(bad_polyline()).unwrap();
    let mut seen = Vec::new();
    diagnose_cad_document_with_progress(
        &cad,
        common::metadata(),
        Default::default(),
        &mut |event| {
            if event.phase == "entity-preparation" {
                seen.push((event.source.as_ref().unwrap().handle.clone(), event.status));
            }
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(
        seen,
        vec![
            (good.to_string(), CadAuditStatus::Passed),
            (bad.to_string(), CadAuditStatus::Failed)
        ]
    );
}

#[test]
fn hatch_boundary_failure_keeps_loop_edge_gap_and_actual_limit() {
    use opencadcodec::entities::hatch::{BoundaryEdge, BoundaryPath, Hatch, LineEdge};
    let mut cad = CadDocument::new();
    let mut h = Hatch::new();
    h.is_solid = true;
    let mut path = BoundaryPath::new();
    path.edges = vec![
        BoundaryEdge::Line(LineEdge {
            start: Vector2::new(0., 0.),
            end: Vector2::new(1., 0.),
        }),
        BoundaryEdge::Line(LineEdge {
            start: Vector2::new(1.1, 0.),
            end: Vector2::new(1., 1.),
        }),
        BoundaryEdge::Line(LineEdge {
            start: Vector2::new(1., 1.),
            end: Vector2::new(0., 0.),
        }),
    ];
    h.paths.push(path);
    let handle = cad.add_entity(EntityType::Hatch(Box::new(h))).unwrap();
    let report = diagnose_cad_document(&cad, common::metadata(), Default::default());
    assert_eq!(
        report.entity(handle).unwrap().status,
        CadAuditStatus::Failed
    );
    let f = report.failures().next().unwrap();
    assert_eq!(f.code, "HatchPreparation/GapExceeded");
    assert_eq!(f.detail["loopIndex"], 0);
    assert!(f.detail["edgeIndex"].is_number());
    assert!(f.detail["gap"]["lower"].as_f64().unwrap() > 0.09);
    assert_eq!(f.detail["gap"]["limit"], 1e-9);
}

#[test]
fn nonfinite_entity_scalars_are_collected_without_blocking_independent_geometry() {
    let mut cad = CadDocument::new();
    let mut bad = Line::from_coords(0., 0., 0., 1., 2., 0.);
    bad.end.x = f64::NAN;
    let a = cad.add_entity(EntityType::Line(bad.clone())).unwrap();
    let b = cad.add_entity(EntityType::Line(bad)).unwrap();
    let good = cad
        .add_entity(EntityType::Line(Line::from_coords(0., 0., 0., 1., 2., 0.)))
        .unwrap();
    let report = diagnose_cad_document(&cad, common::metadata(), Default::default());
    assert!(report.traversal_complete);
    assert_eq!(report.failures().count(), 2);
    assert_eq!(report.entity(a).unwrap().status, CadAuditStatus::Failed);
    assert_eq!(report.entity(b).unwrap().status, CadAuditStatus::Failed);
    assert_eq!(report.entity(good).unwrap().status, CadAuditStatus::Passed);
    assert!(matches!(
        cad_document_to_ifccad_document(&cad, common::metadata(), Default::default()),
        Err(IfccadConversionError::EntitySourceValidation { .. })
    ));
}
