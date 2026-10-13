use crate::*;
use ocdraw::{dimension::*, text::MTextAttachment};
use opencadcodec::{tables::DimStyle, CadDocument, Color, Handle};

pub struct PreparedCadDimensionStyle {
    pub values: CadDimensionStyleValues,
    pub losses: Vec<CadDimensionLoss>,
}
pub struct PreparedCadDimensionTargetStyle {
    pub style: DimStyle,
    pub losses: Vec<CadDimensionLoss>,
}
pub(crate) fn distance_format(
    value: i16,
) -> Result<DimensionDistanceFormat, CadDimensionPreparationError> {
    Ok(match value {
        1 => DimensionDistanceFormat::Scientific,
        2 => DimensionDistanceFormat::Decimal,
        3 => DimensionDistanceFormat::Engineering,
        4 => DimensionDistanceFormat::Architectural,
        5 => DimensionDistanceFormat::Fractional,
        _ => return Err(unsupported("DIMLUNIT", "unqualified linear number format")),
    })
}
pub(crate) fn precision(field: &str, value: i16) -> Result<u8, CadDimensionPreparationError> {
    u8::try_from(value)
        .ok()
        .filter(|v| *v <= 16)
        .ok_or_else(|| invalid(field, "precision must be 0..16"))
}
pub(crate) fn zero_flags(v: i16) -> Result<DimensionZeroSuppression, CadDimensionPreparationError> {
    if !(0..=15).contains(&v) {
        return Err(unsupported("DIMZIN", "unknown zero-suppression bits"));
    }
    Ok(DimensionZeroSuppression {
        leading: v & 4 != 0,
        trailing: v & 8 != 0,
        zero_feet: v & 1 == 0,
        zero_inches: v & 2 == 0,
    })
}
pub(crate) fn post(value: &str) -> Result<(String, String), CadDimensionPreparationError> {
    if value.is_empty() {
        return Ok((String::new(), String::new()));
    }
    let Some((prefix, suffix)) = value.split_once("<>") else {
        return Err(unsupported(
            "DIMPOST",
            "nonempty format without a measurement token",
        ));
    };
    if suffix.contains("<>") || unqualified_text(value) {
        return Err(unsupported(
            "DIMPOST",
            "unqualified repeated token, field or markup",
        ));
    }
    Ok((prefix.into(), suffix.into()))
}
pub(crate) fn unqualified_text(value: &str) -> bool {
    value.contains("%<")
        || value.contains("%%")
        || value.chars().any(|c| matches!(c, '\\' | '{' | '}' | '^'))
}
fn color(
    index: i16,
    rgb: Option<Color>,
) -> Result<DimensionAppearanceChoice<CadColorValue>, CadDimensionPreparationError> {
    if let Some(c) = rgb {
        return cad_presentation_convert::explicit_color_from_cad(c)
            .map(DimensionAppearanceChoice::Explicit)
            .map_err(|_| unsupported("style/color", "unqualified true color"));
    }
    Ok(match index {
        0 => DimensionAppearanceChoice::InheritEntity,
        256 => DimensionAppearanceChoice::ByLayer,
        1..=255 => DimensionAppearanceChoice::Explicit(
            cad_presentation_convert::explicit_color_from_cad(Color::Index(index as u8))
                .map_err(|_| unsupported("style/color", "invalid indexed color"))?,
        ),
        _ => return Err(unsupported("style/color", "unqualified color index")),
    })
}
fn weight(v: i16) -> Result<DimensionAppearanceChoice<f64>, CadDimensionPreparationError> {
    Ok(match v {
        -2 => DimensionAppearanceChoice::InheritEntity,
        -1 => DimensionAppearanceChoice::ByLayer,
        -3 => {
            return Err(unsupported(
                "style/lineWeight",
                "drawing-default weight dependency",
            ))
        }
        0..=211 => DimensionAppearanceChoice::Explicit(f64::from(v) / 100.0),
        _ => return Err(invalid("style/lineWeight", "invalid weight")),
    })
}
fn marker(
    handle: Handle,
    name: &str,
    doc: &CadDocument,
) -> Result<DimensionMarker<Handle>, CadDimensionPreparationError> {
    if handle != Handle::NULL {
        return Ok(DimensionMarker::LocalBlock { block: handle });
    }
    if name.is_empty() {
        return Ok(DimensionMarker::ClosedFilled);
    }
    if let Some(block) = doc.block_records.get(name) {
        return Ok(DimensionMarker::LocalBlock {
            block: block.handle,
        });
    }
    Err(unsupported("style/arrow", "unresolved legacy arrow name"))
}
pub fn prepare_style_from_cad(
    s: &DimStyle,
    doc: &CadDocument,
) -> Result<PreparedCadDimensionStyle, CadDimensionPreparationError> {
    if s.dimscale == 0.0 {
        return Err(unsupported(
            "DIMSCALE",
            "automatic viewport/Paper scaling context is not qualified",
        ));
    }
    if s.annotative {
        return Err(unsupported(
            "style/annotative",
            "annotation scale contexts are a later slice",
        ));
    }
    if s.xref_reference || s.xref_dependent || s.xref_handle != Handle::NULL {
        return Err(unsupported("style/xref", "external style dependencies"));
    }
    if s.dimjust != 0 || !(0..=1).contains(&s.dimtad) || s.dimtvp != 0.0 {
        return Err(unsupported(
            "style/textPlacement",
            "unqualified justification/vertical offset profile",
        ));
    }
    if s.dimtxtdirection {
        return Err(unsupported(
            "style/textDirection",
            "reversed text direction",
        ));
    }
    if s.dimsoxd {
        return Err(unsupported(
            "DIMSOXD",
            "outside arrow suppression has no native fit field",
        ));
    }
    if matches!(s.dimlunit, 4 | 5) && s.dimfrac != 2 {
        return Err(unsupported(
            "DIMFRAC",
            "stacked fraction layout is outside the plain number formatter",
        ));
    }
    if s.dimmzf != 0.0
        || !s.dimmzs.is_empty()
        || (s.dimalt && (s.dimaltmzf != 0.0 || !s.dimaltmzs.is_empty()))
    {
        return Err(unsupported(
            "DIMMZF/DIMMZS",
            "zero-measurement substitution is not represented",
        ));
    }
    if (s.dimtol || s.dimlim) && (s.dimtfac != 1.0 || s.dimtolj != 0 || s.dimtzin != 0 || s.dimalt)
    {
        return Err(unsupported("style/toleranceLayout","tolerance scale, alignment, suppression or alternate tolerance formatting is unqualified"));
    }
    let text = if s.dimtxsty_handle != Handle::NULL {
        s.dimtxsty_handle
    } else {
        doc.text_styles
            .get(&s.dimtxsty)
            .map(|t| t.handle)
            .ok_or_else(|| invalid("DIMTXSTY", "unresolved text style"))?
    };
    let font = doc
        .text_styles
        .iter()
        .find(|t| t.handle == text)
        .ok_or_else(|| invalid("DIMTXSTY", "dangling text style handle"))?;
    if s.dimtxsty_handle != Handle::NULL
        && !s.dimtxsty.is_empty()
        && !font.name.eq_ignore_ascii_case(&s.dimtxsty)
    {
        return Err(invalid("DIMTXSTY", "text style name and handle disagree"));
    }
    if !font.height.is_finite() || font.height < 0.0 {
        return Err(invalid("DIMTXSTY/height", "invalid font height"));
    }
    if font.height != 0.0 {
        return Err(unsupported("DIMTXSTY/fixedHeight","fixed font height overrides dimension height; dependency materialization is unqualified"));
    }
    let mut v = CadDimensionStyleValues::new(text);
    v.presentation_scale = s.dimscale;
    v.dimension_line.extension = s.dimdle;
    v.dimension_line.suppress_first = s.dimsd1;
    v.dimension_line.suppress_second = s.dimsd2;
    v.dimension_line.appearance.color = color(s.dimclrd, s.dimclrd_true_color)?;
    v.dimension_line.appearance.line_weight = weight(s.dimlwd)?;
    if s.dimltex_handle != Handle::NULL {
        v.dimension_line.appearance.line_pattern =
            DimensionAppearanceChoice::Explicit(s.dimltex_handle);
    }
    for (first, e) in [
        (true, &mut v.extension_lines.first),
        (false, &mut v.extension_lines.second),
    ] {
        e.offset = s.dimexo;
        e.extension = s.dimexe;
        e.suppressed = if first { s.dimse1 } else { s.dimse2 };
        e.length = if s.dimfxlon {
            DimensionExtensionLength::Fixed { length: s.dimfxl }
        } else {
            DimensionExtensionLength::Automatic
        };
        e.appearance.color = color(s.dimclre, s.dimclre_true_color)?;
        e.appearance.line_weight = weight(s.dimlwe)?;
        let pattern = if first {
            s.dimltex1_handle
        } else {
            s.dimltex2_handle
        };
        if pattern != Handle::NULL {
            e.appearance.line_pattern = DimensionAppearanceChoice::Explicit(pattern);
        }
    }
    v.arrows.first = marker(
        if s.dimsah { s.dimblk1 } else { s.dimblk },
        if s.dimsah {
            &s.dimblk1_name
        } else {
            &s.dimblk_name
        },
        doc,
    )?;
    v.arrows.second = marker(
        if s.dimsah { s.dimblk2 } else { s.dimblk },
        if s.dimsah {
            &s.dimblk2_name
        } else {
            &s.dimblk_name
        },
        doc,
    )?;
    v.arrows.size = s.dimasz;
    v.arrows.tick_size = s.dimtsz;
    v.arrows.center_mark = if s.dimcen == 0.0 {
        DimensionCenterMark::None
    } else if s.dimcen > 0.0 {
        DimensionCenterMark::Mark { size: s.dimcen }
    } else {
        DimensionCenterMark::Lines { size: -s.dimcen }
    };
    v.text.height = s.dimtxt;
    v.text.gap = s.dimgap;
    v.text.attachment = if s.dimtad == 0 {
        MTextAttachment::MiddleCenter
    } else {
        MTextAttachment::BottomCenter
    };
    v.text.inside_orientation = if s.dimtih {
        DimensionTextOrientation::Horizontal
    } else {
        DimensionTextOrientation::Aligned
    };
    v.text.outside_orientation = if s.dimtoh {
        DimensionTextOrientation::Horizontal
    } else {
        DimensionTextOrientation::Aligned
    };
    v.text.movement = match s.dimtmove {
        0 => DimensionTextMovement::MoveDimensionLine,
        1 => DimensionTextMovement::AddLeader,
        2 => DimensionTextMovement::Free,
        _ => return Err(unsupported("DIMTMOVE", "unknown movement")),
    };
    v.text.appearance.color = color(s.dimclrt, s.dimclrt_true_color)?;
    v.text.background = match s.dimtfill {
        0 => DimensionBackground::None,
        1 => DimensionBackground::Canvas {
            opacity: 1.0,
            padding: DimensionPadding::Relative { factor: 0.0 },
        },
        2 => {
            let DimensionAppearanceChoice::Explicit(c) =
                color(s.dimtfillclr, s.dimtfillclr_true_color)?
            else {
                return Err(unsupported("DIMTFILLCLR", "inherited background color"));
            };
            DimensionBackground::Color {
                color: c,
                opacity: 1.0,
                padding: DimensionPadding::Relative { factor: 0.0 },
            }
        }
        _ => return Err(unsupported("DIMTFILL", "unknown background")),
    };
    v.fit.preferred_placement = match s.dimatfit {
        1 => DimensionFitPlacement::ArrowsInside,
        2 => DimensionFitPlacement::TextInside,
        3 => DimensionFitPlacement::BestFit,
        _ => return Err(unsupported("DIMATFIT", "unqualified fit policy")),
    };
    v.fit.force_text_inside = s.dimtix;
    v.fit.keep_dimension_line_outside = s.dimtofl;
    v.numbers.primary.measurement_factor = s.dimlfac.abs();
    v.numbers.primary.application = if s.dimlfac < 0.0 {
        DimensionFactorApplication::PaperOnly
    } else {
        DimensionFactorApplication::AllOwners
    };
    v.numbers.primary.format = distance_format(s.dimlunit)?;
    v.numbers.primary.precision = precision("DIMDEC", s.dimdec)?;
    v.numbers.primary.decimal_separator =
        char::from_u32(s.dimdsep as u32).ok_or_else(|| invalid("DIMDSEP", "invalid separator"))?;
    v.numbers.primary.rounding_increment = s.dimrnd;
    v.numbers.primary.zero_suppression = zero_flags(s.dimzin)?;
    (v.numbers.primary.prefix, v.numbers.primary.suffix) = post(&s.dimpost)?;
    v.numbers.primary.angle_format = match s.dimaunit {
        0 => DimensionAngleFormat::DecimalDegrees,
        1 => DimensionAngleFormat::DegreesMinutesSeconds,
        2 => DimensionAngleFormat::Grads,
        3 => DimensionAngleFormat::Radians,
        _ => return Err(unsupported("DIMAUNIT", "survey angle format")),
    };
    v.numbers.primary.arc_symbol = match s.dimarcsym {
        0 => DimensionArcSymbol::Before,
        1 => DimensionArcSymbol::Above,
        2 => DimensionArcSymbol::None,
        _ => return Err(unsupported("DIMARCSYM", "unknown arc symbol mode")),
    };
    if s.dimalt {
        let (p, suffix) = post(&s.dimapost)?;
        v.numbers.alternate = Some(DimensionAlternateNumberSettings {
            factor: s.dimaltf,
            format: distance_format(s.dimaltu)?,
            precision: precision("DIMALTD", s.dimaltd)?,
            decimal_separator: v.numbers.primary.decimal_separator,
            rounding_increment: s.dimaltrnd,
            zero_suppression: zero_flags(s.dimaltz)?,
            prefix: p,
            suffix,
        });
    }
    if s.dimtol && s.dimlim {
        return Err(invalid("style/tolerance", "conflicting modes"));
    }
    v.numbers.tolerance = if s.dimlim {
        DimensionToleranceSettings::Limits {
            upper: s.dimtp,
            lower: s.dimtm,
            precision: precision("DIMTDEC", s.dimtdec)?,
        }
    } else if s.dimtol {
        DimensionToleranceSettings::Deviations {
            upper: s.dimtp,
            lower: s.dimtm,
            precision: precision("DIMTDEC", s.dimtdec)?,
        }
    } else {
        DimensionToleranceSettings::None
    };
    validate_dimension_settings(&v).map_err(|e| invalid(&e.field, &e.to_string()))?;
    let mut losses=vec![CadDimensionLoss{field:"DIMDLI".into(),detail:"continuation spacing is CAD creation state, not stored in a native independent dimension".into()}];
    for field in [
        "xref_resolved",
        "DIMUPT",
        "DIMFIT",
        "DIMUNIT",
        "DIMLDRBLK",
        "DIMJOGANG",
        "DIMADEC",
        "DIMAZIN",
    ] {
        losses.push(CadDimensionLoss{field:field.into(),detail:"editing, legacy, leader/radius or angular-only state omitted in the linear/aligned profile".into()});
    }
    if !s.dimfxlon {
        losses.push(CadDimensionLoss {
            field: "DIMFXL".into(),
            detail: "inactive fixed extension length omitted".into(),
        });
    }
    if !s.dimalt {
        losses.push(CadDimensionLoss {
            field: "alternate/inactive".into(),
            detail: "inactive alternate number settings omitted".into(),
        });
    }
    if !(s.dimtol || s.dimlim) {
        losses.push(CadDimensionLoss {
            field: "tolerance/inactive".into(),
            detail: "inactive tolerance values and layout omitted".into(),
        });
    }
    if s.dimsah {
        losses.push(CadDimensionLoss {
            field: "DIMBLK".into(),
            detail: "inactive shared marker choice omitted".into(),
        });
    } else {
        losses.push(CadDimensionLoss {
            field: "DIMBLK1/DIMBLK2".into(),
            detail: "inactive separate marker choices omitted".into(),
        });
    }
    if s.dimtfill != 2 {
        losses.push(CadDimensionLoss {
            field: "DIMTFILLCLR".into(),
            detail: "inactive explicit background color omitted".into(),
        });
    }
    if !matches!(s.dimlunit, 4 | 5) {
        losses.push(CadDimensionLoss {
            field: "DIMFRAC".into(),
            detail: "inactive fraction layout choice omitted".into(),
        });
    }
    Ok(PreparedCadDimensionStyle { values: v, losses })
}

