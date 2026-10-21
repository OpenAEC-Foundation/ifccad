#[path = "support/ifccad_dimension_graphics.rs"]
mod support;
use ifccad::*;

#[test]
fn picture_text_keeps_the_existing_estimated_extent_quality() {
    let catalogue = load_ifccad_bytes(
        include_bytes!("../examples/ifccad/entities.ifcx"),
        Default::default(),
    )
    .unwrap();
    let text = catalogue
        .document()
        .model
        .entities
        .iter()
        .find_map(|e| match &e.as_native()?.kind {
            IfccadEntityKind::Text(t) => Some(t.clone()),
            _ => None,
        })
        .unwrap();
    let mut d = support::drawing();
    support::dimension(&mut d).graphics.as_mut().unwrap().parts =
        vec![support::part(IfccadDimensionGraphicsKind::Text(text))];
    recompute_ifccad_document_bounds(&mut d).unwrap();
    assert!(d.model.bounds.is_some());
    assert_eq!(d.model.bounds_quality, Some(IfccadBoundsQuality::Estimated));
}

#[test]
fn bounds_use_stored_parts_in_parent_frame_and_ignore_reference_helpers() {
    let mut d = support::drawing();
    let dimension = support::dimension(&mut d);
    dimension.placement.origin = [100., 20., 30.];
    let g = dimension.graphics.as_mut().unwrap();
    let mut hidden = support::part(IfccadDimensionGraphicsKind::LineSegment {
        start: [-5., -7., 0.],
        end: [25., 4., 0.],
    });
    hidden.visible = false;
    g.parts.push(hidden);
    let mut helper = support::part(IfccadDimensionGraphicsKind::Point {
        placement: IfccadPlacement {
            origin: [1e10; 3],
            x_axis: [1., 0., 0.],
            y_axis: [0., 1., 0.],
        },
    });
    helper.purpose = IfccadDimensionGraphicsPurpose::ReferencePoint;
    g.parts.push(helper);
    recompute_ifccad_document_bounds(&mut d).unwrap();
    let b = d.model.bounds.expect("stored picture provides bounds");
    for i in 0..3 {
        assert!(b.min[i] <= [95., 13., 30.][i]);
        assert!(b.max[i] >= [125., 24., 30.][i]);
        assert!(b.max[i] - b.min[i] < 40.);
    }
    assert_eq!(d.model.bounds_quality, Some(IfccadBoundsQuality::Enclosing));
    validate_ifccad_document(&d).unwrap();
}

#[test]
fn absent_empty_and_helper_only_pictures_have_distinct_completeness() {
    let mut d = support::drawing();
    for state in 0..3 {
        let dimension = support::dimension(&mut d);
        dimension.graphics = match state {
            0 => None,
            1 => Some(IfccadDimensionGraphics {
                origin: IfccadDimensionGraphicsOrigin::Generated,
                parts: vec![],
            }),
            _ => {
                let mut helper = support::part(IfccadDimensionGraphicsKind::Point {
                    placement: IfccadPlacement {
                        origin: [1e10; 3],
                        x_axis: [1., 0., 0.],
                        y_axis: [0., 1., 0.],
                    },
                });
                helper.purpose = IfccadDimensionGraphicsPurpose::ReferencePoint;
                Some(IfccadDimensionGraphics {
                    origin: IfccadDimensionGraphicsOrigin::Generated,
                    parts: vec![helper],
                })
            }
        };
        let result = derive_ifccad_geometry_completeness(&d).unwrap();
        assert_eq!(
            result[&IfccadScopeId::Layout(d.model.id)],
            if state == 0 {
                IfccadGeometryCompleteness::Unavailable
            } else {
                IfccadGeometryCompleteness::Empty
            }
        );
    }
}
