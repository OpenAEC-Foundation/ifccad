#[path = "support/viewport_clips.rs"]
mod support;
use ocdraw::ocdraw::*;
use ocdraw_convert::*;
use opencadcodec::{EntityType, Handle};

#[test]
fn source_clip_families_preserve_order_and_linkage() {
    for kind in support::KINDS {
        for forward in [false, true] {
            let (d, b, v) = support::source(kind, forward, 1.);
            let outcome =
                cad_document_to_encoded_ocdraw(&d, CadToOcdrawOptions::default()).unwrap();
            let loaded = load_ocdraw_bytes(outcome.encoded().bytes()).unwrap();
            assert!(!outcome.diagnostics().iter().flat_map(|d| d.reasons()).any(|r|
                matches!(r, CadToOcdrawLossReason::UnsupportedSemantic { name } if name == "viewport workspace snap/grid/display state")));

            assert_eq!(
                loaded.viewports().len(),
                1,
                "{kind}/{forward}: {:?}",
                outcome.diagnostics()
            );
            let viewport = &loaded.viewports()[0];
            assert_eq!(
                viewport.paper_clip.boundary_entity_id,
                Some(outcome.entity_mapping()[&b])
            );
            assert!(viewport.paper_clip.enabled);
            assert_eq!(viewport.frame.width, 12.);
            assert!(viewport.view_locked);
            let scope = loaded
                .document()
                .scopes
                .iter()
                .find(|s| s.entities.contains(&viewport.id))
                .unwrap();
            assert_eq!(
                scope
                    .entities
                    .iter()
                    .position(|id| *id == outcome.entity_mapping()[&v])
                    .unwrap()
                    < scope
                        .entities
                        .iter()
                        .position(|id| *id == outcome.entity_mapping()[&b])
                        .unwrap(),
                forward
            );
            assert!(
                !loaded
                    .geometric_entities()
                    .iter()
                    .find(|g| g.id == outcome.entity_mapping()[&b])
                    .unwrap()
                    .visible
            );
        }
    }
}
#[test]
fn source_clip_failures_skip_whole_viewport() {
    for failure in [
        "missing",
        "offplane",
        "open",
        "outside",
        "thickness",
        "wrongscope",
        "unsupported",
    ] {
        let (mut d, b, v) = support::source("polyline", true, 1.);
        if failure == "missing" {
            let EntityType::Viewport(vp) = d.get_entity_mut(v).unwrap() else {
                unreachable!()
            };
            vp.clip_boundary_handle = Handle::new(999999);
        } else if failure == "unsupported" {
            let common = d.get_entity(b).unwrap().common().clone();
            *d.get_entity_mut(b).unwrap() = EntityType::Line(opencadcodec::entities::Line {
                common,
                ..opencadcodec::entities::Line::from_coords(0., 0., 0., 1., 1., 0.)
            });
        } else if failure == "wrongscope" {
            // Both owners remain structurally valid; the viewport references a model line.
            let line = d
                .add_entity(EntityType::Line(opencadcodec::entities::Line::from_coords(
                    0., 0., 0., 1., 1., 0.,
                )))
                .unwrap();
            let EntityType::Viewport(vp) = d.get_entity_mut(v).unwrap() else {
                unreachable!()
            };
            vp.clip_boundary_handle = line;
        } else {
            let EntityType::LwPolyline(p) = d.get_entity_mut(b).unwrap() else {
                unreachable!()
            };
            match failure {
                "offplane" => p.elevation = 1.,
                "open" => p.is_closed = false,
                "outside" => {
                    p.vertices[0].bulge = 20.;
                }
                "thickness" => p.thickness = 1.,
                _ => unreachable!(),
            }
        }
        let o = cad_document_to_ocdraw_document(&d, CadToOcdrawOptions::default()).unwrap();
        assert!(o.document().viewports.is_empty(), "{failure}");
        assert!(!o.entity_mapping().contains_key(&v));
        assert_eq!(
            o.entity_mapping().contains_key(&b),
            failure != "thickness",
            "independent boundary: {failure}"
        );
        assert!(o.diagnostics().iter().any(|diag|matches!(diag.source(),CadToOcdrawDiagnosticSource::Entity{handle,..} if *handle==v)),"{failure}");
        assert!(
            matches!(
                cad_document_to_ocdraw_document(
                    &d,
                    CadToOcdrawOptions {
                        loss_policy: OcdrawLossPolicy::Reject,
                        ..Default::default()
                    }
                ),
                Err(CadToOcdrawError::LossRejected { .. })
            ),
            "{failure}"
        );
    }
}

