use super::*;
use opencadcodec::{entities::EntityCommon, xdata::XDataValue};

/// Copy for common-field auditing, consuming only the exposed ACAD origin slot.
/// All unrelated records, nested payloads and raw transport evidence remain.
pub fn residual_common(h: &Hatch) -> Result<EntityCommon, CadHatchPreparationError> {
    let mut common = h.common.clone();
    if h.is_solid {
        return Ok(common);
    }
    if let Some(mut record) = common.extended_data.remove_record("ACAD") {
        let mut depth = 0usize;
        let mut index = None;
        for (i, v) in record.values.iter().enumerate() {
            match v {
                XDataValue::ControlString(s) if s == "{" => depth += 1,
                XDataValue::ControlString(s) if s == "}" => depth = depth.saturating_sub(1),
                XDataValue::Point3D(p) if depth == 0 => {
                    if ![p.x, p.y, p.z].into_iter().all(f64::is_finite) {
                        return Err(CadPreparationError::InvalidGeometry.into());
                    }
                    if p.z != 0. {
                        return Err(super::super::unsupported(
                            "pattern.origin",
                            "nonplanar ACAD origin metadata",
                        ));
                    }
                    index = Some(i);
                    break;
                }
                _ => {}
            }
        }
        if let Some(i) = index {
            record.values.remove(i);
        }
        if !record.values.is_empty() {
            common.extended_data.add_record(record);
        }
    }
    Ok(common)
}

pub(crate) fn expand_double(
    source: &Hatch,
    literal: &mut Hatch,
) -> Result<(), CadHatchPreparationError> {
    let f = &source.pattern.lines[0];
    residual_common(source)?;
    let origin = source.pattern_origin();
    // Deliberately narrow: exact axis-aligned, continuous, origin-anchored
    // source. Other double phases and already-expanded records need qualification.
    if source.pattern_angle != 0.
        || f.angle != 0.
        || f.base_point != origin
        || f.offset.x != 0.
        || !f.offset.y.is_finite()
        || f.offset.y == 0.
        || f.offset.y.abs() != source.pattern_scale
    {
        return Err(super::super::unsupported(
            "is_double",
            "only origin-anchored axis-aligned continuous single-family double is qualified",
        ));
    }
    let mut orthogonal = f.clone();
    orthogonal.angle = std::f64::consts::FRAC_PI_2;
    orthogonal.offset = Vector2::new(-f.offset.y, 0.);
    literal.pattern.lines.push(orthogonal);
    Ok(())
}
