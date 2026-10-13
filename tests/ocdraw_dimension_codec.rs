#[path = "support/dimension.rs"]
mod support;
use ocdraw::{dimension::*, ocdraw::*};

#[test]
fn dimension_and_unused_type_settings_survive_production_readback() {
    let mut d = support::drawing();
    let mut radius = DimensionStyleOverrides::default();
    radius.text.height = Some(8.0);
    d.dimension_styles[0]
        .type_overrides
        .insert(DimensionKind::Radius, radius);
    d.dimension_entities[0].overrides.text.height = Some(2.5);
    d.dimension_entities[0].text_placement = DimensionTextPlacement::Explicit {
        position: [2000.0, 300.0],
        rotation: 0.0,
        attachment: ocdraw::text::MTextAttachment::MiddleCenter,
    };
    let encoded = encode_ocdraw_document(&d).unwrap();
    let loaded = load_ocdraw_bytes(encoded.bytes()).unwrap();
    assert_eq!(loaded.dimension_entities(), d.dimension_entities.as_slice());
    assert_eq!(loaded.dimension_styles(), d.dimension_styles.as_slice());
    assert_eq!(loaded.document().next_dimension_style_id, 1);
}

#[test]
fn wire_rejects_unknown_nested_fields_and_explicit_null() {
    let d = support::drawing();
    let bytes = encode_ocdraw_document(&d).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(bytes.bytes()).unwrap();
    value["streams"]["dimensionStream"]["definition"][0]["unknown"] = serde_json::json!(true);
    assert!(load_ocdraw_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut value: serde_json::Value = serde_json::from_slice(bytes.bytes()).unwrap();
    value["streams"]["dimensionStream"]["overrides"][0]["text"]["height"] = serde_json::Value::Null;
    assert!(load_ocdraw_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
}
