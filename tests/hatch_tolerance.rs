use ifccad::plot_kernel::{PlotScale, PlotUnit};
use ifccad::{geometry_kernel::hatch::*, *};

#[test]
fn fixed_paper_scale_resolves_physical_limits_but_plot_edits_preserve_stored_tolerance() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../tests/fixtures/ifccad/hello-paper-layouts.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let paper = d.paper_layouts[0].id;
    {
        let p = d.paper_layouts[0].settings.plot_settings.as_mut().unwrap();
        p.plot_unit = PlotUnit::Millimetre;
        p.mapping.scale = PlotScale::Fixed {
            output_length: 2.0,
            scope_length: 1.0,
        };
    }
    let request = HatchJoinToleranceRequest::Millimetres {
        value: 1.0,
        coordinate_fallback: None,
    };
    let resolved =
        resolve_ifccad_hatch_join_tolerance(&d, IfccadScopeId::Layout(paper), request).unwrap();
    assert_eq!(resolved, 0.5);
    let mut e = d
        .model
        .entities
        .iter()
        .find_map(IfccadEntity::as_native)
        .unwrap()
        .clone();
    e.id = d.id_counters.allocate_entity_id().unwrap();
    e.kind = IfccadEntityKind::Hatch(IfccadHatch {
        association_enabled: false,
        gradient: None,
        placement: IfccadPlacement {
            origin: [0.0, 0.0, 0.0],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
        },
        loops: vec![IfccadHatchLoop {
            provenance: None,
            text_island: false,
            boundary: HatchBoundary2::Circle {
                center: [0.0, 0.0],
                radius: 1.0,
            },
            source_entity_ids: vec![],
        }],
        area_rule: HatchAreaRule::Normal,
        join_tolerance: resolved,
        fill: HatchFill::Solid,
    });
    let id = e.id;
    d.paper_layouts[0].entities.push(IfccadEntity::Native(e));
    d.paper_layouts[0]
        .settings
        .plot_settings
        .as_mut()
        .unwrap()
        .mapping
        .scale = PlotScale::Fixed {
        output_length: 4.0,
        scope_length: 1.0,
    };
    assert_eq!(
        resolve_ifccad_hatch_join_tolerance(&d, IfccadScopeId::Layout(paper), request).unwrap(),
        0.25
    );
    recompute_ifccad_document_bounds(&mut d).unwrap();
    let out = encode_ifccad_document(&d).unwrap();
    let read = load_ifccad_bytes(out.bytes(), Default::default()).unwrap();
    let h = read.document().paper_layouts[0]
        .entities
        .iter()
        .find(|e| e.id() == id)
        .unwrap()
        .as_native()
        .unwrap();
    let IfccadEntityKind::Hatch(h) = &h.kind else {
        panic!()
    };
    assert_eq!(h.join_tolerance, 0.5);
}

#[test]
fn invalid_known_paper_scale_is_not_treated_as_unknown_to_enable_a_fallback() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../tests/fixtures/ifccad/hello-paper-layouts.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let owner = IfccadScopeId::Layout(d.paper_layouts[0].id);
    d.paper_layouts[0]
        .settings
        .plot_settings
        .as_mut()
        .unwrap()
        .mapping
        .scale = PlotScale::Fixed {
        output_length: 0.0,
        scope_length: 1.0,
    };
    let request = HatchJoinToleranceRequest::Millimetres {
        value: 1.0,
        coordinate_fallback: Some(0.125),
    };
    assert!(resolve_ifccad_hatch_join_tolerance(&d, owner, request).is_err());
}
