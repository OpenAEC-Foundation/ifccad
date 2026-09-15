use ifccad::ifcdr::IfcdrLengthUnit;
use num_rational::BigRational;
use thiserror::Error;
/// Controls semantic loss acceptance. Proven within-tolerance numerical rounding
/// is accepted by both policies, while remaining loss evidence.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ConversionLossPolicy {
    #[default]
    Allow,
    Reject,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum ToleranceKind {
    Default,
    DrawingUnits(f64),
    Metres(f64),
    Millimetres(f64),
}
/// Hard Euclidean accuracy limit, independent of semantic loss policy.
/// Defaults to exactly one micrometre in known units and zero for unitless data.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConversionGeometryTolerance(ToleranceKind);
impl Default for ConversionGeometryTolerance {
    fn default() -> Self {
        Self(ToleranceKind::Default)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum ConversionToleranceError {
    #[error("geometry tolerance must be finite and nonnegative")]
    InvalidValue,
    #[error("an explicit physical tolerance requires a known drawing unit")]
    PhysicalUnitRequired,
}
impl ConversionGeometryTolerance {
    /// Require zero geometric residual.
    pub fn exact() -> Self {
        Self(ToleranceKind::DrawingUnits(0.0))
    }
    /// Set a finite nonnegative limit in the drawing's coordinate unit.
    pub fn drawing_units(value: f64) -> Result<Self, ConversionToleranceError> {
        Self::checked(ToleranceKind::DrawingUnits(value), value)
    }
    /// Set a physical limit; conversion fails if the drawing is unitless.
    pub fn metres(value: f64) -> Result<Self, ConversionToleranceError> {
        Self::checked(ToleranceKind::Metres(value), value)
    }
    /// Set a physical limit; conversion fails if the drawing is unitless.
    pub fn millimetres(value: f64) -> Result<Self, ConversionToleranceError> {
        Self::checked(ToleranceKind::Millimetres(value), value)
    }
    fn checked(kind: ToleranceKind, value: f64) -> Result<Self, ConversionToleranceError> {
        if !value.is_finite() || value < 0.0 {
            Err(ConversionToleranceError::InvalidValue)
        } else {
            Ok(Self(kind))
        }
    }
    pub(crate) fn resolve(
        self,
        unit: IfcdrLengthUnit,
    ) -> Result<BigRational, ConversionToleranceError> {
        use IfcdrLengthUnit::*;
        let q = |n: i64, d: i64| BigRational::new(n.into(), d.into());
        let factor = match unit {
            Unitless => None,
            Millimetre => Some(q(1, 1000)),
            Centimetre => Some(q(1, 100)),
            Metre => Some(q(1, 1)),
            Kilometre => Some(q(1000, 1)),
            Inch => Some(q(127, 5000)),
            Foot => Some(q(381, 1250)),
        };
        let exact = |v| BigRational::from_float(v).expect("validated tolerance");
        let physical = match self.0 {
            ToleranceKind::DrawingUnits(v) => return Ok(exact(v)),
            ToleranceKind::Default if factor.is_none() => return Ok(q(0, 1)),
            ToleranceKind::Default => q(1, 1000000),
            ToleranceKind::Metres(v) => exact(v),
            ToleranceKind::Millimetres(v) => exact(v) / q(1000, 1),
        };
        Ok(physical / factor.ok_or(ConversionToleranceError::PhysicalUnitRequired)?)
    }
}
/// Policy for converting one validated IFCCAD drawing into CadDocument.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ImportOptions {
    pub loss_policy: ConversionLossPolicy,
    pub geometry_tolerance: ConversionGeometryTolerance,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conversion_tolerance_rejects_invalid_values_and_resolves_exactly() {
        for v in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                ConversionGeometryTolerance::drawing_units(v),
                Err(ConversionToleranceError::InvalidValue)
            );
            assert!(ConversionGeometryTolerance::metres(v).is_err());
            assert!(ConversionGeometryTolerance::millimetres(v).is_err());
        }
        assert_eq!(
            ConversionGeometryTolerance::default()
                .resolve(IfcdrLengthUnit::Millimetre)
                .unwrap(),
            BigRational::new(1.into(), 1000.into())
        );
        assert_eq!(
            ConversionGeometryTolerance::default()
                .resolve(IfcdrLengthUnit::Inch)
                .unwrap(),
            BigRational::new(1.into(), 25400.into())
        );
        assert_eq!(
            ConversionGeometryTolerance::default()
                .resolve(IfcdrLengthUnit::Unitless)
                .unwrap(),
            BigRational::from_integer(0.into())
        );
        assert_eq!(
            ConversionGeometryTolerance::metres(0.0)
                .unwrap()
                .resolve(IfcdrLengthUnit::Unitless),
            Err(ConversionToleranceError::PhysicalUnitRequired)
        );
        assert_eq!(
            ConversionGeometryTolerance::exact()
                .resolve(IfcdrLengthUnit::Unitless)
                .unwrap(),
            BigRational::from_integer(0.into())
        );
    }
}
