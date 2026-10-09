use super::math::*;
use super::*;
use crate::geometry::{
    cad_plane,
    numeric::{exact, round_nearest},
};
use opencadcodec::entities::hatch::HatchPatternLine;
pub(super) fn from_cad(h: &Hatch) -> Result<HatchLinePattern, CadHatchPreparationError> {
    validate_source_scalars(h)?;
    if !h.pattern_angle.is_finite() || !h.pattern_scale.is_finite() || h.pattern_scale <= 0. {
        return Err(CadPreparationError::InvalidGeometry.into());
    }
    if h.pattern.lines.is_empty() {
        return Err(super::super::unsupported(
            "pattern",
            "absent explicit families",
        ));
    }
    let origin = h.pattern_origin();
    if ![origin.x, origin.y].into_iter().all(f64::is_finite) {
        return Err(CadPreparationError::InvalidGeometry.into());
    }
    let mut families = Vec::new();
    for line in &h.pattern.lines {
        if ![
            line.angle,
            line.base_point.x,
            line.base_point.y,
            line.offset.x,
            line.offset.y,
        ]
        .into_iter()
        .chain(line.dash_lengths.iter().copied())
        .all(f64::is_finite)
        {
            return Err(CadPreparationError::InvalidGeometry.into());
        }
        let angle = round_nearest(&(exact(line.angle) - exact(h.pattern_angle)))
            .map_err(|_| CadPreparationError::OutOfRange)?;
        let base = scale(
            &rotate(
                &sub(
                    &point([line.base_point.x, line.base_point.y]),
                    &point([origin.x, origin.y]),
                ),
                -h.pattern_angle,
            ),
            &exact(h.pattern_scale).recip(),
        );
        let offset = scale(
            &rotate(&point([line.offset.x, line.offset.y]), -h.pattern_angle),
            &exact(h.pattern_scale).recip(),
        );
        let d = direction(angle);
        let along = dot(&d, &offset);
        let perpendicular = cross(&d, &offset);
        let perpendicular_value = nearest(&perpendicular)?;
        if perpendicular_value == 0. {
            return Err(CadPreparationError::OutOfRange.into());
        }
        let dashes = line
            .dash_lengths
            .iter()
            .map(|&v| {
                if v == 0. {
                    Ok(HatchDash::Dot)
                } else {
                    let length = round_nearest(&(exact(v.abs()) / exact(h.pattern_scale)))
                        .map_err(|_| CadPreparationError::OutOfRange)?;
                    if length <= 0. {
                        return Err(CadPreparationError::OutOfRange);
                    }
                    Ok(if v > 0. {
                        HatchDash::Dash { length }
                    } else {
                        HatchDash::Gap { length }
                    })
                }
            })
            .collect::<Result<Vec<_>, CadPreparationError>>()?;
        families.push(HatchLineFamily {
            angle,
            base_point: materialize(&base)?,
            offset: [nearest(&along)?, perpendicular_value],
            dashes,
        });
    }
    let p = HatchLinePattern {
        name: Some(h.pattern.name.clone()),
        description: Some(h.pattern.description.clone()),
        origin: [origin.x, origin.y],
        rotation: h.pattern_angle,
        scale: h.pattern_scale,
        families,
    };
    validate_hatch_fill(&HatchFill::LinePattern(p.clone()))
        .map_err(|e| super::super::unsupported("pattern", e.to_string()))?;
    Ok(p)
}

pub(crate) fn validate_source_scalars(h: &Hatch) -> Result<(), CadHatchPreparationError> {
    if h.pattern_scale <= 0.
        || ![h.pattern_angle, h.pattern_scale]
            .into_iter()
            .chain(h.pattern.lines.iter().flat_map(|f| {
                [
                    f.angle,
                    f.base_point.x,
                    f.base_point.y,
                    f.offset.x,
                    f.offset.y,
                ]
                .into_iter()
                .chain(f.dash_lengths.iter().copied())
            }))
            .all(f64::is_finite)
    {
        return Err(CadPreparationError::InvalidGeometry.into());
    }
    Ok(())
}
pub(super) fn to_cad(
    p: &HatchLinePattern,
    plane: CoordinateFrame3,
    normal: Vector3,
) -> Result<(HatchPattern, f64, Vector2), CadHatchPreparationError> {
    let native = super::super::evidence::Frame::native(plane);
    let basis = cad_plane(normal).ok_or(CadPreparationError::InvalidGeometry)?;
    let x = round_nearest(
        &native
            .x
            .iter()
            .zip(basis.u)
            .map(|(a, b)| a * exact(b))
            .sum(),
    )
    .map_err(|_| CadPreparationError::OutOfRange)?;
    let y = round_nearest(
        &native
            .x
            .iter()
            .zip(basis.v)
            .map(|(a, b)| a * exact(b))
            .sum(),
    )
    .map_err(|_| CadPreparationError::OutOfRange)?;
    let phase = y.atan2(x);
    let angle = round_nearest(&(exact(p.rotation) + exact(phase)))
        .map_err(|_| CadPreparationError::OutOfRange)?;
    let project = |v: &V2, translate: bool| -> V2 {
        let world: [crate::geometry::blocks::Range; 3] = std::array::from_fn(|i| {
            v[0].scale(&native.x[i]).add(&v[1].scale(&native.y[i])).add(
                &crate::geometry::blocks::Range::point(if translate {
                    native.o[i].clone()
                } else {
                    exact(0.)
                }),
            )
        });
        [basis.u, basis.v].map(|axis| {
            world.iter().zip(axis).fold(
                crate::geometry::blocks::Range::point(exact(0.)),
                |s, (v, a)| s.add(&v.scale(&exact(a))),
            )
        })
    };
    let o = materialize(&project(&point(p.origin), true))?;
    let mut pattern = HatchPattern::new(p.name.as_deref().unwrap_or("OCDRAW_PATTERN"));
    pattern.description = p.description.clone().unwrap_or_default();
    for f in &p.families {
        let base = add(
            &point(p.origin),
            &scale(&rotate(&point(f.base_point), p.rotation), &exact(p.scale)),
        );
        let d = direction(f.angle);
        let n = [d[1].scale(&exact(-1.)), d[0].clone()];
        let offset = scale(
            &rotate(
                &add(
                    &scale(&d, &exact(f.offset[0])),
                    &scale(&n, &exact(f.offset[1])),
                ),
                p.rotation,
            ),
            &exact(p.scale),
        );
        let b = materialize(&project(&base, true))?;
        let off = materialize(&project(&offset, false))?;
        let a = round_nearest(&(exact(f.angle) + exact(angle)))
            .map_err(|_| CadPreparationError::OutOfRange)?;
        let dash_lengths = f
            .dashes
            .iter()
            .map(|d| match *d {
                HatchDash::Dot => Ok(0.),
                HatchDash::Dash { length } | HatchDash::Gap { length } => {
                    let v = round_nearest(&(exact(length) * exact(p.scale)))
                        .map_err(|_| CadPreparationError::OutOfRange)?;
                    if v <= 0. {
                        return Err(CadPreparationError::OutOfRange);
                    }
                    Ok(if matches!(d, HatchDash::Gap { .. }) {
                        -v
                    } else {
                        v
                    })
                }
            })
            .collect::<Result<Vec<_>, CadPreparationError>>()?;
        pattern.lines.push(HatchPatternLine {
            angle: a,
            base_point: Vector2::new(b[0], b[1]),
            offset: Vector2::new(off[0], off[1]),
            dash_lengths,
        });
    }
    Ok((pattern, angle, Vector2::new(o[0], o[1])))
}
