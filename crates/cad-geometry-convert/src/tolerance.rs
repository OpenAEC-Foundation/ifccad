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
/// Defaults to exactly 1e-9 in each coordinate domain, independent of physical units.
#[derive(Clone, Copy, PartialEq)]
pub struct GeometryTolerance(ToleranceKind, Option<f64>);
impl std::fmt::Debug for GeometryTolerance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut value = f.debug_tuple("GeometryTolerance");
        value.field(&self.0);
        if let Some(fallback) = self.1 {
            value.field(&fallback);
        }
        value.finish()
    }
}
impl Default for GeometryTolerance {
    fn default() -> Self {
        Self(ToleranceKind::Default, None)
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
    #[error("a coordinate fallback is only valid for a physical tolerance")]
    FallbackRequiresPhysicalTolerance,
}
impl GeometryTolerance {
    pub(crate) fn resolve_paper(
        self,
        mapping: &crate::plot_units::PaperMapping,
    ) -> Result<ResolvedTolerance, GeometryToleranceError> {
        use crate::plot_units::PaperMapping;
        let exact = |v| BigRational::from_float(v).expect("validated tolerance");
        if matches!(self.0, ToleranceKind::Default) {
            return Ok(ResolvedTolerance::exact(q(1, 1_000_000_000)));
        }
        if let ToleranceKind::DrawingUnits(v) = self.0 {
            return Ok(ResolvedTolerance::exact(exact(v)));
        }
        let PaperMapping::FixedPhysical {
            metres_per_coordinate,
        } = mapping
        else {
            return self
                .1
                .map(|value| ResolvedTolerance::exact(exact(value)))
                .ok_or(GeometryToleranceError::PhysicalMappingRequired);
        };
        if metres_per_coordinate <= &q(0, 1) {
            return Err(GeometryToleranceError::PhysicalMappingRequired);
        }
        let physical = match self.0 {
            ToleranceKind::Default => unreachable!(),
            ToleranceKind::Metres(v) => exact(v),
            ToleranceKind::Millimetres(v) => exact(v) / q(1000, 1),
            ToleranceKind::DrawingUnits(_) => unreachable!(),
        };
        Ok(ResolvedTolerance::exact(physical / metres_per_coordinate))
    }
    /// Require zero geometric residual.
    pub fn exact() -> Self {
        Self(ToleranceKind::DrawingUnits(0.0), None)
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
            Ok(Self(kind, None))
        }
    }
    /// Explicit coordinate limit for domains whose physical scale is unknown.
    /// Known units and fixed Paper plot mappings retain the physical limit.
    pub fn with_coordinate_fallback(mut self, value: f64) -> Result<Self, GeometryToleranceError> {
        Self::drawing_units(value)?;
        if !matches!(
            self.0,
            ToleranceKind::Metres(_) | ToleranceKind::Millimetres(_)
        ) {
            return Err(GeometryToleranceError::FallbackRequiresPhysicalTolerance);
        }
        self.1 = Some(value);
        Ok(self)
    }
    pub fn resolve(
        self,
        unit: DrawingLengthUnit,
    ) -> Result<ResolvedTolerance, GeometryToleranceError> {
        let exact = |v| BigRational::from_float(v).expect("validated tolerance");
        let physical = match self.0 {
            ToleranceKind::DrawingUnits(v) => return Ok(ResolvedTolerance::exact(exact(v))),
            ToleranceKind::Default => return Ok(ResolvedTolerance::exact(q(1, 1_000_000_000))),
            ToleranceKind::Metres(v) => exact(v),
            ToleranceKind::Millimetres(v) => exact(v) / q(1000, 1),
        };
        ResolvedTolerance::from_metres(physical, unit)
            .or_else(|| self.1.map(|value| ResolvedTolerance::exact(exact(value))))
            .ok_or(GeometryToleranceError::PhysicalUnitRequired)
    }
}
