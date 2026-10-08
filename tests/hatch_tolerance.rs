use ocdraw::plot_kernel::{PlotScale, PlotUnit};
use ocdraw::{geometry_kernel::hatch::*, ifccad::*, ocdraw::*};

#[test]
fn fixed_paper_scale_resolves_physical_limits_but_plot_edits_preserve_stored_tolerance() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-paper-layouts.ifcx"),
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
        placement: IfccadPlacement {
            origin: [0.0, 0.0, 0.0],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
        },
        loops: vec![IfccadHatchLoop {
            boundary: HatchBoundary2::Circle {
                center: [0.0, 0.0],
                radius: 1.0,
            },
            source_entity_id: None,
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
fn ocdraw_paper_uses_fixed_plot_meaning_and_never_infers_from_media() {
    let mut d = load_ocdraw_bytes(include_bytes!(
        "../conformance/next/ocdraw/valid/layout-plot-inch.ocdraw.json"
    ))
    .unwrap()
    .into_document();
    let index = d
        .layouts
        .iter()
        .position(|l| l.kind == DrawingLayoutKind::Paper)
        .unwrap();
    let scope = d.layouts[index].scope_id;
    let request = HatchJoinToleranceRequest::Millimetres {
        value: 1.0,
        coordinate_fallback: None,
    };
    let p = d.layouts[index].settings.plot_settings.as_mut().unwrap();
    p.plot_unit = PlotUnit::Millimetre;
    p.mapping.scale = PlotScale::Fixed {
        output_length: 2.0,
        scope_length: 1.0,
    };
    assert_eq!(
        resolve_ocdraw_hatch_join_tolerance(&d, scope, request).unwrap(),
        0.5
    );
    d.layouts[index].settings.plot_settings = None;
    assert!(resolve_ocdraw_hatch_join_tolerance(&d, scope, request).is_err());
    let fallback = HatchJoinToleranceRequest::Millimetres {
        value: 1.0,
        coordinate_fallback: Some(0.125),
    };
    assert_eq!(
        resolve_ocdraw_hatch_join_tolerance(&d, scope, fallback).unwrap(),
        0.125
    );
}

#[test]
fn invalid_known_paper_scale_is_not_treated_as_unknown_to_enable_a_fallback() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-paper-layouts.ifcx"),
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
    let mut o = load_ocdraw_bytes(include_bytes!(
        "../conformance/next/ocdraw/valid/layout-plot-inch.ocdraw.json"
    ))
    .unwrap()
    .into_document();
    let index = o
        .layouts
        .iter()
        .position(|l| l.kind == DrawingLayoutKind::Paper)
        .unwrap();
    let scope = o.layouts[index].scope_id;
    o.layouts[index]
        .settings
        .plot_settings
        .as_mut()
        .unwrap()
        .mapping
        .scale = PlotScale::Fixed {
        output_length: -1.0,
        scope_length: 1.0,
    };
    assert!(resolve_ocdraw_hatch_join_tolerance(&o, scope, request).is_err());
}
