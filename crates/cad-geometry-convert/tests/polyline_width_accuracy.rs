use cad_geometry_convert::geometry::{
    blocks::EvaluatedBlock,
    numeric::exact,
    polyline_width::{pair_polyline_bands, PolylineBandOperands},
};
use ifccad::geometry_kernel::{BlockTransform, CoordinateFrame3, Point3, Scale3, Vector3};
use num_traits::Zero;
fn operands(bulge: f64) -> PolylineBandOperands {
    PolylineBandOperands {
        origin: [0., 0., 0.].map(exact),
        x_axis: [1., 0., 0.].map(exact),
        y_axis: [0., 1., 0.].map(exact),
        vertices: vec![[0., 0., bulge], [10., 0., 0.]],
        effective_widths: vec![[1., 2.], [0., 0.]],
        closed: false,
    }
}
#[test]
fn straight_taper_band_exposes_error_hidden_by_centerline() {
    let a = operands(0.);
    let mut b = a.clone();
    b.y_axis = [0., 2., 0.].map(exact);
    let bands = pair_polyline_bands(&a, &b).unwrap();
    assert_eq!(bands.len(), 1);
    assert_eq!(bands[0].segment_index(), 0);
    let (lower, upper) = bands[0].squared_deviation().unwrap();
    assert_eq!(lower, exact(1.));
    assert_eq!(upper, exact(1.));
}
#[test]
fn bulged_band_checks_both_sides() {
    let a = operands(1.);
    let mut b = a.clone();
    b.effective_widths[0] = [2., 3.];
    let bands = pair_polyline_bands(&a, &b).unwrap();
    assert!(bands[0].squared_deviation().unwrap().1 >= exact(0.25));
}
#[test]
fn identical_band_is_exact() {
    for bulge in [0., 1., -1., 2., -2., 0.7] {
        let a = operands(bulge);
        let bands = pair_polyline_bands(&a, &a).unwrap();
        assert!(bands[0].squared_deviation().unwrap().1.is_zero());
    }
}
#[test]
fn outer_nonuniform_insert_amplifies_band_error() {
    let a = operands(0.);
    let mut b = a.clone();
    b.y_axis = [0., 2., 0.].map(exact);
    let mut bands = pair_polyline_bands(&a, &b).unwrap();
    let frame = CoordinateFrame3::try_new(
        Point3::new(100., 50., 0.),
        Vector3::new(1., 0., 0.),
        Vector3::new(0., 1., 0.),
    )
    .unwrap();
    let transform = BlockTransform::try_new(frame, 0., Scale3::new(1., 4., 1.)).unwrap();
    let map = EvaluatedBlock::native(transform, [0.; 3]);
    bands[0].apply(&map, &map);
    assert_eq!(bands[0].squared_deviation().unwrap().1, exact(16.));
}
#[test]
fn new_target_collapse_fails_but_source_coincidence_stays_undefined() {
    let a = operands(0.);
    let mut b = a.clone();
    b.vertices[1] = b.vertices[0];
    assert!(pair_polyline_bands(&a, &b).is_err());
    assert!(pair_polyline_bands(&b, &b).unwrap().is_empty());
}

#[test]
fn discard_removes_band_registrations() {
    use cad_geometry_convert::{
        ExchangeState, GeometryAssessment, GeometryPair, GeometrySource, GeometryTolerance,
    };
    use ifccad::geometry_kernel::CoordinateLengthUnit;
    #[derive(Clone, Debug)]
    struct Source;
    impl GeometrySource for Source {
        fn cad_entity(_: opencadcodec::Handle, _: String) -> Self {
            Self
        }
        fn occurrence(_: Vec<Self>, _: Self) -> Self {
            Self
        }
    }
    let assessment = GeometryAssessment::new(
        GeometryTolerance::drawing_units(0.1).unwrap(),
        CoordinateLengthUnit::Unitless,
    )
    .unwrap();
    let mut state = ExchangeState::new(assessment);
    let a = operands(0.);
    let mut b = a.clone();
    b.y_axis = [0., 2., 0.].map(exact);
    let bands = pair_polyline_bands(&a, &b).unwrap();
    state.register_geometry(
        1u64,
        Source,
        GeometryPair {
            points: vec![],
            curves: vec![],
            bands,
        },
    );
    let frame = CoordinateFrame3::try_new(
        Point3::new(0., 0., 0.),
        Vector3::new(1., 0., 0.),
        Vector3::new(0., 1., 0.),
    )
    .unwrap();
    let map = EvaluatedBlock::native(
        BlockTransform::try_new(frame, 0., Scale3::new(1., 1., 1.)).unwrap(),
        [0.; 3],
    );
    state.register_identity(2, Source);
    state.record_instance_parts(2, 9, map.clone(), map);
    let members = std::collections::BTreeMap::from([(9, vec![1])]);
    assert!(state
        .assess_roots(&members, &[2], &mut state.assessment.clone())
        .is_err());
    state.discard_entity(1);
    assert!(state
        .assess_roots(&members, &[2], &mut state.assessment.clone())
        .unwrap()
        .is_empty());
}

#[test]
fn normal_vectors_do_not_acquire_occurrence_translation() {
    let a = operands(1.);
    let mut band = pair_polyline_bands(&a, &a).unwrap().remove(0);
    let frame = |x| {
        CoordinateFrame3::try_new(
            Point3::new(x, 0., 0.),
            Vector3::new(1., 0., 0.),
            Vector3::new(0., 1., 0.),
        )
        .unwrap()
    };
    let map = |x| {
        EvaluatedBlock::native(
            BlockTransform::try_new(frame(x), 0., Scale3::new(1., 1., 1.)).unwrap(),
            [0.; 3],
        )
    };
    band.apply(&map(100.), &map(101.));
    assert_eq!(band.squared_deviation().unwrap().1, exact(1.));
}
#[test]
fn prepared_pair_comparison_keeps_symbolic_band_correlation() {
    use cad_geometry_convert::GeometryPair;
    let a = operands(0.7);
    let pair = GeometryPair {
        points: vec![],
        curves: vec![],
        bands: pair_polyline_bands(&a, &a).unwrap(),
    };
    let selected = GeometryPair::from_prepared_source_and_target(&pair, &pair).unwrap();
    assert_eq!(selected.bands.len(), 1);
    assert!(selected.bands[0].squared_deviation().unwrap().1.is_zero());
    let selected = GeometryPair::from_prepared_sources(&pair, &pair).unwrap();
    assert!(selected.bands[0].squared_deviation().unwrap().1.is_zero());
}
#[test]
fn closed_final_band_and_nonorthogonal_stored_axes_are_retained() {
    let mut a = operands(-2.);
    a.closed = true;
    a.effective_widths[1] = [0.5, 0.];
    a.x_axis = [1., 0., 0.].map(exact);
    a.y_axis = [exact(0.0000000000000001), exact(1.), exact(0.)];
    let bands = pair_polyline_bands(&a, &a).unwrap();
    assert_eq!(bands.len(), 2);
    assert_eq!(bands[1].segment_index(), 1);
    for b in bands {
        assert!(b.squared_deviation().unwrap().1.is_zero());
    }
}
