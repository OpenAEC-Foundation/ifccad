#[path = "support/ifccad_dimension.rs"]
mod support;
use ifccad::*;
fn dimension(d: &mut IfccadDocument) -> &mut IfccadDimension {
    match &mut d.model.entities[0].as_native_mut().unwrap().kind {
        IfccadEntityKind::Dimension(v) => v,
        _ => panic!("Dimension"),
    }
}
fn part(kind: IfccadDimensionGraphicsKind) -> IfccadDimensionGraphicsPart {
    IfccadDimensionGraphicsPart {
        layer_id: None,
        kind,
        appearance: IfccadEntityAppearance {
            color: IfccadMode::ByLayer,
            opacity: IfccadMode::ByLayer,
            line_pattern: IfccadMode::ByLayer,
            line_weight: IfccadMode::ByLayer,
        },
        visible: true,
        line_pattern_scale: 1.0,
        purpose: IfccadDimensionGraphicsPurpose::Display,
    }
}
fn line() -> IfccadDimensionGraphicsPart {
    part(IfccadDimensionGraphicsKind::LineSegment {
        start: [-0.0, 7.0, 0.0],
        end: [13.0, 7.0, 0.0],
        thickness: 0.,
        extrusion_direction: [0., 0., 1.],
    })
}
#[test]
fn graphics_are_owned_values_without_entity_allocations() {
    for origin in [
        IfccadDimensionGraphicsOrigin::SourceSnapshot,
        IfccadDimensionGraphicsOrigin::Generated,
    ] {
        let mut d = support::drawing();
        let counters = d.id_counters;
        for parts in [vec![], vec![line(), line()]] {
            dimension(&mut d).graphics = Some(IfccadDimensionGraphics { origin, parts });
            validate_ifccad_document(&d).unwrap();
            assert_eq!(d.id_counters, counters);
            assert_eq!(d.model.entities.len(), 1);
            if let Some(g) = &dimension(&mut d).graphics {
                if let Some(p) = g.parts.first() {
                    let IfccadDimensionGraphicsKind::LineSegment { start, .. } = &p.kind else {
                        panic!()
                    };
                    assert_eq!(start[0].to_bits(), (-0.0f64).to_bits());
                }
            }
        }
        dimension(&mut d).graphics = None;
        validate_ifccad_document(&d).unwrap();
    }
}
#[test]
fn graphics_do_not_validate_snapshot_freshness() {
    let mut d = support::drawing();
    dimension(&mut d).graphics = Some(IfccadDimensionGraphics {
        origin: IfccadDimensionGraphicsOrigin::SourceSnapshot,
        parts: vec![line()],
    });
    dimension(&mut d).placement.origin = [500., -60., 30.];
    // The stored 13-unit picture is deliberately unrelated to the 4000-unit definition.
    validate_ifccad_document(&d).unwrap();
    assert_eq!(
        dimension(&mut d).graphics.as_ref().unwrap().parts[0],
        line()
    );
}
#[test]
fn reference_point_requires_point() {
    let mut d = support::drawing();
    let mut p = line();
    p.purpose = IfccadDimensionGraphicsPurpose::ReferencePoint;
    dimension(&mut d).graphics = Some(IfccadDimensionGraphics {
        origin: IfccadDimensionGraphicsOrigin::Generated,
        parts: vec![p],
    });
    let e = validate_ifccad_document(&d).unwrap_err();
    assert!(format!("{e:?}").contains("IFCCAD-DIMENSION-GRAPHICS-001"));
    let p = &mut dimension(&mut d).graphics.as_mut().unwrap().parts[0];
    p.kind = IfccadDimensionGraphicsKind::Point {
        placement: IfccadPlacement {
            origin: [1e10, 2., 3.],
            x_axis: [1., 0., 0.],
            y_axis: [0., 1., 0.],
        },
        thickness: 0.,
    };
    validate_ifccad_document(&d).unwrap();
    let p = &mut dimension(&mut d).graphics.as_mut().unwrap().parts[0];
    p.visible = false;
    p.line_pattern_scale = f64::NAN;
    assert!(validate_ifccad_document(&d).is_err());
}
#[test]
fn graphics_validate_all_parts_and_marker_closures() {
    let mut d = support::drawing();
    let base = dimension(&mut d).placement.clone();
    let mut p = part(IfccadDimensionGraphicsKind::Circle {
        placement: base.clone(),
        radius: -1.,
        thickness: 0.,
    });
    p.visible = false;
    dimension(&mut d).graphics = Some(IfccadDimensionGraphics {
        origin: IfccadDimensionGraphicsOrigin::SourceSnapshot,
        parts: vec![line(), p],
    });
    assert!(validate_ifccad_document(&d).is_err());
    let g = dimension(&mut d).graphics.as_mut().unwrap();
    g.parts[1] = part(IfccadDimensionGraphicsKind::BlockInstance {
        definition_id: u64::MAX,
        transform: IfccadBlockTransform {
            placement: base,
            rotation: 0.,
            scale: [1.; 3],
        },
    });
    let e = validate_ifccad_document(&d).unwrap_err();
    assert!(format!("{e:?}").contains("IFCCAD-DIMENSION-GRAPHICS-002"));
    let g = dimension(&mut d).graphics.as_mut().unwrap();
    g.parts.pop();
    g.parts[0].layer_id = Some(u64::MAX);
    assert!(validate_ifccad_document(&d).is_err());
}

