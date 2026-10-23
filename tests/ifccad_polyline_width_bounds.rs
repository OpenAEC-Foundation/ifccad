use ifccad::{geometry_kernel::*, *};
fn plane() -> CoordinateFrame3 {
    CoordinateFrame3::try_new(
        Point3::new(0., 0., 0.),
        Vector3::new(1., 0., 0.),
        Vector3::new(0., 1., 0.),
    )
    .unwrap()
}
#[test]
fn wide_single_segment_encloses_all_four_corners() {
    let vertices = [[0., 0., 0.], [10., 0., 0.]];
    let widths = PlanarPolylineWidths::PerVertex {
        widths: vec![[2., 4.], [0., 0.]],
    };
    let b = polyline_width_display_bounds(GeometryRef::PlanarPolyline {
        placement: plane(),
        vertices: PlanarVertices::Packed(&vertices),
        widths: &widths,
        closed: false,
    })
    .unwrap();
    let b = b.bounds.unwrap();
    assert_eq!(b.min().components(), [0., -2., 0.]);
    assert_eq!(b.max().components(), [10., 2., 0.]);
}
#[test]
fn arc_width_bound_encloses_full_band() {
    let vertices = [[0., 0., 1.], [10., 0., 0.]];
    let widths = PlanarPolylineWidths::Constant { width: 4. };
    let b = polyline_width_display_bounds(GeometryRef::PlanarPolyline {
        placement: plane(),
        vertices: PlanarVertices::Packed(&vertices),
        widths: &widths,
        closed: false,
    })
    .unwrap()
    .bounds
    .unwrap();
    assert!(b.min().x() <= -2. && b.min().y() <= -7.);
    assert!(b.max().x() >= 12. && b.max().y() >= 2.);
}
#[test]
fn general_join_is_partial_not_centerline_enclosing() {
    let vertices = [[0., 0., 0.], [10., 0., 0.], [10., 10., 0.]];
    let widths = PlanarPolylineWidths::Constant { width: 2. };
    let b = polyline_width_display_bounds(GeometryRef::PlanarPolyline {
        placement: plane(),
        vertices: PlanarVertices::Packed(&vertices),
        widths: &widths,
        closed: false,
    })
    .unwrap();
    assert!(b.bounds.is_none());
    assert!(b
        .reasons
        .contains(&PolylineWidthBoundsReason::UnassessedJoin));
    assert_eq!(b.proven_segment_bounds.len(), 2);
}
#[test]
fn wide_clip_uses_centerline_contour() {
    let vertices = [[1., 1., 0.], [9., 1., 0.], [9., 9., 0.], [1., 9., 0.]];
    let widths = PlanarPolylineWidths::Constant { width: 4. };
    let g = GeometryRef::PlanarPolyline {
        placement: plane(),
        vertices: PlanarVertices::Packed(&vertices),
        widths: &widths,
        closed: true,
    };
    let b = geometry_centerline_bounds(g).unwrap();
    assert_eq!(b.min().components(), [1., 1., 0.]);
    assert_eq!(b.max().components(), [9., 9., 0.]);
    validate_paper_boundary(
        PaperFrame {
            center: [5., 5.],
            width: 10.,
            height: 10.,
        },
        g,
    )
    .unwrap();
}
#[test]
fn default_edit_recomputes_only_explicitly() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../conformance/next/ifccad/valid/polyline-defaults-overrides.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let mut e = d
        .model
        .entities
        .iter()
        .filter_map(IfccadEntity::as_native)
        .find(|e| matches!(e.kind, IfccadEntityKind::PlanarPolyline { .. }))
        .unwrap()
        .clone();
    e.kind = IfccadEntityKind::PlanarPolyline {
        placement: IfccadPlacement {
            origin: [0.; 3],
            x_axis: [1., 0., 0.],
            y_axis: [0., 1., 0.],
        },
        vertices: vec![[0., 0.], [10., 0.]],
        bulges: vec![0.; 2],
        closed: false,
        line_pattern_generation: Default::default(),
        widths: IfccadPlanarPolylineWidths::Defaulted {
            default_start_width: 2.,
            default_end_width: 2.,
            overrides: vec![Default::default(); 2],
        },
    };
    d.model.entities = vec![IfccadEntity::Native(e)];
    d.blocks.clear();
    d.model.bounds = None;
    d.model.bounds_quality = None;
    recompute_ifccad_document_bounds(&mut d).unwrap();
    let previous = d.model.bounds.unwrap();
    assert_eq!(previous.min[1], -1.);
    if let IfccadEntityKind::PlanarPolyline {
        widths:
            IfccadPlanarPolylineWidths::Defaulted {
                default_start_width,
                ..
            },
        ..
    } = &mut d.model.entities[0].as_native_mut().unwrap().kind
    {
        *default_start_width = 4.;
    }
    assert_eq!(d.model.bounds, Some(previous));
    recompute_ifccad_document_bounds(&mut d).unwrap();
    assert_eq!(d.model.bounds.unwrap().min[1], -2.);
}

