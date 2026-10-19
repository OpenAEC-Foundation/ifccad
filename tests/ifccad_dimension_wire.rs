#[path = "support/ifccad_dimension.rs"]
mod support;
use ifccad::{dimension::*, *};
#[test]
fn independent_style_and_entity_paths_roundtrip_with_unused_settings() {
    let mut d = support::drawing();
    let mut over = DimensionStyleOverrides::default();
    over.text.height = Some(8.0);
    d.dimension_styles[0]
        .type_overrides
        .insert(DimensionKind::Radius, over);
    if let IfccadEntityKind::Dimension(v) = &mut d.model.entities[0].as_native_mut().unwrap().kind {
        v.overrides.text.height = Some(2.5);
    }
    let bytes = encode_ifccad_document(&d).unwrap();
    let loaded = load_ifccad_bytes(bytes.bytes(), Default::default()).unwrap();
    assert_eq!(loaded.document(), &d);
    let value: serde_json::Value = serde_json::from_slice(bytes.bytes()).unwrap();
    let style = value["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["attributes"].get("ifccad::dimensionStyle").is_some())
        .unwrap();
    assert_eq!(
        style["path"],
        format!("/cad/d{}/dimensionStyle/0", d.drawing_id)
    );
}
#[test]
fn unknown_fields_null_overrides_and_foreign_style_paths_are_rejected() {
    let d = support::drawing();
    let b = encode_ifccad_document(&d).unwrap();
    for kind in ["unknown", "null", "foreign"] {
        let mut v: serde_json::Value = serde_json::from_slice(b.bytes()).unwrap();
        let node = v["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["attributes"].get("ifccad::dimension").is_some())
            .unwrap();
        let body = &mut node["attributes"]["ifccad::dimension"];
        match kind {
            "unknown" => body["unknown"] = serde_json::json!(1),
            "null" => body["overrides"]["text"]["height"] = serde_json::Value::Null,
            _ => body["style"] = serde_json::json!("/cad/d999/dimensionStyle/0"),
        };
        assert!(load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).is_err());
    }
}

#[test]
fn later_wins_replaces_whole_body_and_keeps_original_source_immutable() {
    let mut d = support::drawing();
    if let IfccadEntityKind::Dimension(v) = &mut d.model.entities[0].as_native_mut().unwrap().kind {
        v.overrides.text.height = Some(2.5);
    }
    let mut value: serde_json::Value =
        serde_json::from_slice(encode_ifccad_document(&d).unwrap().bytes()).unwrap();
    let original = value["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["attributes"].get("ifccad::dimension").is_some())
        .unwrap()
        .clone();
    let mut body = original["attributes"]["ifccad::dimension"].clone();
    body["overrides"] = serde_json::json!({});
    body["associations"] = serde_json::json!([]);
    value["data"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"path":original["path"],"attributes":{"ifccad::dimension":body}}));
    let bytes = serde_json::to_vec(&value).unwrap();
    let loaded = load_ifccad_bytes(&bytes, Default::default()).unwrap();
    assert_eq!(loaded.graph().source_bytes(), bytes);
    let IfccadEntityKind::Dimension(v) = &loaded.document().model.entities[0]
        .as_native()
        .unwrap()
        .kind
    else {
        panic!("dimension")
    };
    assert_eq!(v.overrides.text.height, None);
    assert!(load_ifccad_bytes(
        &bytes,
        IfccadReadOptions {
            composition_policy: IfccadCompositionPolicy::RejectConflicts
        }
    )
    .is_err());
}

#[test]
fn dimension_foreground_and_explicit_reset_preserve_native_defaults() {
    let mut d = support::drawing();
    if let IfccadEntityKind::Dimension(v) = &mut d.model.entities[0].as_native_mut().unwrap().kind {
        v.overrides.text.appearance.color =
            Some(DimensionAppearanceChoice::Explicit(IfccadColor::Foreground));
        v.overrides.extension_lines.first.suppressed = Some(false);
    }
    let encoded = encode_ifccad_document(&d).unwrap();
    assert_eq!(
        load_ifccad_bytes(encoded.bytes(), Default::default())
            .unwrap()
            .document(),
        &d
    );
    let value: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    let attrs = &value["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["attributes"].get("ifccad::dimension").is_some())
        .unwrap()["attributes"];
    assert!(attrs["ifccad::entity"].get("visible").is_none());
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
    let encoded = encode_ifccad_document(&d).unwrap();
    let loaded = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    assert_eq!(loaded.document(), &d);
}
