use cad_geometry_convert::{
    plot_units::*,
    units::{q, ResolvedTolerance},
    GeometryTolerance,
};
use ocdraw::{geometry_kernel::CoordinateLengthUnit as Unit, plot_kernel::MediaUnit};
#[test]
fn physical_plot_conversion_is_exact_or_fails() {
    let inch = MediaUnit::Physical(Unit::Inch);
    let mm = MediaUnit::Physical(Unit::Millimetre);
    assert_eq!(convert_plot_length_exact(5., inch, mm), Ok(127.));
    assert_eq!(convert_plot_length_exact(10., inch, mm), Ok(254.));
    assert_eq!(
        convert_plot_length_exact(1., inch, mm),
        Err(PlotNumericError::Inexact)
    );
    assert_eq!(convert_plot_length_exact(-5., inch, mm), Ok(-127.));
    assert_eq!(
        convert_plot_length_exact(123., MediaUnit::Pixel, MediaUnit::Pixel),
        Ok(123.)
    );
    assert_eq!(
        convert_plot_length_exact(1., mm, MediaUnit::Pixel),
        Err(PlotNumericError::UnsupportedUnit)
    );
    assert_eq!(
        convert_plot_length_exact(1., MediaUnit::Physical(Unit::Parsec), mm),
        Err(PlotNumericError::UnsupportedUnit)
    );
    assert_eq!(
        convert_plot_length_exact(f64::MAX, MediaUnit::Physical(Unit::Metre), mm),
        Err(PlotNumericError::OutOfRange)
    );
}
#[test]
fn paper_limit_uses_fixed_mapping_not_medium() {
    let mapping = PaperMapping::FixedPhysical {
        metres_per_coordinate: q(2, 1000),
    };
    assert_eq!(
        resolve_paper_tolerance(GeometryTolerance::default(), &mapping).unwrap(),
        ResolvedTolerance::exact(q(1, 1_000_000_000))
    );
    let inch = PaperMapping::FixedPhysical {
        metres_per_coordinate: q(127, 5000),
    };
    assert_eq!(
        resolve_paper_tolerance(GeometryTolerance::default(), &inch).unwrap(),
        ResolvedTolerance::exact(q(1, 1_000_000_000))
    );
}
#[test]
fn unknown_paper_mapping_never_borrows_physical_units() {
    assert_eq!(
        resolve_paper_tolerance(GeometryTolerance::default(), &PaperMapping::Unknown).unwrap(),
        ResolvedTolerance::exact(q(1, 1_000_000_000))
    );
    assert_eq!(
        resolve_paper_tolerance(
            GeometryTolerance::drawing_units(0.5).unwrap(),
            &PaperMapping::Unknown
        )
        .unwrap(),
        ResolvedTolerance::exact(q(1, 2))
    );
    assert!(resolve_paper_tolerance(
        GeometryTolerance::millimetres(0.).unwrap(),
        &PaperMapping::Unknown
    )
    .is_err());
}
