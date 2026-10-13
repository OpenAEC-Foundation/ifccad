use crate::*;
use ocdraw::dimension::*;
use opencadcodec::{
    xdata::{ExtendedDataRecord, XDataValue as X},
    Handle,
};
use std::collections::BTreeSet;
pub struct CadDimensionOverrides {
    pub values: CadDimensionStyleOverrides,
    pub residual: Vec<ExtendedDataRecord>,
}
fn real(code: i16, v: &X) -> Result<f64, CadDimensionPreparationError> {
    if let X::Real(x) = v {
        if x.is_finite() {
            return Ok(*x);
        }
    }
    Err(invalid(
        &format!("DSTYLE/{code}"),
        "dimvar requires finite group-1040 Real",
    ))
}
fn integer(code: i16, v: &X) -> Result<i16, CadDimensionPreparationError> {
    if let X::Integer16(x) = v {
        Ok(*x)
    } else {
        Err(invalid(
            &format!("DSTYLE/{code}"),
            "dimvar requires group-1070 Integer16",
        ))
    }
}
fn boolean(code: i16, v: &X) -> Result<bool, CadDimensionPreparationError> {
    match integer(code, v)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(invalid(&format!("DSTYLE/{code}"), "boolean must be 0 or 1")),
    }
}
fn string(code: i16, v: &X) -> Result<&str, CadDimensionPreparationError> {
    if let X::String(x) = v {
        Ok(x)
    } else {
        Err(invalid(
            &format!("DSTYLE/{code}"),
            "dimvar requires group-1000 String",
        ))
    }
}
fn handle(code: i16, v: &X) -> Result<Handle, CadDimensionPreparationError> {
    if let X::Handle(x) = v {
        Ok(*x)
    } else {
        Err(invalid(
            &format!("DSTYLE/{code}"),
            "dimvar requires group-1005 Handle",
        ))
    }
}
pub fn parse_dimension_overrides(
    records: &[ExtendedDataRecord],
) -> Result<CadDimensionOverrides, CadDimensionPreparationError> {
    let mut values = CadDimensionStyleOverrides::default();
    let mut residual = Vec::new();
    let mut found = false;
    for record in records {
        if !record.application_name.eq_ignore_ascii_case("ACAD") {
            residual.push(record.clone());
            continue;
        }
        let mut leftover = Vec::new();
        let mut at = 0;
        while at < record.values.len() {
            if record.values[at] != X::String("DSTYLE".into()) {
                leftover.push(record.values[at].clone());
                at += 1;
                continue;
            }
            if found {
                return Err(invalid("DSTYLE", "duplicate override subsection"));
            }
            found = true;
            at += 1;
            if record.values.get(at) != Some(&X::ControlString("{".into())) {
                return Err(invalid("DSTYLE", "missing opening delimiter"));
            }
            at += 1;
            let mut seen = BTreeSet::new();
            loop {
                let item = record
                    .values
                    .get(at)
                    .ok_or_else(|| invalid("DSTYLE", "missing closing delimiter"))?;
                if item == &X::ControlString("}".into()) {
                    at += 1;
                    break;
                }
                let X::Integer16(code) = item else {
                    return Err(invalid("DSTYLE", "dimvar code must be Integer16"));
                };
                let code = *code;
                if !seen.insert(code) {
                    return Err(invalid(&format!("DSTYLE/{code}"), "duplicate dimvar"));
                }
                at += 1;
                let data = record
                    .values
                    .get(at)
                    .ok_or_else(|| invalid("DSTYLE", "missing dimvar value"))?;
                at += 1;
                match code {
                    3 => {
                        let (p, s) = crate::styles::post(string(code, data)?)?;
                        values.numbers.primary.prefix = Some(p);
                        values.numbers.primary.suffix = Some(s);
                    }
                    40 => values.presentation_scale = Some(real(code, data)?),
                    41 => values.arrows.size = Some(real(code, data)?),
                    42 => {
                        let v = Some(real(code, data)?);
                        values.extension_lines.first.offset = v;
                        values.extension_lines.second.offset = v;
                    }
                    44 => {
                        let v = Some(real(code, data)?);
                        values.extension_lines.first.extension = v;
                        values.extension_lines.second.extension = v;
                    }
                    45 => values.numbers.primary.rounding_increment = Some(real(code, data)?),
                    46 => values.dimension_line.extension = Some(real(code, data)?),
                    73 => {
                        values.text.inside_orientation = Some(if boolean(code, data)? {
                            DimensionTextOrientation::Horizontal
                        } else {
                            DimensionTextOrientation::Aligned
                        })
                    }
                    74 => {
                        values.text.outside_orientation = Some(if boolean(code, data)? {
                            DimensionTextOrientation::Horizontal
                        } else {
                            DimensionTextOrientation::Aligned
                        })
                    }
                    75 => values.extension_lines.first.suppressed = Some(boolean(code, data)?),
                    76 => values.extension_lines.second.suppressed = Some(boolean(code, data)?),
                    77 => {
                        values.text.attachment = Some(match integer(code, data)? {
                            0 => ocdraw::text::MTextAttachment::MiddleCenter,
                            1 => ocdraw::text::MTextAttachment::BottomCenter,
                            _ => {
                                return Err(unsupported(
                                    "DSTYLE/77",
                                    "unqualified vertical placement",
                                ))
                            }
                        })
                    }
                    78 => {
                        values.numbers.primary.zero_suppression =
                            Some(crate::styles::zero_flags(integer(code, data)?)?)
                    }
                    140 => values.text.height = Some(real(code, data)?),
                    141 => {
                        let v = real(code, data)?;
                        values.arrows.center_mark = Some(if v == 0.0 {
                            DimensionCenterMark::None
                        } else if v > 0.0 {
                            DimensionCenterMark::Mark { size: v }
                        } else {
                            DimensionCenterMark::Lines { size: -v }
                        });
                    }
                    142 => values.arrows.tick_size = Some(real(code, data)?),
                    144 => {
                        let v = real(code, data)?;
                        values.numbers.primary.measurement_factor = Some(v.abs());
                        values.numbers.primary.application = Some(if v < 0.0 {
                            DimensionFactorApplication::PaperOnly
                        } else {
                            DimensionFactorApplication::AllOwners
                        });
                    }
                    147 => values.text.gap = Some(real(code, data)?),
                    172 => values.fit.keep_dimension_line_outside = Some(boolean(code, data)?),
                    174 => values.fit.force_text_inside = Some(boolean(code, data)?),
                    271 => {
                        values.numbers.primary.precision = Some(crate::styles::precision(
                            "DSTYLE/271",
                            integer(code, data)?,
                        )?)
                    }
                    277 => {
                        values.numbers.primary.format =
                            Some(crate::styles::distance_format(integer(code, data)?)?)
                    }
                    278 => {
                        values.numbers.primary.decimal_separator = Some(
                            char::from_u32(integer(code, data)? as u32)
                                .ok_or_else(|| invalid("DSTYLE/278", "invalid separator"))?,
                        )
                    }
                    279 => {
                        values.text.movement = Some(match integer(code, data)? {
                            0 => DimensionTextMovement::MoveDimensionLine,
                            1 => DimensionTextMovement::AddLeader,
                            2 => DimensionTextMovement::Free,
                            _ => return Err(unsupported("DSTYLE/279", "unknown movement")),
                        })
                    }
                    281 => values.dimension_line.suppress_first = Some(boolean(code, data)?),
                    282 => values.dimension_line.suppress_second = Some(boolean(code, data)?),
                    289 => {
                        values.fit.preferred_placement = Some(match integer(code, data)? {
                            1 => DimensionFitPlacement::ArrowsInside,
                            2 => DimensionFitPlacement::TextInside,
                            3 => DimensionFitPlacement::BestFit,
                            _ => return Err(unsupported("DSTYLE/289", "unqualified fit policy")),
                        })
                    }
                    340 => values.text.style = Some(handle(code, data)?),
                    _ => {
                        return Err(unsupported(
                            &format!("DSTYLE/{code}"),
                            "dimvar is outside the qualified override grammar",
                        ))
                    }
                }
            }
        }
        if !leftover.is_empty() {
            residual.push(ExtendedDataRecord {
                application_name: record.application_name.clone(),
                values: leftover,
            });
        }
    }
    Ok(CadDimensionOverrides { values, residual })
}
/// Emit only exactly representable native leaf overrides; retain other profile restrictions.
pub fn encode_dimension_overrides(
    v: &CadDimensionStyleOverrides,
    base: &CadDimensionStyleValues,
) -> Result<Option<ExtendedDataRecord>, CadDimensionPreparationError> {
    let mut out = Vec::new();
    let mut pair = |code: i16, data: X| {
        out.push(X::Integer16(code));
        out.push(data);
    };
    if let Some(x) = v.text.height {
        pair(140, X::Real(x));
    }
    if let Some(x) = v.text.gap {
        pair(147, X::Real(x));
    }
    if let Some(x) = v.arrows.size {
        pair(41, X::Real(x));
    }
    if let Some(x) = v.arrows.tick_size {
        pair(142, X::Real(x));
    }
    if let Some(x) = v.presentation_scale {
        pair(40, X::Real(x));
    }
    if let Some(x) = v.dimension_line.extension {
        pair(46, X::Real(x));
    }
    if let Some(x) = v.dimension_line.suppress_first {
        pair(281, X::Integer16(i16::from(x)));
    }
    if let Some(x) = v.dimension_line.suppress_second {
        pair(282, X::Integer16(i16::from(x)));
    }
    if let Some(x) = v.extension_lines.first.suppressed {
        pair(75, X::Integer16(i16::from(x)));
    }
    if let Some(x) = v.extension_lines.second.suppressed {
        pair(76, X::Integer16(i16::from(x)));
    }
    if v.extension_lines.first.offset.is_some() || v.extension_lines.second.offset.is_some() {
        let a = v
            .extension_lines
            .first
            .offset
            .unwrap_or(base.extension_lines.first.offset);
        let b = v
            .extension_lines
            .second
            .offset
            .unwrap_or(base.extension_lines.second.offset);
        if a != b {
            return Err(unsupported(
                "overrides/extensionLines/offset",
                "CAD shares both extension offsets",
            ));
        }
        pair(42, X::Real(a));
    }
    if v.extension_lines.first.extension.is_some() || v.extension_lines.second.extension.is_some() {
        let a = v
            .extension_lines
            .first
            .extension
            .unwrap_or(base.extension_lines.first.extension);
        let b = v
            .extension_lines
            .second
            .extension
            .unwrap_or(base.extension_lines.second.extension);
        if a != b {
            return Err(unsupported(
                "overrides/extensionLines/extension",
                "CAD shares both extension lengths",
            ));
        }
        pair(44, X::Real(a));
    }
    if let Some(x) = v.numbers.primary.precision {
        pair(271, X::Integer16(i16::from(x)));
    }
    if let Some(x) = v.numbers.primary.rounding_increment {
        pair(45, X::Real(x));
    }
    if let Some(x) = v.numbers.primary.decimal_separator {
        let x = i16::try_from(x as u32)
            .map_err(|_| unsupported("overrides/decimalSeparator", "CAD separator is int16"))?;
        pair(278, X::Integer16(x));
    }
    if v.numbers.primary.measurement_factor.is_some() || v.numbers.primary.application.is_some() {
        let factor = v
            .numbers
            .primary
            .measurement_factor
            .unwrap_or(base.numbers.primary.measurement_factor);
        let mode = v
            .numbers
            .primary
            .application
            .unwrap_or(base.numbers.primary.application);
        pair(
            144,
            X::Real(if mode == DimensionFactorApplication::PaperOnly {
                -factor
            } else {
                factor
            }),
        );
    }
    if v.numbers.primary.prefix.is_some() || v.numbers.primary.suffix.is_some() {
        pair(
            3,
            X::String(format!(
                "{}<>{}",
                v.numbers
                    .primary
                    .prefix
                    .as_ref()
                    .unwrap_or(&base.numbers.primary.prefix),
                v.numbers
                    .primary
                    .suffix
                    .as_ref()
                    .unwrap_or(&base.numbers.primary.suffix)
            )),
        );
    }
    if let Some(x) = v.text.style {
        pair(340, X::Handle(x));
    }
    if let Some(x) = v.text.inside_orientation {
        pair(
            73,
            X::Integer16(i16::from(x == DimensionTextOrientation::Horizontal)),
        );
    }
    if let Some(x) = v.text.outside_orientation {
        pair(
            74,
            X::Integer16(i16::from(x == DimensionTextOrientation::Horizontal)),
        );
    }
    if let Some(x) = v.text.attachment {
        pair(
            77,
            X::Integer16(match x {
                ocdraw::text::MTextAttachment::MiddleCenter => 0,
                ocdraw::text::MTextAttachment::BottomCenter => 1,
                _ => {
                    return Err(unsupported(
                        "overrides/text/attachment",
                        "unqualified automatic anchor",
                    ))
                }
            }),
        );
    }
    if let Some(x) = v.text.movement {
        pair(
            279,
            X::Integer16(match x {
                DimensionTextMovement::MoveDimensionLine => 0,
                DimensionTextMovement::AddLeader => 1,
                DimensionTextMovement::Free => 2,
            }),
        );
    }
    if let Some(x) = v.arrows.center_mark {
        pair(
            141,
            X::Real(match x {
                DimensionCenterMark::None => 0.0,
                DimensionCenterMark::Mark { size } => size,
                DimensionCenterMark::Lines { size } => -size,
            }),
        );
    }
    if let Some(x) = v.fit.keep_dimension_line_outside {
        pair(172, X::Integer16(i16::from(x)));
    }
    if let Some(x) = v.fit.force_text_inside {
        pair(174, X::Integer16(i16::from(x)));
    }
    if let Some(x) = v.fit.preferred_placement {
        pair(
            289,
            X::Integer16(match x {
                DimensionFitPlacement::ArrowsInside => 1,
                DimensionFitPlacement::TextInside => 2,
                DimensionFitPlacement::BestFit => 3,
            }),
        );
    }
    if let Some(x) = v.numbers.primary.format {
        pair(
            277,
            X::Integer16(match x {
                DimensionDistanceFormat::Scientific => 1,
                DimensionDistanceFormat::Decimal => 2,
                DimensionDistanceFormat::Engineering => 3,
                DimensionDistanceFormat::Architectural => 4,
                DimensionDistanceFormat::Fractional => 5,
            }),
        );
    }
    if let Some(x) = v.numbers.primary.zero_suppression {
        pair(
            78,
            X::Integer16(
                i16::from(!x.zero_feet)
                    + 2 * i16::from(!x.zero_inches)
                    + 4 * i16::from(x.leading)
                    + 8 * i16::from(x.trailing),
            ),
        );
    }
    if out.is_empty() {
        if v != &CadDimensionStyleOverrides::default() {
            return Err(unsupported(
                "overrides",
                "unmapped native override would be omitted",
            ));
        }
        return Ok(None);
    }
    let mut values = vec![X::String("DSTYLE".into()), X::ControlString("{".into())];
    values.extend(out);
    values.push(X::ControlString("}".into()));
    let record = ExtendedDataRecord {
        application_name: "ACAD".into(),
        values,
    };
    if parse_dimension_overrides(std::slice::from_ref(&record))?.values != *v {
        return Err(unsupported(
            "overrides/dependencies",
            "CAD cannot retain this exact sparse override factoring",
        ));
    }
    Ok(Some(record))
}

