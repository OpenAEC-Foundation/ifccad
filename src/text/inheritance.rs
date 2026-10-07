//! Property-wise replacement, never previous-run or previous-paragraph inheritance.
use super::*;

pub fn resolve_character_format<C: Clone>(
    nominal_height: f64,
    style: &TextStyleProperties,
    entity: &CharacterFormat<C>,
    paragraph: &CharacterFormat<C>,
    inline: &CharacterFormat<C>,
) -> Result<ResolvedCharacterFormat<C>, TextValueError> {
    validation::positive(nominal_height, "/height")?;
    validate_text_style(style)?;
    let mut result = ResolvedCharacterFormat {
        font: style.font.clone(),
        color: TextColor::Entity,
        height: nominal_height,
        width_factor: style.width_factor,
        tracking: 1.0,
        oblique_angle: style.oblique_angle,
        underline: false,
        overline: false,
        strike_through: false,
        position: TextPosition::Bottom,
    };
    let mut selected_height = TextHeight::Relative { factor: 1.0 };
    let mut height_scope = "/characterFormat";
    for (scope, overrides) in [
        ("/characterFormat", entity),
        ("/paragraph/characterFormat", paragraph),
        ("/inline/characterFormat", inline),
    ] {
        validate_character_format(overrides, |_| true).map_err(|error| error.prefixed(scope))?;
        if let Some(font) = &overrides.font {
            result.font = font.clone();
        }
        if let Some(color) = &overrides.color {
            result.color = color.clone();
        }
        if let Some(height) = overrides.height {
            selected_height = height;
            height_scope = scope;
        }
        macro_rules! replace {
            ($($field:ident),+ $(,)?) => { $(if let Some(value) = overrides.$field { result.$field = value; })+ };
        }
        replace!(
            width_factor,
            tracking,
            oblique_angle,
            underline,
            overline,
            strike_through,
            position
        );
    }
    result.height = match selected_height {
        TextHeight::Relative { factor } => {
            validation::product(nominal_height, factor, &format!("{height_scope}/height"))?
        }
        TextHeight::Absolute { distance } => distance,
    };
    // These are required before even approximate typographic preparation.
    let advance = validation::product(result.height, result.width_factor, "/effectiveAdvance")?;
    validation::product(advance, result.tracking, "/effectiveAdvance")?;
    Ok(result)
}

pub fn resolve_paragraph_format(
    entity: &ParagraphFormat,
    paragraph: &ParagraphFormat,
) -> Result<ResolvedParagraphFormat, TextValueError> {
    validate_paragraph_format(entity).map_err(|error| error.prefixed("/paragraphFormat"))?;
    validate_paragraph_format(paragraph)
        .map_err(|error| error.prefixed("/paragraph/paragraphFormat"))?;
    macro_rules! value {
        ($field:ident, $default:expr) => {
            paragraph.$field.or(entity.$field).unwrap_or($default)
        };
    }
    Ok(ResolvedParagraphFormat {
        alignment: value!(alignment, TextParagraphAlignment::Left),
        left_indent_factor: value!(left_indent_factor, 0.0),
        right_indent_factor: value!(right_indent_factor, 0.0),
        first_line_indent_factor: value!(first_line_indent_factor, 0.0),
        space_before: value!(space_before, 0.0),
        space_after: value!(space_after, 0.0),
        line_spacing: value!(line_spacing, TextLineSpacing::Multiple { factor: 1.0 }),
        tab_stops: paragraph
            .tab_stops
            .as_ref()
            .or(entity.tab_stops.as_ref())
            .cloned()
            .unwrap_or_default(),
    })
}
