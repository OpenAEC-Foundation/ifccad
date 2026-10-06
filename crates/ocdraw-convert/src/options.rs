pub use cad_geometry_convert::{
    GeometryTolerance as OcdrawGeometryTolerance, GeometryToleranceError as OcdrawToleranceError,
};
/// Controls semantic loss acceptance. Proven within-tolerance numerical rounding
/// is accepted by both policies, while remaining loss evidence.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OcdrawLossPolicy {
    #[default]
    Allow,
    Reject,
}
/// Policy for converting one validated OCDraw drawing into CadDocument.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OcdrawToCadOptions {
    pub loss_policy: OcdrawLossPolicy,
    pub geometry_tolerance: OcdrawGeometryTolerance,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::ResolvedTolerance;
    use num_rational::BigRational;
    use ocdraw::ocdraw::DrawingLengthUnit;
    #[test]
    fn conversion_tolerance_rejects_invalid_values_and_resolves_exactly() {
        for v in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                OcdrawGeometryTolerance::drawing_units(v),
                Err(OcdrawToleranceError::InvalidValue)
            );
            assert!(OcdrawGeometryTolerance::metres(v).is_err());
            assert!(OcdrawGeometryTolerance::millimetres(v).is_err());
        }
        assert_eq!(
            OcdrawGeometryTolerance::default()
                .resolve(DrawingLengthUnit::Millimetre)
                .unwrap(),
            ResolvedTolerance::exact(BigRational::new(1.into(), 1000.into()))
        );
        assert_eq!(
            OcdrawGeometryTolerance::default()
                .resolve(DrawingLengthUnit::Inch)
                .unwrap(),
            ResolvedTolerance::exact(BigRational::new(1.into(), 25400.into()))
        );
        assert_eq!(
            OcdrawGeometryTolerance::default()
                .resolve(DrawingLengthUnit::Unitless)
                .unwrap(),
            ResolvedTolerance::exact(BigRational::from_integer(0.into()))
        );
        assert_eq!(
            OcdrawGeometryTolerance::metres(0.0)
                .unwrap()
                .resolve(DrawingLengthUnit::Unitless),
            Err(OcdrawToleranceError::PhysicalUnitRequired)
        );
        assert_eq!(
            OcdrawGeometryTolerance::exact()
                .resolve(DrawingLengthUnit::Unitless)
                .unwrap(),
            ResolvedTolerance::exact(BigRational::from_integer(0.into()))
        );
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CadToOcdrawOptions {
    pub loss_policy: OcdrawLossPolicy,
    pub geometry_tolerance: OcdrawGeometryTolerance,
}
