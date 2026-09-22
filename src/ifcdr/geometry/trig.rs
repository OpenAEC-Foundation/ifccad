//! Certified enclosures conditional on the pinned backend's documented bound.
//! See docs/geometry/block-trigonometry.md for the bound and qualification.

use super::numeric::Interval;

pub(crate) fn sin_cos_interval(theta: f64) -> Option<(Interval, Interval)> {
    if !theta.is_finite() {
        return None;
    }
    if theta == 0.0 {
        return Some((Interval::point(0.0), Interval::point(1.0)));
    }
    let (sine, cosine) = fpmath::sin_cos(theta);
    Some((one_ulp_enclosure(sine)?, one_ulp_enclosure(cosine)?))
}

fn one_ulp_enclosure(value: f64) -> Option<Interval> {
    if !value.is_finite() {
        return None;
    }
    // One ULP at the exact result may span two neighbors of the computed
    // value at a binade boundary. This is derived from the backend contract,
    // not a guessed safety margin. Intersect with the exact function range.
    let lower = value.next_down().next_down().max(-1.0);
    let upper = value.next_up().next_up().min(1.0);
    (lower <= upper).then_some(Interval { lower, upper })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_zero_does_not_widen_identity() {
        for angle in [0.0, -0.0] {
            let (s, c) = sin_cos_interval(angle).unwrap();
            assert_eq!(s, Interval::point(0.0));
            assert_eq!(c, Interval::point(1.0));
        }
    }

    #[test]
    fn nonfinite_rotation_has_no_certified_result() {
        for angle in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(sin_cos_interval(angle).is_none());
        }
    }

    #[test]
    fn one_ulp_at_true_result_can_cross_a_binade_boundary() {
        use super::super::numeric::exact;
        let computed = 0.5_f64.next_down();
        let true_result = exact(0.5) + exact(2.0_f64.powi(-55));
        assert!(true_result.clone() - exact(computed) < exact(2.0_f64.powi(-53)));
        assert!(exact(computed.next_up()) < true_result);
        let enclosed = one_ulp_enclosure(computed).unwrap();
        assert!(exact(enclosed.lower) <= true_result && true_result <= exact(enclosed.upper));
        let reflected = one_ulp_enclosure(-computed).unwrap();
        assert_eq!(reflected.lower, -enclosed.upper);
        assert_eq!(reflected.upper, -enclosed.lower);
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 2., -2.] {
            assert!(one_ulp_enclosure(invalid).is_none());
        }
    }

    #[test]
    fn intervals_enclose_independent_certified_references() {
        let reference: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/data/block-trig-reference.json"
        ))
        .unwrap();
        let value = |bits: &serde_json::Value| {
            f64::from_bits(u64::from_str_radix(bits.as_str().unwrap(), 16).unwrap())
        };
        for case in reference["cases"].as_array().unwrap() {
            let angle = value(&case["angle_bits"]);
            let (sine, cosine) = sin_cos_interval(angle).unwrap();
            for (field, interval) in [("sin", sine), ("cos", cosine)] {
                let lower = value(&case[field][0]);
                let upper = value(&case[field][1]);
                assert!(
                    interval.lower <= lower && upper <= interval.upper,
                    "{field}({angle:e}): {interval:?} misses [{lower:e}, {upper:e}]"
                );
                assert!(-1.0 <= interval.lower && interval.upper <= 1.0);
            }
        }
    }

    #[test]
    fn tiny_nonzero_angles_are_not_snapped_to_identity() {
        let tiny = f64::from_bits(1);
        let (sine, cosine) = sin_cos_interval(tiny).unwrap();
        assert!(sine.lower < tiny && tiny <= sine.upper);
        assert!(cosine.lower < 1.0 && cosine.upper == 1.0);
        let (negative_sine, negative_cosine) = sin_cos_interval(-tiny).unwrap();
        assert_eq!(negative_sine.lower, -sine.upper);
        assert_eq!(negative_sine.upper, -sine.lower);
        assert_eq!(negative_cosine, cosine);
    }
}
