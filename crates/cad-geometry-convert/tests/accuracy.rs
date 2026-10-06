use cad_geometry_convert::{GeometryAssessment, GeometryFailureReason, GeometryTolerance};
use num_rational::BigRational;
use ocdraw::geometry_kernel::CoordinateLengthUnit as Unit;

#[test]
fn physical_defaults_resolve_exactly_without_guessing_unitless_scale() {
    for (unit, numerator, denominator) in [
        (Unit::Millimetre, 1, 1000),
        (Unit::Metre, 1, 1_000_000),
        (Unit::Inch, 1, 25400),
        (Unit::Unitless, 0, 1),
    ] {
        let resolved = GeometryTolerance::default().resolve(unit).unwrap();
        let expected = BigRational::new(numerator.into(), denominator.into());
        assert_eq!(resolved.lower, expected);
        assert_eq!(resolved.upper, expected);
    }
    assert!(GeometryTolerance::metres(0.)
        .unwrap()
        .resolve(Unit::Unitless)
        .is_err());
    for value in [-1., f64::NAN, f64::INFINITY] {
        assert!(GeometryTolerance::drawing_units(value).is_err());
    }
}

#[test]
fn numerical_verdicts_distinguish_exceedance_from_incomplete_proof() {
    let assessment = GeometryAssessment::<String>::new(
        GeometryTolerance::drawing_units(1.).unwrap(),
        Unit::Unitless,
    )
    .unwrap();
    let source = "primitive".to_owned();
    let q = |n: i64, d: i64| BigRational::new(n.into(), d.into());
    assert_eq!(assessment.check(&source, 0, &q(1, 1)).unwrap(), 1.);
    let error = assessment.check(&source, 0, &q(2, 1)).unwrap_err();
    assert_eq!(error.reason, GeometryFailureReason::ProvenExceedance);
    let error = assessment
        .check_interval(&source, 0, &q(1, 2), &q(2, 1))
        .unwrap_err();
    assert_eq!(
        error.reason,
        GeometryFailureReason::NumericalProofIncomplete
    );
}
