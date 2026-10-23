use ifccad::{geometry_kernel::hatch::*, *};
use ifccad_convert::*;
use opencadcodec::EntityType;
mod common;
#[path = "../../../tests/support/ifccad_hatch.rs"]
mod fixture;

fn drawing(shift: f64) -> IfccadDocument {
    let mut d = fixture::drawing();
    for layer in &mut d.layers {
        layer.appearance.line_weight = 0.09;
    }
    let IfccadEntityKind::Hatch(h) = &mut d.model.entities[0].as_native_mut().unwrap().kind else {
        panic!()
    };
    h.fill = HatchFill::LinePattern(HatchLinePattern {
        name: Some("Original".into()),
        description: Some(String::new()),
        origin: [0., 0.],
        rotation: 0.,
        scale: 1.,
        families: vec![HatchLineFamily {
            angle: 0.,
            base_point: [0., 0.],
            offset: [0., 2.],
            dashes: vec![],
        }],
    });
    h.classic_settings = Some(HatchClassicSettings {
        preferred_fill: Some(HatchClassicFill::LinePattern),
        creation_type: Some(HatchCreationType::Custom),
        name: Some("Original".into()),
        description: Some(String::new()),
        definition_lines: Some(vec![HatchPatternDefinitionLine {
            angle: 0.,
            base_point: [0., shift],
            offset_vector: [0., 2.],
            dash_lengths: vec![],
        }]),
        ..Default::default()
    });
    d
}
fn target(out: &IfccadToCadOutcome) -> &opencadcodec::entities::Hatch {
    let EntityType::Hatch(h) = out
        .document()
        .get_entity(out.mappings().entities.cad_handle(fixture::HATCH).unwrap())
        .unwrap()
    else {
        panic!()
    };
    h
}
#[test]
fn actual_original_candidate_is_selected_within_tolerance_under_both_policies() {
    let d = drawing(2f64.powi(-40));
    // Both independent native encodings must pass production strict readback.
    for cbor in [false, true] {
        let encoded = if cbor {
            encode_ifccad_cbor_document(&d).unwrap().bytes().to_vec()
        } else {
            encode_ifccad_document(&d).unwrap().bytes().to_vec()
        };
        let checked = if cbor {
            load_ifccad_cbor_bytes(&encoded, Default::default())
                .unwrap()
                .into_document()
        } else {
            load_ifccad_bytes(&encoded, Default::default())
                .unwrap()
                .into_document()
        };
        for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
            let out = ifccad_document_to_cad_document(
                &checked,
                IfccadToCadOptions {
                    loss_policy: policy,
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(target(&out).pattern.lines[0].base_point.y, 2f64.powi(-40));
            assert!(target(&out).stored_pattern_origin().is_none());
            assert!(!out
                .diagnostics()
                .iter()
                .any(|d| d.code == "hatch-classic-settings-loss"));
            assert!(out
                .diagnostics()
                .iter()
                .any(|d| matches!(d.action, IfccadDiagnosticAction::RoundedWithinTolerance)));
        }
    }
}

#[test]
fn explicit_recipe_defaults_are_not_foreign_ifcx_or_semantic_loss() {
    let d = drawing(2f64.powi(-40));
    let encoded = encode_ifccad_document(&d).unwrap();
    for field in ["dashLengths", "rotation", "scale", "double"] {
        let mut value: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
        let hatch = value["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find_map(|n| n["attributes"].get_mut("ifccad::hatch"))
            .unwrap();
        let c = &mut hatch["classicSettings"];
        match field {
            "dashLengths" => c["definitionLines"][0][field] = serde_json::json!([]),
            "rotation" => c[field] = serde_json::json!(0.),
            "scale" => c[field] = serde_json::json!(1.),
            _ => c[field] = serde_json::json!(false),
        }
        let source =
            load_ifccad_bytes(&serde_json::to_vec(&value).unwrap(), Default::default()).unwrap();
        let result = ifccad_source_to_cad_document(
            &source,
            IfccadToCadOptions {
                loss_policy: IfccadLossPolicy::Reject,
                ..Default::default()
            },
        );
        let result = result.unwrap_or_else(|e| panic!("explicit default {field}: {e:?}"));
        assert!(target(&result).stored_pattern_origin().is_none());
    }
}

#[test]
fn explicit_construction_defaults_do_not_become_foreign_content() {
    for field in ["seedPoints", "pixelSize"] {
        let mut d = drawing(2f64.powi(-40));
        let IfccadEntityKind::Hatch(h) = &mut d.model.entities[0].as_native_mut().unwrap().kind
        else {
            panic!()
        };
        h.boundary_construction = Some(HatchBoundaryConstruction {
            seed_points: if field == "pixelSize" {
                vec![[1., 2.]]
            } else {
                vec![]
            },
            pixel_size: if field == "seedPoints" { 0.25 } else { 0. },
        });
        let encoded = encode_ifccad_document(&d).unwrap();
        let mut value: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
        let h = value["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find_map(|n| n["attributes"].get_mut("ifccad::hatch"))
            .unwrap();
        h["boundaryConstruction"][field] = if field == "seedPoints" {
            serde_json::json!([])
        } else {
            serde_json::json!(0.)
        };
        let source =
            load_ifccad_bytes(&serde_json::to_vec(&value).unwrap(), Default::default()).unwrap();
        assert!(
            ifccad_source_to_cad_document(
                &source,
                IfccadToCadOptions {
                    loss_policy: IfccadLossPolicy::Reject,
                    ..Default::default()
                }
            )
            .is_ok(),
            "{field}"
        );
    }
}
#[test]
fn zero_tolerance_falls_back_with_located_loss_and_reject_refuses_it() {
    let d = drawing(2f64.powi(-40));
    let options = IfccadToCadOptions {
        geometry_tolerance: IfccadGeometryTolerance::exact(),
        ..Default::default()
    };
    let out = ifccad_document_to_cad_document(&d, options).unwrap();
    assert_eq!(target(&out).pattern.lines[0].base_point.y, 0.);
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "hatch-classic-settings-loss"
            && d.location.contains(&fixture::HATCH.to_string())));
    assert!(ifccad_document_to_cad_document(
        &d,
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..options
        }
    )
    .is_err());
}

