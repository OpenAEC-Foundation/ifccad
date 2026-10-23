use ifccad::{IfccadPlanarPolylineWidths as Widths, IfccadPolylineVertexWidthOverride as Override};
#[test]
fn default_edit_preserves_explicit_zero_and_equal_overrides() {
    let w = Widths::Defaulted {
        default_start_width: 0.6,
        default_end_width: 0.8,
        overrides: vec![
            Override {
                start: None,
                end: None,
            },
            Override {
                start: Some(0.),
                end: None,
            },
            Override {
                start: None,
                end: Some(0.),
            },
            Override {
                start: Some(0.2),
                end: Some(0.4),
            },
        ],
    };
    assert_eq!(w.effective_at(0, 4).unwrap(), [0.6, 0.8]);
    assert_eq!(w.effective_at(1, 4).unwrap(), [0., 0.8]);
    assert_eq!(w.effective_at(2, 4).unwrap(), [0.6, 0.]);
    assert_eq!(w.effective_at(3, 4).unwrap(), [0.2, 0.4]);
    assert!(w.effective_at(4, 4).is_err());
}
#[test]
fn dormant_values_and_unused_defaults_are_validated() {
    let w = Widths::Defaulted {
        default_start_width: -1.,
        default_end_width: 0.,
        overrides: vec![
            Override {
                start: Some(1.),
                end: Some(1.)
            };
            2
        ],
    };
    assert!(w.validate(2).is_err());
    let w = Widths::PerVertex {
        widths: vec![[0., 0.], [f64::NAN, 0.]],
    };
    assert!(w.validate(2).is_err());
    assert!(w.validate(1).is_err());
    assert!(Widths::Constant { width: -0.0 }.validate(2).is_ok());
}
