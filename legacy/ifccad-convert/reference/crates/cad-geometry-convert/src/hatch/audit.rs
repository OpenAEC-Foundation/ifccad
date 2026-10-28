use super::{unsupported, CadHatchPreparationError, HatchSourceLoss};
use opencadcodec::entities::hatch::{Hatch, HatchPatternType};
use opencadcodec::{Color, Vector2};

pub(super) fn audit(h: &Hatch) -> Result<Vec<HatchSourceLoss>, CadHatchPreparationError> {
    if h.common
        .extended_data
        .records()
        .iter()
        .any(|r| r.application_name.eq_ignore_ascii_case("AcadAnnotative"))
    {
        return Err(unsupported(
            "common.extended_data",
            "annotative Hatch contexts are unqualified",
        ));
    }
    if h.common.xdictionary_handle.is_some_and(|h| !h.is_null()) {
        return Err(unsupported("common.xdictionary_handle","extension dictionary may contain active Hatch context; requires separate qualification"));
    }
    if h.is_mpolygon {
        return Err(unsupported(
            "is_mpolygon",
            "MPOLYGON is outside the native Hatch profile",
        ));
    }
    if !h.gradient_color.enabled
        && !h.is_solid
        && h.is_double
        && h.pattern_type == HatchPatternType::UserDefined
    {
        return Err(unsupported(
            "is_double",
            "active user-defined double pattern requires a qualified orthogonal expansion",
        ));
    }
    let mut losses = Vec::new();
    let mut loss = |field, detail: &str| {
        losses.push(HatchSourceLoss {
            field,
            detail: detail.into(),
        })
    };
    if !h.gradient_color.enabled && !h.is_solid && h.pattern_type == HatchPatternType::UserDefined {
        return Err(unsupported(
            "pattern_type",
            "user-defined pattern needs model-aware continuous linetype qualification",
        ));
    }
    if h.mpolygon_hatch_color != Color::ByLayer
        || h.mpolygon_x_direction != Vector2::ZERO
        || !h.mpolygon_invalid_loops.is_empty()
    {
        loss("mpolygon_fields", "inactive MPOLYGON state is not retained");
    }
    super::prepare_loop_flags_from_cad(h)?;
    Ok(losses)
}
