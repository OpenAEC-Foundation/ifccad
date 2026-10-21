mod common;
use ifccad::*;
use ifccad_convert::*;
use opencadcodec::EntityType;
fn drawing(c: [[f64; 2]; 4], trace: bool, h: f64) -> IfccadDocument {
    let mut d = common::empty();
    let q = IfccadFilledQuadrilateral {
        placement: common::placement([0., 0., 5.]),
        corners: c,
        thickness: h,
    };
    d.model
        .entities
        .push(IfccadEntity::Native(IfccadNativeEntity {
            id: 9007199254740993,
            layer_id: 0,
            visible: true,
            line_pattern_scale: 1.,
            appearance: common::modes(),
            kind: if trace {
                IfccadEntityKind::Trace(q)
            } else {
                IfccadEntityKind::Solid(q)
            },
        }));
    d.id_counters.next_entity_id = 9007199254740994;
    d
}
#[test]
fn filled_unassessed_surface_is_not_loss_or_recovery() {
    let cases: serde_json::Value = serde_json::from_str(include_str!(
        "../../../docs/geometry/fixtures/solid-trace-profile/cases.json"
    ))
    .unwrap();
    for c in cases["cases"].as_array().unwrap() {
        for trace in [false, true] {
            for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
                for errors in [
                    CadImportErrorPolicy::Abort,
                    CadImportErrorPolicy::SkipInvalidEntities,
                ] {
                    let d = drawing(
                        serde_json::from_value(c["corners"].clone()).unwrap(),
                        trace,
                        -2.,
                    );
                    let cad = ifccad_document_to_cad_document(
                        &d,
                        IfccadToCadOptions {
                            loss_policy: policy,
                            ..Default::default()
                        },
                    )
                    .unwrap();
                    assert!(!cad.diagnostics().iter().any(|d| d.is_loss()));
                    let evidence = cad.geometry_assessment();
                    assert_eq!(evidence.status(), IfccadGeometryStatus::Exact);
                    assert_eq!(evidence.filled_surfaces().len(), 1);
                    let proven = c["simpleSurfaceCoverCandidate"] == true;
                    assert_eq!(evidence.is_complete(), proven);
                    assert_eq!(
                        evidence.filled_surfaces()[0].coverage(),
                        if proven {
                            IfccadFilledSurfaceCoverage::Proven
                        } else {
                            IfccadFilledSurfaceCoverage::Unassessed
                        }
                    );
                    let handle = cad
                        .mappings()
                        .entities
                        .cad_handle(9007199254740993)
                        .unwrap();
                    let EntityType::Solid(s) = cad.document().get_entity(handle).unwrap() else {
                        panic!()
                    };
                    assert_eq!(s.is_trace, trace);
                    assert_eq!(s.thickness, -2.);
                    let back = cad_document_to_encoded_ifccad(
                        cad.document(),
                        common::metadata(),
                        CadToIfccadOptions {
                            loss_policy: policy,
                            error_policy: errors,
                            ..Default::default()
                        },
                    )
                    .unwrap();
                    assert!(!back.diagnostics().iter().any(|d| d.is_loss()));
                    assert_eq!(back.geometry_assessment().filled_surfaces().len(), 1);
                    let restored = back.validated_source().document().model.entities[0]
                        .as_native()
                        .unwrap();
                    assert_eq!(matches!(restored.kind, IfccadEntityKind::Trace(_)), trace);
                }
            }
        }
    }
}
#[test]
fn filled_invalid_scalar_precedes_unsupported_metadata() {
    let d = drawing([[0., 0.]; 4], false, 0.);
    let mut cad = ifccad_document_to_cad_document(&d, Default::default())
        .unwrap()
        .into_document();
    let h = cad
        .block_records
        .iter()
        .find(|b| b.handle == cad.header.model_space_block_handle)
        .unwrap()
        .entity_handles[0];
    if let Some(EntityType::Solid(s)) = cad.get_entity_mut(h) {
        s.first_corner.x = f64::NAN;
        s.fourth_corner.z = 7.;
    }
    assert!(cad_document_to_ifccad_document(&cad, common::metadata(), Default::default()).is_err());
}

