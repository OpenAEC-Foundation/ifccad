use ocdraw::ocdraw::{encode_ocdraw_document, load_ocdraw_bytes};
use ocdraw_convert::*;
use opencadcodec::{
    tables::TextStyle, CadDocument, EntityType, Handle, LineWeight, MText, Text, Vector3,
};

fn text_source() -> CadDocument {
    let mut d = CadDocument::new();
    let mut t = Text::with_value("Ordinary text", Vector3::ZERO);
    t.common.line_weight = LineWeight::ByLayer;
    d.add_entity(EntityType::Text(t)).unwrap();
    d
}
fn shape(name: &str, handle: u64, font: &str) -> TextStyle {
    let mut s = TextStyle::new(name);
    s.handle = Handle::new(handle);
    s.font_file = font.into();
    s.is_shape_file = true;
    s
}
fn is_skipped_table(d: &CadToOcdrawDiagnostic) -> bool {
    d.action() == CadToOcdrawAction::Skipped
        && matches!(d.source(),CadToOcdrawDiagnosticSource::Table{kind} if kind.starts_with("text_styles/"))
}

#[test]
fn distinct_anonymous_shape_resources_do_not_collide_in_text_style_lookup() {
    // Independently reconstructed STYLE roles observed in Sample_AC1032:
    // two empty names, distinct handles/fonts, both explicitly shape-file records.
    let mut d = text_source();
    d.text_styles
        .add_allow_duplicate(shape("", 0x27f, "ltypeshp.shx"));
    d.text_styles
        .add_allow_duplicate(shape("", 0x887, "test_shape.shx"));
    let out = cad_document_to_ocdraw_document(&d, Default::default()).unwrap();
    assert_eq!(out.document().text_entities.len(), 1);
    assert_eq!(out.document().text_styles.len(), 1);
    assert_eq!(out.text_style_mapping().len(), 1);
    assert_eq!(
        out.diagnostics()
            .iter()
            .filter(|d| is_skipped_table(d))
            .count(),
        2
    );
    for handle in ["0x27F", "0x887"] {
        assert!(out.diagnostics().iter().any(|d| is_skipped_table(d) && matches!(d.source(), CadToOcdrawDiagnosticSource::Table{kind} if kind == &format!("text_styles/shape/{handle}"))));
    }
    assert!(!out.text_style_mapping().contains_key(&Handle::new(0x27f)));
    assert!(!out.text_style_mapping().contains_key(&Handle::new(0x887)));
    let encoded = encode_ocdraw_document(out.document()).unwrap();
    assert_eq!(
        load_ocdraw_bytes(encoded.bytes()).unwrap().text_entities(),
        out.document().text_entities
    );
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
fn shape_names_do_not_shadow_ordinary_styles_or_turn_shape_targets_into_text_fonts() {
    let mut d = text_source();
    d.text_styles
        .add_allow_duplicate(shape("Standard", 0x501, "shape.shx"));
    let out = cad_document_to_ocdraw_document(&d, Default::default()).unwrap();
    assert_eq!(out.document().text_entities.len(), 1);
    assert_eq!(out.document().text_styles.len(), 1);
    assert_eq!(out.document().text_styles[0].name, "Standard");
    assert_eq!(
        out.diagnostics()
            .iter()
            .filter(|d| is_skipped_table(d))
            .count(),
        1
    );
    // A name found only in the unsupported shape role remains a located skip.
    d.text_styles
        .add_allow_duplicate(shape("Only shapes", 0x502, "shape2.shx"));
    let h = d
        .entities()
        .find(|e| matches!(e, EntityType::Text(_)))
        .unwrap()
        .common()
        .handle;
    if let EntityType::Text(t) = d.get_entity_mut(h).unwrap() {
        t.style = "Only shapes".into();
    }
    let out = cad_document_to_ocdraw_document(&d, Default::default()).unwrap();
    assert!(out.document().text_entities.is_empty());
    assert!(!out.entity_mapping().contains_key(&h));
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.action() == CadToOcdrawAction::Skipped
            && matches!(d.source(),CadToOcdrawDiagnosticSource::Entity{handle,..} if *handle==h)));

    // The ordinary font must also win when the shape record is encountered first.
    let mut d = text_source();
    let ordinary = d.text_styles.get("Standard").unwrap().clone();
    d.text_styles.clear();
    d.text_styles
        .add_allow_duplicate(shape("STANDARD", 0x503, "shape3.shx"));
    d.text_styles.add_allow_duplicate(ordinary);
    let out = cad_document_to_ocdraw_document(&d, Default::default()).unwrap();
    assert_eq!(out.document().text_entities.len(), 1);
    assert_eq!(out.document().text_styles[0].name, "Standard");
}

#[test]
fn actual_text_lookup_conflicts_and_duplicate_allocated_handles_stay_fatal() {
    let mut d = text_source();
    let mut duplicate = TextStyle::standard();
    duplicate.name = "STANDARD".into();
    duplicate.handle = Handle::new(0x601);
    d.text_styles.add_allow_duplicate(duplicate);
    assert!(matches!(
        cad_document_to_ocdraw_document(&d, Default::default()),
        Err(CadToOcdrawError::DrawingBuild(_))
    ));
    let mut d = text_source();
    let handle = d.text_styles.get("Standard").unwrap().handle.value();
    d.text_styles
        .add_allow_duplicate(shape("", handle, "shape.shx"));
    assert!(matches!(
        cad_document_to_ocdraw_document(&d, Default::default()),
        Err(CadToOcdrawError::DrawingBuild(_))
    ));
}

#[test]
fn resolving_shape_names_does_not_mask_invalid_mtext_column_heights() {
    // A negative nonfinal height is invalid; the final manual-column height
    // is now qualified as a cache/sentinel for the automatic tail.
    let mut d = text_source();
    d.text_styles
        .add_allow_duplicate(shape("", 0x27f, "ltypeshp.shx"));
    d.text_styles
        .add_allow_duplicate(shape("", 0x887, "test_shape.shx"));
    let mut m = MText::new();
    m.common.line_weight = LineWeight::ByLayer;
    m.value = "this is a Mtext\nwith multiple lines in it".into();
    m.height = 1.;
    m.rectangle_width = 15.741414244987482;
    m.column_data.column_type = 2;
    m.column_data.column_count = 2;
    m.column_data.width = m.rectangle_width;
    m.column_data.gutter = 110.22316703811953;
    m.rectangle_width = 2. * m.column_data.width + m.column_data.gutter;
    m.column_data.heights = vec![-2.9442928603910867, 0.];
    let handle = d.add_entity(EntityType::MText(m)).unwrap();
    for loss_policy in [OcdrawLossPolicy::Allow, OcdrawLossPolicy::Reject] {
        assert!(
            matches!(cad_document_to_ocdraw_document(&d, CadToOcdrawOptions {
            loss_policy,
            ..Default::default()
        }), Err(CadToOcdrawError::TextPreparation { handle: failed, .. }) if failed == handle)
        );
    }
}
