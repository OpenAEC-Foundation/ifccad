use ifccad::line_pattern::*;

#[test]
fn simple_and_complex_period_rules() {
    let simple: Vec<LinePatternElement<()>> = [6., -2., 0., -2.]
        .into_iter()
        .map(LinePatternElement::simple)
        .collect();
    assert!(validate_line_pattern_values(&simple).is_ok());
    assert!(validate_line_pattern_values(&[LinePatternElement::<()>::simple(1.)]).is_err());
    assert!(validate_line_pattern_values(&[LinePatternElement {
        length: 1.,
        marker: Some(())
    }])
    .is_ok());
    assert!(validate_line_pattern_values(&[LinePatternElement {
        length: 0.,
        marker: Some(())
    }])
    .is_err());
    assert!(validate_line_pattern_values(&[LinePatternElement {
        length: -1.,
        marker: Some(())
    }])
    .is_err());
    assert!(validate_line_pattern_values(&[
        LinePatternElement::<()>::simple(f64::MAX),
        LinePatternElement::simple(f64::MAX)
    ])
    .is_err());
}

#[test]
fn transform_and_source_rules() {
    assert!(validate_line_pattern_transform(&LinePatternTransform::default()).is_ok());
    for scale in [0., -1., f64::NAN, f64::INFINITY] {
        assert!(validate_line_pattern_transform(&LinePatternTransform {
            scale,
            ..Default::default()
        })
        .is_err());
    }
    for file_name in ["", ".", "..", "fonts/a.shx", "fonts\\a.shx", "a\0.shx"] {
        assert!(validate_symbol_source(&SymbolSource::Shx {
            file_name: file_name.into()
        })
        .is_err());
    }
    assert!(validate_symbol_source(&SymbolSource::Shx {
        file_name: "MySymbols.SHX".into()
    })
    .is_ok());
}

#[test]
fn upright_rotation_requires_a_finite_angle() {
    use ifccad::line_pattern::*;
    let mut transform = LinePatternTransform::default();
    for angle in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        transform.rotation = LinePatternRotation::Upright(angle);
        assert!(validate_line_pattern_transform(&transform).is_err());
    }
}