fn nested(mut d: IfccadDocument, paper: bool) -> IfccadDocument {
    let mut inner = common::instance(fixture::HATCH + 1, 7, [0.; 3]);
    let mut outer = common::instance(fixture::HATCH + 2, 8, [0.; 3]);
    for (e, scale) in [(&mut inner, [2., 1e6, 1.]), (&mut outer, [1., 2., 1.])] {
        let IfccadEntityKind::BlockInstance { transform, .. } =
            &mut e.as_native_mut().unwrap().kind
        else {
            panic!()
        };
        transform.rotation = 0.;
        transform.scale = scale;
    }
    let block = |id, name: &str, entities| IfccadBlockDefinition {
        description: String::new(),
        anonymous: false,
        explodable: true,
        uniform_scaling: false,
        bounds_quality: None,
        id,
        name: name.into(),
        base_point: [0.; 3],
        insertion_unit: "unitless".into(),
        bounds: None,
        entities,
    };
    d.blocks = vec![
        block(7, "Leaf", std::mem::take(&mut d.model.entities)),
        block(8, "Outer", vec![inner]),
    ];
    if paper {
        d.paper_layouts = vec![IfccadPaperLayout {
            bounds_quality: None,
            canvas: None,
            id: 42,
            name: "Sheet".into(),
            tab_index: 1,
            bounds: None,
            settings: common::paper_settings(ifccad::plot_kernel::PlotUnit::Millimetre, 1.),
            entities: vec![outer],
        }];
        d.id_counters.next_layout_id = 43;
    } else {
        d.model.entities = vec![outer];
    }
    d.id_counters.next_entity_id = fixture::HATCH + 3;
    d.id_counters.next_block_id = 9;
    d
}
#[test]
fn nested_model_and_paper_occurrences_choose_fallback_without_trial_rounding_leaks() {
    for paper in [false, true] {
        for unit in ["unitless", "mm", "m"] {
            let mut d = nested(drawing(2f64.powi(-40)), paper);
            d.length_unit = unit.into();
            let out = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
            assert_eq!(target(&out).pattern.lines[0].base_point.y, 0.);
            assert!(out
                .diagnostics()
                .iter()
                .any(|d| d.code == "hatch-classic-settings-loss"));
            assert!(
                !out.diagnostics()
                    .iter()
                    .any(|d| d.code == "GeometryRoundedWithinTolerance"),
                "failed trial leaked: {:?}",
                out.diagnostics()
            );
            assert!(ifccad_document_to_cad_document(
                &d,
                IfccadToCadOptions {
                    loss_policy: IfccadLossPolicy::Reject,
                    ..Default::default()
                }
            )
            .is_err());
        }
    }
}
#[test]
fn tiny_active_edit_can_qualify_but_discrete_dash_structure_cannot() {
    let mut d = drawing(0.);
    let IfccadEntityKind::Hatch(h) = &mut d.model.entities[0].as_native_mut().unwrap().kind else {
        panic!()
    };
    let HatchFill::LinePattern(p) = &mut h.fill else {
        panic!()
    };
    p.families[0].base_point[1] = 2f64.powi(-40);
    let out = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    assert_eq!(target(&out).pattern.lines[0].base_point.y, 0.);
    assert!(!out
        .diagnostics()
        .iter()
        .any(|d| d.code == "hatch-classic-settings-loss"));
    let IfccadEntityKind::Hatch(h) = &mut d.model.entities[0].as_native_mut().unwrap().kind else {
        panic!()
    };
    let HatchFill::LinePattern(p) = &mut h.fill else {
        panic!()
    };
    p.families[0].dashes = vec![
        HatchDash::Dash { length: 1. },
        HatchDash::Gap { length: 1. },
        HatchDash::Dot,
    ];
    h.classic_settings
        .as_mut()
        .unwrap()
        .definition_lines
        .as_mut()
        .unwrap()[0]
        .dash_lengths = vec![-1., 1., 0.];
    let out = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    assert_eq!(
        target(&out).pattern.lines[0].dash_lengths,
        vec![1., -1., 0.]
    );
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "hatch-classic-settings-loss"));
}

