use cad_presentation_convert::*;
use opencadcodec::{Color, LineWeight, Transparency};

#[test]
fn byte_derived_opacity_returns_the_original_byte_without_loss() {
    for byte in 0u8..=255 {
        let opacity = 1.0 - f64::from(byte) / 255.0;
        let mapped = opacity_to_cad(opacity).unwrap();
        assert_eq!(mapped.transparency, Transparency::Explicit(byte));
        assert_eq!(mapped.roundtrip, opacity);
        assert!(!mapped.changed, "byte {byte}");
    }
}

#[test]
fn authored_opacity_uses_upward_transparency_and_reports_the_change() {
    let mapped = opacity_to_cad(0.5).unwrap();
    assert_eq!(mapped.transparency, Transparency::Explicit(128));
    assert_eq!(mapped.roundtrip, 127.0 / 255.0);
    assert!(mapped.changed);
}

#[test]
fn invalid_presentation_scalars_are_refused_instead_of_clamped() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.01, 1.01] {
        assert!(opacity_to_cad(value).is_err(), "opacity {value}");
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.01] {
        assert!(lineweight_to_cad(value).is_err(), "weight {value}");
    }
}

#[test]
fn standard_weights_survive_the_dwg_table_without_quantization() {
    for hundredths in [
        0, 5, 9, 13, 15, 18, 20, 25, 30, 35, 40, 50, 53, 60, 70, 80, 90, 100, 106, 120, 140, 158,
        200, 211,
    ] {
        let millimetres = f64::from(hundredths) / 100.0;
        let mapped = lineweight_to_cad(millimetres).unwrap();
        assert_eq!(mapped.weight, LineWeight::Value(hundredths));
        assert_eq!(
            LineWeight::from_dwg_index(mapped.weight.to_dwg_index()),
            mapped.weight
        );
        assert_eq!(mapped.roundtrip_mm, millimetres);
        assert!(!mapped.changed);
    }
}

#[test]
fn nonstandard_weights_choose_the_nearest_table_value_with_lower_ties() {
    assert_eq!((0.225_f64 - 0.20).abs(), (0.25_f64 - 0.225).abs());
    for (input, hundredths) in [(0.225, 20), (0.236, 25), (10.0, 211), (f64::MAX, 211)] {
        let mapped = lineweight_to_cad(input).unwrap();
        assert_eq!(mapped.weight, LineWeight::Value(hundredths));
        assert_eq!(mapped.roundtrip_mm, f64::from(hundredths) / 100.0);
        assert!(mapped.changed);
    }
}

#[test]
fn source_colors_preserve_concrete_rgb_and_aci_identity() {
    assert_eq!(
        explicit_color_from_cad(Color::from_rgb(12, 34, 56)).unwrap(),
        CadColorValue {
            rgb: [12, 34, 56],
            indexed: None,
            named: None,
        }
    );
    assert_eq!(
        explicit_color_from_cad(Color::Index(1)).unwrap(),
        CadColorValue {
            rgb: [255, 0, 0],
            indexed: Some(("ACI".into(), 1)),
            named: None,
        }
    );
    for color in [Color::ByLayer, Color::ByBlock, Color::None, Color::Index(0)] {
        assert!(explicit_color_from_cad(color).is_err(), "{color:?}");
    }
}

#[test]
fn target_color_retains_matching_index_and_named_identity() {
    let value = CadColorValue {
        rgb: [255, 0, 0],
        indexed: Some(("aci".into(), 1)),
        named: Some(("Example".into(), "Red".into())),
    };
    let mapped = color_to_cad(&value).unwrap();
    assert_eq!(mapped.color, Color::Index(1));
    assert_eq!(mapped.named, value.named);
    assert!(mapped.losses.is_empty());
}

#[test]
fn incompatible_index_never_replaces_the_native_rgb_fallback() {
    for (system, index, expected) in [
        ("ACI", 1, CadColorLoss::InconsistentIndex),
        ("custom", 1, CadColorLoss::UnsupportedIndex),
        ("ACI", 0, CadColorLoss::UnsupportedIndex),
        ("ACI", 256, CadColorLoss::UnsupportedIndex),
        ("ACI", u64::MAX, CadColorLoss::UnsupportedIndex),
    ] {
        let value = CadColorValue {
            rgb: [12, 34, 56],
            indexed: Some((system.into(), index)),
            named: None,
        };
        let mapped = color_to_cad(&value).unwrap();
        assert_eq!(mapped.color, Color::from_rgb(12, 34, 56));
        assert_eq!(mapped.losses, [expected]);
    }
}

#[test]
fn incomplete_color_identity_is_not_accepted_as_valid_input() {
    for value in [
        CadColorValue {
            rgb: [0; 3],
            indexed: Some((String::new(), 1)),
            named: None,
        },
        CadColorValue {
            rgb: [0; 3],
            indexed: None,
            named: Some((String::new(), "Red".into())),
        },
        CadColorValue {
            rgb: [0; 3],
            indexed: None,
            named: Some(("Example".into(), String::new())),
        },
    ] {
        assert!(color_to_cad(&value).is_err());
    }
}
