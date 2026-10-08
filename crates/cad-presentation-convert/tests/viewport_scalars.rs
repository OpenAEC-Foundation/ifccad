use cad_presentation_convert::*;
use opencadcodec::objects::XRecordValue;
use opencadcodec::{LineWeight, Transparency};

#[test]
fn literal_rgb_and_aci_method_values() {
    // AcCmColorBase packs method in the high byte; ACI 1 is 0xc3000001.
    let rgb = decode_override_color(&XRecordValue::Int32(-1039523821)).unwrap(); // c20a2013
    assert_eq!(rgb.rgb, [0x0a, 0x20, 0x13]);
    let aci = decode_override_color(&XRecordValue::Int32(-1023410175)).unwrap();
    assert_eq!(aci.rgb, [255, 0, 0]);
    assert_eq!(aci.indexed, Some(("ACI".into(), 1)));
    assert_eq!(
        encode_override_color(&aci).unwrap().0,
        XRecordValue::Int32(-1023410175)
    );
}

#[test]
fn literal_alpha_and_hundredths_of_mm() {
    assert_eq!(
        decode_override_opacity(&XRecordValue::Int32(33554559)).unwrap(),
        127.0 / 255.0
    );
    assert_eq!(
        decode_override_opacity(&XRecordValue::Int32(50331775)).unwrap(),
        127.0 / 255.0
    );
    assert_eq!(
        encode_override_opacity(Transparency::Explicit(128)).unwrap(),
        XRecordValue::Int32(33554559)
    );
    assert_eq!(
        decode_override_lineweight(&XRecordValue::Int32(25)).unwrap(),
        0.25
    );
    assert_eq!(
        encode_override_lineweight(LineWeight::Value(25)).unwrap(),
        XRecordValue::Int32(25)
    );
}

#[test]
fn unknown_methods_types_and_reserved_payload_are_rejected() {
    for value in [
        0x00112233_u32,
        0xc1000000,
        0xc8000000,
        0xc3010001,
        0xc3000000,
    ] {
        assert!(decode_override_color(&XRecordValue::Int32(value as i32)).is_err());
    }
    assert!(decode_override_color(&XRecordValue::String("red".into())).is_err());
    for value in [0_u32, 0x010000ff, 0x040000ff, 0x020001ff] {
        assert!(decode_override_opacity(&XRecordValue::Int32(value as i32)).is_err());
    }
    assert!(decode_override_lineweight(&XRecordValue::Int32(-1)).is_err());
    assert!(encode_override_opacity(Transparency::ByLayer).is_err());
    assert!(encode_override_lineweight(LineWeight::ByLayer).is_err());
}

#[test]
fn every_override_alpha_byte_returns_the_original_byte_without_quantization_loss() {
    for transparency in 0u8..=255 {
        let alpha = 255 - transparency;
        for method in [2u32, 3] {
            let source = XRecordValue::Int32(((method << 24) | u32::from(alpha)) as i32);
            let opacity = decode_override_opacity(&source).unwrap();
            let mapped = opacity_to_cad(opacity).unwrap();
            assert_eq!(
                mapped.transparency,
                Transparency::Explicit(transparency),
                "byte {transparency}"
            );
            assert!(!mapped.changed, "byte {transparency}");
            assert_eq!(
                encode_override_opacity(mapped.transparency).unwrap(),
                XRecordValue::Int32((0x02000000 | u32::from(alpha)) as i32)
            );
        }
    }
}