#[test]
fn unrelated_geometry_and_an_unproved_fallback_remain_hard_under_both_policies() {
    for unrelated in [false, true] {
        for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
            let mut d = drawing(1.);
            if unrelated {
                let IfccadEntityKind::Circle { radius, placement } =
                    &mut d.model.entities[1].as_native_mut().unwrap().kind
                else {
                    panic!()
                };
                *radius = 1e8;
                placement.x_axis[0] = 1. + 2e-13;
            } else {
                let IfccadEntityKind::Hatch(h) =
                    &mut d.model.entities[0].as_native_mut().unwrap().kind
                else {
                    panic!()
                };
                let HatchFill::LinePattern(p) = &mut h.fill else {
                    panic!()
                };
                p.rotation = 0.2;
                p.families[0].angle = 0.1;
                h.loops = vec![IfccadHatchLoop {
                    boundary: HatchBoundary2::Circle {
                        center: [1e9, 1e9],
                        radius: 10.,
                    },
                    source_entity_ids: vec![],
                    provenance: None,
                    text_island: false,
                }];
                h.association_enabled = false;
            }
            let Err(IfccadConversionError::Geometry(failure)) = ifccad_document_to_cad_document(
                &d,
                IfccadToCadOptions {
                    loss_policy: policy,
                    ..Default::default()
                },
            ) else {
                panic!("final geometry must have a hard certificate")
            };
            if unrelated {
                assert!(matches!(
                    failure.source,
                    IfccadGeometryEntitySource::NativeEntity {
                        entity_id: fixture::SOURCE,
                        ..
                    }
                ));
            }
        }
    }
}
