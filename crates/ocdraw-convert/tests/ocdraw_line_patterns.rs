use ocdraw::ocdraw::load_drawing_bytes;
use ocdraw_convert::cadcodec::tables::{
    LineTypeComplexContent, LineTypeComplexData, LineTypeElement,
};
use ocdraw_convert::cadcodec::{CadDocument, LineType};
use ocdraw_convert::{
    cad_document_to_drawing, ocdraw_to_cad_document, ConversionLossPolicy, ExportOptions,
    ImportOptions,
};

#[test]
fn exports_custom_and_unused_simple_patterns() {
    let mut source = CadDocument::new();
    let mut p = LineType::new("Custom");
    p.description = "User pattern".into();
    p.elements = vec![
        LineTypeElement::dash(6.0),
        LineTypeElement::space(2.0),
        LineTypeElement::dot(),
        LineTypeElement::space(2.0),
    ];
    p.pattern_length = 10.0;
    source.line_types.add(p).unwrap();
    source.layers.get_mut("0").unwrap().line_type = "Custom".into();
    source.header.linetype_scale = 2.0;
    let exported = cad_document_to_drawing(&source, ExportOptions::default()).unwrap();
    let read = load_drawing_bytes(exported.drawing().bytes());
    let drawing = read.validated_drawing().unwrap();
    assert_eq!(drawing.line_pattern_scale(), 2.0);
    let p = drawing
        .line_patterns()
        .iter()
        .find(|p| p.name == "Custom")
        .unwrap();
    assert_eq!(p.pattern, vec![6.0, -2.0, 0.0, -2.0]);
    assert_eq!(drawing.typed_layers()[0].line_pattern_id, p.id);
    let returned = ocdraw_to_cad_document(drawing, ImportOptions::default()).unwrap();
    assert_eq!(
        returned
            .document()
            .line_types
            .get("Custom")
            .unwrap()
            .pattern_length,
        10.0
    );
    assert_eq!(returned.document().header.linetype_scale, 2.0);
}

#[test]
fn unused_complex_pattern_falls_back_and_rejects() {
    let mut source = CadDocument::new();
    let mut p = LineType::new("GAS_LEIDING");
    p.description = "Gasleiding".into();
    let mut dash = LineTypeElement::dash(6.0);
    dash.complex = Some(LineTypeComplexData {
        content: LineTypeComplexContent::Text { text: "GAS".into() },
        ..Default::default()
    });
    p.elements = vec![dash, LineTypeElement::space(2.0)];
    p.pattern_length = 8.0;
    source.line_types.add(p).unwrap();
    let out = cad_document_to_drawing(&source, ExportOptions::default()).unwrap();
    let loaded = load_drawing_bytes(out.drawing().bytes());
    let p = loaded
        .validated_drawing()
        .unwrap()
        .line_patterns()
        .iter()
        .find(|p| p.name == "GAS_LEIDING")
        .unwrap();
    assert!(p.pattern.is_empty());
    assert_eq!(p.description.as_deref(), Some("Gasleiding"));
    assert_eq!(
        out.diagnostics()
            .iter()
            .filter(|d| d.reasons().iter().any(|r| matches!(
                r,
                ocdraw_convert::ExportLossReason::ComplexLinePatternFallback { .. }
            )))
            .count(),
        1
    );
    let options = ExportOptions {
        loss_policy: ConversionLossPolicy::Reject,
        ..Default::default()
    };
    assert!(cad_document_to_drawing(&source, options).is_err());
}

#[test]
fn source_lookup_uses_cad_unicode_case_rules() {
    let mut source = CadDocument::new();
    source.line_types.add(LineType::new("Σ")).unwrap();
    source.layers.get_mut("0").unwrap().line_type = "ς".into();
    let output = cad_document_to_drawing(&source, ExportOptions::default()).unwrap();
    let read = load_drawing_bytes(output.drawing().bytes());
    let drawing = read.validated_drawing().unwrap();
    let p = drawing
        .line_patterns()
        .iter()
        .find(|p| p.name == "Σ")
        .unwrap();
    assert_eq!(drawing.typed_layers()[0].line_pattern_id, p.id);
}

