//! Capture authored parameters before active UserDefined/double materialization.
use super::*;
use ifccad::geometry_kernel::hatch::*;
use opencadcodec::{entities::hatch::*, Vector2};
pub struct PreparedHatchEditing {
    pub classic_settings: Option<HatchClassicSettings>,
    pub boundary_construction: Option<HatchBoundaryConstruction>,
    pub losses: Vec<HatchSourceLoss>,
}
fn invalid(e: HatchEditingValidationError) -> CadHatchPreparationError {
    CadHatchPreparationError::Editing(e)
}
fn same_axes(h: &Hatch, p: CoordinateFrame3) -> bool {
    let a = super::evidence::Frame::native(p);
    let b = super::evidence::Frame::cad(h);
    a.x == b.x && a.y == b.y
}
fn validate_frame(h: &Hatch) -> Result<(), CadHatchPreparationError> {
    if !h.elevation.is_finite()
        || ![h.normal.x, h.normal.y, h.normal.z]
            .into_iter()
            .all(f64::is_finite)
        || (h.normal.x == 0. && h.normal.y == 0. && h.normal.z == 0.)
    {
        return Err(CadPreparationError::InvalidGeometry.into());
    }
    let basis = opencadcodec::types::Matrix3::arbitrary_axis(h.normal).m;
    if !basis.iter().flatten().all(|x| x.is_finite())
        || (0..3).any(|j| (0..3).all(|i| basis[i][j] == 0.))
    {
        return Err(CadPreparationError::InvalidGeometry.into());
    }
    Ok(())
}
pub fn prepare_editing_from_cad(
    h: &Hatch,
    placement: CoordinateFrame3,
) -> Result<PreparedHatchEditing, CadHatchPreparationError> {
    validate_frame(h)?;
    super::pattern::validate_source_scalars(h)?;
    for line in &h.pattern.lines {
        let period = line
            .dash_lengths
            .iter()
            .map(|v| crate::geometry::numeric::exact(v.abs()))
            .sum::<num_rational::BigRational>();
        if period > crate::geometry::numeric::exact(f64::MAX) {
            return Err(CadPreparationError::InvalidGeometry.into());
        }
        if line.offset == Vector2::ZERO
            || (!line.dash_lengths.is_empty() && line.dash_lengths.iter().all(|d| *d == 0.))
        {
            return Err(CadPreparationError::InvalidGeometry.into());
        }
    }
    let construction = HatchBoundaryConstruction {
        seed_points: h.seed_points.iter().map(|p| [p.x, p.y]).collect(),
        pixel_size: h.pixel_size,
    };
    if !construction.is_empty() {
        validate_hatch_boundary_construction(&construction).map_err(invalid)?;
    }
    let mut losses = Vec::new();
    let origin_unqualified = match super::pattern::residual_common(h) {
        Ok(_) => false,
        Err(CadHatchPreparationError::Unsupported { detail, .. })
            if h.is_solid || h.gradient_color.enabled =>
        {
            losses.push(HatchSourceLoss {
                field: "classicSettings",
                detail,
            });
            true
        }
        Err(e) => return Err(e),
    };
    let scaffolding = h.is_solid
        && h.pattern.name == "SOLID"
        && h.pattern.description.is_empty()
        && h.pattern.lines.is_empty()
        && h.pattern_type == HatchPatternType::Predefined
        && h.pattern_angle == 0.
        && h.pattern_scale == 1.
        && !h.is_double
        && h.stored_pattern_origin().is_none();
    let classic = if scaffolding || origin_unqualified {
        None
    } else {
        let definition_lines = Some(
            h.pattern
                .lines
                .iter()
                .map(|l| HatchPatternDefinitionLine {
                    angle: l.angle,
                    base_point: [l.base_point.x, l.base_point.y],
                    offset_vector: [l.offset.x, l.offset.y],
                    dash_lengths: l.dash_lengths.clone(),
                })
                .collect(),
        );
        let c = HatchClassicSettings {
            preferred_fill: Some(if h.is_solid {
                HatchClassicFill::Solid
            } else {
                HatchClassicFill::LinePattern
            }),
            creation_type: Some(match h.pattern_type {
                HatchPatternType::UserDefined => HatchCreationType::UserDefined,
                HatchPatternType::Predefined => HatchCreationType::Predefined,
                HatchPatternType::Custom => HatchCreationType::Custom,
            }),
            name: Some(h.pattern.name.clone()),
            description: Some(h.pattern.description.clone()),
            origin: h.stored_pattern_origin().map(|p| [p.x, p.y]),
            rotation: h.pattern_angle,
            scale: h.pattern_scale,
            double: h.is_double,
            definition_lines,
        };
        validate_hatch_classic_settings(&c).map_err(invalid)?;
        Some(c)
    };
    let mut classic_settings = classic;
    let mut boundary_construction = (!construction.is_empty()).then_some(construction);
    if !same_axes(h, placement) {
        if classic_settings.take().is_some() {
            losses.push(loss(
                "classicSettings",
                "local XY axes differ from the CAD OCS",
            ));
        }
        if boundary_construction.take().is_some() {
            losses.push(loss(
                "boundaryConstruction",
                "local XY axes differ from the CAD OCS",
            ));
        }
    }
    Ok(PreparedHatchEditing {
        classic_settings,
        boundary_construction,
        losses,
    })
}
fn loss(field: &'static str, detail: &str) -> HatchSourceLoss {
    HatchSourceLoss {
        field,
        detail: detail.into(),
    }
}
fn point_to_cad(
    h: &Hatch,
    p: CoordinateFrame3,
    xy: [f64; 2],
) -> Result<[f64; 2], CadHatchPreparationError> {
    use crate::geometry::numeric::{exact, round_nearest};
    let native = super::evidence::Frame::native(p);
    let cad = super::evidence::Frame::cad(h);
    use num_traits::Zero;
    let delta: [_; 3] = std::array::from_fn(|i| &native.o[i] - &cad.o[i]);
    let dot = |a: &[_; 3], b: &[_; 3]| {
        a.iter()
            .zip(b)
            .map(|(a, b)| a * b)
            .sum::<num_rational::BigRational>()
    };
    let xx = dot(&cad.x, &cad.x);
    let yy = dot(&cad.y, &cad.y);
    let cross = dot(&cad.x, &cad.y);
    let ox = dot(&delta, &cad.x);
    let oy = dot(&delta, &cad.y);
    let determinant = &xx * &yy - &cross * &cross;
    if determinant.is_zero() {
        return Err(CadPreparationError::InvalidGeometry.into());
    }
    let shifts = [
        (&ox * &yy - &oy * &cross) / &determinant,
        (&oy * &xx - &ox * &cross) / &determinant,
    ];
    let mut result = xy;
    for i in 0..2 {
        if shifts[i].is_zero() {
            continue;
        }
        let wanted = exact(xy[i]) + &shifts[i];
        let value = round_nearest(&wanted).map_err(|_| CadPreparationError::OutOfRange)?;
        if exact(value) != wanted {
            return Err(CadPreparationError::OutOfRange.into());
        }
        result[i] = value;
    }
    Ok(result)
}
#[allow(clippy::too_many_arguments)]
fn apply_editing_impl(
    h: &mut Hatch,
    placement: CoordinateFrame3,
    fill: &HatchFill,
    classic: Option<&HatchClassicSettings>,
    construction: Option<&HatchBoundaryConstruction>,
) -> Result<Vec<HatchSourceLoss>, CadHatchPreparationError> {
    validate_frame(h)?;
    validate_hatch_fill(fill).map_err(|_| CadPreparationError::InvalidGeometry)?;
    if let Some(c) = classic {
        validate_hatch_classic_settings(c).map_err(invalid)?;
    }
    if let Some(c) = construction {
        validate_hatch_boundary_construction(c).map_err(invalid)?;
    }
    let mut losses = Vec::new();
    if !same_axes(h, placement) {
        if classic.is_some() {
            losses.push(loss(
                "classicSettings",
                "local XY axes differ from target CAD OCS",
            ));
        }
        if construction.is_some() {
            losses.push(loss(
                "boundaryConstruction",
                "local XY axes differ from target CAD OCS",
            ));
        }
        return Ok(losses);
    }
    if let Some(c) = construction {
        match c.seed_points.iter().map(|&p|point_to_cad(h,placement,p)).collect::<Result<Vec<_>,_>>() {
            Ok(points)=>{h.seed_points=points.into_iter().map(|p|Vector2::new(p[0],p[1])).collect();h.pixel_size=c.pixel_size;}
            Err(CadHatchPreparationError::Primitive(CadPreparationError::OutOfRange))=>losses.push(loss("boundaryConstruction","finite seed coordinates cannot be represented exactly in the target OCS; the whole construction record is omitted")),
            Err(e)=>return Err(e),
        }
    }
    if let Some(c) = classic {
        let mut candidate = h.clone();
        candidate.pattern_type = match c.creation_type.unwrap_or(HatchCreationType::Custom) {
            HatchCreationType::UserDefined => HatchPatternType::UserDefined,
            HatchCreationType::Predefined => HatchPatternType::Predefined,
            HatchCreationType::Custom => HatchPatternType::Custom,
        };
        candidate.is_double = c.double;
        candidate.pattern_angle = c.rotation;
        candidate.pattern_scale = c.scale;
        if let Some(name) = &c.name {
            candidate.pattern.name = name.clone();
        }
        if let Some(description) = &c.description {
            candidate.pattern.description = description.clone();
        }
        if let Some(f) = c.preferred_fill {
            candidate.is_solid = f == HatchClassicFill::Solid;
        }
        let mut expected = c.clone();
        if c.origin.is_none() {
            // The active literal writer may have materialized a default origin.
            // Restore authored absence while preserving unrelated XDATA records.
            let mut probe = candidate.clone();
            probe.is_solid = false;
            probe.gradient_color.enabled = false;
            candidate.common = super::pattern::residual_common(&probe)?;
        }
        if let Some(origin) = c.origin {
            match point_to_cad(h, placement, origin) {
                Ok(p) => {
                    expected.origin = Some(p);
                    candidate.record_pattern_origin(Vector2::new(p[0], p[1]));
                }
                Err(CadHatchPreparationError::Primitive(CadPreparationError::OutOfRange)) => {
                    losses.push(loss("classicSettings","finite classic origin cannot be represented exactly in the target OCS; the whole record is omitted"));
                    return Ok(losses);
                }
                Err(e) => return Err(e),
            }
        }
        if let Some(lines) = &c.definition_lines {
            let translated = lines
                .iter()
                .map(|line| {
                    let p = point_to_cad(h, placement, line.base_point)?;
                    Ok(HatchPatternLine {
                        angle: line.angle,
                        base_point: Vector2::new(p[0], p[1]),
                        offset: Vector2::new(line.offset_vector[0], line.offset_vector[1]),
                        dash_lengths: line.dash_lengths.clone(),
                    })
                })
                .collect::<Result<Vec<_>, CadHatchPreparationError>>();
            match translated {
                Ok(lines) => {
                    expected.definition_lines = Some(
                        lines
                            .iter()
                            .map(|l| HatchPatternDefinitionLine {
                                angle: l.angle,
                                base_point: [l.base_point.x, l.base_point.y],
                                offset_vector: [l.offset.x, l.offset.y],
                                dash_lengths: l.dash_lengths.clone(),
                            })
                            .collect(),
                    );
                    candidate.pattern.lines = lines;
                }
                Err(CadHatchPreparationError::Primitive(CadPreparationError::OutOfRange)) => {
                    losses.push(loss("classicSettings/definitionLines", "finite definition points cannot be translated exactly to the target OCS; the whole record is omitted"));
                    return Ok(losses);
                }
                Err(e) => return Err(e),
            }
        }
        let native = prepare_editing_from_cad(&candidate, placement)?.classic_settings;
        let presence_ok = native.as_ref() == Some(&expected);
        let active_ok = if matches!(fill, HatchFill::LinePattern(_)) {
            if candidate.is_solid {
                false
            } else {
                let HatchFill::LinePattern(expected) = fill else {
                    unreachable!()
                };
                Some(candidate.pattern.name.as_str()) == expected.name.as_deref()
                    && Some(candidate.pattern.description.as_str())
                        == expected.description.as_deref()
            }
        } else {
            candidate.is_solid == h.is_solid || matches!(fill, HatchFill::Gradient)
        };
        if presence_ok && active_ok {
            *h = candidate;
        } else {
            let field = if presence_ok {
                "classicSettings/recipe"
            } else {
                native.as_ref().map_or("classicSettings", |actual| {
                    classic_difference(&expected, actual)
                })
            };
            losses.push(loss(field,if presence_ok {"the recipe fill kind or literal metadata does not match the active fill; the complete editing record is omitted"}else{"CAD slots cannot retain the authored presence/value at this field; the complete editing record is omitted"}));
        }
    }
    Ok(losses)
}
fn classic_difference(a: &HatchClassicSettings, b: &HatchClassicSettings) -> &'static str {
    if a.preferred_fill != b.preferred_fill {
        "classicSettings/preferredFill"
    } else if a.creation_type != b.creation_type {
        "classicSettings/creationType"
    } else if a.name != b.name {
        "classicSettings/name"
    } else if a.description != b.description {
        "classicSettings/description"
    } else if a.origin != b.origin {
        "classicSettings/origin"
    } else if a.rotation != b.rotation {
        "classicSettings/rotation"
    } else if a.scale != b.scale {
        "classicSettings/scale"
    } else if a.double != b.double {
        "classicSettings/double"
    } else {
        "classicSettings/definitionLines"
    }
}

