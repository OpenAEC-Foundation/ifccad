use ocdraw::ocdraw::{load_drawing_bytes, DrawingLoadStatus};
use serde_json::{json, Value};

fn drawing() -> Value {
    json!({"header":{"format":"open_cad_drawing","version":"0.1.0","drawingId":"patterns","unit":"mm","nextEntityId":1,"nextLayerId":0,"nextLayoutId":1,"nextLinePatternId":3},
    "layouts":[{"id":0,"scopeId":0,"kind":"model","name":"Model","tabIndex":0}],
    "scopes":[{"id":0,"kind":0,"entities":[],"bounds":null}],"streams":{},
    "linePatterns":[{"id":0,"name":"Continuous","pattern":[]},{"id":1,"name":"DashDot","pattern":[6,-2,0,-2]},{"id":2,"name":"GAS_LEIDING","pattern":[]}]})
}

#[test]
fn named_patterns_read_including_unused_empty_definitions() {
    let value = drawing();
    let result = load_drawing_bytes(&serde_json::to_vec(&value).unwrap());
    assert_eq!(
        result.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        result.diagnostics()
    );
}

#[test]
fn invalid_pattern_definitions_are_rejected() {
    for pattern in [
        json!([0, 0]),
        json!([-1, 2]),
        json!([1]),
        json!([1e308, -1e308]),
    ] {
        let mut value = drawing();
        value["linePatterns"][1]["pattern"] = pattern;
        assert_eq!(
            load_drawing_bytes(&serde_json::to_vec(&value).unwrap()).status(),
            DrawingLoadStatus::Invalid
        );
    }
}

#[test]
fn pattern_names_references_and_watermarks_are_checked() {
    let mut invalids = Vec::new();
    let mut v = drawing();
    v["header"]["nextLinePatternId"] = json!(2);
    invalids.push(v);
    let mut v = drawing();
    v["linePatterns"][2]["id"] = json!(1);
    invalids.push(v);
    let mut v = drawing();
    v["linePatterns"][1]["name"] = json!("Straße");
    v["linePatterns"][2]["name"] = json!("STRASSE");
    invalids.push(v);
    let mut v = drawing();
    v["linePatterns"][2]["name"] = json!("ByLayer");
    invalids.push(v);
    for value in invalids {
        assert_eq!(
            load_drawing_bytes(&serde_json::to_vec(&value).unwrap()).status(),
            DrawingLoadStatus::Invalid
        );
    }
    let mut v = drawing();
    v["linePatternScale"] = json!(0);
    assert_eq!(
        load_drawing_bytes(&serde_json::to_vec(&v).unwrap()).status(),
        DrawingLoadStatus::Invalid
    );
}

#[test]
fn inherited_modes_cannot_carry_ids_and_explicit_ids_resolve() {
    use ocdraw::ocdraw::*;
    let mut builder = DrawingBuilder::new(DrawingOptions::new("references", "mm")).unwrap();
    let pattern = builder.ensure_continuous_line_pattern().unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new("0", RgbColor::new(0, 0, 0), pattern))
        .unwrap();
    builder
        .add_line(LineDefinition::new(layer, [0.0; 3], [1.0, 0.0, 0.0]))
        .unwrap();
    let encoded = builder.finish().unwrap();
    let value: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    for (mode, id) in [
        ("ByLayer", json!(0)),
        ("ByBlock", json!(0)),
        ("Explicit", Value::Null),
        ("Explicit", json!(99)),
    ] {
        let mut v = value.clone();
        v["streams"]["lineStream"]["linePatternMode"] = json!([mode]);
        v["streams"]["lineStream"]["linePatternId"] = json!([id]);
        assert_eq!(
            load_drawing_bytes(&serde_json::to_vec(&v).unwrap()).status(),
            DrawingLoadStatus::Invalid
        );
    }
    let mut v = value;
    v["streams"]["lineStream"]["linePatternId"] = json!([]);
    assert_eq!(
        load_drawing_bytes(&serde_json::to_vec(&v).unwrap()).status(),
        DrawingLoadStatus::Invalid
    );
}

#[test]
fn invalid_writer_pattern_values_and_scales_are_rejected() {
    use ocdraw::ocdraw::*;
    let mut b = DrawingBuilder::new(DrawingOptions::new("invalid", "mm")).unwrap();
    for pattern in [
        vec![f64::NAN, -1.0],
        vec![f64::INFINITY, -1.0],
        vec![f64::MAX, f64::MAX],
        vec![0.0, 0.0],
    ] {
        assert!(b
            .add_line_pattern(LinePatternDefinition {
                name: "Bad".into(),
                description: None,
                pattern
            })
            .is_err());
    }
    for scale in [0.0, -1.0, f64::INFINITY, f64::NAN] {
        assert!(b.set_line_pattern_scale(scale).is_err());
    }
    assert!(b
        .add_line_pattern(LinePatternDefinition {
            name: "Dots".into(),
            description: None,
            pattern: vec![0.0, -2.0]
        })
        .is_ok());
    assert!(b
        .add_line_pattern(LinePatternDefinition {
            name: "Solid".into(),
            description: None,
            pattern: vec![1.0, 2.0]
        })
        .is_ok());
}

