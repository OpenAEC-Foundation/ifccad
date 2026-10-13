use super::{HatchGradient, HatchGradientColorMode};

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("gradient {field}, color {color_index:?}: {reason}")]
pub struct HatchGradientValidationError {
    pub field: &'static str,
    pub color_index: Option<usize>,
    pub reason: HatchGradientFailureReason,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum HatchGradientFailureReason {
    #[error("active gradient requires stored settings")]
    MissingState,
    #[error("active gradient requires a selected type")]
    MissingType,
    #[error("active gradient requires both endpoint colors")]
    MissingColors,
    #[error("value must be finite")]
    NonFinite,
    #[error("value must lie in [0, 1]")]
    OutOfRange,
    #[error("invalid concrete endpoint color")]
    InvalidColor,
    #[error("all-default dormant settings must be absent")]
    EmptyDormantState,
}

/// Validates stored settings only, without normalizing, deriving colors or
/// evaluating a gradient. Color semantics are owned by the caller's model.
pub fn validate_hatch_gradient<C>(
    active: bool,
    gradient: Option<&HatchGradient<C>>,
    valid_color: impl Fn(&C) -> bool,
) -> Result<(), HatchGradientValidationError> {
    use HatchGradientFailureReason as R;
    let error = |field, color_index, reason| HatchGradientValidationError {
        field,
        color_index,
        reason,
    };
    let Some(g) = gradient else {
        return if active {
            Err(error("gradient", None, R::MissingState))
        } else {
            Ok(())
        };
    };
    for (field, value, bounded) in [
        ("rotation", g.rotation, false),
        ("shift", g.shift, true),
        ("tint", g.tint, true),
    ] {
        if !value.is_finite() {
            return Err(error(field, None, R::NonFinite));
        }
        if bounded && !(0.0..=1.0).contains(&value) {
            return Err(error(field, None, R::OutOfRange));
        }
    }
    if let Some(colors) = &g.colors {
        for (index, color) in colors.iter().enumerate() {
            if !valid_color(color) {
                return Err(error("colors", Some(index), R::InvalidColor));
            }
        }
    }
    if active {
        if g.gradient_type.is_none() {
            return Err(error("type", None, R::MissingType));
        }
        if g.colors.is_none() {
            return Err(error("colors", None, R::MissingColors));
        }
    } else if g.gradient_type.is_none()
        && g.colors.is_none()
        && g.rotation == 0.0
        && g.shift == 0.0
        && g.color_mode == HatchGradientColorMode::TwoColor
        && g.tint == 0.0
    {
        return Err(error("gradient", None, R::EmptyDormantState));
    }
    Ok(())
}
