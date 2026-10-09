use super::{HatchDash, HatchFill};
use crate::geometry_kernel::numeric::exact;
use num_rational::BigRational;
use num_traits::Zero;

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
#[error("pattern family {family_index:?}, dash {dash_index:?}: {reason}")]
pub struct HatchPatternValidationError {
    pub family_index: Option<usize>,
    pub dash_index: Option<usize>,
    pub reason: HatchPatternFailureReason,
}
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum HatchPatternFailureReason {
    #[error("invalid finite pattern value: {0}")]
    InvalidValue(&'static str),
    #[error("perpendicular spacing must be nonzero")]
    ZeroSpacing,
    #[error("nonempty dash sequence needs a positive finite total period")]
    InvalidPeriod,
    #[error("line pattern requires at least one family")]
    EmptyFamilies,
}
/// Intrinsic stored-value checks only; never generates lines or evaluates fill.
pub fn validate_hatch_fill(fill: &HatchFill) -> Result<(), HatchPatternValidationError> {
    use HatchPatternFailureReason as R;
    let fail = |family_index, dash_index, reason| HatchPatternValidationError {
        family_index,
        dash_index,
        reason,
    };
    let HatchFill::LinePattern(p) = fill else {
        return Ok(());
    };
    if !p
        .origin
        .into_iter()
        .chain([p.rotation, p.scale])
        .all(f64::is_finite)
        || p.scale <= 0.
    {
        return Err(fail(None, None, R::InvalidValue("transform")));
    }
    if p.families.is_empty() {
        return Err(fail(None, None, R::EmptyFamilies));
    }
    for (i, f) in p.families.iter().enumerate() {
        if ![f.angle]
            .into_iter()
            .chain(f.base_point)
            .chain(f.offset)
            .all(f64::is_finite)
        {
            return Err(fail(Some(i), None, R::InvalidValue("family")));
        }
        if f.offset[1] == 0. {
            return Err(fail(Some(i), None, R::ZeroSpacing));
        }
        let mut period = BigRational::zero();
        for (j, dash) in f.dashes.iter().enumerate() {
            let length = match dash {
                HatchDash::Dash { length } | HatchDash::Gap { length } => *length,
                HatchDash::Dot => continue,
            };
            if !length.is_finite() || length <= 0. {
                return Err(fail(Some(i), Some(j), R::InvalidValue("dash/gap length")));
            }
            period += exact(length);
        }
        if !f.dashes.is_empty() && (period.is_zero() || period > exact(f64::MAX)) {
            return Err(fail(Some(i), None, R::InvalidPeriod));
        }
    }
    Ok(())
}
