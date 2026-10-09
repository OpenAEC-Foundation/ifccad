use ocdraw::geometry_kernel::hatch::*;
fn fill(dashes: Vec<HatchDash>) -> HatchFill {
    HatchFill::LinePattern(HatchLinePattern {
        name: None,
        description: None,
        origin: [2., 3.],
        rotation: 0.25,
        scale: 2.,
        families: vec![HatchLineFamily {
            angle: 0.5,
            base_point: [1., -2.],
            offset: [3., -4.],
            dashes,
        }],
    })
}
#[test]
fn continuous_ordered_dashes_and_signed_offsets_are_authoritative() {
    for dashes in [
        vec![],
        vec![
            HatchDash::Gap { length: 2. },
            HatchDash::Dot,
            HatchDash::Dash { length: 1. },
            HatchDash::Dot,
        ],
        vec![HatchDash::Gap { length: 2. }],
    ] {
        let value = fill(dashes);
        let copy = value.clone();
        validate_hatch_fill(&value).unwrap();
        assert_eq!(value, copy);
    }
}
#[test]
fn invalid_pattern_parameters_are_located_without_mutation() {
    let mut value = fill(vec![HatchDash::Dot]);
    assert_eq!(
        validate_hatch_fill(&value).unwrap_err().reason,
        HatchPatternFailureReason::InvalidPeriod
    );
    let HatchFill::LinePattern(p) = &mut value else {
        panic!()
    };
    p.families[0].dashes = vec![HatchDash::Dash { length: -1. }];
    let e = validate_hatch_fill(&value).unwrap_err();
    assert_eq!(e.family_index, Some(0));
    assert_eq!(e.dash_index, Some(0));
    let HatchFill::LinePattern(p) = &mut value else {
        panic!()
    };
    p.families[0].dashes.clear();
    p.families[0].offset[1] = -0.;
    assert_eq!(
        validate_hatch_fill(&value).unwrap_err().reason,
        HatchPatternFailureReason::ZeroSpacing
    );
    for scale in [0., -1., f64::NAN, f64::INFINITY] {
        let mut v = fill(vec![]);
        let HatchFill::LinePattern(p) = &mut v else {
            panic!()
        };
        p.scale = scale;
        assert!(validate_hatch_fill(&v).is_err());
    }
}
#[test]
fn extreme_periods_are_checked_exactly_and_small_spacing_is_not_clamped() {
    assert!(validate_hatch_fill(&fill(vec![
        HatchDash::Dash { length: f64::MAX },
        HatchDash::Gap { length: f64::MAX }
    ]))
    .is_err());
    let mut v = fill(vec![HatchDash::Dash {
        length: f64::from_bits(1),
    }]);
    let HatchFill::LinePattern(p) = &mut v else {
        panic!()
    };
    p.families[0].offset[1] = f64::from_bits(1);
    let before = v.clone();
    validate_hatch_fill(&v).unwrap();
    assert_eq!(v, before);
}
