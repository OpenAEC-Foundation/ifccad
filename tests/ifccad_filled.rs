use ifccad::*;
fn value(corners: [[f64; 2]; 4], thickness: f64) -> IfccadFilledQuadrilateral {
    IfccadFilledQuadrilateral {
        placement: IfccadPlacement {
            origin: [0.; 3],
            x_axis: [1., 0., 0.],
            y_axis: [0., 1., 0.],
        },
        corners,
        thickness,
    }
}
fn document_with(kind: IfccadEntityKind) -> IfccadDocument {
    let mut d = load_ifccad_bytes(
        include_bytes!("fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    d.model.entities[0].as_native_mut().unwrap().kind = kind;
    d.model.bounds = None;
    for b in &mut d.blocks {
        b.bounds = None;
    }
    for p in &mut d.paper_layouts {
        p.bounds = None;
    }
    d
}
fn restored(d: &IfccadDocument) -> Vec<IfccadDocument> {
    let j = encode_ifccad_document(d).unwrap();
    let b = encode_ifccad_cbor_document(d).unwrap();
    vec![
        load_ifccad_bytes(j.bytes(), Default::default())
            .unwrap()
            .into_document(),
        load_ifccad_cbor_bytes(b.bytes(), Default::default())
            .unwrap()
            .into_document(),
    ]
}
fn get(d: &IfccadDocument) -> &IfccadFilledQuadrilateral {
    match &d.model.entities[0].as_native().unwrap().kind {
        IfccadEntityKind::Solid(v) | IfccadEntityKind::Trace(v) => v,
        _ => panic!("filled kind lost"),
    }
}
#[test]
fn filled_all_finite_records_survive_strict_readback() {
    let v: serde_json::Value = serde_json::from_str(include_str!(
        "../docs/geometry/fixtures/solid-trace-profile/cases.json"
    ))
    .unwrap();
    for c in v["cases"].as_array().unwrap() {
        for trace in [false, true] {
            for h in [0., 3., -3.] {
                let q = value(serde_json::from_value(c["corners"].clone()).unwrap(), h);
                let d = document_with(if trace {
                    IfccadEntityKind::Trace(q.clone())
                } else {
                    IfccadEntityKind::Solid(q.clone())
                });
                for out in restored(&d) {
                    assert_eq!(get(&out), &q);
                    assert_eq!(
                        matches!(
                            out.model.entities[0].as_native().unwrap().kind,
                            IfccadEntityKind::Trace(_)
                        ),
                        trace
                    );
                    assert!(out.model.bounds.is_none());
                }
            }
        }
    }
}
#[test]
fn filled_native_float_bits_survive() {
    let v = value([[-0., f64::from_bits(1)]; 4], -0.);
    for out in restored(&document_with(IfccadEntityKind::Trace(v))) {
        assert_eq!(get(&out).corners[0][0].to_bits(), (-0f64).to_bits());
        assert_eq!(get(&out).corners[0][1].to_bits(), 1);
        assert_eq!(get(&out).thickness.to_bits(), (-0f64).to_bits());
    }
}
#[test]
fn filled_native_validation_reports_value_rules() {
    let mut d = document_with(IfccadEntityKind::Solid(value([[0.; 2]; 4], f64::INFINITY)));
    assert!(
        format!("{:?}", validate_ifccad_document(&d).unwrap_err()).contains("IFCCAD-FILLED-001")
    );
    if let IfccadEntityKind::Solid(v) = &mut d.model.entities[0].as_native_mut().unwrap().kind {
        v.thickness = 0.;
        v.placement.x_axis = [0.; 3];
    }
    assert!(
        format!("{:?}", validate_ifccad_document(&d).unwrap_err()).contains("IFCCAD-FILLED-002")
    );
}

#[test]
fn filled_recompute_is_atomic_and_save_does_not_recompute() {
    let mut d = document_with(IfccadEntityKind::Solid(value([[0., 0.]; 4], -2.)));
    for out in restored(&d) {
        assert!(out.model.bounds.is_none());
    }
    if let IfccadEntityKind::Solid(q) = &mut d.model.entities[0].as_native_mut().unwrap().kind {
        q.placement.origin[0] = f64::MAX;
        q.corners[0][0] = f64::MAX;
    }
    let before = d.clone();
    assert!(recompute_ifccad_document_bounds(&mut d).is_err());
    assert_eq!(d, before);
}
#[test]
fn filled_same_path_replacement_and_payload_exclusivity() {
    let d = document_with(IfccadEntityKind::Solid(value([[0., 0.]; 4], 0.)));
    let bytes = encode_ifccad_document(&d).unwrap();
    let mut v: serde_json::Value = serde_json::from_slice(bytes.bytes()).unwrap();
    let e = v["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["attributes"]["ifccad::geom::solid"].is_object())
        .unwrap();
    let path = e["path"].clone();
    v["data"].as_array_mut().unwrap().push(serde_json::json!({"path":path,"attributes":{"ifccad::geom::solid":{"corners":[[1.,2.],[1.,2.],[1.,2.],[1.,2.]],"thickness":-3.}}}));
    let loaded = load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).unwrap();
    assert_eq!(get(loaded.document()).corners, [[1., 2.]; 4]);
    v["data"].as_array_mut().unwrap().push(serde_json::json!({"path":path,"attributes":{"ifccad::geom::trace":{"corners":[[0.,0.],[0.,0.],[0.,0.],[0.,0.]]}}}));
    assert!(load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).is_err());
}

#[path = "support/ifccad_hatch.rs"]
mod hatch_fixture;
#[test]
fn filled_remains_ineligible_for_hatch_clip_and_dimension_selectors() {
    let kind = IfccadEntityKind::Solid(value([[0., 0.], [4., 0.], [0., 3.], [4., 3.]], 0.));
    assert!(!kind.is_hatch_boundary_source());
    let mut d = hatch_fixture::drawing();
    d.model
        .entities
        .iter_mut()
        .find(|e| e.id() == hatch_fixture::SOURCE)
        .unwrap()
        .as_native_mut()
        .unwrap()
        .kind = kind.clone();
    assert!(format!("{:?}", validate_ifccad_document(&d).unwrap_err()).contains("HATCH"));
    let mut d = load_ifccad_bytes(
        include_bytes!("../conformance/next/ifccad/valid/dimension-linear-aligned.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let source = d
        .model
        .entities
        .iter()
        .filter_map(IfccadEntity::as_native)
        .find_map(|e| {
            if let IfccadEntityKind::Dimension(d) = &e.kind {
                d.associations.first().map(|a| a.source_entity_id)
            } else {
                None
            }
        })
        .unwrap();
    d.model
        .entities
        .iter_mut()
        .find(|e| e.id() == source)
        .unwrap()
        .as_native_mut()
        .unwrap()
        .kind = kind.clone();
    assert!(format!("{:?}", validate_ifccad_document(&d).unwrap_err()).contains("DIMENSION"));
    let mut d = load_ifccad_bytes(
        include_bytes!("../conformance/next/ifccad/valid-circle.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let source = d
        .paper_layouts
        .iter()
        .flat_map(|p| &p.entities)
        .filter_map(IfccadEntity::as_native)
        .find_map(|e| {
            if let IfccadEntityKind::Viewport(v) = &e.kind {
                v.paper_clip.boundary_entity_id
            } else {
                None
            }
        })
        .unwrap();
    let e = d
        .paper_layouts
        .iter_mut()
        .flat_map(|p| &mut p.entities)
        .find(|e| e.id() == source)
        .unwrap()
        .as_native_mut()
        .unwrap();
    let IfccadEntityKind::Circle { placement, .. } = &e.kind else {
        panic!()
    };
    let mut q = value([[0., 0.], [4., 0.], [0., 3.], [4., 3.]], 0.);
    q.placement = placement.clone();
    e.kind = IfccadEntityKind::Trace(q);
    assert!(format!("{:?}", validate_ifccad_document(&d).unwrap_err()).contains("VIEWPORT"));
}
#[test]
fn filled_bounds_ignore_dormant_pattern_markers() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../conformance/next/ifccad/valid/line-pattern-text.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let mut e = d.model.entities.remove(0);
    let n = e.as_native_mut().unwrap();
    n.kind = IfccadEntityKind::Trace(value([[0., 0.], [4., 0.], [0., 3.], [4., 3.]], -2.));
    n.appearance.line_weight = IfccadMode::Explicit(100.);
    d.model.entities = vec![e];
    d.model.bounds = None;
    d.blocks.clear();
    d.paper_layouts.clear();
    recompute_ifccad_document_bounds(&mut d).unwrap();
    assert_eq!(
        d.model.bounds,
        Some(IfccadBounds3d {
            min: [0., 0., -2.],
            max: [4., 3., 0.]
        })
    );
    for out in restored(&d) {
        assert_eq!(out.model.bounds, d.model.bounds);
    }
}

#[test]
fn filled_native_roundtrip_keeps_kind_and_slots_in_all_owners() {
    let mut d = document_with(IfccadEntityKind::Solid(value(
        [[0., 0.], [4., 0.], [0., 3.], [4., 3.]],
        -3.,
    )));
    let template = d.model.entities[0].clone();
    for trace in [false, true] {
        let mut e = template.clone();
        let id = d.id_counters.allocate_entity_id().unwrap();
        let n = e.as_native_mut().unwrap();
        n.id = id;
        let q = value([[4., 4.]; 4], 3.);
        n.kind = if trace {
            IfccadEntityKind::Trace(q)
        } else {
            IfccadEntityKind::Solid(q)
        };
        d.blocks[0].entities.push(e);
    }
    let mut entities = Vec::new();
    for trace in [false, true] {
        let mut e = template.clone();
        let id = d.id_counters.allocate_entity_id().unwrap();
        let n = e.as_native_mut().unwrap();
        n.id = id;
        let q = value([[0., 0.], [4., 0.], [0., 3.], [0., 3.]], -2.);
        n.kind = if trace {
            IfccadEntityKind::Trace(q)
        } else {
            IfccadEntityKind::Solid(q)
        };
        entities.push(e);
    }
    let id = d.id_counters.allocate_layout_id().unwrap();
    d.paper_layouts.push(IfccadPaperLayout {
        bounds_quality: None,
        canvas: None,
        id,
        name: "Filled sheet".into(),
        tab_index: 1,
        settings: Default::default(),
        bounds: None,
        entities,
    });
    for out in restored(&d) {
        assert_eq!(out.model.entities, d.model.entities);
        assert_eq!(out.blocks, d.blocks);
        assert_eq!(out.paper_layouts, d.paper_layouts);
    }
}
