use ocdraw::dimension::*;
use serde_json::json;

#[test]
fn fraction_defaults_keep_old_documents_inline() {
    let original = DimensionNumberSettings::default();
    let mut old = serde_json::to_value(&original).unwrap();
    old["primary"]
        .as_object_mut()
        .unwrap()
        .remove("fractionLayout");
    old["primary"]
        .as_object_mut()
        .unwrap()
        .remove("fractionScale");
    let read: DimensionNumberSettings = serde_json::from_value(old).unwrap();
    let returned = serde_json::to_value(read).unwrap();
    assert_eq!(returned["primary"]["fractionLayout"], "inline");
    assert_eq!(returned["primary"]["fractionScale"], 1.0);
}

#[test]
fn fraction_reset_retains_authored_presence() {
    let mut value =
        serde_json::to_value(DimensionStyleSettings::<u32, u32, [u8; 3], u32>::new(7)).unwrap();
    value["numbers"]["primary"]["fractionLayout"] = json!("diagonal");
    value["numbers"]["primary"]["fractionScale"] = json!(0.75);
    let base = serde_json::from_value(value).unwrap();
    let over: DimensionStyleOverrides<u32, u32, [u8; 3], u32> = serde_json::from_value(json!({
        "numbers": {"primary": {"fractionLayout": "inline", "fractionScale": 1.0}}
    }))
    .unwrap();
    let effective =
        resolve_dimension_settings(&base, DimensionKind::Aligned, &Default::default(), &over);
    let read = serde_json::to_value(&effective).unwrap();
    assert_eq!(read["numbers"]["primary"]["fractionLayout"], "inline");
    assert_eq!(read["numbers"]["primary"]["fractionScale"], 1.0);
    let stored = serde_json::to_value(&over).unwrap();
    assert_eq!(stored["numbers"]["primary"]["fractionLayout"], "inline");
    assert_eq!(stored["numbers"]["primary"]["fractionScale"], 1.0);
    assert!(
        serde_json::from_value::<DimensionStyleOverrides<u32, u32, [u8; 3], u32>>(json!({
            "numbers":{"primary":{"fractionScale":null}}
        }))
        .is_err()
    );
}

#[test]
fn fraction_scale_is_validated_even_when_inactive() {
    let mut value = serde_json::to_value(DimensionNumberSettings::default()).unwrap();
    value["primary"]["fractionLayout"] = json!("horizontal");
    value["primary"]["fractionScale"] = json!(0.75);
    let valid: DimensionNumberSettings = serde_json::from_value(value.clone()).unwrap();
    validate_dimension_numbers(&valid).unwrap();
    for invalid in [0.0, -1.0] {
        value["primary"]["fractionScale"] = json!(invalid);
        let settings: DimensionNumberSettings = serde_json::from_value(value.clone()).unwrap();
        assert!(validate_dimension_numbers(&settings).is_err());
    }
    for invalid in [json!(null), json!("wrong")] {
        value["primary"]["fractionLayout"] = invalid;
        assert!(serde_json::from_value::<DimensionNumberSettings>(value.clone()).is_err());
    }
}

#[test]
fn non_finite_fraction_scales_and_alternate_scales_are_rejected() {
    for value in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut settings = DimensionNumberSettings::default();
        settings.primary.fraction_scale = value;
        assert!(validate_dimension_numbers(&settings).is_err());
        settings.primary.fraction_scale = 1.0;
        settings.alternate = Some(DimensionAlternateNumberSettings {
            fraction_scale: value,
            ..Default::default()
        });
        assert!(validate_dimension_numbers(&settings).is_err());
    }
}
