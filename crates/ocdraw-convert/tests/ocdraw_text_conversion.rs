use ocdraw::ocdraw::*;
use ocdraw_convert::*;
use opencadcodec::{CadDocument, EntityType, Line, LineWeight, MText, Text, Vector3};

fn source() -> CadDocument {
    let mut d = CadDocument::new();
    let mut unused = opencadcodec::tables::TextStyle::new("Unused");
    unused.font_file = "requested.shx".into();
    unused.handle = d.allocate_handle();
    d.text_styles.add(unused).unwrap();
    let mut line = Line::new();
    line.end = Vector3::new(1., 1., 0.);
    line.common.line_weight = LineWeight::ByLayer;
    d.add_entity(EntityType::Line(line)).unwrap();
    let mut t = Text::with_value("Literal %%uunderlined%%u", Vector3::new(4., 5., 0.));
    t.common.line_weight = LineWeight::ByLayer;
    d.add_entity(EntityType::Text(t)).unwrap();
    let mut m = MText::new();
    m.value = "First\\PSecond".into();
    m.common.line_weight = LineWeight::ByLayer;
    d.add_entity(EntityType::MText(m)).unwrap();
    d
}

#[test]
fn text_styles_and_mixed_order_pass_both_production_conversion_routes() {
    let d = source();
    let outcome = cad_document_to_ocdraw_document(&d, Default::default()).unwrap();
    let native = outcome.document();
    assert_eq!(native.text_entities.len(), 1);
    assert_eq!(native.mtext_entities.len(), 1);
    assert!(native.text_styles.iter().any(|s| s.name == "Unused"));
    assert_eq!(outcome.text_style_mapping().len(), native.text_styles.len());
    assert!(!outcome.geometry_assessment().is_complete());
    let order = &native.scopes[0].entities;
    assert_eq!(order.len(), 3);
    assert_eq!(order[1], native.text_entities[0].id);
    assert_eq!(order[2], native.mtext_entities[0].id);
    assert_eq!(
        native.scopes[0].bounds_quality,
        Some(OcdrawBoundsQuality::Estimated)
    );
    let bytes = encode_ocdraw_document(native).unwrap();
    let read = load_ocdraw_bytes(bytes.bytes()).unwrap();
    let exported = ocdraw_document_to_cad_document(read.document(), Default::default()).unwrap();
    assert_eq!(exported.entity_mapping().len(), 3);
    assert_eq!(
        exported.text_style_mapping().len(),
        native.text_styles.len()
    );
    assert!(!exported.geometry_assessment().is_complete());
    assert_eq!(
        exported
            .document()
            .entities()
            .filter(|e| matches!(e, EntityType::Text(_)))
            .count(),
        1
    );
    assert_eq!(
        exported
            .document()
            .entities()
            .filter(|e| matches!(e, EntityType::MText(_)))
            .count(),
        1
    );
}

