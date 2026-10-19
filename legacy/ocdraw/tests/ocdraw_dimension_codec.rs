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

#[test]
fn dimension_common_defaults_are_compact_and_mixed_visibility_is_complete() {
    let mut d = support::drawing();
    let encoded = encode_ocdraw_document(&d).unwrap();
    let v: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    let stream = &v["streams"]["dimensionStream"];
    assert!(stream.get("visible").is_none());
    for key in ["color", "linePattern", "lineWeight", "linePatternScale"] {
        assert!(stream.get(key).is_none(), "{key}");
    }
    assert_eq!(
        load_ocdraw_bytes(encoded.bytes())
            .unwrap()
            .dimension_entities(),
        d.dimension_entities.as_slice()
    );
    let mut second = d.dimension_entities[0].clone();
    second.id = d.next_entity_id;
    d.next_entity_id += 1;
    d.scopes[0].entities.push(second.id);
    d.dimension_entities.push(second);
    d.dimension_entities[0].visible = false;
    let encoded = encode_ocdraw_document(&d).unwrap();
    let v: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    assert_eq!(
        v["streams"]["dimensionStream"]["visible"],
        serde_json::json!([false, true])
    );
    assert_eq!(
        load_ocdraw_bytes(encoded.bytes())
            .unwrap()
            .dimension_entities(),
        d.dimension_entities.as_slice()
    );
}

#[test]
fn dimension_foreground_colors_and_explicit_reset_leaves_survive() {
    let mut d = support::drawing();
    d.dimension_entities[0].overrides.text.appearance.color = Some(
        ocdraw::dimension::DimensionAppearanceChoice::Explicit(DrawingColor::Foreground),
    );
    d.dimension_entities[0].overrides.text.height = Some(2.5);
    let encoded = encode_ocdraw_document(&d).unwrap();
    let loaded = load_ocdraw_bytes(encoded.bytes()).unwrap();
    assert_eq!(loaded.dimension_entities(), d.dimension_entities.as_slice());
}

#[test]
fn fraction_styles_and_sparse_resets_survive_strict_native_reader() {
    let mut d = support::drawing();
    let primary = &mut d.dimension_styles[0].settings.numbers.primary;
    primary.fraction_layout = DimensionFractionLayout::Diagonal;
    primary.fraction_scale = 0.75;
    let mut over = DimensionStyleOverrides::default();
    over.numbers.primary.fraction_layout = Some(DimensionFractionLayout::Inline);
    over.numbers.primary.fraction_scale = Some(1.0);
    d.dimension_styles[0]
        .type_overrides
        .insert(DimensionKind::Aligned, over);
    d.dimension_styles[0].settings.numbers.alternate = Some(DimensionAlternateNumberSettings {
        fraction_layout: DimensionFractionLayout::Horizontal,
        fraction_scale: 0.8,
        ..Default::default()
    });
    let encoded = encode_ocdraw_document(&d).unwrap();
    let loaded = load_ocdraw_bytes(encoded.bytes()).unwrap();
    assert_eq!(loaded.dimension_styles(), d.dimension_styles.as_slice());
    assert_eq!(loaded.dimension_entities(), d.dimension_entities.as_slice());
}
