use cad_geometry_convert::{
    plot_units::{resolve_paper_tolerance, PaperMapping},
    units::q,
    GeometryTolerance,
};
use ocdraw::geometry_kernel::CoordinateLengthUnit as Unit;

#[test]
fn physical_tolerance_fallback_is_explicit_and_domain_specific() {
    let strict = GeometryTolerance::millimetres(0.001).unwrap();
    assert!(strict.resolve(Unit::Unitless).is_err());
    assert!(resolve_paper_tolerance(strict, &PaperMapping::Unknown).is_err());
    let tolerance = strict.with_coordinate_fallback(1e-9).unwrap();
    let coordinate = GeometryTolerance::drawing_units(1e-9)
        .unwrap()
        .resolve(Unit::Unitless)
        .unwrap();
    assert_eq!(tolerance.resolve(Unit::Unitless).unwrap(), coordinate);
    assert_eq!(
        resolve_paper_tolerance(tolerance, &PaperMapping::Unknown).unwrap(),
        coordinate
    );
    assert_eq!(
        tolerance.resolve(Unit::Millimetre).unwrap(),
        strict.resolve(Unit::Millimetre).unwrap()
    );
    let mapping = PaperMapping::FixedPhysical {
        metres_per_coordinate: q(2, 1000),
    };
    assert_eq!(
        resolve_paper_tolerance(tolerance, &mapping).unwrap(),
        resolve_paper_tolerance(strict, &mapping).unwrap()
    );
    for value in [-1., f64::NAN, f64::INFINITY] {
        assert!(strict.with_coordinate_fallback(value).is_err());
    }
    assert!(GeometryTolerance::exact()
        .with_coordinate_fallback(1e-9)
        .is_err());
    assert!(GeometryTolerance::default()
        .with_coordinate_fallback(1e-9)
        .is_err());
}
