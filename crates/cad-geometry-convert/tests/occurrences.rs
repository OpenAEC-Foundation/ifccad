use cad_geometry_convert::{
    ExchangeState, GeometryAssessment, GeometryFailureReason, GeometryPair, GeometrySource,
    GeometryTolerance,
};
use ocdraw::geometry_kernel::{BlockTransform, CoordinateFrame3, CoordinateLengthUnit, Scale3};
use opencadcodec::Handle;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
enum Source {
    Entity(u64),
    Occurrence {
        path: Vec<Source>,
        leaf: Box<Source>,
    },
}
impl GeometrySource for Source {
    fn cad_entity(h: Handle, _: String) -> Self {
        Self::Entity(h.value())
    }
    fn occurrence(path: Vec<Self>, leaf: Self) -> Self {
        Self::Occurrence {
            path,
            leaf: Box::new(leaf),
        }
    }
}
fn assessment(limit: f64) -> GeometryAssessment<Source> {
    GeometryAssessment::new(
        GeometryTolerance::drawing_units(limit).unwrap(),
        CoordinateLengthUnit::Unitless,
    )
    .unwrap()
}

#[test]
fn occurrence_limit_uses_root_coordinates_without_narrowing_ids() {
    let high = 9_007_199_254_740_993_u64;
    let mut state = ExchangeState::new(assessment(1.));
    state
        .record_geometry(
            high,
            Source::Entity(high),
            GeometryPair::points(&[[1., 0., 0.]], &[[1.25, 0., 0.]]).unwrap(),
        )
        .unwrap();
    let scale = |value| {
        BlockTransform::try_new(
            CoordinateFrame3::default(),
            0.,
            Scale3::new(value, value, value),
        )
        .unwrap()
    };
    let a = cad_geometry_convert::geometry::blocks::EvaluatedBlock::native(scale(-8.), [0.; 3]);
    let b = cad_geometry_convert::geometry::blocks::EvaluatedBlock::native(scale(2.), [0.; 3]);
    state.register_identity(3, Source::Entity(3));
    state.record_instance_parts(3, 10, a.clone(), a);
    state.register_identity(7, Source::Entity(7));
    state.record_instance_parts(7, 5, b.clone(), b);
    let members = BTreeMap::from([(10, vec![high]), (5, vec![3])]);
    let error = state
        .assess_roots(&members, &[7], &mut assessment(1.))
        .unwrap_err();
    assert_eq!(error.reason, GeometryFailureReason::ProvenExceedance);
    let Source::Occurrence { path, leaf } = error.source else {
        panic!("occurrence source")
    };
    assert_eq!(path, vec![Source::Entity(7), Source::Entity(3)]);
    assert_eq!(*leaf, Source::Entity(high));
    let records = state
        .assess_roots(&members, &[7], &mut assessment(4.))
        .unwrap();
    assert_eq!(records[0].root, 7);
    assert_eq!(records[0].deviation.lower(), 4.);
    assert_eq!(records[0].deviation.upper(), 4.);
}

#[test]
fn equal_endpoints_do_not_certify_curve_interiors() {
    use cad_geometry_convert::geometry::{
        blocks::{PairedCurve, Range},
        numeric::exact,
    };
    let axis = |v: [f64; 3]| v.map(|n| Range::point(exact(n)));
    let pair = GeometryPair {
        points: GeometryPair::points(
            &[[-1., 0., 0.], [1., 0., 0.]],
            &[[-1., 0., 0.], [1., 0., 0.]],
        )
        .unwrap()
        .points,
        curves: vec![PairedCurve::new(
            [0.; 3].map(exact),
            [axis([1., 0., 0.]), axis([0., 1., 0.])],
            [0.; 3].map(exact),
            [axis([1., 0., 0.]), axis([0., 2., 0.])],
        )],
    };
    let mut state = ExchangeState::new(assessment(0.5));
    assert!(state
        .record_geometry(1, Source::Entity(1), pair.clone())
        .is_err());
    let mut state = ExchangeState::new(assessment(1.));
    assert_eq!(
        state.record_geometry(1, Source::Entity(1), pair).unwrap(),
        1.
    );
}
