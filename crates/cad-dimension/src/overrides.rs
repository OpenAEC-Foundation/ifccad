use crate::*;
use ocdraw::dimension::*;
use opencadcodec::xdata::{ExtendedDataRecord, XDataValue as X};
pub struct CadDimensionOverrides {
    pub values: CadDimensionStyleOverrides,
    pub effective: CadDimensionStyleValues,
    pub residual: Vec<ExtendedDataRecord>,
    pub losses: Vec<CadDimensionLoss>,
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
    if let Some(layout) = v.numbers.primary.fraction_layout {
        pair(
            276,
            X::Integer16(match layout {
                DimensionFractionLayout::Horizontal => 0,
                DimensionFractionLayout::Diagonal => 1,
                DimensionFractionLayout::Inline => 2,
            }),
        );
    }
    if let Some(scale) = v.numbers.primary.fraction_scale {
        if base.numbers.alternate.is_some()
            && base
                .numbers
                .alternate
                .as_ref()
                .is_some_and(|a| a.fraction_scale != scale)
        {
            return Err(unsupported(
                "overrides/fractionScale",
                "CAD shares alternate fraction scale",
            ));
        }
        if !matches!(base.numbers.tolerance, DimensionToleranceSettings::None) && scale != 1.0 {
            return Err(unsupported(
                "overrides/fractionScale",
                "CAD couples fraction and tolerance scale",
            ));
        }
        pair(146, X::Real(scale));
    }
    if v.extension_lines.first.length.is_some() || v.extension_lines.second.length.is_some() {
        let first = v
            .extension_lines
            .first
            .length
            .unwrap_or(base.extension_lines.first.length);
        let second = v
            .extension_lines
            .second
            .length
            .unwrap_or(base.extension_lines.second.length);
        if first != second {
            return Err(unsupported(
                "overrides/extensionLines/length",
                "CAD shares fixed extension length",
            ));
        }
        match first {
            DimensionExtensionLength::Automatic => pair(290, X::Integer16(0)),
            DimensionExtensionLength::Fixed { length } => {
                pair(49, X::Real(length));
                pair(290, X::Integer16(1));
            }
        }
    }
    for (code, color) in [
        (176, v.dimension_line.appearance.color.as_ref()),
        (178, v.text.appearance.color.as_ref()),
    ] {
        if let Some(color) = color {
            let (index, rgb) = crate::styles::target_color(color)?;
            if rgb.is_some() {
                return Err(unsupported(
                    "overrides/color",
                    "DSTYLE ACI override cannot preserve true color",
                ));
            }
            pair(code, X::Integer16(index));
        }
    }
    if v.extension_lines.first.appearance.color.is_some()
        || v.extension_lines.second.appearance.color.is_some()
    {
        let first = v
            .extension_lines
            .first
            .appearance
            .color
            .as_ref()
            .unwrap_or(&base.extension_lines.first.appearance.color);
        let second = v
            .extension_lines
            .second
            .appearance
            .color
            .as_ref()
            .unwrap_or(&base.extension_lines.second.appearance.color);
        if first != second {
            return Err(unsupported(
                "overrides/extensionLines/color",
                "CAD shares extension color",
            ));
        }
        let (index, rgb) = crate::styles::target_color(first)?;
        if rgb.is_some() {
            return Err(unsupported(
                "overrides/color",
                "DSTYLE ACI override cannot preserve true color",
            ));
        }
        pair(177, X::Integer16(index));
    }
    if v.arrows.first.is_some() || v.arrows.second.is_some() {
        pair(173, X::Integer16(1));
        if let Some(marker) = &v.arrows.first {
            pair(343, X::Handle(crate::styles::target_marker(marker)?));
        }
        if let Some(marker) = &v.arrows.second {
            pair(344, X::Handle(crate::styles::target_marker(marker)?));
        }
    }
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
    let parsed = parse_dimension_overrides(std::slice::from_ref(&record))?;
    let mut cad_base = prepare_style_to_cad("Override coverage", base)?.style;
    for entry in &parsed.entries {
        entry.apply_to(&mut cad_base);
    }
    let mut effective = base.clone();
    v.apply(&mut effective);
    if crate::source_overrides::project_source_overrides(&parsed.entries, &cad_base, &effective)
        != *v
    {
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
