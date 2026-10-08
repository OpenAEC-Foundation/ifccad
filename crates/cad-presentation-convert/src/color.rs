use crate::{CadColorLoss, CadColorMapping, CadColorValue, PresentationValueError};
use opencadcodec::Color;

/// Decode a concrete CAD color, leaving inherited modes to the adapter.
pub fn explicit_color_from_cad(color: Color) -> Result<CadColorValue, PresentationValueError> {
    let indexed = match color {
        Color::Index(1..=255) => Some(("ACI".into(), u64::from(color.index().unwrap()))),
        Color::Rgb { .. } => None,
        _ => return Err(PresentationValueError::UnsupportedColor),
    };
    let (r, g, b) = color
        .rgb()
        .ok_or(PresentationValueError::UnsupportedColor)?;
    Ok(CadColorValue {
        rgb: [r, g, b],
        indexed,
        named: None,
    })
}

/// Preserve RGB fallback when an indexed identity cannot denote that color.
/// Named identity is returned separately for context-specific serialization.
pub fn color_to_cad(value: &CadColorValue) -> Result<CadColorMapping, PresentationValueError> {
    if value
        .indexed
        .as_ref()
        .is_some_and(|(system, _)| system.is_empty())
        || value
            .named
            .as_ref()
            .is_some_and(|(catalog, name)| catalog.is_empty() || name.is_empty())
    {
        return Err(PresentationValueError::InvalidColorIdentity);
    }
    let [r, g, b] = value.rgb;
    let mut color = Color::from_rgb(r, g, b);
    let mut losses = Vec::new();
    if let Some((system, index)) = &value.indexed {
        if system.eq_ignore_ascii_case("ACI") && (1..=255).contains(index) {
            let candidate = Color::Index(*index as u8);
            if candidate.rgb() == Some((r, g, b)) {
                color = candidate;
            } else {
                losses.push(CadColorLoss::InconsistentIndex);
            }
        } else {
            losses.push(CadColorLoss::UnsupportedIndex);
        }
    }
    Ok(CadColorMapping {
        color,
        named: value.named.clone(),
        losses,
    })
}
