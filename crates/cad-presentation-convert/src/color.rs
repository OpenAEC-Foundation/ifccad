use crate::{CadColorLoss, CadColorMapping, CadColorValue, PresentationValueError};
use opencadcodec::Color;

/// Decode a qualified explicit CAD color, leaving inherited modes to the adapter.
pub fn explicit_color_from_cad(color: Color) -> Result<CadColorValue, PresentationValueError> {
    if color == Color::Index(7) {
        return Ok(CadColorValue::Foreground);
    }
    let indexed = match color {
        Color::Index(1..=255) => Some(("ACI".into(), u64::from(color.index().unwrap()))),
        Color::Rgb { .. } => None,
        _ => return Err(PresentationValueError::UnsupportedColor),
    };
    let (r, g, b) = color
        .rgb()
        .ok_or(PresentationValueError::UnsupportedColor)?;
    Ok(CadColorValue::Concrete {
        rgb: [r, g, b],
        indexed,
        named: None,
    })
}

/// Emit Foreground as ACI 7, or preserve concrete RGB when its identity cannot map.
/// Named identity is returned separately for context-specific serialization.
pub fn color_to_cad(value: &CadColorValue) -> Result<CadColorMapping, PresentationValueError> {
    let CadColorValue::Concrete {
        rgb,
        indexed,
        named,
    } = value
    else {
        return Ok(CadColorMapping {
            color: Color::Index(7),
            named: None,
            losses: Vec::new(),
        });
    };
    if indexed
        .as_ref()
        .is_some_and(|(system, _)| system.is_empty())
        || named
            .as_ref()
            .is_some_and(|(catalog, name)| catalog.is_empty() || name.is_empty())
    {
        return Err(PresentationValueError::InvalidColorIdentity);
    }
    let [r, g, b] = *rgb;
    let mut color = Color::from_rgb(r, g, b);
    let mut losses = Vec::new();
    if let Some((system, index)) = indexed {
        if system.eq_ignore_ascii_case("ACI") && *index == 7 {
            return Err(PresentationValueError::InvalidColorIdentity);
        }
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
        named: named.clone(),
        losses,
    })
}