/// Moves the CAD per-entity fields out of DSTYLE; returns located dependency/transport losses.
pub fn apply_entity_overrides_to_cad(
    entity: &mut opencadcodec::entities::Dimension,
    overrides: &CadDimensionStyleOverrides,
    base: &CadDimensionStyleValues,
) -> Result<Vec<CadDimensionLoss>, CadDimensionPreparationError> {
    let mut effective = base.clone();
    overrides.apply(&mut effective);
    validate_dimension_settings(&effective).map_err(|e| invalid(&e.field, &e.to_string()))?;
    let mut dstyle = overrides.clone();
    dstyle.arrows.flip_first = None;
    dstyle.arrows.flip_second = None;
    dstyle.text.line_spacing = None;
    let record = encode_dimension_overrides(&dstyle, base)?;
    let target = entity.base_mut();
    target.flip_arrow1 = effective.arrows.flip_first;
    target.flip_arrow2 = effective.arrows.flip_second;
    match effective.text.line_spacing {
        DimensionLineSpacing::AtLeast { factor } => {
            target.line_spacing_style = 1;
            target.line_spacing_factor = factor
        }
        DimensionLineSpacing::Exact { factor } => {
            target.line_spacing_style = 2;
            target.line_spacing_factor = factor
        }
    }
    if let Some(record) = record {
        target.common.extended_data.add_record(record)
    }
    let mut losses = Vec::new();
    if overrides.arrows.flip_first.is_none()
        || overrides.arrows.flip_second.is_none()
        || overrides.text.line_spacing.is_none()
    {
        losses.push(CadDimensionLoss{field:"overrides/perEntityDependencies".into(),detail:"CAD stores arrow flips/line spacing per entity; inherited native dependencies materialized".into()});
    }
    if target.flip_arrow1 || target.flip_arrow2 {
        losses.push(CadDimensionLoss{field:"arrows/flips".into(),detail:"pinned DIMENSION DXF does not emit arrow flip fields; DWG transport must be assessed separately".into()});
    }
    if target.line_spacing_factor != 1.0 && (target.line_spacing_factor - 1.0).abs() <= 1e-10 {
        losses.push(CadDimensionLoss {
            field: "text/lineSpacing".into(),
            detail: "pinned DXF suppresses this nondefault factor using an epsilon".into(),
        });
    }
    Ok(losses)
}