#[test]
fn graphics_marker_closure_rejects_nested_dimensions_and_cycles() {
    let mut d = support::drawing();
    let mut b = load_ifccad_bytes(
        include_bytes!("fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .document()
    .blocks[0]
        .clone();
    b.id = 1;
    b.bounds = None;
    b.bounds_quality = None;
    b.entities = d.model.entities.clone();
    b.entities[0].as_native_mut().unwrap().id = 1;
    d.blocks = vec![b];
    d.id_counters.next_block_id = 2;
    let placement = dimension(&mut d).placement.clone();
    dimension(&mut d).graphics = Some(IfccadDimensionGraphics {
        origin: IfccadDimensionGraphicsOrigin::SourceSnapshot,
        parts: vec![part(IfccadDimensionGraphicsKind::BlockInstance {
            definition_id: 1,
            transform: IfccadBlockTransform {
                placement: placement.clone(),
                rotation: 0.,
                scale: [1.; 3],
            },
        })],
    });
    assert!(format!("{:?}", validate_ifccad_document(&d).unwrap_err())
        .contains("IFCCAD-DIMENSION-GRAPHICS-001"));
    d.blocks[0].entities[0].as_native_mut().unwrap().kind = IfccadEntityKind::BlockInstance {
        definition_id: 1,
        transform: IfccadBlockTransform {
            placement,
            rotation: 0.,
            scale: [1.; 3],
        },
    };
    assert!(format!("{:?}", validate_ifccad_document(&d).unwrap_err()).contains("cycle"));
}
#[test]
fn graphics_preserve_all_finite_filled_topologies_and_signed_thickness() {
    let mut d = support::drawing();
    let placement = dimension(&mut d).placement.clone();
    let mut parts = vec![];
    for corners in [
        [[0., 0.], [4., 0.], [0., 3.], [4., 3.]],
        [[0., 0.], [4., 3.], [0., 3.], [4., 0.]],
        [[0.; 2]; 4],
    ] {
        for thickness in [-3., -0., 0., 3.] {
            let v = IfccadFilledQuadrilateral {
                placement: placement.clone(),
                corners,
                thickness,
            };
            parts.push(part(IfccadDimensionGraphicsKind::Solid(v.clone())));
            parts.push(part(IfccadDimensionGraphicsKind::Trace(v)));
        }
    }
    dimension(&mut d).graphics = Some(IfccadDimensionGraphics {
        origin: IfccadDimensionGraphicsOrigin::Generated,
        parts,
    });
    validate_ifccad_document(&d).unwrap();
    // Both production writers must retain every stored part.
    assert!(encode_ifccad_document(&d).is_ok());
    assert!(encode_ifccad_cbor_document(&d).is_ok());
}

#[test]
fn definition_only_markers_keep_the_existing_ordinary_entity_profile() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../examples/ifccad/entities.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let mut e=d.model.entities.iter().find_map(|e|match e.as_native(){Some(n) if matches!(&n.kind,IfccadEntityKind::Hatch(h) if !matches!(h.fill,geometry_kernel::hatch::HatchFill::Solid))=>Some(e.clone()),_=>None}).unwrap();
    e.as_native_mut().unwrap().id = d.id_counters.allocate_entity_id().unwrap();
    let id = d.id_counters.allocate_block_id().unwrap();
    let mut block = d.blocks[0].clone();
    block.id = id;
    block.name = "Ordinary custom marker".into();
    block.entities = vec![e];
    block.bounds = None;
    block.bounds_quality = None;
    d.blocks.push(block);
    d.dimension_styles[0].settings.arrows.first =
        ifccad::dimension::DimensionMarker::LocalBlock { block: id };
    validate_ifccad_document(&d).unwrap();
}