#[test]
fn overall_canvas_is_not_an_authored_boundary_claimant() {
    let (mut d, b, _) = support::source("circle", true, 1.);
    let canvas = d
        .entities()
        .find_map(|e| {
            if let EntityType::Viewport(v) = e {
                (v.id == 1).then_some(v.common.handle)
            } else {
                None
            }
        })
        .unwrap();
    let EntityType::Viewport(v) = d.get_entity_mut(canvas).unwrap() else {
        unreachable!()
    };
    v.clip_boundary_handle = b;
    let o = cad_document_to_ocdraw_document(&d, CadToOcdrawOptions::default()).unwrap();
    assert_eq!(o.document().viewports.len(), 1);
}
#[test]
fn shared_source_clip_rejects_every_claimant() {
    for forward in [false, true] {
        let (mut d, _, v) = support::source("circle", forward, 1.);
        let mut second = d.get_entity(v).unwrap().clone();
        second.common_mut().handle = Handle::NULL;
        if let EntityType::Viewport(v) = &mut second {
            v.id = 3;
            // A dormant claimant still participates in ownership uniqueness.
            v.status = opencadcodec::entities::ViewportStatusFlags::from_bits(
                v.status.to_bits() & !0x10000,
            );
        }
        let second = d.add_entity_to_layout(second, "Layout1").unwrap();
        let o = cad_document_to_ocdraw_document(&d, CadToOcdrawOptions::default()).unwrap();
        assert!(o.document().viewports.is_empty());
        for h in [v, second] {
            assert!(!o.entity_mapping().contains_key(&h));
        }
    }
}
#[test]
fn clip_width_loss_is_retained() {
    let (mut d, b, _) = support::source("polyline", true, 1.);
    let EntityType::LwPolyline(p) = d.get_entity_mut(b).unwrap() else {
        unreachable!()
    };
    p.constant_width = 1.;
    let o = cad_document_to_ocdraw_document(&d, CadToOcdrawOptions::default()).unwrap();
    assert_eq!(o.document().viewports.len(), 1);
    assert!(o.diagnostics().iter().any(|d| d
        .reasons()
        .iter()
        .any(|r| matches!(r, CadToOcdrawLossReason::PolylineWidth))));
    assert!(matches!(
        cad_document_to_ocdraw_document(
            &d,
            CadToOcdrawOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(CadToOcdrawError::LossRejected { .. })
    ));
}
#[test]
fn clip_accuracy_failure_remains_fatal() {
    let (d, _, _) = support::source("ellipse", true, 1.);
    assert!(matches!(
        cad_document_to_ocdraw_document(
            &d,
            CadToOcdrawOptions {
                geometry_tolerance: OcdrawGeometryTolerance::drawing_units(0.).unwrap(),
                ..Default::default()
            }
        ),
        Err(CadToOcdrawError::Geometry(_))
    ));
}
#[test]
fn target_forward_clips_preserve_order_and_linkage() {
    for kind in support::KINDS {
        for forward in [false, true] {
            let d = support::native(kind, forward);
            validate_ocdraw_document(&d).unwrap();
            let o = ocdraw_document_to_cad_document(&d, OcdrawToCadOptions::default()).unwrap();
            let EntityType::Viewport(v) = o
                .document()
                .get_entity(o.entity_mapping()[&1])
                .expect("viewport retained")
            else {
                unreachable!()
            };
            assert_eq!(v.clip_boundary_handle, o.entity_mapping()[&2]);
            assert_ne!(v.status.to_bits() & 0x10000, 0);
            let owner = v.common.owner_handle;
            let record = o
                .document()
                .block_records
                .iter()
                .find(|r| r.handle == owner)
                .unwrap();
            let actual = record
                .entity_handles
                .iter()
                .filter_map(|h| {
                    o.entity_mapping()
                        .iter()
                        .find_map(|(id, m)| (*m == *h).then_some(*id))
                })
                .collect::<Vec<_>>();
            assert_eq!(actual, d.scopes[1].entities);
        }
    }
}
#[test]
fn dormant_target_clip_preserves_reference_without_activation() {
    let mut d = support::native("circle", true);
    d.viewports[0].paper_clip.enabled = false;
    let o = ocdraw_document_to_cad_document(&d, OcdrawToCadOptions::default()).unwrap();
    let EntityType::Viewport(v) = o.document().get_entity(o.entity_mapping()[&1]).unwrap() else {
        unreachable!()
    };
    assert_eq!(v.clip_boundary_handle, o.entity_mapping()[&2]);
    assert_eq!(v.status.to_bits() & 0x10000, 0);
    assert!(!o
        .diagnostics()
        .iter()
        .any(|d| d.code == "VIEWPORT_UNSUPPORTED"));
    ocdraw_document_to_cad_document(
        &d,
        OcdrawToCadOptions {
            loss_policy: OcdrawLossPolicy::Reject,
            ..Default::default()
        },
    )
    .unwrap();
}

#[test]
fn target_clip_accuracy_failure_returns_no_document() {
    let mut d = support::native("polyline", true);
    d.unit = "unitless".into();
    let placement = CoordinateFrame3::try_new(
        Point3::new(9007199254740992., 0., 0.),
        Vector3::new(1., 0., 0.),
        Vector3::new(0., 1., 0.),
    )
    .unwrap();
    d.geometric_entities[0].geometry = DrawingGeometry::PlanarPolyline {
        placement,
        vertices: vec![[1., 0., 0.], [3., 2., 0.], [5., 0., 0.]],
        closed: true,
        line_pattern_generation: LinePatternGeneration::PerSegment,
    };
    d.viewports[0].frame = DrawingViewportFrame {
        center: Point2::new(9007199254740996., 0.),
        width: 16.,
        height: 8.,
    };
    recompute_ocdraw_document_bounds(&mut d).unwrap();
    validate_ocdraw_document(&d).unwrap();
    assert!(matches!(
        ocdraw_document_to_cad_document(&d, OcdrawToCadOptions::default()),
        Err(OcdrawToCadError::Geometry(_))
    ));
}

#[test]
fn target_rejects_invalid_relationships_before_construction() {
    let mut d = support::native("circle", true);
    d.viewports[0].paper_clip.boundary_entity_id = Some(99);
    assert!(matches!(
        ocdraw_document_to_cad_document(&d, OcdrawToCadOptions::default()),
        Err(OcdrawToCadError::InvalidDocument(_))
    ));
    let mut d = support::native("circle", true);
    d.scopes[1].entities.pop();
    d.scopes[0].entities.push(2);
    assert!(matches!(
        ocdraw_document_to_cad_document(&d, OcdrawToCadOptions::default()),
        Err(OcdrawToCadError::InvalidDocument(_))
    ));
}

#[test]
fn source_clip_activation_is_independent_of_boundary_presence() {
    for enabled in [false, true] {
        for boundary in [false, true] {
            let (mut source, b, v) = support::source("circle", true, 1.);
            let EntityType::Viewport(viewport) = source.get_entity_mut(v).unwrap() else {
                panic!()
            };
            viewport.status = opencadcodec::entities::ViewportStatusFlags::from_bits(
                viewport.status.to_bits() & !0x10000 | if enabled { 0x10000 } else { 0 },
            );
            viewport.clip_boundary_handle = if boundary { b } else { Handle::NULL };
            let result =
                cad_document_to_encoded_ocdraw(&source, CadToOcdrawOptions::default()).unwrap();
            let loaded = load_ocdraw_bytes(result.encoded().bytes()).unwrap();
            if enabled && !boundary {
                assert!(loaded.viewports().is_empty());
                assert!(result.diagnostics().iter().any(|d| matches!(d.source(), CadToOcdrawDiagnosticSource::Entity { handle, .. } if *handle == v)));
                assert!(matches!(
                    cad_document_to_ocdraw_document(
                        &source,
                        CadToOcdrawOptions {
                            loss_policy: OcdrawLossPolicy::Reject,
                            ..Default::default()
                        }
                    ),
                    Err(CadToOcdrawError::LossRejected { .. })
                ));
            } else {
                assert_eq!(loaded.viewports().len(), 1);
                assert_eq!(loaded.viewports()[0].paper_clip.enabled, enabled);
                assert_eq!(
                    loaded.viewports()[0].paper_clip.boundary_entity_id,
                    boundary.then(|| result.entity_mapping()[&b])
                );
            }
        }
    }
}

#[test]
fn dormant_source_boundary_does_not_require_active_clip_eligibility() {
    let (mut source, b, v) = support::source("circle", true, 1.);
    let common = source.get_entity(b).unwrap().common().clone();
    *source.get_entity_mut(b).unwrap() = EntityType::Line(opencadcodec::entities::Line {
        common,
        ..opencadcodec::entities::Line::from_coords(-5., 0., 0., 5., 0., 0.)
    });
    let EntityType::Viewport(viewport) = source.get_entity_mut(v).unwrap() else {
        panic!()
    };
    viewport.status = opencadcodec::entities::ViewportStatusFlags::from_bits(
        viewport.status.to_bits() & !0x10000,
    );
    let result = cad_document_to_encoded_ocdraw(&source, CadToOcdrawOptions::default()).unwrap();
    let loaded = load_ocdraw_bytes(result.encoded().bytes()).unwrap();
    assert_eq!(loaded.viewports().len(), 1);
    assert!(!loaded.viewports()[0].paper_clip.enabled);
    assert_eq!(
        loaded.viewports()[0].paper_clip.boundary_entity_id,
        Some(result.entity_mapping()[&b])
    );
    let imported = ocdraw_source_to_cad_document(&loaded, OcdrawToCadOptions::default()).unwrap();
    let EntityType::Viewport(viewport) = imported
        .document()
        .get_entity(imported.entity_mapping()[&loaded.viewports()[0].id])
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(viewport.status.to_bits() & 0x10000, 0);
    assert!(matches!(
        imported
            .document()
            .get_entity(viewport.clip_boundary_handle),
        Some(EntityType::Line(_))
    ));
}
