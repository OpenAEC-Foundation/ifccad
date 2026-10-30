use ifccad::geometry_kernel::hatch::{validate_hatch_boundaries, HatchBoundary2};
use ifccad::geometry_kernel::*;
use ifccad::*;

#[test]
fn coincident_bulges_are_points_with_exact_bounds_and_retained_native_parameters() {
    for bulge in [0.414213562373095, -0.414213562373095, f64::MAX] {
        let x = if bulge == f64::MAX { -0. } else { 2. };
        let next_x = if x == 0. { 0. } else { x };
        let vertices = [[x, 3., bulge], [next_x, 3., -0.]];
        let g = GeometryRef::PlanarPolyline {
            widths: &Default::default(),
            placement: CoordinateFrame3::default(),
            vertices: PlanarVertices::Packed(&vertices),
            closed: true,
            thickness: 0.,
        };
        let bounds = geometry_bounds(g).unwrap();
        assert_eq!(bounds.min().components(), [x, 3., 0.]);
        assert_eq!(bounds.max().components(), [next_x, 3., 0.]);
        let mut d = load_ifccad_bytes(
            include_bytes!("fixtures/ifccad/hello-cad.ifcx"),
            Default::default(),
        )
        .unwrap()
        .into_document();
        d.model.bounds = None;
        d.model.entities[0].as_native_mut().unwrap().kind = IfccadEntityKind::PlanarPolyline {
            widths: Default::default(),
            vertices: vec![[x, 3.], [next_x, 3.]],
            bulges: vec![bulge, -0.],
            closed: true,
            placement: IfccadPlacement {
                origin: [0.; 3],
                x_axis: [1., 0., 0.],
                y_axis: [0., 1., 0.],
            },
            line_pattern_generation: IfccadLinePatternGeneration::Continuous,
            thickness: 0.,
        };
        let json = encode_ifccad_document(&d).unwrap();
        let json = load_ifccad_bytes(json.bytes(), Default::default()).unwrap();
        let cbor = encode_ifccad_cbor_document(json.document()).unwrap();
        let cbor = load_ifccad_cbor_bytes(cbor.bytes(), Default::default()).unwrap();
        let IfccadEntityKind::PlanarPolyline {
            vertices, bulges, ..
        } = &cbor.document().model.entities[0].as_native().unwrap().kind
        else {
            panic!()
        };
        assert_eq!(vertices, &vec![[x, 3.], [next_x, 3.]]);
        assert_eq!(vertices[0][0].to_bits(), x.to_bits());
        assert_eq!(vertices[1][0].to_bits(), next_x.to_bits());
        assert_eq!(bulges[0].to_bits(), bulge.to_bits());
        assert_eq!(bulges[1].to_bits(), (-0_f64).to_bits());
    }
}

#[test]
fn coincident_bulges_do_not_supply_curvature_for_two_point_contours() {
    let vertices = [[0., 0., 1.], [0., 0., 0.], [1., 0., 0.]];
    let g = GeometryRef::PlanarPolyline {
        widths: &Default::default(),
        placement: CoordinateFrame3::default(),
        vertices: PlanarVertices::Packed(&vertices),
        closed: true,
        thickness: 0.,
    };
    validate_geometry(g).unwrap();
    let frame = PaperFrame {
        center: [0., 0.],
        width: 10.,
        height: 10.,
    };
    assert!(validate_paper_boundary(frame, g).is_err());
    let boundary = HatchBoundary2::Polyline {
        vertices: vec![[0., 0.], [0., 0.], [1., 0.]],
        bulges: vec![1., 0., 0.],
    };
    assert!(validate_hatch_boundaries([&boundary], 1e-9).is_err());
    let vertices = [[0., 0., 1.], [0., 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    let g = GeometryRef::PlanarPolyline {
        widths: &Default::default(),
        placement: CoordinateFrame3::default(),
        vertices: PlanarVertices::Packed(&vertices),
        closed: true,
        thickness: 0.,
    };
    validate_paper_boundary(frame, g).unwrap();
    let boundary = HatchBoundary2::Polyline {
        vertices: vec![[0., 0.], [0., 0.], [1., 0.], [0., 1.]],
        bulges: vec![1., 0., 0., 0.],
    };
    validate_hatch_boundaries([&boundary], 1e-9).unwrap();
}
