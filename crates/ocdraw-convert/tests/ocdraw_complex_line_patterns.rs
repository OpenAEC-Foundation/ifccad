use ocdraw_convert::*;
use opencadcodec::{
    tables::{LineTypeComplexContent, LineTypeComplexData, LineTypeElement, TextStyle},
    CadDocument, Handle, LineType,
};

fn source() -> CadDocument {
    let mut doc = CadDocument::new();
    let text = doc.text_styles.get("Standard").unwrap().handle;
    for handle in [900, 901] {
        let mut shape = TextStyle::new("");
        shape.handle = Handle::new(handle);
        shape.is_shape_file = true;
        shape.font_file = "custom.shx".into();
        doc.text_styles.add_allow_duplicate(shape);
    }
    let mut pattern = LineType::new("Mixed");
    pattern.description = "GAS symbols".into();
    pattern.elements = vec![
        LineTypeElement {
            length: 2.,
            complex: Some(LineTypeComplexData {
                content: LineTypeComplexContent::Text {
                    text: "GAS β".into(),
                },
                style_handle: text,
                scale: 0.1,
                rotation: 0.25,
                offset: [-1., 2.],
                ..Default::default()
            }),
        },
        LineTypeElement {
            length: 0.,
            complex: Some(LineTypeComplexData {
                source_flags: 0,
                content: LineTypeComplexContent::Shape { shape_number: 42 },
                style_handle: Handle::new(900),
                scale: 0.1,
                rotation: -0.5,
                absolute_rotation: true,
                offset: [-1., 2.],
            }),
        },
        LineTypeElement::space(2.),
    ];
    pattern.pattern_length = 4.;
    pattern.handle = doc.allocate_handle();
    doc.line_types.add(pattern).unwrap();
    doc
}

#[test]
fn mixed_markers_and_distinct_sources_survive_native_readback() {
    let source = source();
    let out = cad_document_to_encoded_ocdraw(&source, Default::default()).unwrap();
    assert!(!out.diagnostics().iter().any(|d| d
        .reasons()
        .iter()
        .any(|r| matches!(r, CadToOcdrawLossReason::ComplexLinePatternFallback { .. }))));
    let read = ocdraw::ocdraw::load_ocdraw_bytes(out.encoded().bytes()).unwrap();
    assert_eq!(read.symbol_sources().len(), 2);
    let returned = ocdraw_source_to_cad_document(&read, Default::default()).unwrap();
    let original = source.line_types.get("Mixed").unwrap();
    let result = returned.document().line_types.get("Mixed").unwrap();
    for (a, b) in original.elements.iter().zip(&result.elements) {
        assert_eq!(a.length, b.length);
        match (&a.complex, &b.complex) {
            (Some(a), Some(b)) => {
                assert_eq!(a.content, b.content);
                assert_eq!(a.scale, b.scale);
                assert_eq!(a.rotation, b.rotation);
                assert_eq!(a.offset, b.offset);
                assert_eq!(a.absolute_rotation, b.absolute_rotation);
                let style = returned
                    .document()
                    .text_styles
                    .iter()
                    .find(|s| s.handle == b.style_handle)
                    .unwrap();
                assert_eq!(style.is_shape_file, b.is_shape());
            }
            (None, None) => {}
            _ => panic!("marker lost"),
        }
    }
    assert_eq!(
        returned
            .document()
            .text_styles
            .iter()
            .filter(|s| s.is_shape_file)
            .count(),
        2
    );
}

#[test]
fn unresolved_or_wrong_role_marker_style_is_invalid() {
    for handle in [Handle::NULL, Handle::new(9999), Handle::new(900)] {
        let mut source = source();
        source.line_types.get_mut("Mixed").unwrap().elements[0]
            .complex
            .as_mut()
            .unwrap()
            .style_handle = handle;
        assert!(matches!(
            cad_document_to_ocdraw_document(&source, Default::default()),
            Err(CadToOcdrawError::InvalidSourceStructure { .. })
        ));
    }
}

#[test]
fn unsupported_active_marker_falls_back_as_a_whole() {
    let mut source = source();
    source.line_types.get_mut("Mixed").unwrap().elements[1]
        .complex
        .as_mut()
        .unwrap()
        .scale = 0.;
    let out = cad_document_to_ocdraw_document(&source, Default::default()).unwrap();
    assert!(out
        .document()
        .line_patterns
        .iter()
        .find(|p| p.name == "Mixed")
        .unwrap()
        .pattern
        .is_empty());
    assert!(cad_document_to_ocdraw_document(
        &source,
        CadToOcdrawOptions {
            loss_policy: OcdrawLossPolicy::Reject,
            ..Default::default()
        }
    )
    .is_err());
}

#[test]
fn marker_footprint_is_unassessed_without_semantic_loss() {
    let mut doc = source();
    doc.layers.get_mut("0").unwrap().line_type = "Mixed".into();
    let mut line = opencadcodec::Line::new();
    line.end = opencadcodec::Vector3::new(10., 0., 0.);
    doc.add_entity(opencadcodec::EntityType::Line(line))
        .unwrap();
    let out = cad_document_to_encoded_ocdraw(&doc, Default::default()).unwrap();
    assert!(!out.geometry_assessment().unassessed_sources().is_empty());
    let read = ocdraw::ocdraw::load_ocdraw_bytes(out.encoded().bytes()).unwrap();
    let returned = ocdraw_source_to_cad_document(&read, Default::default()).unwrap();
    assert!(!returned
        .geometry_assessment()
        .unassessed_sources()
        .is_empty());
}

#[test]
fn source_locator_loss_is_reported_once_per_resource_and_rejected() {
    let mut doc = source();
    for s in doc.text_styles.iter_mut().filter(|s| s.is_shape_file) {
        s.font_file = "C:\\library\\Custom.SHX".into();
    }
    let out = cad_document_to_ocdraw_document(&doc, Default::default()).unwrap();
    assert_eq!(out.document().symbol_sources.len(), 2);
    assert_eq!(out.diagnostics().iter().filter(|d|d.reasons().iter().any(|r|matches!(r,CadToOcdrawLossReason::UnsupportedSemantic{name} if name.contains("locator")))).count(),2);
    assert!(matches!(
        cad_document_to_ocdraw_document(
            &doc,
            CadToOcdrawOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(CadToOcdrawError::LossRejected { .. })
    ));
}
