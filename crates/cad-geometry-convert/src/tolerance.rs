use crate::units::{q, ResolvedTolerance};
use num_rational::BigRational;
use ocdraw::geometry_kernel::CoordinateLengthUnit as DrawingLengthUnit;
use thiserror::Error;
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
pub struct GeometryTolerance(ToleranceKind);
impl Default for GeometryTolerance {
    fn default() -> Self {
        Self(ToleranceKind::Default)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum GeometryToleranceError {
    #[error("geometry tolerance must be finite and nonnegative")]
    InvalidValue,
    #[error("an explicit physical tolerance requires a known drawing unit")]
    PhysicalUnitRequired,
    #[error("an explicit physical Paper tolerance requires a fixed physical plot mapping")]
    PhysicalMappingRequired,
}
impl GeometryTolerance {
    pub(crate) fn resolve_paper(
        self,
        mapping: &crate::plot_units::PaperMapping,
    ) -> Result<ResolvedTolerance, GeometryToleranceError> {
        use crate::plot_units::PaperMapping;
        let exact = |v| BigRational::from_float(v).expect("validated tolerance");
        if let ToleranceKind::DrawingUnits(v) = self.0 {
            return Ok(ResolvedTolerance::exact(exact(v)));
        }
        let PaperMapping::FixedPhysical {
            metres_per_coordinate,
        } = mapping
        else {
            return if matches!(self.0, ToleranceKind::Default) {
                Ok(ResolvedTolerance::exact(q(0, 1)))
            } else {
                Err(GeometryToleranceError::PhysicalMappingRequired)
            };
        };
        if metres_per_coordinate <= &q(0, 1) {
            return Err(GeometryToleranceError::PhysicalMappingRequired);
        }
        let physical = match self.0 {
            ToleranceKind::Default => q(1, 1_000_000),
            ToleranceKind::Metres(v) => exact(v),
            ToleranceKind::Millimetres(v) => exact(v) / q(1000, 1),
            ToleranceKind::DrawingUnits(_) => unreachable!(),
        };
        Ok(ResolvedTolerance::exact(physical / metres_per_coordinate))
    }
    /// Require zero geometric residual.
    pub fn exact() -> Self {
        Self(ToleranceKind::DrawingUnits(0.0))
    }
    /// Set a finite nonnegative limit in the drawing's coordinate unit.
    pub fn drawing_units(value: f64) -> Result<Self, GeometryToleranceError> {
        Self::checked(ToleranceKind::DrawingUnits(value), value)
    }
    /// Set a physical limit; conversion fails if the drawing is unitless.
    pub fn metres(value: f64) -> Result<Self, GeometryToleranceError> {
        Self::checked(ToleranceKind::Metres(value), value)
    }
    /// Set a physical limit; conversion fails if the drawing is unitless.
    pub fn millimetres(value: f64) -> Result<Self, GeometryToleranceError> {
        Self::checked(ToleranceKind::Millimetres(value), value)
    }
    fn checked(kind: ToleranceKind, value: f64) -> Result<Self, GeometryToleranceError> {
        if !value.is_finite() || value < 0.0 {
            Err(GeometryToleranceError::InvalidValue)
        } else {
            Ok(Self(kind))
        }
    }
    pub fn resolve(
        self,
        unit: DrawingLengthUnit,
    ) -> Result<ResolvedTolerance, GeometryToleranceError> {
        let exact = |v| BigRational::from_float(v).expect("validated tolerance");
        let physical = match self.0 {
            ToleranceKind::DrawingUnits(v) => return Ok(ResolvedTolerance::exact(exact(v))),
            ToleranceKind::Default if unit == DrawingLengthUnit::Unitless => {
                return Ok(ResolvedTolerance::exact(q(0, 1)))
            }
            ToleranceKind::Default => q(1, 1000000),
            ToleranceKind::Metres(v) => exact(v),
            ToleranceKind::Millimetres(v) => exact(v) / q(1000, 1),
        };
        ResolvedTolerance::from_metres(physical, unit)
            .ok_or(GeometryToleranceError::PhysicalUnitRequired)
    }
}