/// Apply editing values for bounded non-pattern users. Patterned conversion
/// should retain alternatives and assess the actual candidate before selection.
pub fn apply_editing_to_cad(
    h: &mut Hatch,
    placement: CoordinateFrame3,
    fill: &HatchFill,
    classic: Option<&HatchClassicSettings>,
    construction: Option<&HatchBoundaryConstruction>,
    _continuous_linetype: bool,
) -> Result<Vec<HatchSourceLoss>, CadHatchPreparationError> {
    validate_hatch_fill(fill).map_err(|_| CadPreparationError::InvalidGeometry)?;
    if matches!(fill, HatchFill::LinePattern(_)) {
        return Err(unsupported(
            "classicSettings/recipe",
            "patterned output requires prepared candidates and actual occurrence assessment",
        ));
    }
    apply_editing_impl(h, placement, fill, classic, construction)
}
pub struct PreparedHatchEditingCandidates {
    pub preferred: Option<PreparedCadHatch>,
    pub fallback: PreparedCadHatch,
}
pub fn prepare_hatch_editing_candidates(
    mut fallback: PreparedCadHatch,
    placement: CoordinateFrame3,
    fill: &HatchFill,
    classic: Option<&HatchClassicSettings>,
    construction: Option<&HatchBoundaryConstruction>,
    continuous_linetype: bool,
) -> Result<PreparedHatchEditingCandidates, CadHatchPreparationError> {
    let losses = apply_editing_impl(&mut fallback.hatch, placement, fill, None, construction)?;
    fallback.losses.extend(losses);
    if classic.is_none() {
        return Ok(PreparedHatchEditingCandidates {
            preferred: None,
            fallback,
        });
    }
    let mut candidate = fallback.clone();
    let losses = apply_editing_impl(&mut candidate.hatch, placement, fill, classic, None)?;
    if !losses.is_empty() {
        fallback.losses.extend(losses);
        return Ok(PreparedHatchEditingCandidates {
            preferred: None,
            fallback,
        });
    }
    if let HatchFill::LinePattern(pattern) = fill {
        let pair = match super::pattern::pattern_candidate_pair_to_cad(
            pattern,
            placement,
            &candidate.native_boundaries,
            &candidate.hatch,
            continuous_linetype,
        ) {
            Ok(pair) => pair,
            Err(e) => {
                let detail = match &e {
                    CadHatchPreparationError::Unsupported { detail, .. } => {
                        format!("Original recipe semantics are not qualified: {detail}")
                    }
                    _ => format!("Original recipe compatibility proof is unavailable: {e}"),
                };
                fallback
                    .losses
                    .push(loss("classicSettings/recipe", &detail));
                return Ok(PreparedHatchEditingCandidates {
                    preferred: None,
                    fallback,
                });
            }
        };
        *candidate
            .pairs
            .last_mut()
            .expect("prepared pattern certificate") = pair;
    }
    Ok(PreparedHatchEditingCandidates {
        preferred: Some(candidate),
        fallback,
    })
}
