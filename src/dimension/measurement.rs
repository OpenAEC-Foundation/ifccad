use super::*;
use crate::geometry_kernel::numeric::{exact, round_nearest};
use num_rational::BigRational;
use num_traits::{Signed, Zero};

/// A deterministic measure of the stored definition; never follows associations.
pub fn evaluate_dimension(
    definition: &DimensionDefinition,
) -> Result<DimensionMeasurement, DimensionValueError> {
    validate_dimension_definition(definition)?;
    let (a, b) = definition.points();
    let dx = exact(b[0]) - exact(a[0]);
    let dy = exact(b[1]) - exact(a[1]);
    let value = match definition {
        DimensionDefinition::Linear {
            direction_angle, ..
        } => {
            let (sin, cos) = fpmath::sin_cos(*direction_angle);
            if !sin.is_finite() || !cos.is_finite() {
                return Err(failure(
                    "definition/directionAngle",
                    DimensionValueReason::NumericUnavailable,
                ));
            }
            round_nearest(&(dx * exact(cos) + dy * exact(sin)).abs())
                .map_err(|_| failure("measurement", DimensionValueReason::NumericUnavailable))?
        }
        DimensionDefinition::Aligned { .. } => sqrt_nearest(&(dx.clone() * dx + dy.clone() * dy))?,
    };
    Ok(DimensionMeasurement::Distance(value))
}

// Search the representable positive range using exact square comparisons. This
// avoids intermediate subtraction/square overflow and preserves cancellation.
fn sqrt_nearest(square: &BigRational) -> Result<f64, DimensionValueError> {
    if square.is_zero() {
        return Ok(0.0);
    }
    if square > &(exact(f64::MAX) * exact(f64::MAX)) {
        return Err(failure(
            "measurement",
            DimensionValueReason::NumericUnavailable,
        ));
    }
    let (mut lo, mut hi) = (0_u64, f64::MAX.to_bits());
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        let value = exact(f64::from_bits(mid));
        if value.clone() * value <= *square {
            lo = mid
        } else {
            hi = mid
        }
    }
    let lower = f64::from_bits(lo);
    let q = exact(lower);
    if q.clone() * q == *square {
        return Ok(lower);
    }
    let upper = f64::from_bits(hi);
    let midpoint = (exact(lower) + exact(upper)) / BigRational::from_integer(2.into());
    let midpoint_square = midpoint.clone() * midpoint;
    Ok(
        if square < &midpoint_square || (square == &midpoint_square && lo & 1 == 0) {
            lower
        } else {
            upper
        },
    )
}
