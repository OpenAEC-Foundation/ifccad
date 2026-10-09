use ifccad_convert::*;
use ocdraw::ifccad::*;
use opencadcodec::EntityType;
#[path = "../../../tests/support/ifccad_hatch.rs"]
mod fixture;
fn metadata() -> IfccadTargetMetadata {
    let d = fixture::drawing();
    IfccadTargetMetadata {
        header: d.header,
        drawing_id: d.drawing_id,
    }
}

#[test]
fn user_defined_active_linetype_is_resolved_in_the_document() {
    use ocdraw::geometry_kernel::hatch::HatchFill;
    use opencadcodec::{entities::hatch::*, Vector2};
    for (mode, accepted) in [
        ("Continuous", true),
        ("ByLayer", true),
        ("ByBlock", false),
        ("DashedLayer", false),
    ] {
        for double in [false, true] {
            let cad =
                ifccad_document_to_cad_document(&fixture::drawing(), Default::default()).unwrap();
            let hh = cad.mappings().entities.cad_handle(fixture::HATCH).unwrap();
            let mut d = cad.document().clone();
            if mode == "DashedLayer" {
                let mut lt = opencadcodec::LineType::new("ActiveDash");
                lt.handle = d.allocate_handle();
                lt.elements
                    .push(opencadcodec::tables::LineTypeElement::dash(1.));
                lt.elements
                    .push(opencadcodec::tables::LineTypeElement::space(1.));
                lt.pattern_length = 2.;
                d.line_types.add(lt).unwrap();
                let layer = d.get_entity(hh).unwrap().common().layer.clone();
                d.layers.get_mut(&layer).unwrap().line_type = "ActiveDash".into();
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
            h.common.linetype_handle = None;
            h.pattern = HatchPattern::new("U");
            h.pattern.lines.push(HatchPatternLine {
                angle: 0.,
                base_point: Vector2::ZERO,
                offset: Vector2::new(0., 1.),
                dash_lengths: vec![],
            });
            let out = cad_document_to_ifccad_document(&d, metadata(), Default::default()).unwrap();
            let fills = out
                .document()
                .model
                .entities
                .iter()
                .filter_map(IfccadEntity::as_native)
                .filter_map(|e| {
                    if let IfccadEntityKind::Hatch(h) = &e.kind {
                        Some(&h.fill)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            assert_eq!(fills.len(), usize::from(accepted), "{mode} double={double}");
            if accepted {
                let HatchFill::LinePattern(p) = fills[0] else {
                    panic!()
                };
                assert_eq!(p.families.len(), if double { 2 } else { 1 });
            }
        }
    }
}
#[test]
fn independent_route_binds_forward_source_and_preserves_the_hole() {
    for reverse in [false, true] {
        let mut d = fixture::drawing();
        if reverse {
            d.model.entities.reverse();
        }
        let cad = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
        let h = cad.mappings().entities.cad_handle(fixture::HATCH).unwrap();
        let source = cad.mappings().entities.cad_handle(fixture::SOURCE).unwrap();
        let EntityType::Hatch(hatch) = cad.document().get_entity(h).unwrap() else {
            panic!()
        };
        assert_eq!(hatch.paths[1].boundary_handles, vec![source]);
        let back = cad_document_to_ifccad_document(cad.document(), metadata(), Default::default())
            .unwrap();
        let hatch = back
            .document()
            .model
            .entities
            .iter()
            .filter_map(IfccadEntity::as_native)
            .find_map(|e| {
                if let IfccadEntityKind::Hatch(h) = &e.kind {
                    Some(h)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(hatch.loops.len(), 2);
        assert_eq!(
            hatch.loops[1].source_entity_id,
            back.mappings().entities.ifccad_id(source)
        );
        let bytes = encode_ifccad_document(back.document()).unwrap();
        assert!(load_ifccad_bytes(bytes.bytes(), Default::default()).is_ok());
        assert!(!back.geometry_assessment().is_complete());
    }
}
#[test]
fn unsupported_relation_detaches_only_source_and_nondefault_limit_is_loss() {
    let mut d = fixture::drawing();
    let IfccadEntityKind::Hatch(h) = &mut d.model.entities[0].as_native_mut().unwrap().kind else {
        panic!()
    };
    h.join_tolerance = 0.;
    let out = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "hatch-join-policy"));
    assert!(matches!(
        ifccad_document_to_cad_document(
            &d,
            IfccadToCadOptions {
                loss_policy: IfccadLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(IfccadConversionError::Unsupported(_))
    ));
    let mut cad = out.into_document();
    let hh = cad
        .entities()
        .find_map(|e| {
            if let EntityType::Hatch(h) = e {
                Some(h.common.handle)
            } else {
                None
            }
        })
        .unwrap();
    let EntityType::Hatch(h) = cad.get_entity_mut(hh).unwrap() else {
        panic!()
    };
    let source = h.paths[1].boundary_handles[0];
    h.paths[1].boundary_handles.push(source);
    let back = cad_document_to_ifccad_document(&cad, metadata(), Default::default()).unwrap();
    let IfccadEntityKind::Hatch(h) = &back.document().model.entities[0].as_native().unwrap().kind
    else {
        panic!()
    };
    assert_eq!(h.loops.len(), 2);
    assert_eq!(h.loops[1].source_entity_id, None);
    assert!(back.diagnostics().iter().any(|d| d.code == "hatch-source"));
    assert!(matches!(
        cad_document_to_ifccad_document(
            &cad,
            metadata(),
            CadToIfccadOptions {
                loss_policy: IfccadLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(IfccadConversionError::Unsupported(_))
    ));
}
#[test]
fn omitted_hatch_defaults_are_native_but_foreign_graph_data_remains_loss() {
    let d = fixture::drawing();
    let bytes = encode_ifccad_document(&d).unwrap();
    let mut raw: serde_json::Value = serde_json::from_slice(bytes.bytes()).unwrap();
    for node in raw["data"].as_array_mut().unwrap() {
        if let Some(h) = node["attributes"].get_mut("ifccad::hatch") {
            h.as_object_mut().unwrap().remove("areaRule");
            h.as_object_mut().unwrap().remove("joinTolerance");
            h["loops"][0]["boundary"]
                .as_object_mut()
                .unwrap()
                .remove("bulges");
        }
    }
    let loaded = load_ifccad_bytes(&serde_json::to_vec(&raw).unwrap(), Default::default()).unwrap();
    let out = ifccad_source_to_cad_document(&loaded, Default::default()).unwrap();
    assert!(!out.diagnostics().iter().any(|d| d.code == "foreign-ifcx"));
    raw["data"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"path":"/foreign/hatch-test","attributes":{}}));
    let loaded = load_ifccad_bytes(&serde_json::to_vec(&raw).unwrap(), Default::default()).unwrap();
    let out = ifccad_source_to_cad_document(&loaded, Default::default()).unwrap();
    assert!(out.diagnostics().iter().any(|d| d.code == "foreign-ifcx"));
}

#[test]
fn omitted_pattern_defaults_are_native_but_foreign_graph_data_remains_loss() {
    let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../conformance/next/ifccad/valid/hatch-pattern.ifcx"
    ))
    .unwrap();
    for node in raw["data"].as_array_mut().unwrap() {
        if let Some(h) = node["attributes"].get_mut("ifccad::hatch") {
            for field in ["origin", "rotation", "scale"] {
                h["fill"].as_object_mut().unwrap().remove(field);
            }
        }
    }
    let loaded = load_ifccad_bytes(&serde_json::to_vec(&raw).unwrap(), Default::default()).unwrap();
    let out = ifccad_source_to_cad_document(&loaded, Default::default()).unwrap();
    assert!(!out.diagnostics().iter().any(|d| d.code == "foreign-ifcx"));
    raw["data"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"path":"/foreign/pattern-test","attributes":{}}));
    let loaded = load_ifccad_bytes(&serde_json::to_vec(&raw).unwrap(), Default::default()).unwrap();
    let out = ifccad_source_to_cad_document(&loaded, Default::default()).unwrap();
    assert!(out.diagnostics().iter().any(|d| d.code == "foreign-ifcx"));
}
#[test]
fn paper_producer_limit_uses_fixed_mapping_without_changing_accuracy_budget() {
    use ocdraw::{
        geometry_kernel::hatch::*,
        plot_kernel::{PlotScale, PlotUnit},
    };
    let mut d = load_ifccad_bytes(
        include_bytes!("../../../examples/ifccad/hello-paper-layouts.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let mut e = d
        .model
        .entities
        .iter()
        .find_map(IfccadEntity::as_native)
        .unwrap()
        .clone();
    e.id = d.id_counters.allocate_entity_id().unwrap();
    e.kind = IfccadEntityKind::Hatch(IfccadHatch {
        placement: fixture::plane(),
        loops: vec![IfccadHatchLoop {
            boundary: HatchBoundary2::Circle {
                center: [0., 0.],
                radius: 1.,
            },
            source_entity_id: None,
        }],
        area_rule: HatchAreaRule::Normal,
        join_tolerance: 1e-9,
        fill: HatchFill::Solid,
    });
    d.model.entities.clear();
    d.blocks.clear();
    d.workspace_state = None;
    d.model_windows.clear();
    d.model_view_state = None;
    d.paper_layouts.retain(|l| {
        l.settings
            .plot_settings
            .as_ref()
            .is_some_and(|p| p.plot_unit == PlotUnit::Millimetre)
    });
    let plot = d.paper_layouts[0].settings.plot_settings.as_mut().unwrap();
    plot.plot_unit = PlotUnit::Millimetre;
    plot.mapping.scale = PlotScale::Fixed {
        output_length: 2.,
        scope_length: 1.,
    };
    d.paper_layouts[0].bounds = None;
    d.paper_layouts[0].entities = vec![IfccadEntity::Native(e)];
    let out = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    let back = cad_document_to_ifccad_document(
        out.document(),
        metadata(),
        CadToIfccadOptions {
            hatch_join_tolerance: HatchJoinToleranceRequest::Millimetres {
                value: 1.,
                coordinate_fallback: None,
            },
            ..Default::default()
        },
    )
    .unwrap();
    let h = back.document().paper_layouts[0]
        .entities
        .iter()
        .filter_map(IfccadEntity::as_native)
        .find_map(|e| {
            if let IfccadEntityKind::Hatch(h) = &e.kind {
                Some(h)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(h.join_tolerance, 0.5);
    assert!(back
        .geometry_assessment()
        .domains()
        .iter()
        .all(|d| d.requested_tolerance() == IfccadGeometryTolerance::default()));
}
