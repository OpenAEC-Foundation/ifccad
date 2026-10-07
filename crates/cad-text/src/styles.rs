use crate::CadTextError;
use ocdraw::text::*;
use opencadcodec::tables::TextStyle;

pub struct PreparedTextStyle {
    pub name: String,
    pub properties: TextStyleProperties,
}

pub fn import_text_style(source: &TextStyle) -> Result<PreparedTextStyle, CadTextError> {
    if source.annotative {
        return Err(CadTextError::Unsupported("annotative style contexts"));
    }
    if source.is_shape_file {
        return Err(CadTextError::Unsupported("shape-file style role"));
    }
    if source.xref_dependent || !source.xref_block_record_handle.is_null() {
        return Err(CadTextError::Unsupported("external text style context"));
    }
    name(&source.name)?;
    let optional = |value: &String| {
        if value.is_empty() {
            None
        } else {
            Some(value.clone())
        }
    };
    let font = FontRequest {
        family: optional(&source.true_type_font),
        cad_font_name: optional(&source.font_file),
        big_font_name: optional(&source.big_font_file),
        ..Default::default()
    };
    let mut properties = TextStyleProperties::new(font);
    properties.width_factor = source.width_factor;
    properties.oblique_angle = source.oblique_angle;
    properties.vertical = source.is_vertical;
    properties.creation_height = if source.height == 0. {
        None
    } else {
        Some(source.height)
    };
    properties.last_used_height = Some(source.last_height);
    properties.creation_backward = source.flags.backward;
    properties.creation_upside_down = source.flags.upside_down;
    validate_text_style(&properties)?;
    Ok(PreparedTextStyle {
        name: source.name.clone(),
        properties,
    })
}

pub fn prepare_text_style_to_cad(
    style_name: &str,
    properties: &TextStyleProperties,
) -> Result<TextStyle, CadTextError> {
    name(style_name)?;
    validate_text_style(properties)?;
    let font = &properties.font;
    if font.bold.is_some()
        || font.italic.is_some()
        || font.charset.is_some()
        || font.pitch.is_some()
    {
        return Err(CadTextError::Unsupported(
            "style font face flags not exposed by pinned TextStyle",
        ));
    }
    let mut target = TextStyle::new(style_name);
    target.font_file = font.cad_font_name.clone().unwrap_or_default();
    target.big_font_file = font.big_font_name.clone().unwrap_or_default();
    target.true_type_font = font.family.clone().unwrap_or_default();
    target.width_factor = properties.width_factor;
    target.oblique_angle = properties.oblique_angle;
    target.is_vertical = properties.vertical;
    target.flags.backward = properties.creation_backward;
    target.flags.upside_down = properties.creation_upside_down;
    target.height = properties.creation_height.unwrap_or(0.);
    target.last_height = properties.last_used_height.unwrap_or(2.5);
    Ok(target)
}
pub(crate) fn name(value: &str) -> Result<(), CadTextError> {
    if value.is_empty() || value.contains('\0') {
        Err(CadTextError::InvalidSource(
            "style name must be nonempty and NUL-free",
        ))
    } else {
        Ok(())
    }
}