#[test]
fn dynamic_field_source_is_skipped_under_allow_and_rejected_under_reject() {
    let mut d = source();
    let handle = d
        .entities()
        .find(|e| matches!(e, EntityType::MText(_)))
        .unwrap()
        .common()
        .handle;
    if let EntityType::MText(m) = d.get_entity_mut(handle).unwrap() {
        m.value = "%<field>".into();
    }
    let allowed = cad_document_to_ocdraw_document(&d, Default::default()).unwrap();
    assert!(allowed.document().mtext_entities.is_empty());
    assert!(!allowed.entity_mapping().contains_key(&handle));
    assert!(allowed.diagnostics().iter().any(
        |d| matches!(d.source(),CadToOcdrawDiagnosticSource::Entity{handle:h,..} if *h==handle)
    ));
    assert!(matches!(
        cad_document_to_ocdraw_document(
            &d,
            CadToOcdrawOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(CadToOcdrawError::LossRejected { .. })
    ));
}

#[test]
fn unallocated_style_handles_do_not_merge_distinct_named_source_records() {
    let mut d = source();
    for name in ["First unallocated", "Second unallocated"] {
        let mut s = opencadcodec::tables::TextStyle::new(name);
        s.font_file = "txt.shx".into();
        d.text_styles.add(s).unwrap();
    }
    let converted = cad_document_to_ocdraw_document(&d, Default::default()).unwrap();
    assert!(converted
        .document()
        .text_styles
        .iter()
        .any(|s| s.name == "First unallocated"));
    assert!(converted
        .document()
        .text_styles
        .iter()
        .any(|s| s.name == "Second unallocated"));
}

#[test]
fn native_absent_last_height_is_diagnosed_when_cad_requires_a_default_value() {
    let converted = cad_document_to_ocdraw_document(&source(), Default::default()).unwrap();
    let mut d = converted.into_document();
    d.text_styles[0].properties.last_used_height = None;
    let result = ocdraw_document_to_cad_document(&d, Default::default()).unwrap();
    assert!(result
        .diagnostics()
        .iter()
        .any(|d| d.code == "TEXT_STYLE_LAST_HEIGHT_DEFAULTED"));
    assert!(matches!(
        ocdraw_document_to_cad_document(
            &d,
            OcdrawToCadOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(OcdrawToCadError::LossRejected { .. })
    ));
}

#[test]
fn cad_noop_color_reset_does_not_silently_erase_authored_inheritance() {
    let outcome = cad_document_to_ocdraw_document(&source(), Default::default()).unwrap();
    let mut d = outcome.into_document();
    let ocdraw::text::MTextInline::Run {
        character_format, ..
    } = &mut d.mtext_entities[0].content[0].inlines[0]
    else {
        panic!()
    };
    character_format.color = Some(ocdraw::text::TextColor::Entity);
    let output = ocdraw_document_to_cad_document(&d, Default::default()).unwrap();
    assert!(output
        .diagnostics()
        .iter()
        .any(|e| e.code == "TEXT_DEPENDENCY_CHANGED"
            && e.message.contains("AuthoredFormattingNormalized")));
    assert!(matches!(
        ocdraw_document_to_cad_document(
            &d,
            OcdrawToCadOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(OcdrawToCadError::LossRejected { .. })
    ));
}

#[test]
fn invalid_source_text_parameters_are_fatal_under_both_loss_policies() {
    let mut d = source();
    let handle = d
        .entities()
        .find(|e| matches!(e, EntityType::Text(_)))
        .unwrap()
        .common()
        .handle;
    if let EntityType::Text(t) = d.get_entity_mut(handle).unwrap() {
        t.insertion_point.x = f64::NAN;
    }
    for loss_policy in [OcdrawLossPolicy::Allow, OcdrawLossPolicy::Reject] {
        assert!(matches!(
            cad_document_to_ocdraw_document(
                &d,
                CadToOcdrawOptions {
                    loss_policy,
                    ..Default::default()
                }
            ),
            Err(CadToOcdrawError::TextPreparation { .. })
        ));
    }
}

#[test]
fn annotative_style_skips_dependent_text_and_native_mirrors_remain_explicit_restrictions() {
    let mut d = source();
    d.text_styles.get_mut("Standard").unwrap().annotative = true;
    let converted = cad_document_to_ocdraw_document(&d, Default::default()).unwrap();
    assert!(converted.document().text_entities.is_empty());
    assert!(converted.document().mtext_entities.is_empty());
    let plain = cad_document_to_ocdraw_document(&source(), Default::default()).unwrap();
    let mut native = plain.into_document();
    native.mtext_entities[0].backward = true;
    recompute_ocdraw_document_bounds(&mut native).unwrap();
    let converted = ocdraw_document_to_cad_document(&native, Default::default()).unwrap();
    assert!(!converted
        .entity_mapping()
        .contains_key(&native.mtext_entities[0].id));
    assert!(converted
        .diagnostics()
        .iter()
        .any(|d| d.code == "TEXT_UNSUPPORTED"));
}
