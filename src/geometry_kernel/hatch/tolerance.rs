use super::DEFAULT_HATCH_JOIN_TOLERANCE;
use crate::geometry_kernel::numeric::{exact, round_down};
use num_rational::BigRational;
use num_traits::{Signed, Zero};

/// Creation policy; documents store its resolved coordinate value.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum HatchJoinToleranceRequest {
    #[default]
    Default,
    Coordinates(f64),
    Metres {
        value: f64,
        coordinate_fallback: Option<f64>,
    },
    Millimetres {
        value: f64,
        coordinate_fallback: Option<f64>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum HatchJoinToleranceError {
    #[error("tolerance values and known physical scales must be finite and nonnegative/positive")]
    InvalidValue,
    #[error("physical tolerance needs a known scale or an explicit coordinate fallback")]
    PhysicalMeaningRequired,
    #[error(
        "resolved tolerance cannot be represented without enlarging it or losing a nonzero limit"
    )]
    LimitOutOfRange,
}

fn checked(value: f64) -> Result<f64, HatchJoinToleranceError> {
    if value.is_finite() && value >= 0.0 {
        Ok(value)
    } else {
        Err(HatchJoinToleranceError::InvalidValue)
    }
}

/// Resolve an authored local limit using a known metres-per-coordinate scale.
/// An interval-valued unit must supply its conservative upper physical scale.
/// Unknown scale requires an explicit coordinate fallback for physical requests.
pub fn resolve_hatch_join_tolerance(
    request: HatchJoinToleranceRequest,
    metres_per_coordinate: Option<&BigRational>,
) -> Result<f64, HatchJoinToleranceError> {
    let (value, divisor, fallback) = match request {
        HatchJoinToleranceRequest::Default => return Ok(DEFAULT_HATCH_JOIN_TOLERANCE),
        HatchJoinToleranceRequest::Coordinates(value) => return checked(value),
        HatchJoinToleranceRequest::Metres {
            value,
            coordinate_fallback,
        } => (value, 1.0, coordinate_fallback),
        HatchJoinToleranceRequest::Millimetres {
            value,
            coordinate_fallback,
        } => (value, 1000.0, coordinate_fallback),
    };
    checked(value)?;
    if let Some(fallback) = fallback {
        checked(fallback)?;
    }
    let Some(scale) = metres_per_coordinate else {
        return fallback.ok_or(HatchJoinToleranceError::PhysicalMeaningRequired);
    };
    if !scale.is_positive() {
        return Err(HatchJoinToleranceError::InvalidValue);
    }
    let limit = exact(value) / exact(divisor) / scale;
    let resolved = round_down(&limit).map_err(|_| HatchJoinToleranceError::LimitOutOfRange)?;
    if !limit.is_zero() && resolved == 0.0 {
        return Err(HatchJoinToleranceError::LimitOutOfRange);
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry_kernel::numeric::exact;
    use num_rational::BigRational;

    #[test]
    fn default_and_exact_are_local_policies() {
        assert_eq!(
            resolve_hatch_join_tolerance(HatchJoinToleranceRequest::Default, None).unwrap(),
            1e-9
        );
        assert_eq!(
            resolve_hatch_join_tolerance(HatchJoinToleranceRequest::Coordinates(0.0), None)
                .unwrap(),
            0.0
        );
    }

    #[test]
    fn physical_requests_need_known_scale_or_explicit_fallback() {
        let request = HatchJoinToleranceRequest::Metres {
            value: 0.01,
            coordinate_fallback: None,
        };
        assert_eq!(
            resolve_hatch_join_tolerance(request, None),
            Err(HatchJoinToleranceError::PhysicalMeaningRequired)
        );
        let fallback = HatchJoinToleranceRequest::Metres {
            value: 0.01,
            coordinate_fallback: Some(0.125),
        };
        assert_eq!(resolve_hatch_join_tolerance(fallback, None).unwrap(), 0.125);
    }

    #[test]
    fn known_physical_limits_round_down_and_ignore_coordinate_fallback() {
        let request = HatchJoinToleranceRequest::Millimetres {
            value: 1.0,
            coordinate_fallback: Some(99.0),
        };
        let value = resolve_hatch_join_tolerance(request, Some(&exact(1.0))).unwrap();
        let mathematical_limit = BigRational::new(1.into(), 1000.into());
        assert!(exact(value) <= mathematical_limit);
        assert!(exact(value.next_up()) > mathematical_limit);
        assert!(value < 0.001);
    }

    #[test]
    fn nonzero_underflow_and_overflow_are_not_clamped() {
        let tiny = HatchJoinToleranceRequest::Metres {
            value: f64::from_bits(1),
            coordinate_fallback: None,
        };
        assert_eq!(
            resolve_hatch_join_tolerance(tiny, Some(&exact(2.0))),
            Err(HatchJoinToleranceError::LimitOutOfRange)
        );
        let huge = HatchJoinToleranceRequest::Metres {
            value: f64::MAX,
            coordinate_fallback: None,
        };
        assert_eq!(
            resolve_hatch_join_tolerance(huge, Some(&exact(0.5))),
            Err(HatchJoinToleranceError::LimitOutOfRange)
        );
    }

    #[test]
    fn invalid_known_scale_is_not_masked_by_a_fallback() {
        let request = HatchJoinToleranceRequest::Metres {
            value: 1.0,
            coordinate_fallback: Some(1.0),
        };
        assert_eq!(
            resolve_hatch_join_tolerance(request, Some(&exact(0.0))),
            Err(HatchJoinToleranceError::InvalidValue)
        );
    }

    #[test]
    fn invalid_request_values_are_rejected_before_exact_arithmetic() {
        for value in [-1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(
                resolve_hatch_join_tolerance(HatchJoinToleranceRequest::Coordinates(value), None),
                Err(HatchJoinToleranceError::InvalidValue)
            );
            assert_eq!(
                resolve_hatch_join_tolerance(
                    HatchJoinToleranceRequest::Metres {
                        value,
                        coordinate_fallback: Some(1.0)
                    },
                    Some(&exact(1.0))
                ),
                Err(HatchJoinToleranceError::InvalidValue)
            );
            assert_eq!(
                resolve_hatch_join_tolerance(
                    HatchJoinToleranceRequest::Millimetres {
                        value: 1.0,
                        coordinate_fallback: Some(value)
                    },
                    Some(&exact(1.0))
                ),
                Err(HatchJoinToleranceError::InvalidValue)
            );
        }
    }
}