#[test]
fn filled_kind_survives_owners() {
    let mut d = common::nested([1., 2., 3.]);
    for (index, b) in d.blocks.iter_mut().enumerate() {
        let q = IfccadFilledQuadrilateral {
            placement: common::placement([0., 0., 5.]),
            corners: if index == 0 {
                [[4., 4.]; 4]
            } else {
                [[0., 0.], [8., 0.], [0., 8.], [8., 8.]]
            },
            thickness: -2.,
        };
        let mut e = drawing([[0.; 2]; 4], false, 0.).model.entities.remove(0);
        let e_native = e.as_native_mut().unwrap();
        e_native.id = 100 + index as u64;
        e_native.kind = IfccadEntityKind::Trace(q);
        b.entities.push(e);
    }
    let mut e = drawing([[0.; 2]; 4], true, 2.).model.entities.remove(0);
    e.as_native_mut().unwrap().id = 200;
    let paper_id = d.id_counters.allocate_layout_id().unwrap();
    d.paper_layouts.push(IfccadPaperLayout {
        bounds_quality: None,
        canvas: None,
        id: paper_id,
        name: "Filled sheet".into(),
        tab_index: 1,
        settings: common::paper_settings(ifccad::plot_kernel::PlotUnit::Millimetre, 1.),
        bounds: None,
        entities: vec![e],
    });
    let out = ifccad_document_to_cad_document(
        &d,
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        },
    )
    .unwrap();
    let surfaces = out.geometry_assessment().filled_surfaces();
    assert_eq!(surfaces.len(), 9);
    assert!(surfaces.iter().any(|s| matches!(
        s.source(),
        IfccadGeometryEntitySource::BlockOccurrence { .. }
    )));
    assert!(surfaces.iter().any(|s| matches!(
        s.source(),
        IfccadGeometryEntitySource::NativeEntity {
            owner: IfccadGeometryOwner::PaperLayout(_),
            ..
        }
    )));
    let back =
        cad_document_to_encoded_ifccad(out.document(), common::metadata(), Default::default())
            .unwrap();
    assert!(back
        .diagnostics()
        .iter()
        .filter(|d| d.is_semantic_loss())
        .all(|d| d.code == "workspace"));
    assert_eq!(back.geometry_assessment().filled_surfaces().len(), 9);
    let doc = back.validated_source().document();
    assert!(matches!(
        doc.paper_layouts[0].entities[0].as_native().unwrap().kind,
        IfccadEntityKind::Trace(_)
    ));
    assert_eq!(
        doc.blocks
            .iter()
            .flat_map(|b| &b.entities)
            .filter(|e| matches!(
                e.as_native().map(|e| &e.kind),
                Some(IfccadEntityKind::Trace(_))
            ))
            .count(),
        3
    );
}
#[test]
fn filled_source_graph_audits_defaults_foreign_fields_and_numbers() {
    use serde_json::{json, Value};
    let d = drawing([[0.; 2]; 4], true, 0.);
    let bytes = encode_ifccad_document(&d).unwrap();
    let base: Value = serde_json::from_slice(bytes.bytes()).unwrap();
    for explicit in [false, true] {
        let mut v = base.clone();
        let n = v["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["attributes"]["ifccad::geom::trace"].is_object())
            .unwrap();
        if explicit {
            n["attributes"]["ifccad::geom::trace"]["thickness"] = json!(0.);
        }
        let bytes = serde_json::to_vec(&v).unwrap();
        let saved = bytes.clone();
        let src = load_ifccad_bytes(&bytes, Default::default()).unwrap();
        let out = ifccad_source_to_cad_document(
            &src,
            IfccadToCadOptions {
                loss_policy: IfccadLossPolicy::Reject,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!out.diagnostics().iter().any(|d| d.is_loss()));
        assert_eq!(bytes, saved);
    }
    let mut foreign = base.clone();
    foreign["data"]
        .as_array_mut()
        .unwrap()
        .push(json!({"path":"/foreign","attributes":{"test::value":42}}));
    let src =
        load_ifccad_bytes(&serde_json::to_vec(&foreign).unwrap(), Default::default()).unwrap();
    let out = ifccad_source_to_cad_document(&src, Default::default()).unwrap();
    assert!(out.diagnostics().iter().any(|d| d.code == "foreign-ifcx"));
    let mut imprecise = base;
    let n = imprecise["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"]["ifccad::geom::trace"].is_object())
        .unwrap();
    n["attributes"]["ifccad::geom::trace"]["corners"][0][0] = json!(9007199254740993u64);
    let src =
        load_ifccad_bytes(&serde_json::to_vec(&imprecise).unwrap(), Default::default()).unwrap();
    for p in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        assert!(
            matches!(ifccad_source_to_cad_document(&src,IfccadToCadOptions{loss_policy:p,..Default::default()}),Err(IfccadConversionError::Unsupported(d)) if d.iter().any(|d|d.code=="precision"))
        );
    }
    assert!(ifccad_document_to_cad_document(src.document(), Default::default()).is_ok());
}
#[test]
fn filled_source_policy_matrix() {
    let d = drawing([[0.; 2]; 4], false, 0.);
    let mut cad = ifccad_document_to_cad_document(&d, Default::default())
        .unwrap()
        .into_document();
    let h = cad
        .block_records
        .iter()
        .find(|b| b.handle == cad.header.model_space_block_handle)
        .unwrap()
        .entity_handles[0];
    if let Some(EntityType::Solid(s)) = cad.get_entity_mut(h) {
        s.fourth_corner.z += 1.;
    }
    for errors in [
        CadImportErrorPolicy::Abort,
        CadImportErrorPolicy::SkipInvalidEntities,
    ] {
        let out = cad_document_to_ifccad_document(
            &cad,
            common::metadata(),
            CadToIfccadOptions {
                error_policy: errors,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(out.document().model.entities.is_empty());
        assert!(out.geometry_assessment().filled_surfaces().is_empty());
        assert!(out.diagnostics().iter().any(|d| d.code == "geometry"));
        assert!(cad_document_to_ifccad_document(
            &cad,
            common::metadata(),
            CadToIfccadOptions {
                error_policy: errors,
                loss_policy: IfccadLossPolicy::Reject,
                ..Default::default()
            }
        )
        .is_err());
    }
}

#[test]
fn filled_rounding_may_change_topology_without_waiving_position_limit() {
    let mut d = drawing([[0., 0.], [1., 0.], [0., 1.], [1., 1.]], false, 0.);
    let IfccadEntityKind::Solid(q) = &mut d.model.entities[0].as_native_mut().unwrap().kind else {
        panic!()
    };
    q.placement.origin = [1e20, 1e20, 0.];
    assert!(matches!(
        ifccad_document_to_cad_document(
            &d,
            IfccadToCadOptions {
                geometry_tolerance: IfccadGeometryTolerance::exact(),
                ..Default::default()
            }
        ),
        Err(IfccadConversionError::Geometry(_))
    ));
    let out = ifccad_document_to_cad_document(
        &d,
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            geometry_tolerance: IfccadGeometryTolerance::drawing_units(2.).unwrap(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        out.geometry_assessment().status(),
        IfccadGeometryStatus::RoundedWithinTolerance
    );
    assert!(!out.geometry_assessment().is_complete());
    assert_eq!(
        out.geometry_assessment().filled_surfaces()[0].coverage(),
        IfccadFilledSurfaceCoverage::Unassessed
    );
    assert_eq!(
        out.geometry_assessment().domains()[0].assessed_vertices(),
        8
    );
    assert!(!out.diagnostics().iter().any(|d| d.is_semantic_loss()));
}
#[test]
fn filled_paper_tolerance_uses_coordinate_mapping() {
    let mut d = drawing([[0., 0.]; 4], true, 0.);
    d.length_unit = "m".into();
    let id = d.id_counters.allocate_layout_id().unwrap();
    let entities = std::mem::take(&mut d.model.entities);
    d.paper_layouts.push(IfccadPaperLayout {
        bounds_quality: None,
        canvas: None,
        id,
        name: "Filled physical sheet".into(),
        tab_index: 1,
        settings: common::paper_settings(ifccad::plot_kernel::PlotUnit::Millimetre, 1.),
        bounds: None,
        entities,
    });
    let out = ifccad_document_to_cad_document(
        &d,
        IfccadToCadOptions {
            geometry_tolerance: IfccadGeometryTolerance::millimetres(0.001).unwrap(),
            ..Default::default()
        },
    )
    .unwrap();
    let paper = out
        .geometry_assessment()
        .domains()
        .iter()
        .find(|a| a.domain() == IfccadGeometryDomain::PaperLayout(id))
        .unwrap();
    assert_eq!(paper.assessed_vertices(), 8);
    assert!(paper.resolved_tolerance().upper() <= 0.0010000000001);
    let model = out
        .geometry_assessment()
        .domains()
        .iter()
        .find(|a| a.domain() == IfccadGeometryDomain::Drawing)
        .unwrap();
    assert!(model.resolved_tolerance().upper() <= 0.0000010000000001);
    d.paper_layouts[0].settings = Default::default();
    assert!(matches!(
        ifccad_document_to_cad_document(
            &d,
            IfccadToCadOptions {
                geometry_tolerance: IfccadGeometryTolerance::millimetres(0.001).unwrap(),
                ..Default::default()
            }
        ),
        Err(IfccadConversionError::PaperTolerance { .. })
    ));
}

#[test]
fn filled_partial_import_handles_real_failures_without_orphan_evidence() {
    for bad in [0, 1, 2] {
        let d = drawing([[0.; 2]; 4], false, 0.);
        let mut cad = ifccad_document_to_cad_document(&d, Default::default())
            .unwrap()
            .into_document();
        let h = cad
            .block_records
            .iter()
            .find(|b| b.handle == cad.header.model_space_block_handle)
            .unwrap()
            .entity_handles[0];
        let Some(EntityType::Solid(s)) = cad.get_entity_mut(h) else {
            panic!()
        };
        match bad {
            0 => s.thickness = f64::INFINITY,
            1 => s.first_corner.x = f64::NAN,
            _ => s.normal = opencadcodec::Vector3::ZERO,
        };
        s.fourth_corner.z += 1.;
        assert!(
            cad_document_to_ifccad_document(&cad, common::metadata(), Default::default()).is_err()
        );
        let out = cad_document_to_encoded_ifccad(
            &cad,
            common::metadata(),
            CadToIfccadOptions {
                error_policy: CadImportErrorPolicy::SkipInvalidEntities,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(out.validated_source().document().model.entities.is_empty());
        assert!(out.geometry_assessment().filled_surfaces().is_empty());
        assert!(!out.partial_open_report().unwrap().errors.is_empty());
        assert!(cad_document_to_ifccad_document(
            &cad,
            common::metadata(),
            CadToIfccadOptions {
                error_policy: CadImportErrorPolicy::SkipInvalidEntities,
                loss_policy: IfccadLossPolicy::Reject,
                ..Default::default()
            }
        )
        .is_err());
    }
}
