use ocdraw_convert::*;
use opencadcodec::entities::hatch::*;
use opencadcodec::{CadDocument, Circle, EntityType, Vector2};

fn source(
    hatch_first: bool,
    multiple: bool,
) -> (CadDocument, opencadcodec::Handle, opencadcodec::Handle) {
    let mut d = CadDocument::new();
    let mut h = Hatch::solid();
    let mut p = BoundaryPath::new();
    p.add_edge(BoundaryEdge::CircularArc(CircularArcEdge {
        center: Vector2::ZERO,
        radius: 3.,
        start_angle: 0.,
        end_angle: std::f64::consts::TAU,
        counter_clockwise: true,
    }));
    h.paths.push(p);
    h.is_associative = true;
    let hh = if hatch_first {
        Some(d.add_entity(EntityType::Hatch(h.clone())).unwrap())
    } else {
        None
    };
    let mut circle = Circle::new();
    circle.radius = 3.;
    let ch = d.add_entity(EntityType::Circle(circle)).unwrap();
    let hh = hh.unwrap_or_else(|| d.add_entity(EntityType::Hatch(h)).unwrap());
    let EntityType::Hatch(h) = d.get_entity_mut(hh).unwrap() else {
        panic!()
    };
    h.paths[0].boundary_handles = if multiple { vec![ch, ch] } else { vec![ch] };
    (d, hh, ch)
}

#[test]
fn user_defined_active_linetype_is_resolved_in_the_document() {
    use ocdraw::geometry_kernel::hatch::HatchFill;
    for (mode, accepted) in [
        ("Continuous", true),
        ("ByLayer", true),
        ("ByBlock", false),
        ("DashedLayer", false),
    ] {
        for double in [false, true] {
            let (mut d, hh, _) = source(true, false);
            if mode == "DashedLayer" {
                let mut lt = opencadcodec::LineType::new("ActiveDash");
                lt.handle = d.allocate_handle();
                lt.elements
                    .push(opencadcodec::tables::LineTypeElement::dash(1.));
                lt.elements
                    .push(opencadcodec::tables::LineTypeElement::space(1.));
                lt.pattern_length = 2.;
                d.line_types.add(lt).unwrap();
                d.layers.get_mut("0").unwrap().line_type = "ActiveDash".into();
            }
            let EntityType::Hatch(h) = d.get_entity_mut(hh).unwrap() else {
                panic!()
            };
            h.is_solid = false;
            h.pattern_type = HatchPatternType::UserDefined;
            h.is_double = double;
            h.common.linetype = if mode == "DashedLayer" {
                "ByLayer"
            } else {
                mode
            }
            .into();
            h.pattern = HatchPattern::new("U");
            h.pattern.lines.push(HatchPatternLine {
                angle: 0.,
                base_point: Vector2::ZERO,
                offset: Vector2::new(0., 1.),
                dash_lengths: vec![],
            });
            let out =
                cad_document_to_ocdraw_document_with_id(&d, "user-pattern", Default::default())
                    .unwrap();
            assert_eq!(
                out.document().hatch_entities.len(),
                usize::from(accepted),
                "{mode} double={double}"
            );
            if accepted {
                let HatchFill::LinePattern(p) = &out.document().hatch_entities[0].fill else {
                    panic!()
                };
                assert_eq!(p.families.len(), if double { 2 } else { 1 });
            }
        }
    }
}
#[test]
fn source_binding_is_independent_of_draw_order_and_strict_readback() {
    for first in [true, false] {
        let (d, hh, ch) = source(first, false);
        let out =
            cad_document_to_ocdraw_document_with_id(&d, "hatch", CadToOcdrawOptions::default())
                .unwrap();
        assert_eq!(out.document().hatch_entities.len(), 1);
        let h = &out.document().hatch_entities[0];
        assert_eq!(h.id, out.entity_mapping()[&hh]);
        assert_eq!(h.loops[0].source_entity_id, Some(out.entity_mapping()[&ch]));
        let encoded = ocdraw::ocdraw::encode_ocdraw_document(out.document()).unwrap();
        assert!(ocdraw::ocdraw::load_ocdraw_bytes(encoded.bytes()).is_ok());
        assert!(
            !out.geometry_assessment().is_complete(),
            "fill evaluation is unassessed"
        );
    }
}
#[test]
fn multiple_sources_detach_only_the_relation_and_reject_refuses_loss() {
    let (d, _, _) = source(true, true);
    let out = cad_document_to_ocdraw_document_with_id(&d, "hatch", CadToOcdrawOptions::default())
        .unwrap();
    assert_eq!(out.document().hatch_entities.len(), 1);
    assert_eq!(
        out.document().hatch_entities[0].loops[0].source_entity_id,
        None
    );
    assert!(!out.diagnostics().is_empty());
    assert!(matches!(
        cad_document_to_ocdraw_document_with_id(
            &d,
            "hatch",
            CadToOcdrawOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(CadToOcdrawError::LossRejected { .. })
    ));
}

#[test]
fn large_native_ids_and_stale_relations_export_stored_contours() {
    let (d, _, _) = source(true, false);
    let out = cad_document_to_ocdraw_document_with_id(&d, "hatch", CadToOcdrawOptions::default())
        .unwrap();
    let mut n = out.into_document();
    let old_h = n.hatch_entities[0].id;
    let old_c = n.hatch_entities[0].loops[0].source_entity_id.unwrap();
    let h = 9_007_199_254_740_993;
    let c = h + 1;
    n.hatch_entities[0].id = h;
    n.hatch_entities[0].loops[0].source_entity_id = Some(c);
    let pattern = ocdraw::ocdraw::load_ocdraw_bytes(include_bytes!(
        "../../../conformance/next/ocdraw/valid/hatch-pattern.ocdraw.json"
    ))
    .unwrap();
    n.hatch_entities[0].fill = pattern.hatch_entities()[0].fill.clone();
    let circle = n
        .geometric_entities
        .iter_mut()
        .find(|e| e.id() == old_c)
        .unwrap();
    circle.id = c;
    let ocdraw::ocdraw::DrawingGeometry::Circle { radius, .. } = &mut circle.geometry else {
        panic!()
    };
    *radius = 7.;
    for s in &mut n.scopes {
        for id in &mut s.entities {
            if *id == old_h {
                *id = h;
            } else if *id == old_c {
                *id = c;
            }
        }
    }
    n.next_entity_id = c + 1;
    ocdraw::ocdraw::recompute_ocdraw_document_bounds(&mut n).unwrap();
    let out = ocdraw_document_to_cad_document(&n, OcdrawToCadOptions::default()).unwrap();
    let EntityType::Hatch(target) = out.document().get_entity(out.entity_mapping()[&h]).unwrap()
    else {
        panic!()
    };
    assert_eq!(
        target.paths[0].boundary_handles,
        vec![out.entity_mapping()[&c]]
    );
    let BoundaryEdge::CircularArc(a) = &target.paths[0].edges[0] else {
        panic!()
    };
    assert_eq!(a.radius, 3.);
    assert_eq!(target.pattern_origin(), Vector2::new(0.5, -0.5));
    assert_eq!(target.pattern.lines.len(), 2);
    let EntityType::Circle(a) = out.document().get_entity(out.entity_mapping()[&c]).unwrap() else {
        panic!()
    };
    assert_eq!(a.radius, 7.);
}

#[test]
fn native_invalid_reference_is_hard_and_nondefault_limit_is_located_loss() {
    let (d, _, _) = source(false, false);
    let mut n = cad_document_to_ocdraw_document_with_id(&d, "hatch", CadToOcdrawOptions::default())
        .unwrap()
        .into_document();
    n.hatch_entities[0].join_tolerance = 0.;
    let out = ocdraw_document_to_cad_document(&n, OcdrawToCadOptions::default()).unwrap();
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "HATCH_JOIN_POLICY"));
    assert!(matches!(
        ocdraw_document_to_cad_document(
            &n,
            OcdrawToCadOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(OcdrawToCadError::LossRejected { .. })
    ));
    n.hatch_entities[0].loops[0].source_entity_id = Some(u64::MAX - 1);
    for policy in [OcdrawLossPolicy::Allow, OcdrawLossPolicy::Reject] {
        assert!(matches!(
            ocdraw_document_to_cad_document(
                &n,
                OcdrawToCadOptions {
                    loss_policy: policy,
                    ..Default::default()
                }
            ),
            Err(OcdrawToCadError::InvalidDocument(_))
        ));
    }
}