#[test]
fn shape_and_mixed_fallback_keep_references_and_modes() {
    use ocdraw::ocdraw::AppearanceSelection;
    use ocdraw_convert::cadcodec::{EntityType, Line};
    for include_text in [false, true] {
        let mut source = CadDocument::new();
        let mut pattern = LineType::new("Symbols");
        let mut symbol = LineTypeElement::dash(2.0);
        symbol.complex = Some(LineTypeComplexData {
            content: LineTypeComplexContent::Shape { shape_number: 7 },
            ..Default::default()
        });
        pattern.elements = vec![symbol, LineTypeElement::space(1.0)];
        if include_text {
            pattern.elements[1].complex = Some(LineTypeComplexData {
                content: LineTypeComplexContent::Text { text: "GAS".into() },
                ..Default::default()
            });
        }
        pattern.pattern_length = 3.0;
        source.line_types.add(pattern).unwrap();
        source.layers.get_mut("0").unwrap().line_type = "Symbols".into();
        for selection in ["Symbols", "ByLayer", "ByBlock"] {
            let mut line = Line::new();
            line.common.linetype = selection.into();
            source.add_entity(EntityType::Line(line)).unwrap();
        }
        let out = cad_document_to_drawing(&source, ExportOptions::default()).unwrap();
        let loaded = load_drawing_bytes(out.drawing().bytes());
        let drawing = loaded.validated_drawing().unwrap();
        let pattern = drawing
            .line_patterns()
            .iter()
            .find(|p| p.name == "Symbols")
            .unwrap();
        assert!(pattern.pattern.is_empty());
        assert_eq!(drawing.typed_layers()[0].line_pattern_id, pattern.id);
        let selections: Vec<_> = drawing
            .geometric_entities()
            .iter()
            .map(|e| e.appearance().line_pattern.clone())
            .collect();
        assert_eq!(
            selections,
            vec![
                AppearanceSelection::Explicit(pattern.id),
                AppearanceSelection::ByLayer,
                AppearanceSelection::ByBlock
            ]
        );
        let reasons: Vec<_> = out
            .diagnostics()
            .iter()
            .flat_map(|d| d.reasons())
            .filter(|r| {
                matches!(
                    r,
                    ocdraw_convert::ExportLossReason::ComplexLinePatternFallback { .. }
                )
            })
            .collect();
        assert_eq!(reasons.len(), 1);
        assert!(
            matches!(reasons[0], ocdraw_convert::ExportLossReason::ComplexLinePatternFallback {
            text, shapes: true, ..
        } if *text == include_text)
        );
        assert!(cad_document_to_drawing(
            &source,
            ExportOptions {
                loss_policy: ConversionLossPolicy::Reject,
                ..Default::default()
            }
        )
        .is_err());
    }
}

#[test]
fn inconsistent_pattern_references_fail_under_both_policies() {
    use ocdraw_convert::cadcodec::{EntityType, Handle, Line};
    for (name, handle) in [
        ("Missing", None),
        ("Continuous", Some(Handle::new(0xFFFF))),
        ("ByLayer", Some(Handle::new(0x1234))),
    ] {
        let mut source = CadDocument::new();
        source.line_types.get_mut("Continuous").unwrap().handle = Handle::new(0x1234);
        let mut line = Line::new();
        line.common.linetype = name.into();
        line.common.linetype_handle = handle;
        source.add_entity(EntityType::Line(line)).unwrap();
        for loss_policy in [ConversionLossPolicy::Allow, ConversionLossPolicy::Reject] {
            assert!(matches!(
                cad_document_to_drawing(
                    &source,
                    ExportOptions {
                        loss_policy,
                        ..Default::default()
                    }
                ),
                Err(ocdraw_convert::DirectExportError::InvalidSourceStructure { .. })
            ));
        }
    }
}