fn target_color(
    v: &DimensionAppearanceChoice<CadColorValue>,
) -> Result<(i16, Option<Color>), CadDimensionPreparationError> {
    Ok(match v {
        DimensionAppearanceChoice::InheritEntity => (0, None),
        DimensionAppearanceChoice::ByLayer => (256, None),
        DimensionAppearanceChoice::ByBlock => {
            return Err(unsupported(
                "component/color",
                "outer occurrence ByBlock cannot be represented independently of entity color",
            ))
        }
        DimensionAppearanceChoice::Explicit(v) => {
            let c = cad_presentation_convert::color_to_cad(v)
                .map_err(|_| invalid("component/color", "invalid color"))?;
            if !c.losses.is_empty() || c.named.is_some() {
                return Err(unsupported(
                    "component/color",
                    "unqualified indexed/named color",
                ));
            }
            match c.color {
                Color::Index(v) => (i16::from(v), None),
                c => (0, Some(c)),
            }
        }
    })
}
fn target_weight(v: &DimensionAppearanceChoice<f64>) -> Result<i16, CadDimensionPreparationError> {
    Ok(match v {
        DimensionAppearanceChoice::InheritEntity => -2,
        DimensionAppearanceChoice::ByLayer => -1,
        DimensionAppearanceChoice::ByBlock => {
            return Err(unsupported(
                "component/lineWeight",
                "outer occurrence ByBlock",
            ))
        }
        DimensionAppearanceChoice::Explicit(v) => {
            let q = cad_presentation_convert::lineweight_to_cad(*v)
                .map_err(|_| invalid("component/lineWeight", "invalid weight"))?;
            if q.changed {
                return Err(unsupported(
                    "component/lineWeight",
                    "target quantization would change weight",
                ));
            }
            (q.roundtrip_mm * 100.0).round() as i16
        }
    })
}
fn target_pattern(
    v: &DimensionAppearanceChoice<Handle>,
) -> Result<Handle, CadDimensionPreparationError> {
    match v {
        DimensionAppearanceChoice::InheritEntity => Ok(Handle::NULL),
        DimensionAppearanceChoice::Explicit(h) if !h.is_null() => Ok(*h),
        DimensionAppearanceChoice::Explicit(_) => Err(invalid(
            "component/linePattern",
            "null explicit pattern reference",
        )),
        _ => Err(unsupported(
            "component/linePattern",
            "unqualified inherited pattern mode",
        )),
    }
}
fn target_marker(v: &DimensionMarker<Handle>) -> Result<Handle, CadDimensionPreparationError> {
    match v {
        DimensionMarker::ClosedFilled => Ok(Handle::NULL),
        DimensionMarker::LocalBlock { block } if !block.is_null() => Ok(*block),
        DimensionMarker::LocalBlock { .. } => Err(invalid("arrows/marker", "null block reference")),
        _ => Err(unsupported(
            "arrows/marker",
            "this built-in marker needs a qualified CAD block definition",
        )),
    }
}
fn format_code(v: DimensionDistanceFormat) -> i16 {
    match v {
        DimensionDistanceFormat::Scientific => 1,
        DimensionDistanceFormat::Decimal => 2,
        DimensionDistanceFormat::Engineering => 3,
        DimensionDistanceFormat::Architectural => 4,
        DimensionDistanceFormat::Fractional => 5,
    }
}
fn zero_code(v: DimensionZeroSuppression) -> i16 {
    i16::from(!v.zero_feet)
        + 2 * i16::from(!v.zero_inches)
        + 4 * i16::from(v.leading)
        + 8 * i16::from(v.trailing)
}
pub fn prepare_style_to_cad(
    name: &str,
    v: &CadDimensionStyleValues,
) -> Result<PreparedCadDimensionTargetStyle, CadDimensionPreparationError> {
    validate_dimension_settings(v).map_err(|e| invalid(&e.field, &e.to_string()))?;
    if v.text.style.is_null() {
        return Err(invalid("text/style", "null target TextStyle reference"));
    }
    if matches!(v.arrows.center_mark,DimensionCenterMark::Mark{size}|DimensionCenterMark::Lines{size} if size==0.0)
    {
        return Err(unsupported(
            "arrows/centerMark",
            "zero-sized Mark/Lines choice would become None in CAD",
        ));
    }
    if v.extension_lines.first.offset != v.extension_lines.second.offset
        || v.extension_lines.first.extension != v.extension_lines.second.extension
        || v.extension_lines.first.length != v.extension_lines.second.length
    {
        return Err(unsupported(
            "extensionLines",
            "CAD shares offset, extension and fixed length across both lines",
        ));
    }
    if v.extension_lines.first.appearance.color != v.extension_lines.second.appearance.color
        || v.extension_lines.first.appearance.line_weight
            != v.extension_lines.second.appearance.line_weight
    {
        return Err(unsupported(
            "extensionLines/appearance",
            "CAD shares extension color and weight",
        ));
    }
    for a in [
        &v.dimension_line.appearance,
        &v.extension_lines.first.appearance,
        &v.extension_lines.second.appearance,
        &v.text.appearance,
    ] {
        if a.opacity != DimensionAppearanceChoice::InheritEntity {
            return Err(unsupported(
                "component/opacity",
                "CAD has no independent component opacity",
            ));
        }
    }
    let mut s = DimStyle::new(name);
    s.dimscale = v.presentation_scale;
    s.dimdle = v.dimension_line.extension;
    s.dimsd1 = v.dimension_line.suppress_first;
    s.dimsd2 = v.dimension_line.suppress_second;
    (s.dimclrd, s.dimclrd_true_color) = target_color(&v.dimension_line.appearance.color)?;
    s.dimlwd = target_weight(&v.dimension_line.appearance.line_weight)?;
    s.dimltex_handle = target_pattern(&v.dimension_line.appearance.line_pattern)?;
    s.dimexo = v.extension_lines.first.offset;
    s.dimexe = v.extension_lines.first.extension;
    s.dimse1 = v.extension_lines.first.suppressed;
    s.dimse2 = v.extension_lines.second.suppressed;
    (s.dimclre, s.dimclre_true_color) = target_color(&v.extension_lines.first.appearance.color)?;
    s.dimlwe = target_weight(&v.extension_lines.first.appearance.line_weight)?;
    s.dimltex1_handle = target_pattern(&v.extension_lines.first.appearance.line_pattern)?;
    s.dimltex2_handle = target_pattern(&v.extension_lines.second.appearance.line_pattern)?;
    match v.extension_lines.first.length {
        DimensionExtensionLength::Automatic => s.dimfxlon = false,
        DimensionExtensionLength::Fixed { length } => {
            s.dimfxlon = true;
            s.dimfxl = length
        }
    }
    s.dimasz = v.arrows.size;
    s.dimtsz = v.arrows.tick_size;
    s.dimsah = true;
    s.dimblk1 = target_marker(&v.arrows.first)?;
    s.dimblk2 = target_marker(&v.arrows.second)?;
    s.dimcen = match v.arrows.center_mark {
        DimensionCenterMark::None => 0.0,
        DimensionCenterMark::Mark { size } => size,
        DimensionCenterMark::Lines { size } => -size,
    };
    s.dimtxsty_handle = v.text.style;
    s.dimtxt = v.text.height;
    s.dimgap = v.text.gap;
    s.dimtad = match v.text.attachment {
        MTextAttachment::MiddleCenter => 0,
        MTextAttachment::BottomCenter => 1,
        _ => {
            return Err(unsupported(
                "text/attachment",
                "unqualified automatic text anchor",
            ))
        }
    };
    s.dimtih = v.text.inside_orientation == DimensionTextOrientation::Horizontal;
    s.dimtoh = v.text.outside_orientation == DimensionTextOrientation::Horizontal;
    s.dimtmove = match v.text.movement {
        DimensionTextMovement::MoveDimensionLine => 0,
        DimensionTextMovement::AddLeader => 1,
        DimensionTextMovement::Free => 2,
    };
    (s.dimclrt, s.dimclrt_true_color) = target_color(&v.text.appearance.color)?;
    if v.text.appearance.line_pattern != DimensionAppearanceChoice::InheritEntity
        || v.text.appearance.line_weight != DimensionAppearanceChoice::InheritEntity
    {
        return Err(unsupported(
            "text/appearance",
            "text stroke pattern/weight cannot be set independently in CAD",
        ));
    }
    match &v.text.background {
        DimensionBackground::None => s.dimtfill = 0,
        DimensionBackground::Canvas {
            opacity: 1.0,
            padding: DimensionPadding::Relative { factor: 0.0 },
        } => s.dimtfill = 1,
        DimensionBackground::Color {
            color,
            opacity: 1.0,
            padding: DimensionPadding::Relative { factor: 0.0 },
        } => {
            s.dimtfill = 2;
            (s.dimtfillclr, s.dimtfillclr_true_color) =
                target_color(&DimensionAppearanceChoice::Explicit(color.clone()))?;
        }
        _ => {
            return Err(unsupported(
                "text/background",
                "unqualified transparency or padding",
            ))
        }
    }
    s.dimatfit = match v.fit.preferred_placement {
        DimensionFitPlacement::ArrowsInside => 1,
        DimensionFitPlacement::TextInside => 2,
        DimensionFitPlacement::BestFit => 3,
    };
    s.dimtix = v.fit.force_text_inside;
    s.dimtofl = v.fit.keep_dimension_line_outside;
    let p = &v.numbers.primary;
    s.dimlfac = if p.application == DimensionFactorApplication::PaperOnly {
        -p.measurement_factor
    } else {
        p.measurement_factor
    };
    s.dimlunit = format_code(p.format);
    s.dimdec = i16::from(p.precision);
    s.dimdsep = i16::try_from(p.decimal_separator as u32)
        .map_err(|_| unsupported("decimalSeparator", "CAD int16 separator"))?;
    s.dimrnd = p.rounding_increment;
    s.dimzin = zero_code(p.zero_suppression);
    if p.prefix.contains("<>")
        || p.suffix.contains("<>")
        || p.prefix.contains('\\')
        || p.suffix.contains('\\')
    {
        return Err(unsupported(
            "numbers/prefixSuffix",
            "token-ambiguous CAD format",
        ));
    }
    s.dimpost = format!("{}<>{}", p.prefix, p.suffix);
    s.dimaunit = match p.angle_format {
        DimensionAngleFormat::DecimalDegrees => 0,
        DimensionAngleFormat::DegreesMinutesSeconds => 1,
        DimensionAngleFormat::Grads => 2,
        DimensionAngleFormat::Radians => 3,
    };
    s.dimfrac = 2;
    s.dimarcsym = match p.arc_symbol {
        DimensionArcSymbol::Before => 0,
        DimensionArcSymbol::Above => 1,
        DimensionArcSymbol::None => 2,
    };
    if let Some(a) = &v.numbers.alternate {
        s.dimalt = true;
        s.dimaltf = a.factor;
        s.dimaltu = format_code(a.format);
        s.dimaltd = i16::from(a.precision);
        s.dimaltrnd = a.rounding_increment;
        s.dimaltz = zero_code(a.zero_suppression);
        s.dimapost = format!("{}<>{}", a.prefix, a.suffix);
        if a.decimal_separator != p.decimal_separator {
            return Err(unsupported(
                "alternate/decimalSeparator",
                "CAD shares the separator",
            ));
        }
    } else {
        s.dimalt = false
    }
    post(&s.dimpost)?;
    if s.dimalt {
        post(&s.dimapost)?;
    }
    match v.numbers.tolerance {
        DimensionToleranceSettings::None => {
            s.dimtol = false;
            s.dimlim = false
        }
        DimensionToleranceSettings::Deviations {
            upper,
            lower,
            precision,
        } => {
            s.dimtol = true;
            s.dimlim = false;
            s.dimtp = upper;
            s.dimtm = lower;
            s.dimtdec = i16::from(precision)
        }
        DimensionToleranceSettings::Limits {
            upper,
            lower,
            precision,
        } => {
            s.dimtol = false;
            s.dimlim = true;
            s.dimtp = upper;
            s.dimtm = lower;
            s.dimtdec = i16::from(precision)
        }
    }
    s.dimtzin = 0;
    if s.dimalt && (s.dimtol || s.dimlim) {
        return Err(unsupported(
            "numbers/tolerance",
            "alternate tolerance formatting is not qualified",
        ));
    }
    let mut losses = Vec::new();
    if s.dimclrd_true_color.is_some()
        || s.dimclre_true_color.is_some()
        || s.dimclrt_true_color.is_some()
        || s.dimtfillclr_true_color.is_some()
    {
        losses.push(CadDimensionLoss {
            field: "style/trueColor".into(),
            detail: "pinned DIMSTYLE DXF does not transport DWG-only true-color fields".into(),
        });
    }
    if v.text.line_spacing != (DimensionLineSpacing::AtLeast { factor: 1.0 }) {
        losses.push(CadDimensionLoss {
            field: "text/lineSpacing".into(),
            detail: "CAD stores line spacing on entities; style dependency must be materialized"
                .into(),
        });
    }
    if v.arrows.flip_first || v.arrows.flip_second {
        losses.push(CadDimensionLoss {
            field: "arrows/flips".into(),
            detail: "CAD stores arrow flips on entities; style dependency must be materialized"
                .into(),
        });
    }
    Ok(PreparedCadDimensionTargetStyle { style: s, losses })
}
