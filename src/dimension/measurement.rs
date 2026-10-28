use super::*;
use crate::geometry_kernel::numeric::{exact, round_nearest};
use num_rational::BigRational;
use num_traits::{Signed, Zero};

/// A deterministic measure of the stored definition; never follows associations.
pub fn evaluate_dimension(
    definition: &DimensionDefinition,
) -> Result<DimensionMeasurement, DimensionValueError> {
    validate_dimension_definition(definition)?;
    use DimensionDefinition as D;
    let rounded = |q: &BigRational| {
        round_nearest(q)
            .map_err(|_| failure("measurement", DimensionValueReason::NumericUnavailable))
    };
    let distance = |a: [f64; 3], b: [f64; 3]| {
        let (x, y) = delta(a, b);
        sqrt_nearest(&(x.clone() * x + y.clone() * y))
    };
    let value = match definition {
        D::Linear {
            first_point,
            second_point,
            direction_angle,
            ..
        } => {
            let (x, y) = delta(*first_point, *second_point);
            let (s, c) = fpmath::sin_cos(*direction_angle);
            rounded(&(x * exact(c) + y * exact(s)).abs())?
        }
        D::Aligned {
            first_point,
            second_point,
            ..
        } => distance(*first_point, *second_point)?,
        D::Radius {
            center,
            curve_point,
            ..
        }
        | D::JoggedRadius {
            center,
            curve_point,
            ..
        } => distance(*center, *curve_point)?,
        D::Diameter {
            first_diameter_point,
            second_diameter_point,
            ..
        } => distance(*first_diameter_point, *second_diameter_point)?,
        D::Ordinate {
            origin,
            feature_point,
            axis_angle,
            value_mode,
            ..
        } => {
            let (x, y) = delta(*origin, *feature_point);
            let (s, c) = fpmath::sin_cos(*axis_angle);
            let q = x * exact(c) + y * exact(s);
            rounded(&if *value_mode == DimensionOrdinateMode::Absolute {
                q.abs()
            } else {
                q
            })?
        }
        D::ArcLength { radius, sweep, .. } => {
            rounded(&(exact(*radius) * exact(sweep.radians()).abs()))?
        }
        D::AngularThreePoints {
            vertex,
            first_point,
            second_point,
            traversal,
            ..
        } => {
            return sector(
                delta(*vertex, *first_point),
                delta(*vertex, *second_point),
                *traversal,
            )
        }
        D::AngularTwoLines {
            line1_start,
            line1_end,
            line2_start,
            line2_end,
            ray1_sign,
            ray2_sign,
            traversal,
            ..
        } => {
            let ray = |a, b, sign| {
                let (x, y) = delta(a, b);
                if sign == DimensionRaySign::Negative {
                    (-x, -y)
                } else {
                    (x, y)
                }
            };
            return sector(
                ray(*line1_start, *line1_end, *ray1_sign),
                ray(*line2_start, *line2_end, *ray2_sign),
                *traversal,
            );
        }
    };
    if matches!(definition, D::ArcLength { .. }) && value == 0. {
        return Err(failure(
            "measurement",
            DimensionValueReason::NumericUnavailable,
        ));
    }
    Ok(DimensionMeasurement::Distance(value))
}
fn delta(a: [f64; 3], b: [f64; 3]) -> (BigRational, BigRational) {
    (exact(b[0]) - exact(a[0]), exact(b[1]) - exact(a[1]))
}
fn sector(
    a: (BigRational, BigRational),
    b: (BigRational, BigRational),
    traversal: DimensionTraversal,
) -> Result<DimensionMeasurement, DimensionValueError> {
    let cross = &a.0 * &b.1 - &a.1 * &b.0;
    let dot = &a.0 * &b.0 + &a.1 * &b.1;
    if cross.is_zero() && dot >= BigRational::zero() {
        return Err(failure(
            "definition/sector",
            DimensionValueReason::InvalidDefinition,
        ));
    }
    let scale = cross.abs().max(dot.abs());
    let y = round_nearest(&(cross / &scale))
        .map_err(|_| failure("measurement", DimensionValueReason::NumericUnavailable))?;
    let x = round_nearest(&(dot / scale))
        .map_err(|_| failure("measurement", DimensionValueReason::NumericUnavailable))?;
    let mut angle = fpmath::atan2(y, x);
    if traversal == DimensionTraversal::Clockwise {
        angle = -angle;
    }
    if angle < 0. {
        angle += std::f64::consts::TAU;
    }
    if !angle.is_finite() || angle <= 0. || angle >= std::f64::consts::TAU {
        return Err(failure(
            "measurement",
            DimensionValueReason::NumericUnavailable,
        ));
    }
    Ok(DimensionMeasurement::Angle(angle))
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
