mod common;
use common::*;
use ifccad_convert::*;
use ocdraw::ifccad::*;

#[test]
fn unsupported_medium_does_not_remove_paper_geometry_domain() {
    let mut d = common::viewport_drawing();
    d.paper_layouts[0].settings.media = Some(IfccadLayoutMedia {
        unit: IfccadMediaUnit::from_token("pc").unwrap(),
        width: 1000.,
        height: 1000.,
    });
    let out = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    assert!(out.diagnostics().iter().any(|i| i.code == "paper-medium"));
    assert_eq!(out.geometry_assessment().domains().len(), 2);
}

#[test]
fn unitless_media_does_not_enable_physical_tolerance() {
    let mut d = empty();
    let id = d.id_counters.allocate_layout_id().unwrap();
    d.paper_layouts.push(IfccadPaperLayout {
        settings: ocdraw::ifccad::IfccadLayoutSettings {
            media: Some(ocdraw::ifccad::IfccadLayoutMedia {
                width: 100.,
                height: 200.,
                unit: ocdraw::ifccad::IfccadMediaUnit::from_token("mm").unwrap(),
            }),
            ..Default::default()
        },
        id,
        name: "Unitless paper".into(),
        tab_index: 1,
        bounds: None,
        entities: vec![],
    });
    let error = ifccad_document_to_cad_document(
        &d,
        IfccadToCadOptions {
            geometry_tolerance: IfccadGeometryTolerance::millimetres(0.001).unwrap(),
            ..Default::default()
        },
    )
    .err()
    .expect("unitless coordinate domain");
    assert!(matches!(error, IfccadConversionError::PaperTolerance{layout_id,..} if layout_id==id));
    let outcome = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    assert_eq!(
        outcome.geometry_assessment().status(),
        IfccadGeometryStatus::Empty
    );
    let paper = outcome
        .geometry_assessment()
        .domains()
        .iter()
        .find(|d| d.domain() == IfccadGeometryDomain::PaperLayout(id))
        .unwrap();
    assert_eq!(paper.resolved_tolerance().upper(), 1e-9);
}
#[test]
fn within_limit_geometric_rounding_is_accepted_under_semantic_reject() {
    let mut d = primitives();
    let IfccadEntityKind::PlanarPolyline {
        placement,
        vertices,
        ..
    } = &mut d.model.entities[1].as_native_mut().unwrap().kind
    else {
        panic!()
    };
    placement.origin[0] = 1e20;
    vertices[0][0] = 1.;
    let exact = ifccad_document_to_cad_document(
        &d,
        IfccadToCadOptions {
            preservation: Default::default(),
            loss_policy: IfccadLossPolicy::Reject,
            geometry_tolerance: IfccadGeometryTolerance::exact(),
        },
    )
    .err()
    .expect("exact residual");
    assert!(matches!(exact, IfccadConversionError::Geometry(_)));
    let outcome = ifccad_document_to_cad_document(
        &d,
        IfccadToCadOptions {
            preservation: Default::default(),
            loss_policy: IfccadLossPolicy::Reject,
            geometry_tolerance: IfccadGeometryTolerance::drawing_units(10.).unwrap(),
        },
    )
    .unwrap();
    assert_eq!(
        outcome.geometry_assessment().status(),
        IfccadGeometryStatus::RoundedWithinTolerance
    );
    assert!(outcome
        .diagnostics()
        .iter()
        .any(|d| d.code == "GeometryRoundedWithinTolerance" && d.is_loss()));
}

#[test]
fn physical_limits_are_resolved_separately_for_each_paper_unit() {
    let mut d = empty();
    d.length_unit = "m".into();
    let id = d.id_counters.allocate_layout_id().unwrap();
    d.paper_layouts.push(IfccadPaperLayout {
        settings: common::paper_settings(ocdraw::plot_kernel::PlotUnit::Millimetre, 1.),
        id,
        name: "Millimetre sheet".into(),
        tab_index: 1,
        bounds: None,
        entities: vec![],
    });
    let outcome = ifccad_document_to_cad_document(
        &d,
        IfccadToCadOptions {
            geometry_tolerance: IfccadGeometryTolerance::millimetres(0.001).unwrap(),
            ..Default::default()
        },
    )
    .unwrap();
    let domains = outcome.geometry_assessment().domains();
    let drawing = domains
        .iter()
        .find(|v| v.domain() == IfccadGeometryDomain::Drawing)
        .unwrap();
    let paper = domains
        .iter()
        .find(|v| v.domain() == IfccadGeometryDomain::PaperLayout(id))
        .unwrap();
    assert!(
        drawing.resolved_tolerance().lower() <= 1e-6
            && drawing.resolved_tolerance().upper() >= 1e-6
    );
    assert!(
        paper.resolved_tolerance().lower() <= 0.001 && paper.resolved_tolerance().upper() >= 0.001
    );
}
