//! Exact plot scalar conversion and Paper-coordinate output interpretation.
use crate::{
    units::{q, ResolvedTolerance},
    GeometryTolerance, GeometryToleranceError,
};
use num_rational::BigRational;
use num_traits::ToPrimitive;
use ocdraw::{
    geometry_kernel::CoordinateLengthUnit,
    plot_kernel::{MediaUnit, PlotScale, PlotSettings, PlotUnit},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PaperMapping {
    Unknown,
    FixedPhysical { metres_per_coordinate: BigRational },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GeometryCoordinateMeaning {
    DrawingUnit(CoordinateLengthUnit),
    PaperCoordinates { mapping: PaperMapping },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PlotNumericError {
    #[error("unsupported exact plot unit conversion")]
    UnsupportedUnit,
    #[error("nonfinite plot scalar")]
    NonFinite,
    #[error("plot scalar conversion exceeds binary64 range")]
    OutOfRange,
    #[error("plot scalar conversion is not exact in binary64")]
    Inexact,
}
pub fn convert_plot_length_exact(
    value: f64,
    from: MediaUnit,
    to: MediaUnit,
) -> Result<f64, PlotNumericError> {
    let value_q = BigRational::from_float(value).ok_or(PlotNumericError::NonFinite)?;
    if from == to {
        return Ok(value);
    }
    let (MediaUnit::Physical(from), MediaUnit::Physical(to)) = (from, to) else {
        return Err(PlotNumericError::UnsupportedUnit);
    };
    let from =
        ResolvedTolerance::from_metres(q(1, 1), from).ok_or(PlotNumericError::UnsupportedUnit)?;
    let to =
        ResolvedTolerance::from_metres(q(1, 1), to).ok_or(PlotNumericError::UnsupportedUnit)?;
    if from.lower != from.upper || to.lower != to.upper {
        return Err(PlotNumericError::UnsupportedUnit);
    }
    let exact = value_q * to.lower / from.lower;
    let out = exact
        .to_f64()
        .filter(|v| v.is_finite())
        .ok_or(PlotNumericError::OutOfRange)?;
    if BigRational::from_float(out).as_ref() != Some(&exact) {
        return Err(PlotNumericError::Inexact);
    }
    Ok(out)
}
pub fn paper_mapping(plot: Option<&PlotSettings>) -> PaperMapping {
    let Some(plot) = plot else {
        return PaperMapping::Unknown;
    };
    let PlotScale::Fixed {
        output_length,
        scope_length,
    } = plot.mapping.scale
    else {
        return PaperMapping::Unknown;
    };
    let factor = match plot.plot_unit {
        PlotUnit::Millimetre => q(1, 1000),
        PlotUnit::Inch => q(127, 5000),
        PlotUnit::Pixel => return PaperMapping::Unknown,
    };
    if !output_length.is_finite()
        || output_length <= 0.
        || !scope_length.is_finite()
        || scope_length <= 0.
    {
        return PaperMapping::Unknown;
    }
    PaperMapping::FixedPhysical {
        metres_per_coordinate: factor * BigRational::from_float(output_length).unwrap()
            / BigRational::from_float(scope_length).unwrap(),
    }
}
pub fn resolve_paper_tolerance(
    requested: GeometryTolerance,
    mapping: &PaperMapping,
) -> Result<ResolvedTolerance, GeometryToleranceError> {
    requested.resolve_paper(mapping)
}

/// Materialize a scalar difference without silently rounding a page boundary.
pub fn subtract_plot_lengths_exact(a: f64, b: f64) -> Result<f64, PlotNumericError> {
    let a = BigRational::from_float(a).ok_or(PlotNumericError::NonFinite)?;
    let b = BigRational::from_float(b).ok_or(PlotNumericError::NonFinite)?;
    let exact = a - b;
    let value = exact
        .to_f64()
        .filter(|v| v.is_finite())
        .ok_or(PlotNumericError::OutOfRange)?;
    if BigRational::from_float(value).as_ref() != Some(&exact) {
        return Err(PlotNumericError::Inexact);
    }
    Ok(value)
}
