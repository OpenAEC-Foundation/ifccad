use ifccad::geometry_kernel::hatch::*;

fn state() -> HatchGradient<[u8; 3]> {
    HatchGradient {
        gradient_type: Some(HatchGradientType::InverseSpherical),
        colors: Some([[220, 80, 30], [30, 60, 150]]),
        rotation: -7.0,
        shift: 0.5,
        color_mode: HatchGradientColorMode::OneColor,
        tint: 0.8,
    }
}

#[test]
fn active_gradient_requires_explicit_type_and_both_colors() {
    assert_eq!(
        validate_hatch_gradient::<[u8; 3]>(true, None, |_| true)
            .unwrap_err()
            .reason,
        HatchGradientFailureReason::MissingState
    );
    assert!(validate_hatch_gradient::<[u8; 3]>(false, None, |_| true).is_ok());
    let mut g = state();
    g.gradient_type = None;
    let error = validate_hatch_gradient(true, Some(&g), |_| true).unwrap_err();
    assert_eq!(error.reason, HatchGradientFailureReason::MissingType);
    assert_eq!(error.field, "type");
    g.gradient_type = Some(HatchGradientType::Linear);
    g.colors = None;
    assert_eq!(
        validate_hatch_gradient(true, Some(&g), |_| true)
            .unwrap_err()
            .reason,
        HatchGradientFailureReason::MissingColors
    );
}

#[test]
fn inactive_preferences_need_no_invented_type_or_colors() {
    let mut g = state();
    g.gradient_type = None;
    g.colors = None;
    let copy = g.clone();
    validate_hatch_gradient(false, Some(&g), |_| true).unwrap();
    assert_eq!(g, copy);
    g.rotation = 0.0;
    g.shift = 0.0;
    g.color_mode = HatchGradientColorMode::TwoColor;
    g.tint = 0.0;
    assert_eq!(
        validate_hatch_gradient(false, Some(&g), |_| true)
            .unwrap_err()
            .reason,
        HatchGradientFailureReason::EmptyDormantState
    );
    g.gradient_type = Some(HatchGradientType::Curved);
    assert!(validate_hatch_gradient(false, Some(&g), |_| true).is_ok());
    g.gradient_type = None;
    g.colors = Some([[1, 2, 3], [4, 5, 6]]);
    assert!(validate_hatch_gradient(false, Some(&g), |_| true).is_ok());
}

#[test]
fn authorship_mode_tint_and_stored_second_color_are_independent() {
    for mode in [
        HatchGradientColorMode::OneColor,
        HatchGradientColorMode::TwoColor,
    ] {
        for colors in [[[220, 80, 30], [30, 60, 150]], [[1, 2, 3], [1, 2, 3]]] {
            let mut g = state();
            g.color_mode = mode;
            g.colors = Some(colors);
            let copy = g.clone();
            validate_hatch_gradient(true, Some(&g), |_| true).unwrap();
            assert_eq!(g, copy);
        }
    }
}

#[test]
fn finite_angle_and_bounded_shift_tint_are_checked_when_dormant_too() {
    for active in [false, true] {
        for field in ["rotation", "shift", "tint"] {
            for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                let mut g = state();
                match field {
                    "rotation" => g.rotation = bad,
                    "shift" => g.shift = bad,
                    _ => g.tint = bad,
                }
                let error = validate_hatch_gradient(active, Some(&g), |_| true).unwrap_err();
                assert_eq!(error.field, field);
                assert_eq!(error.reason, HatchGradientFailureReason::NonFinite);
            }
        }
        for bad in [-0.1, 1.1] {
            let mut g = state();
            g.shift = bad;
            assert_eq!(
                validate_hatch_gradient(active, Some(&g), |_| true)
                    .unwrap_err()
                    .field,
                "shift"
            );
            g.shift = 0.5;
            g.tint = bad;
            assert_eq!(
                validate_hatch_gradient(active, Some(&g), |_| true)
                    .unwrap_err()
                    .reason,
                HatchGradientFailureReason::OutOfRange
            );
        }
        for value in [0.0, 0.5, 1.0] {
            let mut g = state();
            g.shift = value;
            g.tint = value;
            validate_hatch_gradient(active, Some(&g), |_| true).unwrap();
        }
    }
}

#[test]
fn invalid_colors_are_located_even_when_gradient_is_inactive() {
    let g = state();
    for active in [false, true] {
        for index in [0, 1] {
            let bad = g.colors.as_ref().unwrap()[index];
            let error = validate_hatch_gradient(active, Some(&g), |c| *c != bad).unwrap_err();
            assert_eq!(error.reason, HatchGradientFailureReason::InvalidColor);
            assert_eq!(error.field, "colors");
            assert_eq!(error.color_index, Some(index));
        }
    }
}
