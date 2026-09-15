use num_rational::BigRational;
use num_traits::{Signed, ToPrimitive, Zero};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NumericRangeError;

pub(crate) fn exact(value: f64) -> BigRational {
    BigRational::from_float(value).expect("finite numeric operand")
}

/// Bracket a non-negative rational by adjacent finite floats.
fn positive_bracket(value: &BigRational) -> Result<(f64, f64), NumericRangeError> {
    if value > &exact(f64::MAX) {
        return Err(NumericRangeError);
    }
    if value.is_zero() {
        return Ok((0.0, 0.0));
    }
    // The library estimate is only a candidate; exact comparisons certify it.
    if let Some(candidate) = value.to_f64().filter(|v| v.is_finite() && *v >= 0.0) {
        let candidate_value = exact(candidate);
        match candidate_value.cmp(value) {
            Ordering::Equal => return Ok((candidate, candidate)),
            Ordering::Less => {
                let upper = candidate.next_up();
                if upper.is_finite() && value <= &exact(upper) {
                    if value == &exact(upper) {
                        return Ok((upper, upper));
                    }
                    return Ok((candidate, upper));
                }
            }
            Ordering::Greater => {
                let lower = candidate.next_down();
                if lower >= 0.0 && &exact(lower) <= value {
                    if value == &exact(lower) {
                        return Ok((lower, lower));
                    }
                    return Ok((lower, candidate));
                }
            }
        }
    }
    Ok(search_positive_bracket(value))
}

fn search_positive_bracket(value: &BigRational) -> (f64, f64) {
    let (mut lower, mut upper) = (0_u64, f64::MAX.to_bits());
    while upper - lower > 1 {
        let middle = lower + (upper - lower) / 2;
        match exact(f64::from_bits(middle)).cmp(value) {
            Ordering::Equal => {
                let result = f64::from_bits(middle);
                return (result, result);
            }
            Ordering::Less => lower = middle,
            Ordering::Greater => upper = middle,
        }
    }
    let (lower, upper) = (f64::from_bits(lower), f64::from_bits(upper));
    if exact(lower) == *value {
        (lower, lower)
    } else if exact(upper) == *value {
        (upper, upper)
    } else {
        (lower, upper)
    }
}

fn bracket(value: &BigRational) -> Result<(f64, f64), NumericRangeError> {
    if value.is_negative() {
        let (lower, upper) = positive_bracket(&-value)?;
        Ok((-upper, -lower))
    } else {
        positive_bracket(value)
    }
}

pub(crate) fn round_nearest(value: &BigRational) -> Result<f64, NumericRangeError> {
    let (lower, upper) = bracket(value)?;
    if lower == upper {
        return Ok(lower);
    }
    let midpoint = (exact(lower) + exact(upper)) / BigRational::from_integer(2.into());
    Ok(match value.cmp(&midpoint) {
        Ordering::Less => lower,
        Ordering::Greater => upper,
        Ordering::Equal if lower.to_bits() & 1 == 0 => lower,
        Ordering::Equal => upper,
    })
}

pub(crate) fn round_down(value: &BigRational) -> Result<f64, NumericRangeError> {
    bracket(value).map(|(lower, _)| lower)
}

pub(crate) fn round_up(value: &BigRational) -> Result<f64, NumericRangeError> {
    bracket(value).map(|(_, upper)| upper)
}

pub(crate) fn sqrt_interval(value: &BigRational) -> Option<(f64, f64)> {
    if value.is_zero() {
        return Some((0.0, 0.0));
    }
    if value > &(exact(f64::MAX) * exact(f64::MAX)) {
        return None;
    }
    let square = |x| exact(x) * exact(x);
    if let Some(c) = value.to_f64().filter(|v| v.is_finite()).map(f64::sqrt) {
        match square(c).cmp(value) {
            Ordering::Equal => return Some((c, c)),
            Ordering::Less if c.next_up().is_finite() && square(c.next_up()) >= *value => {
                return Some((c, c.next_up()))
            }
            Ordering::Greater if c.next_down() >= 0.0 && square(c.next_down()) <= *value => {
                return Some((c.next_down(), c))
            }
            _ => {}
        }
    }
    let (mut lo, mut hi) = (0_u64, f64::MAX.to_bits());
    while hi - lo > 1 {
        let mid = lo + (hi - lo) / 2;
        match square(f64::from_bits(mid)).cmp(value) {
            Ordering::Equal => {
                let v = f64::from_bits(mid);
                return Some((v, v));
            }
            Ordering::Less => lo = mid,
            Ordering::Greater => hi = mid,
        }
    }
    Some((f64::from_bits(lo), f64::from_bits(hi)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn certified_candidate_and_exact_search_agree() {
        let samples = [
            0.0,
            f64::from_bits(1),
            f64::MIN_POSITIVE,
            0.1,
            1.0,
            1e100,
            f64::MAX,
        ];
        for value in samples {
            let q = exact(value);
            assert_eq!(positive_bracket(&q).unwrap(), search_positive_bracket(&q));
            if value > 0.0 {
                let middle = (&q + exact(value.next_down())) / exact(2.0);
                assert_eq!(
                    positive_bracket(&middle).unwrap(),
                    search_positive_bracket(&middle)
                );
                let rounded = round_nearest(&middle).unwrap();
                assert_eq!(rounded.to_bits() & 1, 0);
                assert_eq!(round_nearest(&-&middle).unwrap(), -rounded);
            }
        }
        assert!(round_nearest(&(exact(f64::MAX) + exact(1.0))).is_err());
    }
    #[test]
    fn distance_intervals_enclose_extreme_exact_squared_values() {
        for distance in [
            0.0,
            f64::from_bits(1),
            f64::MIN_POSITIVE,
            0.1,
            1.0,
            1e200,
            f64::MAX,
        ] {
            let squared = exact(distance) * exact(distance);
            let (lo, hi) = sqrt_interval(&squared).unwrap();
            assert!(exact(lo) * exact(lo) <= squared);
            assert!(exact(hi) * exact(hi) >= squared);
            assert!(lo == hi || lo.next_up() == hi);
        }
        assert!(sqrt_interval(&(exact(f64::MAX) * exact(f64::MAX) + exact(1.0))).is_none());
    }
}
