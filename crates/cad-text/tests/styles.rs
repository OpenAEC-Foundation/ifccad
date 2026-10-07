use cad_text::*;
use ocdraw::text::*;
use opencadcodec::tables::TextStyle;

#[test]
fn local_style_requests_and_creation_metadata_roundtrip_without_loading_fonts() {
    let mut source = TextStyle::new("Labels");
    source.font_file = "simplex.shx".into();
    source.big_font_file = "bigfont.shx".into();
    source.true_type_font = "Arial".into();
    source.width_factor = 0.8;
    source.oblique_angle = 0.1;
    source.height = 3.;
    source.last_height = 0.;
    source.flags.backward = true;
    source.is_vertical = true;
    let native = import_text_style(&source).unwrap();
    assert_eq!(native.name, "Labels");
    assert_eq!(
        native.properties.font.cad_font_name.as_deref(),
        Some("simplex.shx")
    );
    assert_eq!(native.properties.font.family.as_deref(), Some("Arial"));
    assert_eq!(native.properties.creation_height, Some(3.));
    assert_eq!(native.properties.last_used_height, Some(0.));
    assert!(native.properties.creation_backward && native.properties.vertical);
    let target = prepare_text_style_to_cad(&native.name, &native.properties).unwrap();
    assert_eq!(target.font_file, source.font_file);
    assert_eq!(target.big_font_file, source.big_font_file);
    assert_eq!(target.true_type_font, source.true_type_font);
    assert_eq!(target.last_height, 0.);
    assert_eq!(target.flags, source.flags);
    assert_eq!(target.width_factor, 0.8);
}

#[test]
fn font_and_style_restrictions_are_separate_from_numeric_invalidity() {
    let mut source = TextStyle::new("annotative");
    source.annotative = true;
    assert!(matches!(
        import_text_style(&source),
        Err(CadTextError::Unsupported(_))
    ));
    source.annotative = false;
    source.is_shape_file = true;
    assert!(matches!(
        import_text_style(&source),
        Err(CadTextError::Unsupported(_))
    ));
    source.is_shape_file = false;
    source.width_factor = 0.;
    assert!(matches!(
        import_text_style(&source),
        Err(CadTextError::Values(_))
    ));
    let mut props = TextStyleProperties::new(FontRequest::family("Arial"));
    props.font.bold = Some(true);
    assert!(matches!(
        prepare_text_style_to_cad("Bold", &props),
        Err(CadTextError::Unsupported(_))
    ));
}