#[test]
fn active_gradient_skips_the_whole_hatch_and_invalid_creation_limit_is_hard() {
    let (mut d, hh, _) = source(false, false);
    let EntityType::Hatch(h) = d.get_entity_mut(hh).unwrap() else {
        panic!()
    };
    h.gradient_color.enabled = true;
    let out = cad_document_to_ocdraw_document_with_id(&d, "hatch", CadToOcdrawOptions::default())
        .unwrap();
    assert!(out.document().hatch_entities.is_empty());
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| matches!(d.action(), CadToOcdrawAction::Skipped)));
    assert!(cad_document_to_ocdraw_document_with_id(
        &d,
        "hatch",
        CadToOcdrawOptions {
            hatch_join_tolerance:
                ocdraw::geometry_kernel::hatch::HatchJoinToleranceRequest::Coordinates(-1.),
            ..Default::default()
        }
    )
    .is_err());
}
#[test]
fn unitless_physical_join_request_requires_fallback_and_keeps_numeric_budget() {
    use ocdraw::geometry_kernel::hatch::HatchJoinToleranceRequest;
    let (d, _, _) = source(false, false);
    let options = CadToOcdrawOptions {
        hatch_join_tolerance: HatchJoinToleranceRequest::Millimetres {
            value: 1.,
            coordinate_fallback: None,
        },
        ..Default::default()
    };
    assert!(cad_document_to_ocdraw_document_with_id(&d, "physical", options).is_err());
    let options = CadToOcdrawOptions {
        hatch_join_tolerance: HatchJoinToleranceRequest::Millimetres {
            value: 1.,
            coordinate_fallback: Some(0.25),
        },
        ..Default::default()
    };
    let out = cad_document_to_ocdraw_document_with_id(&d, "fallback", options).unwrap();
    assert_eq!(out.document().hatch_entities[0].join_tolerance, 0.25);
    assert!(out
        .geometry_assessment()
        .domains()
        .iter()
        .all(|d| d.requested_tolerance() == OcdrawGeometryTolerance::default()));
}
