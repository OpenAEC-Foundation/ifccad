use ocdraw::text::{
    resolve_character_format, CharacterFormat, FontRequest, TextHeight, TextStyleProperties,
};

#[test]
fn public_text_values_resolve_overrides_without_a_drawing_or_cad_runtime() {
    let style = TextStyleProperties::new(FontRequest::family("Noto Sans"));
    let entity = CharacterFormat::<String> {
        height: Some(TextHeight::Relative { factor: 1.5 }),
        ..Default::default()
    };
    let resolved = resolve_character_format(
        2.5,
        &style,
        &entity,
        &Default::default(),
        &Default::default(),
    )
    .unwrap();
    assert_eq!(resolved.height, 3.75);
    assert_eq!(entity.height, Some(TextHeight::Relative { factor: 1.5 }));
}
