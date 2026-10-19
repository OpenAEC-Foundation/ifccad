use ocdraw::ocdraw::*;
use serde_json::{json, Value};

fn drawing() -> Value {
    json!({"header":{"format":"open_cad_drawing","version":"0.1.0","drawingId":"markers","unit":"mm","nextEntityId":1,"nextLayerId":0,"nextLayoutId":1,"nextLinePatternId":1,"nextSymbolSourceId":2},
      "layouts":[{"id":0,"scopeId":0,"kind":"model","name":"Model","tabIndex":0}],
      "scopes":[{"id":0,"kind":0,"entities":[],"bounds":null}],"streams":{},
      "symbolSources":[{"id":0,"kind":"shx","fileName":"missing.shx"},{"id":1,"kind":"shx","fileName":"missing.shx"}],
      "linePatterns":[{"id":0,"name":"Shapes","pattern":[6,{"length":0,"marker":{"kind":"symbol","symbolSourceId":1,"shapeNumber":42,"scale":0.1,"offset":[-1,2],"rotation":{"mode":"absolute","angle":0.5}}},-2]}]})
}

#[test]
fn complex_native_readback_without_assets() {
    let native = load_ocdraw_bytes(&serde_json::to_vec(&drawing()).unwrap()).unwrap();
    let encoded = encode_ocdraw_document(native.document()).unwrap();
    let reopened = load_ocdraw_bytes(encoded.bytes()).unwrap();
    assert_eq!(
        native.document().line_patterns,
        reopened.document().line_patterns
    );
    let wire: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    assert!(wire["linePatterns"][0]["pattern"][0].is_number());
    assert_eq!(
        wire["linePatterns"][0]["pattern"][1]["length"].as_f64(),
        Some(0.0)
    );
    assert_eq!(
        wire["linePatterns"][0]["pattern"][1]["marker"]["symbolSourceId"],
        1
    );
}

#[test]
fn unknown_marker_fields_rejected() {
    let mut wire = drawing();
    wire["linePatterns"][0]["pattern"][1]["marker"]["uri"] = json!("symbols.shx");
    assert!(load_ocdraw_bytes(&serde_json::to_vec(&wire).unwrap()).is_err());
}

#[test]
fn marker_sources_are_independent_ids() {
    let mut wire = drawing();
    wire["linePatterns"][0]["pattern"][1]["marker"]["symbolSourceId"] = json!(2);
    assert!(load_ocdraw_bytes(&serde_json::to_vec(&wire).unwrap()).is_err());
    let mut wire = drawing();
    wire["header"]["nextSymbolSourceId"] = json!(1);
    assert!(load_ocdraw_bytes(&serde_json::to_vec(&wire).unwrap()).is_err());
    let mut wire = drawing();
    wire["symbolSources"][0]["id"] = json!(1);
    assert!(load_ocdraw_bytes(&serde_json::to_vec(&wire).unwrap()).is_err());
}

#[test]
fn text_marker_defaults_and_references_roundtrip() {
    let mut document = OcdrawBuilder::new(OcdrawBuildOptions::new("text-pattern", "mm"))
        .unwrap()
        .build_document()
        .unwrap();
    document.text_styles.push(DrawingTextStyle {
        id: OcdrawTextStyleId(0),
        name: "Requested face".into(),
        properties: ocdraw::text::TextStyleProperties::new(ocdraw::text::FontRequest::family(
            "Missing face",
        )),
    });
    document.line_patterns.push(DrawingLinePattern {
        id: LinePatternId(0),
        name: "Gas".into(),
        description: None,
        pattern: vec![ocdraw::line_pattern::LinePatternElement {
            length: 1.,
            marker: Some(DrawingLinePatternMarker::Text {
                text: "GAS α".into(),
                text_style_id: OcdrawTextStyleId(0),
                transform: Default::default(),
            }),
        }],
    });
    document.next_line_pattern_id = 1;
    let encoded = encode_ocdraw_document(&document).unwrap();
    let loaded = load_ocdraw_bytes(encoded.bytes()).unwrap();
    assert_eq!(document.line_patterns, loaded.document().line_patterns);
    let mut wire: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    for field in ["scale", "offset", "rotation"] {
        wire["linePatterns"][0]["pattern"][0]["marker"]
            .as_object_mut()
            .unwrap()
            .remove(field);
    }
    let loaded = load_ocdraw_bytes(&serde_json::to_vec(&wire).unwrap()).unwrap();
    assert_eq!(document.line_patterns, loaded.document().line_patterns);
    wire["linePatterns"][0]["pattern"][0]["marker"]["textStyleId"] = json!(1);
    assert!(load_ocdraw_bytes(&serde_json::to_vec(&wire).unwrap()).is_err());
}

#[test]
fn source_allocation_is_atomic() {
    use ocdraw::line_pattern::SymbolSource;
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("sources", "mm")).unwrap();
    assert!(builder
        .add_symbol_source(SymbolSource::Shx {
            file_name: "a/b.shx".into()
        })
        .is_err());
    assert_eq!(
        builder
            .add_symbol_source(SymbolSource::Shx {
                file_name: "a.shx".into()
            })
            .unwrap(),
        SymbolSourceId(0)
    );
    assert_eq!(
        builder
            .add_symbol_source(SymbolSource::Shx {
                file_name: "a.shx".into()
            })
            .unwrap(),
        SymbolSourceId(1)
    );
    let document = builder.build_document().unwrap();
    assert_eq!(document.next_symbol_source_id, 2);
    assert_eq!(document.symbol_sources.len(), 2);
}

#[test]
fn invalid_physical_marker_values_are_rejected() {
    for (field, value) in [
        ("scale", json!(null)),
        ("scale", json!(0)),
        ("offset", json!([1])),
        ("offset", json!(null)),
        ("rotation", json!({"mode":"upright","angle":0})),
        ("symbolSourceId", json!(1.0)),
        ("shapeNumber", json!(42.0)),
    ] {
        let mut wire = drawing();
        wire["linePatterns"][0]["pattern"][1]["marker"][field] = value;
        assert!(
            load_ocdraw_bytes(&serde_json::to_vec(&wire).unwrap()).is_err(),
            "{field}"
        );
    }
    for name in [".", "..", "dir\\a.shx", ""] {
        let mut wire = drawing();
        wire["symbolSources"][0]["fileName"] = json!(name);
        assert!(load_ocdraw_bytes(&serde_json::to_vec(&wire).unwrap()).is_err());
    }
}

#[test]
fn builder_rejects_invalid_marker_before_allocating_pattern() {
    use ocdraw::line_pattern::*;
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("atomic-pattern", "mm")).unwrap();
    let source = builder
        .add_symbol_source(SymbolSource::Shx {
            file_name: "a.shx".into(),
        })
        .unwrap();
    let pattern = |scale| LinePatternDefinition {
        name: "Shapes".into(),
        description: None,
        pattern: vec![LinePatternElement {
            length: 1.,
            marker: Some(DrawingLinePatternMarker::Symbol {
                symbol_source_id: source,
                shape_number: 42,
                transform: LinePatternTransform {
                    scale,
                    ..Default::default()
                },
            }),
        }],
    };
    assert!(builder.add_line_pattern(pattern(0.)).is_err());
    assert_eq!(
        builder.add_line_pattern(pattern(1.)).unwrap(),
        LinePatternId(0)
    );
}

#[test]
fn physical_definition_ids_need_exact_integer_backings() {
    let mut wire = drawing();
    wire["linePatterns"][0]["id"] = json!(0.0);
    assert!(load_ocdraw_bytes(&serde_json::to_vec(&wire).unwrap()).is_err());
}
