//! Stops use factors relative to nominal height, from the left paragraph indent.
use super::*;

pub fn next_tab_stop(
    stops: &[TextTabStop],
    position_factor: f64,
) -> Result<TextTabStop, TextValueError> {
    validation::finite(position_factor, "/tabPosition")?;
    validation::validate_tab_stops(stops)?;
    if let Some(stop) = stops
        .iter()
        .find(|stop| stop.position_factor > position_factor)
    {
        return Ok(*stop);
    }
    // The fallback grid stays anchored at the original indent, not the last stop.
    let next = ((position_factor / 4.0).floor() + 1.0).max(1.0) * 4.0;
    if !next.is_finite() || next <= position_factor {
        return Err(TextValueError::new(
            TextValueErrorCode::DerivedOutOfRange,
            "/tabPosition",
            "next grid stop is not representable",
        ));
    }
    Ok(TextTabStop {
        position_factor: next,
        alignment: TextTabAlignment::Left,
    })
}