#[test]
fn polyline_generation_and_scales_roundtrip() {
    use ocdraw::ocdraw::*;
    let mut builder = DrawingBuilder::new(DrawingOptions::new("patterns", "mm")).unwrap();
    let pattern = builder
        .add_line_pattern(LinePatternDefinition {
            name: "DashDot".into(),
            description: None,
            pattern: vec![6.0, -2.0, 0.0, -2.0],
        })
        .unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new("0", RgbColor::new(0, 0, 0), pattern))
        .unwrap();
    builder.set_line_pattern_scale(2.0).unwrap();
    let mut poly =
        PlanarPolylineDefinition::new(layer, vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]], false);
    poly.line_pattern_generation = LinePatternGeneration::Continuous;
    poly.appearance.line_pattern_scale = 0.5;
    builder.add_planar_polyline(poly).unwrap();
    let encoded = builder.finish().unwrap();
    let loaded = load_drawing_bytes(encoded.bytes());
    let drawing = loaded.validated_drawing().unwrap();
    assert_eq!(drawing.line_pattern_scale(), 2.0);
    assert_eq!(
        drawing.line_patterns()[0].pattern,
        vec![6.0, -2.0, 0.0, -2.0]
    );
    assert_eq!(
        drawing.geometric_entities()[0]
            .appearance()
            .line_pattern_scale,
        0.5
    );
    assert!(matches!(
        drawing.geometric_entities()[0].geometry(),
        DrawingGeometry::PlanarPolyline {
            line_pattern_generation: LinePatternGeneration::Continuous,
            ..
        }
    ));
}

#[test]
fn empty_drawing_and_omitted_defaults_do_not_synthesize_patterns() {
    use ocdraw::ocdraw::*;
    let empty = DrawingBuilder::new(DrawingOptions::new("empty", "mm"))
        .unwrap()
        .finish()
        .unwrap();
    let read = load_drawing_bytes(empty.bytes());
    assert!(read.validated_drawing().unwrap().line_patterns().is_empty());
    let mut builder = DrawingBuilder::new(DrawingOptions::new("defaults", "mm")).unwrap();
    let authored = builder
        .add_line_pattern(LinePatternDefinition {
            name: "Authored".into(),
            description: None,
            pattern: vec![1.0, -1.0],
        })
        .unwrap();
    let continuous = builder.ensure_continuous_line_pattern().unwrap();
    assert_ne!(continuous, authored);
    assert_eq!(
        continuous,
        builder.ensure_continuous_line_pattern().unwrap()
    );
    let layer = builder
        .add_layer(LayerDefinition::new("0", RgbColor::new(0, 0, 0), authored))
        .unwrap();
    builder
        .add_spatial_polyline(SpatialPolylineDefinition::new(
            layer,
            vec![[0.0; 3], [1.0, 2.0, 3.0]],
            false,
        ))
        .unwrap();
    let encoded = builder.finish().unwrap();
    let mut value: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    value.as_object_mut().unwrap().remove("linePatternScale");
    let stream = value["streams"]["spatialPolylineStream"]
        .as_object_mut()
        .unwrap();
    stream.remove("linePatternScale");
    stream.remove("linePatternGeneration");
    let read = load_drawing_bytes(&serde_json::to_vec(&value).unwrap());
    let drawing = read.validated_drawing().unwrap();
    assert_eq!(drawing.line_pattern_scale(), 1.0);
    assert_eq!(
        drawing.geometric_entities()[0]
            .appearance()
            .line_pattern_scale,
        1.0
    );
    assert!(matches!(
        drawing.geometric_entities()[0].geometry(),
        DrawingGeometry::SpatialPolyline {
            line_pattern_generation: LinePatternGeneration::PerSegment,
            ..
        }
    ));
}

#[test]
fn entity_scale_and_generation_columns_are_strict() {
    let bytes = include_bytes!("../conformance/next/ocdraw/valid/named-line-patterns.ocdraw.json");
    let base: Value = serde_json::from_slice(bytes).unwrap();
    for bad in [json!([0]), json!([-1]), json!([])] {
        let mut value = base.clone();
        value["streams"]["planarPolylineStream"]["linePatternScale"] = bad;
        assert_eq!(
            load_drawing_bytes(&serde_json::to_vec(&value).unwrap()).status(),
            DrawingLoadStatus::Invalid
        );
    }
    let mut value = base;
    value["streams"]["planarPolylineStream"]["linePatternGeneration"] = json!(["phase"]);
    assert_eq!(
        load_drawing_bytes(&serde_json::to_vec(&value).unwrap()).status(),
        DrawingLoadStatus::Invalid
    );
}