#[test]
fn coincident_wide_segment_is_partial_and_continuous_circle_is_enclosing() {
    let widths = PlanarPolylineWidths::Constant { width: 2. };
    for (vertices, closed, complete) in [
        (vec![[0., 0., 0.], [0., 0., 0.]], false, false),
        (vec![[0., 0., 1.], [10., 0., 1.]], true, true),
        (
            vec![[0., 0., 0.], [10., 0., 0.], [0., 0., 0.]],
            false,
            false,
        ),
    ] {
        let b = polyline_width_display_bounds(GeometryRef::PlanarPolyline {
            placement: plane(),
            vertices: PlanarVertices::Packed(&vertices),
            widths: &widths,
            closed,
        })
        .unwrap();
        assert_eq!(b.bounds.is_some(), complete);
    }
}
#[test]
fn hatch_source_keeps_centerline_identity_and_loop_records() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../conformance/next/ifccad/valid/hatch-associations-unbound.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    d.model.bounds = None;
    d.model.bounds_quality = None;
    let before = d
        .model
        .entities
        .iter()
        .filter_map(IfccadEntity::as_native)
        .find_map(|e| {
            if let IfccadEntityKind::Hatch(h) = &e.kind {
                Some(h.clone())
            } else {
                None
            }
        })
        .unwrap();
    let source = d
        .model
        .entities
        .iter_mut()
        .filter_map(IfccadEntity::as_native_mut)
        .find(|e| matches!(e.kind, IfccadEntityKind::Circle { .. }))
        .unwrap();
    let placement = if let IfccadEntityKind::Circle { placement, .. } = &source.kind {
        placement.clone()
    } else {
        unreachable!()
    };
    source.kind = IfccadEntityKind::PlanarPolyline {
        vertices: vec![[-1., -1.], [1., -1.], [1., 1.], [-1., 1.]],
        bulges: vec![0.; 4],
        closed: true,
        placement,
        line_pattern_generation: Default::default(),
        widths: IfccadPlanarPolylineWidths::Constant { width: 4. },
    };
    validate_ifccad_document(&d).unwrap();
    let bytes = encode_ifccad_document(&d).unwrap();
    let back = load_ifccad_bytes(bytes.bytes(), Default::default()).unwrap();
    let after = back
        .document()
        .model
        .entities
        .iter()
        .filter_map(IfccadEntity::as_native)
        .find_map(|e| {
            if let IfccadEntityKind::Hatch(h) = &e.kind {
                Some(h)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(after, &before);
}
#[test]
fn wide_partial_box_checks_known_contributions_without_claiming_verified_enclosure() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../conformance/next/ifccad/valid/polyline-defaults-overrides.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let e = d
        .model
        .entities
        .iter()
        .filter_map(IfccadEntity::as_native)
        .find(|e| matches!(e.kind, IfccadEntityKind::PlanarPolyline { .. }))
        .unwrap()
        .clone();
    d.model.entities = vec![IfccadEntity::Native(e)];
    d.blocks.clear();
    d.model.bounds = Some(IfccadBounds3d {
        min: [-10., -10., -1.],
        max: [40., 10., 1.],
    });
    d.model.bounds_quality = Some(IfccadBoundsQuality::Enclosing);
    validate_ifccad_document(&d).unwrap();
    let a = assess_ifccad_document_bounds(&d).unwrap();
    let s = &a.scopes[&IfccadScopeId::Layout(d.model.id)];
    assert_eq!(s.quality, Some(IfccadBoundsQuality::Partial));
    assert!(!s.enclosure_verified);
    assert!(s.bounds.is_none());
    assert!(!s.polyline_width_reasons.is_empty());
    assert!(s.polyline_width_reasons[0].source_path.contains("/e1/"));
    d.model.bounds.as_mut().unwrap().max[0] = 1.;
    assert!(validate_ifccad_document(&d).is_err());
}
#[test]
fn overflowed_band_recomputation_is_transactional() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../conformance/next/ifccad/valid/polyline-widths.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let e = d
        .model
        .entities
        .iter_mut()
        .filter_map(IfccadEntity::as_native_mut)
        .find(|e| matches!(e.kind, IfccadEntityKind::PlanarPolyline { .. }))
        .unwrap();
    if let IfccadEntityKind::PlanarPolyline {
        vertices,
        bulges,
        widths,
        ..
    } = &mut e.kind
    {
        *vertices = vec![[f64::MAX, 0.], [f64::MAX, 1.]];
        *bulges = vec![0.; 2];
        *widths = IfccadPlanarPolylineWidths::Constant { width: f64::MAX };
    }
    let before = d.clone();
    assert!(recompute_ifccad_document_bounds(&mut d).is_err());
    assert_eq!(d, before);
}
