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
    #[cfg(test)]
    metrics::CANDIDATES.set(metrics::CANDIDATES.get() + 1);
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
    #[cfg(test)]
    metrics::SEARCHES.set(metrics::SEARCHES.get() + 1);
    Ok(search_positive_bracket(value))
}

#[cfg(test)]
pub(crate) mod metrics {
    use std::cell::Cell;
    thread_local! {
        pub static EXACT_COORDINATES: Cell<usize> = const { Cell::new(0) };
        pub static CANDIDATES: Cell<usize> = const { Cell::new(0) };
        pub static SEARCHES: Cell<usize> = const { Cell::new(0) };
    }
    pub fn reset() {
        EXACT_COORDINATES.set(0);
        CANDIDATES.set(0);
        SEARCHES.set(0);
    }
    pub fn counts() -> (usize, usize, usize) {
        (EXACT_COORDINATES.get(), CANDIDATES.get(), SEARCHES.get())
    }
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Interval {
    pub lower: f64,
    pub upper: f64,
}

impl Interval {
    pub fn point(value: f64) -> Self {
        assert!(value.is_finite());
        Self {
            lower: value,
            upper: value,
        }
    }

    fn widened(lower: f64, upper: f64) -> Option<Self> {
        let (lower, upper) = (lower.next_down(), upper.next_up());
        (lower.is_finite() && upper.is_finite()).then_some(Self { lower, upper })
    }

    pub fn add(self, other: Self) -> Option<Self> {
        if self == Self::point(0.0) {
            return Some(other);
        }
        if other == Self::point(0.0) {
            return Some(self);
        }
        Self::widened(self.lower + other.lower, self.upper + other.upper)
    }

    pub fn mul(self, other: Self) -> Option<Self> {
        if self == Self::point(0.0) || other == Self::point(0.0) {
            return Some(Self::point(0.0));
        }
        if self == Self::point(1.0) {
            return Some(other);
        }
        if other == Self::point(1.0) {
            return Some(self);
        }
        let products = [
            self.lower * other.lower,
            self.lower * other.upper,
            self.upper * other.lower,
            self.upper * other.upper,
        ];
        if products.iter().any(|v| !v.is_finite()) {
            return None;
        }
        Self::widened(
            products.into_iter().fold(f64::INFINITY, f64::min),
            products.into_iter().fold(f64::NEG_INFINITY, f64::max),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;

    fn q(value: f64) -> BigRational {
        BigRational::from_float(value).unwrap()
    }

    #[test]
    fn nonsingleton_interval_does_not_prove_one_rounded_value() {
        let lower = 1.0_f64;
        let upper = lower.next_up();
        let interval = Interval { lower, upper };
        let near_lower = q(lower) + q(2.0_f64.powi(-55));
        let near_upper = q(upper) - q(2.0_f64.powi(-55));
        for value in [&near_lower, &near_upper] {
            assert!(q(interval.lower) <= *value && *value <= q(interval.upper));
        }
        assert_eq!(round_nearest(&near_lower), Ok(lower));
        assert_eq!(round_nearest(&near_upper), Ok(upper));
    }

    #[test]
    fn cancellation_retains_the_unit_term() {
        let huge = q(2.0_f64.powi(100));
        let value = huge.clone() + q(1.0) - huge;
        assert_eq!(round_nearest(&value), Ok(1.0));
    }

    #[test]
    fn halfway_rounds_to_even_in_both_directions() {
        let midpoint = q(1.0) + q(2.0_f64.powi(-53));
        let next = f64::from_bits(1.0_f64.to_bits() + 1);
        assert_eq!(round_nearest(&midpoint), Ok(1.0));
        assert_eq!(round_down(&midpoint), Ok(1.0));
        assert_eq!(round_up(&midpoint), Ok(next));
        assert_eq!(round_nearest(&-midpoint.clone()), Ok(-1.0));
        assert_eq!(round_down(&-midpoint.clone()), Ok(-next));
        assert_eq!(round_up(&-midpoint), Ok(-1.0));

        let odd_midpoint = q(next) + q(2.0_f64.powi(-53));
        assert_eq!(
            round_nearest(&odd_midpoint),
            Ok(f64::from_bits(1.0_f64.to_bits() + 2))
        );
    }

    #[test]
    fn directed_rounding_encloses_values_below_the_minimum_subnormal() {
        let half = BigRational::new(BigInt::from(1), BigInt::from(1) << 1075);
        assert_eq!(round_nearest(&half), Ok(0.0));
        assert_eq!(round_down(&half), Ok(0.0));
        assert_eq!(round_up(&half), Ok(f64::from_bits(1)));
        assert_eq!(round_down(&-half.clone()), Ok(-f64::from_bits(1)));
        assert_eq!(round_up(&-half), Ok(0.0));
    }

    #[test]
    fn finite_range_is_checked_before_rounding() {
        let maximum = q(f64::MAX);
        assert_eq!(round_up(&maximum), Ok(f64::MAX));
        let beyond = maximum + q(1.0);
        assert_eq!(round_nearest(&beyond), Err(NumericRangeError));
        assert_eq!(round_down(&beyond), Err(NumericRangeError));
        assert_eq!(round_up(&-beyond), Err(NumericRangeError));
    }

    #[test]
    fn arbitrary_rationals_have_adjacent_enclosing_floats() {
        for (numerator, denominator) in [(1, 3), (-7, 11), (1, 10), (997, 101)] {
            let value = BigRational::new(numerator.into(), denominator.into());
            let lower = round_down(&value).unwrap();
            let upper = round_up(&value).unwrap();
            assert!(q(lower) <= value && value <= q(upper));
            assert_eq!(upper, lower.next_up());
            let nearest = round_nearest(&value).unwrap();
            assert!(nearest == lower || nearest == upper);
        }
    }

    #[test]
    fn interval_arithmetic_encloses_signed_and_subnormal_products() {
        for (a, b) in [
            (1.0, 2.0_f64.powi(-53)),
            (-7.0, 0.3),
            (f64::from_bits(1), 0.5),
        ] {
            let sum = Interval::point(a).add(Interval::point(b)).unwrap();
            let product = Interval::point(a).mul(Interval::point(b)).unwrap();
            let exact_sum = q(a) + q(b);
            let exact_product = q(a) * q(b);
            assert!(q(sum.lower) <= exact_sum && exact_sum <= q(sum.upper));
            assert!(q(product.lower) <= exact_product && exact_product <= q(product.upper));
        }
        let product = Interval {
            lower: -2.0,
            upper: -1.0,
        }
        .mul(Interval {
            lower: 3.0,
            upper: 4.0,
        })
        .unwrap();
        assert!(product.lower <= -8.0 && product.upper >= -3.0);
    }

    #[test]
    fn intervals_keep_exact_identity_and_defer_overflow() {
        let point = Interval::point(f64::MAX);
        assert_eq!(point.add(Interval::point(0.0)), Some(point));
        assert_eq!(point.mul(Interval::point(1.0)), Some(point));
        assert_eq!(point.mul(Interval::point(0.0)), Some(Interval::point(0.0)));
        assert!(point.add(point).is_none());
        assert!(point.mul(Interval::point(2.0)).is_none());
    }

    #[test]
    fn candidate_rounding_matches_the_full_search() {
        let mut bits = 17_u64;
        for _ in 0..128 {
            bits = bits.wrapping_mul(6364136223846793005).wrapping_add(1);
            let value = q(f64::from_bits(bits & 0x7fefffffffffffff)) / q(3.0);
            assert_eq!(
                positive_bracket(&value).unwrap(),
                search_positive_bracket(&value)
            );
        }
    }
}
