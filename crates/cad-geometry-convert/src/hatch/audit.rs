use super::{unsupported, CadHatchPreparationError, HatchSourceLoss};
use opencadcodec::entities::hatch::{
    BoundaryPathFlags, Hatch, HatchGradientPattern, HatchPatternType,
};
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
    if h.gradient_color.enabled {
        return Err(unsupported("gradient_color", "active gradient fill"));
    }
    if !h.is_solid {
        return Err(unsupported(
            "is_solid",
            "pattern fill requires the second Hatch slice",
        ));
    }
    let mut losses = Vec::new();
    let mut loss = |field, detail: &str| {
        losses.push(HatchSourceLoss {
            field,
            detail: detail.into(),
        })
    };
    if h.pattern.name != "SOLID" || !h.pattern.description.is_empty() || !h.pattern.lines.is_empty()
    {
        loss("pattern", "inactive Solid pattern metadata is not retained");
    }
    if h.pattern_type != HatchPatternType::Predefined
        || h.pattern_angle != 0.
        || h.pattern_scale != 1.
        || h.is_double
    {
        loss(
            "pattern_context",
            "inactive Solid pattern creation settings are not retained",
        );
    }
    if h.gradient_color != HatchGradientPattern::default() {
        loss(
            "gradient_color",
            "disabled gradient editing state is not retained",
        );
    }
    if !h.seed_points.is_empty() {
        loss(
            "seed_points",
            "boundary construction seed points are not retained",
        );
    }
    if h.pixel_size != 0. {
        loss(
            "pixel_size",
            "boundary construction pixel size is not retained",
        );
    }
    if h.mpolygon_hatch_color != Color::ByLayer
        || h.mpolygon_x_direction != Vector2::ZERO
        || !h.mpolygon_invalid_loops.is_empty()
    {
        loss("mpolygon_fields", "inactive MPOLYGON state is not retained");
    }
    for p in &h.paths {
        let hints = BoundaryPathFlags::EXTERNAL.bits()
            | BoundaryPathFlags::DERIVED.bits()
            | BoundaryPathFlags::OUTERMOST.bits();
        let backing = BoundaryPathFlags::POLYLINE.bits();
        // POLYLINE is encoding; EXTERNAL/DERIVED/OUTERMOST are source hints.
        // Text islands, invalid/open/duplicate loops and unknown flags require
        // an actual interpretation, not removal into a normal fill.
        if p.flags.bits() & !(hints | backing) != 0 {
            return Err(unsupported(
                "paths.flags",
                format!("unqualified active loop flags {:#x}", p.flags.bits()),
            ));
        }
        if p.flags.bits() & hints != 0 {
            loss("paths.flags", "source provenance/nesting hints are not retained; native area rules use stored contours");
        }
    }
    Ok(losses)
}
