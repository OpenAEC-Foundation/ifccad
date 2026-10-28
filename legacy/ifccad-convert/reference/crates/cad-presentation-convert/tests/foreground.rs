use cad_presentation_convert::{
    color_to_cad, decode_override_color, encode_override_color, explicit_color_from_cad,
    CadColorValue,
};
use opencadcodec::{objects::XRecordValue, Color};

#[test]
fn aci_seven_has_symbolic_meaning_without_an_rgb_fallback() {
    assert_eq!(
        explicit_color_from_cad(Color::Index(7)).unwrap(),
        CadColorValue::Foreground
    );
    let mapped = color_to_cad(&CadColorValue::Foreground).unwrap();
    assert_eq!(mapped.color, Color::Index(7));
    assert!(mapped.named.is_none());
    assert!(mapped.losses.is_empty());
    for rgb in [[0, 0, 0], [255, 255, 255]] {
        let value = CadColorValue::Concrete {
            rgb,
            indexed: None,
            named: None,
        };
        assert_eq!(
            color_to_cad(&value).unwrap().color,
            Color::from_rgb(rgb[0], rgb[1], rgb[2])
        );
    }
}

#[test]
fn override_aci_seven_keeps_the_indexed_method() {
    let packed = XRecordValue::Int32(0xc3000007u32 as i32);
    assert_eq!(
        decode_override_color(&packed).unwrap(),
        CadColorValue::Foreground
    );
    let (restored, losses) = encode_override_color(&CadColorValue::Foreground).unwrap();
    assert_eq!(restored, packed);
    assert!(losses.is_empty());
}
