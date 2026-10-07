use ocdraw_convert::{
    opencadcodec::{objects::ObjectType, CadDocument},
    *,
};
fn source() -> CadDocument {
    let mut c = CadDocument::new();
    c.header.insertion_units = 4;
    c.add_layout("Second").unwrap();
    for o in c.objects.values_mut() {
        if let ObjectType::Layout(l) = o {
            if l.name != "Model" {
                l.paper_width = 1000.;
                l.paper_height = 1000.;
                l.plot_paper_units = 1;
                l.plot_flags.use_standard_scale = false;
                l.plot_scale_numerator = if l.name == "Second" { 2. } else { 1. };
                l.plot_scale_denominator = 1.;
                l.plot_scale_type = 0;
            }
        }
    }
    c
}
#[test]
fn paper_limits_and_meaning_are_per_layout() {
    let out = cad_document_to_ocdraw_document(&source(), Default::default()).unwrap();
    let domains = out.geometry_assessment().domains();
    assert_eq!(domains.len(), 3);
    let first = domains
        .iter()
        .find(|d| d.domain() == OcdrawGeometryDomain::PaperLayout(1))
        .unwrap();
    let second = domains
        .iter()
        .find(|d| d.domain() == OcdrawGeometryDomain::PaperLayout(2))
        .unwrap();
    assert!(
        first.resolved_tolerance().lower() <= 0.001 && first.resolved_tolerance().upper() >= 0.001
    );
    assert!(
        second.resolved_tolerance().lower() <= 0.0005
            && second.resolved_tolerance().upper() >= 0.0005
    );
}
#[test]
fn explicit_physical_request_refuses_unknown_paper_even_if_empty() {
    let mut c = CadDocument::new();
    c.header.insertion_units = 4;
    c.add_layout("Empty").unwrap();
    assert!(matches!(
        cad_document_to_ocdraw_document(
            &c,
            CadToOcdrawOptions {
                geometry_tolerance: OcdrawGeometryTolerance::millimetres(0.).unwrap(),
                ..Default::default()
            }
        ),
        Err(CadToOcdrawError::PaperTolerance(e)) if e.layout_id==1 && e.reason==OcdrawToleranceError::PhysicalMappingRequired
    ));
}

#[test]
fn physical_request_cannot_certify_a_target_that_lost_its_plot_mapping() {
    let native = ocdraw::ocdraw::load_ocdraw_bytes(include_bytes!(
        "../../../conformance/next/ocdraw/valid/layout-plot-inch.ocdraw.json"
    ))
    .unwrap();
    let mut doc = native.into_document();
    doc.unit = "mm".into();
    doc.layouts[1].settings.media.as_mut().unwrap().unit = ocdraw::plot_kernel::MediaUnit::Physical(
        ocdraw::geometry_kernel::CoordinateLengthUnit::Parsec,
    );
    assert!(matches!(
        ocdraw_document_to_cad_document(
            &doc,
            OcdrawToCadOptions {
                geometry_tolerance: OcdrawGeometryTolerance::millimetres(1.).unwrap(),
                ..Default::default()
            }
        ),
        Err(OcdrawToCadError::PaperTolerance(e)) if e.layout_id==1 && e.reason==OcdrawToleranceError::PhysicalMappingRequired
    ));
}

#[test]
fn nested_signed_scaling_is_checked_in_paper_output_domain() {
    use ocdraw::ocdraw::*;
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("nested-paper", "mm")).unwrap();
    b.ensure_continuous_line_pattern().unwrap();
    let layer = b
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(255, 255, 255),
            LinePatternId(0),
        ))
        .unwrap();
    let paper = b.add_paper_layout("Sheet").unwrap();
    let leaf = b
        .add_block_definition(BlockDefinition::new("Leaf"))
        .unwrap();
    let outer = b
        .add_block_definition(BlockDefinition::new("Outer"))
        .unwrap();
    let placement = CoordinateFrame3::try_new(
        Point3::new(1e20, 0., 0.),
        Vector3::new(1., 0., 0.),
        Vector3::new(0., 1., 0.),
    )
    .unwrap();
    b.add_geometric_entity(GeometricEntityDefinition::new(
        leaf,
        layer,
        DrawingGeometry::PlanarPolyline {
            placement,
            vertices: vec![[0.25, 0., 0.], [0.5, 1., 0.]],
            closed: false,
            line_pattern_generation: LinePatternGeneration::PerSegment,
        },
    ))
    .unwrap();
    b.add_block_instance(BlockInstanceDefinition::new(layer, leaf, [0.; 3]).in_scope(outer))
        .unwrap();
    let transform =
        BlockTransform::try_new(CoordinateFrame3::default(), 0., Scale3::new(-8., 2., 1.)).unwrap();
    b.add_geometric_entity(GeometricEntityDefinition::new(
        paper,
        layer,
        DrawingGeometry::BlockInstance {
            definition_scope_id: outer,
            transform,
        },
    ))
    .unwrap();
    let template = load_ocdraw_bytes(include_bytes!(
        "../../../conformance/next/ocdraw/valid/layout-plot-inch.ocdraw.json"
    ))
    .unwrap();
    let mut settings = template.document().layouts[1].settings.clone();
    settings.plot_settings.as_mut().unwrap().plot_unit = PlotUnit::Millimetre;
    settings.plot_settings.as_mut().unwrap().mapping.scale = PlotScale::Fixed {
        output_length: 2.,
        scope_length: 1.,
    };
    b.set_layout_settings(paper, settings).unwrap();
    let mut doc = b.build_document().unwrap();
    for loss_policy in [OcdrawLossPolicy::Allow, OcdrawLossPolicy::Reject] {
        let err = ocdraw_document_to_cad_document(
            &doc,
            OcdrawToCadOptions {
                loss_policy,
                geometry_tolerance: OcdrawGeometryTolerance::millimetres(1.).unwrap(),
                ..Default::default()
            },
        )
        .err()
        .expect("Paper occurrence must exceed");
        assert!(
            matches!(err,OcdrawToCadError::Geometry(f) if f.domain==OcdrawGeometryDomain::PaperLayout(paper) && matches!(&f.source,OcdrawGeometryEntitySource::BlockOccurrence{path,..} if path.len()==2))
        );
    }
    doc.layouts[1]
        .settings
        .plot_settings
        .as_mut()
        .unwrap()
        .mapping
        .scale = PlotScale::Fixed {
        output_length: 0.1,
        scope_length: 1.,
    };
    let out = ocdraw_document_to_cad_document(
        &doc,
        OcdrawToCadOptions {
            geometry_tolerance: OcdrawGeometryTolerance::millimetres(1.).unwrap(),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        out.geometry_assessment()
            .domains()
            .iter()
            .find(|d| d.domain() == OcdrawGeometryDomain::PaperLayout(paper))
            .unwrap()
            .rounded_entities()
            > 0
    );
}
