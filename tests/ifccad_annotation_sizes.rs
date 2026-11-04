use ifccad::{IfccadAnnotationScale, IfccadAnnotationScaleId};

fn scale(paper_units: f64, drawing_units: f64) -> IfccadAnnotationScale {
    IfccadAnnotationScale {
        id: IfccadAnnotationScaleId(1),
        name: "saved ratio".into(),
        paper_units,
        drawing_units,
        unit_scale: false,
    }
}

#[test]
fn ratio_uses_both_saved_unit_values_and_preserves_their_designation() {
    let mut reference = scale(2.0, 40.0);
    reference.unit_scale = true;
    let selected = scale(1.0, 50.0);
    assert_eq!(reference.size_ratio_to(&selected).unwrap(), 2.5);
    assert_eq!(reference.paper_units, 2.0);
    assert_eq!(reference.drawing_units, 40.0);
    assert!(reference.unit_scale);
}

#[test]
fn individually_extreme_ratios_can_have_a_finite_requested_factor() {
    let smallest = f64::from_bits(1);
    let reference = scale(smallest, f64::MAX);
    let selected = scale(smallest, f64::MAX);
    // Neither individual drawing/paper quotient fits in binary64.
    assert_eq!(reference.size_ratio_to(&selected).unwrap(), 1.0);
}

#[test]
fn nonrepresentable_requested_factors_and_invalid_saved_values_are_reported() {
    let smallest = f64::from_bits(1);
    assert!(scale(1.0, smallest)
        .size_ratio_to(&scale(1.0, f64::MAX))
        .is_err());
    assert!(scale(1.0, f64::MAX)
        .size_ratio_to(&scale(1.0, smallest))
        .is_err());
    for invalid in [0.0, -0.0, -1.0, f64::INFINITY, f64::NAN] {
        assert!(scale(invalid, 1.0).size_ratio_to(&scale(1.0, 1.0)).is_err());
        assert!(scale(1.0, 1.0).size_ratio_to(&scale(1.0, invalid)).is_err());
    }
}
