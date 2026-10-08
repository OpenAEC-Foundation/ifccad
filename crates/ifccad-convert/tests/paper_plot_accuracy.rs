mod common;
use ifccad_convert::{
    opencadcodec::{objects::ObjectType, CadDocument},
    *,
};
#[test]
fn paper_mapping_is_exposed_without_an_authored_coordinate_unit() {
    let mut c = CadDocument::new();
    c.header.insertion_units = 4;
    for o in c.objects.values_mut() {
        if let ObjectType::Layout(l) = o {
            if l.name == "Layout1" {
                l.paper_width = 1000.;
                l.paper_height = 1000.;
                l.plot_paper_units = 1;
                l.plot_flags.use_standard_scale = false;
                l.plot_scale_numerator = 2.;
                l.plot_scale_denominator = 1.;
                l.plot_scale_type = 0;
            }
        }
    }
    let out = cad_document_to_ifccad_document(
        &c,
        common::metadata(),
        CadToIfccadOptions {
            geometry_tolerance: IfccadGeometryTolerance::millimetres(0.001).unwrap(),
            ..Default::default()
        },
    )
    .unwrap();
    let paper = out
        .geometry_assessment()
        .domains()
        .iter()
        .find(|d| matches!(d.domain(), IfccadGeometryDomain::PaperLayout(_)))
        .unwrap();
    assert!(matches!(
        paper.coordinate_meaning(),
        cad_geometry_convert::plot_units::GeometryCoordinateMeaning::PaperCoordinates { .. }
    ));
    assert!(
        paper.resolved_tolerance().lower() <= 0.0005
            && paper.resolved_tolerance().upper() >= 0.0005
    );
}

#[test]
fn missing_target_mapping_blocks_an_explicit_physical_request() {
    let native = ocdraw::ifccad::load_ifccad_bytes(
        include_bytes!("../../../conformance/next/ifccad/valid/layout-plot-inch.ifcx"),
        Default::default(),
    )
    .unwrap();
    let mut doc = native.into_document();
    doc.paper_layouts[0].settings.media.as_mut().unwrap().unit =
        ocdraw::ifccad::IfccadMediaUnit::Physical(
            ocdraw::geometry_kernel::CoordinateLengthUnit::Parsec,
        );
    assert!(matches!(
        ifccad_document_to_cad_document(
            &doc,
            IfccadToCadOptions {
                geometry_tolerance: IfccadGeometryTolerance::millimetres(1.).unwrap(),
                ..Default::default()
            }
        ),
        Err(IfccadConversionError::PaperTolerance {
            layout_id: 2,
            reason: IfccadToleranceError::PhysicalMappingRequired
        })
    ));
}

#[test]
fn nested_signed_occurrence_uses_paper_output_limit_under_both_policies() {
    use ocdraw::ifccad::*;
    let mut d = common::empty();
    let mut leaf = common::primitives().model.entities[1].clone();
    leaf.as_native_mut().unwrap().id = 1;
    if let IfccadEntityKind::PlanarPolyline {
        vertices,
        bulges,
        placement,
        closed,
        ..
    } = &mut leaf.as_native_mut().unwrap().kind
    {
        *vertices = vec![[0.25, 0.], [0.5, 1.]];
        *bulges = vec![0.; 2];
        *closed = false;
        placement.origin = [1e20, 0., 0.];
    } else {
        panic!("polyline fixture");
    }
    let mut inner = common::instance(2, 7, [0.; 3]);
    if let IfccadEntityKind::BlockInstance { transform, .. } =
        &mut inner.as_native_mut().unwrap().kind
    {
        transform.rotation = 0.;
        transform.scale = [1.; 3];
    }
    let mut outer = common::instance(3, 8, [0.; 3]);
    if let IfccadEntityKind::BlockInstance { transform, .. } =
        &mut outer.as_native_mut().unwrap().kind
    {
        transform.rotation = 0.;
        transform.scale = [-8., 2., 1.];
    }
    d.blocks = vec![
        IfccadBlockDefinition {
            bounds_quality: None,
            id: 7,
            name: "Leaf".into(),
            base_point: [0.; 3],
            insertion_unit: "mm".into(),
            bounds: None,
            entities: vec![leaf],
        },
        IfccadBlockDefinition {
            bounds_quality: None,
            id: 8,
            name: "Outer".into(),
            base_point: [0.; 3],
            insertion_unit: "mm".into(),
            bounds: None,
            entities: vec![inner],
        },
    ];
    d.paper_layouts = vec![IfccadPaperLayout {
        bounds_quality: None,
        canvas: None,

        id: 42,
        name: "Sheet".into(),
        tab_index: 1,
        bounds: None,
        settings: common::paper_settings(ocdraw::plot_kernel::PlotUnit::Millimetre, 2.),
        entities: vec![outer],
    }];
    d.id_counters.next_entity_id = 4;
    d.id_counters.next_block_id = 9;
    d.id_counters.next_layout_id = 43;
    for loss_policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        let err = ifccad_document_to_cad_document(
            &d,
            IfccadToCadOptions {
                preservation: Default::default(),
                loss_policy,
                geometry_tolerance: IfccadGeometryTolerance::millimetres(1.).unwrap(),
            },
        )
        .err()
        .expect("Paper exceedance");
        assert!(
            matches!(err,IfccadConversionError::Geometry(f) if f.domain==IfccadGeometryDomain::PaperLayout(42) && matches!(&f.source,IfccadGeometryEntitySource::BlockOccurrence{path,..} if path.len()==2))
        );
    }
    d.paper_layouts[0].settings =
        common::paper_settings(ocdraw::plot_kernel::PlotUnit::Millimetre, 0.1);
    let out = ifccad_document_to_cad_document(
        &d,
        IfccadToCadOptions {
            geometry_tolerance: IfccadGeometryTolerance::millimetres(1.).unwrap(),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        out.geometry_assessment()
            .domains()
            .iter()
            .find(|a| a.domain() == IfccadGeometryDomain::PaperLayout(42))
            .unwrap()
            .rounded_entities()
            > 0
    );
}
