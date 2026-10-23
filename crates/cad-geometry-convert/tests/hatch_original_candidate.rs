use cad_geometry_convert::{
    geometry::{blocks::EvaluatedBlock, numeric::exact},
    hatch::pattern::pattern_candidate_pair_to_cad,
    ExchangeState, GeometryAssessment, GeometryPair, GeometrySource, GeometryTolerance,
};
use ifccad::geometry_kernel::{hatch::*, BlockTransform, CoordinateLengthUnit, Scale3};
use opencadcodec::{entities::hatch::*, Handle, Vector2};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
struct Source;
impl GeometrySource for Source {
    fn cad_entity(_: Handle, _: String) -> Self {
        Self
    }
    fn occurrence(_: Vec<Self>, _: Self) -> Self {
        Self
    }
}
fn assessment(limit: f64) -> GeometryAssessment<Source> {
    GeometryAssessment::new(
        GeometryTolerance::drawing_units(limit).unwrap(),
        CoordinateLengthUnit::Unitless,
    )
    .unwrap()
}
fn fixture(radius: f64) -> (HatchLinePattern, Hatch, Vec<HatchBoundary2>) {
    let p = HatchLinePattern {
        name: Some("Original".into()),
        description: Some(String::new()),
        origin: [0., 0.],
        rotation: 0.,
        scale: 1.,
        families: vec![HatchLineFamily {
            angle: 0.,
            base_point: [0., 0.],
            offset: [0., 2.],
            dashes: vec![],
        }],
    };
    let mut h = Hatch::with_pattern(HatchPattern::new("Original"));
    h.pattern.lines.push(HatchPatternLine {
        angle: 0.,
        base_point: Vector2::ZERO,
        offset: Vector2::new(0., 2.),
        dash_lengths: vec![],
    });
    (
        p,
        h,
        vec![HatchBoundary2::Circle {
            center: [0., 0.],
            radius,
        }],
    )
}
fn accepted(pair: &GeometryPair, limit: f64) -> bool {
    let a = assessment(limit);
    pair.points.iter().enumerate().all(|(i, (p, _))| {
        let (lo, hi) = p.squared_deviation();
        a.check_interval(&Source, i, &lo, &hi).is_ok()
    })
}

#[test]
fn original_candidate_not_regenerated_from_native() {
    let (p, mut h, b) = fixture(10.);
    h.pattern.lines[0].base_point.y = 2f64.powi(-40);
    let before = h.clone();
    let pair = pattern_candidate_pair_to_cad(&p, Default::default(), &b, &h, true).unwrap();
    assert!(!pair.points.is_empty());
    assert!(pair
        .points
        .iter()
        .all(|(p, _)| p.squared_deviation().0 > exact(0.)));
    assert_eq!(h, before);
}

#[test]
fn candidate_with_rounding_difference_is_certifiable() {
    let (p, mut h, b) = fixture(10.);
    h.pattern.lines[0].base_point.y = 2f64.powi(-40);
    let pair = pattern_candidate_pair_to_cad(&p, Default::default(), &b, &h, true).unwrap();
    assert!(accepted(&pair, 1e-9));
    assert!(!accepted(&pair, 0.));
}

#[test]
fn repetition_and_occurrence_amplify_small_parameter_differences() {
    let (p, mut h, b) = fixture(1e6);
    h.pattern.lines[0].offset.y += 2f64.powi(-40);
    let pair = pattern_candidate_pair_to_cad(&p, Default::default(), &b, &h, true).unwrap();
    assert!(!accepted(&pair, 1e-9));
    let (p, mut h, b) = fixture(10.);
    h.pattern.lines[0].base_point.y = 2f64.powi(-40);
    let pair = pattern_candidate_pair_to_cad(&p, Default::default(), &b, &h, true).unwrap();
    assert!(accepted(&pair, 1e-9));
    let mut state = ExchangeState::new(assessment(1e-9));
    state.register_geometry(1, Source, pair);
    for (key, definition, scale) in [
        (2, 10, Scale3::new(2., 1e6, 1.)),
        (3, 20, Scale3::new(1., 2., 1.)),
    ] {
        let block = EvaluatedBlock::native(
            BlockTransform::try_new(Default::default(), 0., scale).unwrap(),
            [0.; 3],
        );
        state.register_identity(key, Source);
        state.record_instance_parts(key, definition, block.clone(), block);
    }
    let members = BTreeMap::from([(10, vec![1]), (20, vec![2])]);
    assert!(state
        .assess_roots(&members, &[3], &mut assessment(1e-9))
        .is_err());
}

#[test]
fn family_and_dash_gap_dot_structure_must_match() {
    let (mut p, mut h, b) = fixture(10.);
    h.pattern.lines.push(h.pattern.lines[0].clone());
    assert!(pattern_candidate_pair_to_cad(&p, Default::default(), &b, &h, true).is_err());
    h.pattern.lines.pop();
    p.families[0].dashes = vec![
        HatchDash::Dash { length: 1. },
        HatchDash::Gap { length: 1. },
        HatchDash::Dot,
    ];
    for dashes in [vec![-1., 1., 0.], vec![1., 0., -1.], vec![1., -1.]] {
        h.pattern.lines[0].dash_lengths = dashes;
        assert!(pattern_candidate_pair_to_cad(&p, Default::default(), &b, &h, true).is_err());
    }
    h.pattern.lines[0].dash_lengths = vec![1., -1., -0.];
    assert!(pattern_candidate_pair_to_cad(&p, Default::default(), &b, &h, true).is_ok());
}

#[test]
fn effective_candidate_does_not_need_an_invertible_pat_normalization() {
    let (mut p, mut h, b) = fixture(10.);
    let smallest = f64::from_bits(1);
    p.families[0].dashes = vec![
        HatchDash::Dash { length: smallest },
        HatchDash::Gap { length: 1. },
    ];
    h.pattern.lines[0].dash_lengths = vec![smallest, -1.];
    // General creation scale is editing metadata; these definition lengths
    // are already effective. Dividing them by MAX would lose the tiny dash.
    h.pattern_scale = f64::MAX;
    let pair = pattern_candidate_pair_to_cad(&p, Default::default(), &b, &h, true).unwrap();
    assert!(accepted(&pair, 0.));
}
