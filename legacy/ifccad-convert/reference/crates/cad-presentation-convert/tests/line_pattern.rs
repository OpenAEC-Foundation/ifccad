use cad_presentation_convert::*;
use opencadcodec::{
    tables::{LineTypeComplexContent, LineTypeComplexData},
    Handle,
};

fn marker() -> CadLinePatternMarkerValues {
    CadLinePatternMarkerValues {
        content: CadLinePatternMarkerContent::Shape(42),
        scale: 0.1,
        rotation: 0.25,
        absolute_rotation: false,
        upright: false,
        offset: [-1., 2.],
    }
}

#[test]
fn newly_retained_unknown_linetype_flags_are_not_discarded() {
    let mut cad = marker_values_to_cad(&marker(), Handle::new(7)).unwrap();
    cad.source_flags = 0x04;
    marker_values_from_cad(&cad).unwrap();
    cad.source_flags |= 0x80;
    assert!(matches!(
        marker_values_from_cad(&cad),
        Err(CadLinePatternValueError::Unsupported(_))
    ));
}

#[test]
fn marker_parameters_survive_value_roundtrip() {
    for absolute in [false, true] {
        let mut value = marker();
        value.absolute_rotation = absolute;
        if absolute {
            value.rotation = -0.5;
            value.content = CadLinePatternMarkerContent::Text("GAS".into());
        }
        let cad = marker_values_to_cad(&value, Handle::new(7)).unwrap();
        assert_eq!(marker_values_from_cad(&cad).unwrap(), value);
    }
}

#[test]
fn negative_shape_selector_is_unqualified() {
    let cad = LineTypeComplexData {
        content: LineTypeComplexContent::Shape { shape_number: -1 },
        ..Default::default()
    };
    assert!(matches!(
        marker_values_from_cad(&cad),
        Err(CadLinePatternValueError::Unsupported(_))
    ));
    let mut value = marker();
    value.content = CadLinePatternMarkerContent::Shape(32768);
    assert!(matches!(
        marker_values_to_cad(&value, Handle::new(7)),
        Err(CadLinePatternValueError::Unsupported(_))
    ));
}

#[test]
fn source_locator_loss_is_explicit() {
    for input in ["C:\\cad\\Custom.SHX", "/cad/Custom.SHX"] {
        assert_eq!(
            shx_file_name_from_cad(input).unwrap(),
            CadShxFileName {
                file_name: "Custom.SHX".into(),
                discarded_locator: true
            }
        );
    }
    assert!(
        !shx_file_name_from_cad("Custom.SHX")
            .unwrap()
            .discarded_locator
    );
    for invalid in ["", ".", "..", "/cad/", "a\0.shx"] {
        assert!(shx_file_name_from_cad(invalid).is_err());
    }
}

#[test]
fn target_handle_must_resolve() {
    assert!(matches!(
        marker_values_to_cad(&marker(), Handle::NULL),
        Err(CadLinePatternValueError::InvalidValue(_))
    ));
}

#[test]
fn nonfinite_is_invalid_and_zero_scale_is_unsupported() {
    let mut value = marker();
    value.scale = 0.;
    assert!(matches!(
        marker_values_to_cad(&value, Handle::new(7)),
        Err(CadLinePatternValueError::Unsupported(_))
    ));
    value.rotation = f64::NAN;
    assert!(matches!(
        marker_values_to_cad(&value, Handle::new(7)),
        Err(CadLinePatternValueError::InvalidValue(_))
    ));
}

#[test]
fn typed_upright_without_raw_flags_is_supported() {
    let mut cad = marker_values_to_cad(&marker(), Handle::new(7)).unwrap();
    cad.upright = true;
    assert_eq!(cad.source_flags, 0);
    assert!(marker_values_from_cad(&cad).is_ok());
}
