//! Bounded source DSTYLE grammar and context-dependent preparation.
use crate::*;
use ifccad::dimension::*;
use opencadcodec::{
    tables::DimStyle,
    xdata::{ExtendedDataRecord, XDataValue as X},
    CadDocument, Handle,
};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq)]
pub enum CadDimensionSourceOverride {
    Dimpost(String),
    Dimscale(f64),
    Dimasz(f64),
    Dimexo(f64),
    Dimexe(f64),
    Dimrnd(f64),
    Dimdle(f64),
    Dimfxl(f64),
    Dimtih(bool),
    Dimtoh(bool),
    Dimse1(bool),
    Dimse2(bool),
    Dimtad(i16),
    Dimzin(i16),
    Dimtxt(f64),
    Dimcen(f64),
    Dimtsz(f64),
    Dimlfac(f64),
    Dimtfac(f64),
    Dimgap(f64),
    Dimtofl(bool),
    Dimsah(bool),
    Dimtix(bool),
    Dimclrd(i16),
    Dimclre(i16),
    Dimclrt(i16),
    Dimdec(i16),
    Dimfrac(i16),
    Dimlunit(i16),
    Dimdsep(i16),
    Dimtmove(i16),
    Dimsd1(bool),
    Dimsd2(bool),
    Dimatfit(i16),
    Dimfxlon(bool),
    Dimtxsty(Handle),
    Dimblk(Handle),
    Dimblk1(Handle),
    Dimblk2(Handle),
}
pub struct CadDimensionParsedOverrides {
    pub entries: Vec<CadDimensionSourceOverride>,
    pub residual: Vec<ExtendedDataRecord>,
}
impl CadDimensionSourceOverride {
    pub(crate) fn apply_to(&self, source: &mut DimStyle) {
        match self {
            Self::Dimpost(value) => {
                source.dimpost = value.clone();
            }
            Self::Dimscale(value) => {
                source.dimscale = *value;
            }
            Self::Dimasz(value) => {
                source.dimasz = *value;
            }
            Self::Dimexo(value) => {
                source.dimexo = *value;
            }
            Self::Dimexe(value) => {
                source.dimexe = *value;
            }
            Self::Dimrnd(value) => {
                source.dimrnd = *value;
            }
            Self::Dimdle(value) => {
                source.dimdle = *value;
            }
            Self::Dimfxl(value) => {
                source.dimfxl = *value;
            }
            Self::Dimtih(value) => {
                source.dimtih = *value;
            }
            Self::Dimtoh(value) => {
                source.dimtoh = *value;
            }
            Self::Dimse1(value) => {
                source.dimse1 = *value;
            }
            Self::Dimse2(value) => {
                source.dimse2 = *value;
            }
            Self::Dimtad(value) => {
                source.dimtad = *value;
            }
            Self::Dimzin(value) => {
                source.dimzin = *value;
            }
            Self::Dimtxt(value) => {
                source.dimtxt = *value;
            }
            Self::Dimcen(value) => {
                source.dimcen = *value;
            }
            Self::Dimtsz(value) => {
                source.dimtsz = *value;
            }
            Self::Dimlfac(value) => {
                source.dimlfac = *value;
            }
            Self::Dimtfac(value) => {
                source.dimtfac = *value;
            }
            Self::Dimgap(value) => {
                source.dimgap = *value;
            }
            Self::Dimtofl(value) => {
                source.dimtofl = *value;
            }
            Self::Dimsah(value) => {
                source.dimsah = *value;
            }
            Self::Dimtix(value) => {
                source.dimtix = *value;
            }
            Self::Dimclrd(value) => {
                source.dimclrd = *value;
                source.dimclrd_true_color = None;
            }
            Self::Dimclre(value) => {
                source.dimclre = *value;
                source.dimclre_true_color = None;
            }
            Self::Dimclrt(value) => {
                source.dimclrt = *value;
                source.dimclrt_true_color = None;
            }
            Self::Dimdec(value) => {
                source.dimdec = *value;
            }
            Self::Dimfrac(value) => {
                source.dimfrac = *value;
            }
            Self::Dimlunit(value) => {
                source.dimlunit = *value;
            }
            Self::Dimdsep(value) => {
                source.dimdsep = *value;
            }
            Self::Dimtmove(value) => {
                source.dimtmove = *value;
            }
            Self::Dimsd1(value) => {
                source.dimsd1 = *value;
            }
            Self::Dimsd2(value) => {
                source.dimsd2 = *value;
            }
            Self::Dimatfit(value) => {
                source.dimatfit = *value;
            }
            Self::Dimfxlon(value) => {
                source.dimfxlon = *value;
            }
            Self::Dimtxsty(value) => {
                source.dimtxsty_handle = *value;
                source.dimtxsty.clear();
            }
            Self::Dimblk(value) => {
                source.dimblk = *value;
                source.dimblk_name.clear();
            }
            Self::Dimblk1(value) => {
                source.dimblk1 = *value;
                source.dimblk1_name.clear();
            }
            Self::Dimblk2(value) => {
                source.dimblk2 = *value;
                source.dimblk2_name.clear();
            }
        }
    }
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
) -> Result<CadDimensionParsedOverrides, CadDimensionPreparationError> {
    let mut entries = Vec::new();
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
                entries.push(match code {
                    3 => CadDimensionSourceOverride::Dimpost(string(code, data)?.to_owned()),
                    40 => CadDimensionSourceOverride::Dimscale(real(code, data)?),
                    41 => CadDimensionSourceOverride::Dimasz(real(code, data)?),
                    42 => CadDimensionSourceOverride::Dimexo(real(code, data)?),
                    44 => CadDimensionSourceOverride::Dimexe(real(code, data)?),
                    45 => CadDimensionSourceOverride::Dimrnd(real(code, data)?),
                    46 => CadDimensionSourceOverride::Dimdle(real(code, data)?),
                    49 => CadDimensionSourceOverride::Dimfxl(real(code, data)?),
                    73 => CadDimensionSourceOverride::Dimtih(boolean(code, data)?),
                    74 => CadDimensionSourceOverride::Dimtoh(boolean(code, data)?),
                    75 => CadDimensionSourceOverride::Dimse1(boolean(code, data)?),
                    76 => CadDimensionSourceOverride::Dimse2(boolean(code, data)?),
                    77 => CadDimensionSourceOverride::Dimtad(integer(code, data)?),
                    78 => CadDimensionSourceOverride::Dimzin(integer(code, data)?),
                    140 => CadDimensionSourceOverride::Dimtxt(real(code, data)?),
                    141 => CadDimensionSourceOverride::Dimcen(real(code, data)?),
                    142 => CadDimensionSourceOverride::Dimtsz(real(code, data)?),
                    144 => CadDimensionSourceOverride::Dimlfac(real(code, data)?),
                    146 => CadDimensionSourceOverride::Dimtfac(real(code, data)?),
                    147 => CadDimensionSourceOverride::Dimgap(real(code, data)?),
                    172 => CadDimensionSourceOverride::Dimtofl(boolean(code, data)?),
                    173 => CadDimensionSourceOverride::Dimsah(boolean(code, data)?),
                    174 => CadDimensionSourceOverride::Dimtix(boolean(code, data)?),
                    176 => CadDimensionSourceOverride::Dimclrd(integer(code, data)?),
                    177 => CadDimensionSourceOverride::Dimclre(integer(code, data)?),
                    178 => CadDimensionSourceOverride::Dimclrt(integer(code, data)?),
                    271 => CadDimensionSourceOverride::Dimdec(integer(code, data)?),
                    276 => CadDimensionSourceOverride::Dimfrac(integer(code, data)?),
                    277 => CadDimensionSourceOverride::Dimlunit(integer(code, data)?),
                    278 => CadDimensionSourceOverride::Dimdsep(integer(code, data)?),
                    279 => CadDimensionSourceOverride::Dimtmove(integer(code, data)?),
                    281 => CadDimensionSourceOverride::Dimsd1(boolean(code, data)?),
                    282 => CadDimensionSourceOverride::Dimsd2(boolean(code, data)?),
                    289 => CadDimensionSourceOverride::Dimatfit(integer(code, data)?),
                    290 => CadDimensionSourceOverride::Dimfxlon(boolean(code, data)?),
                    340 => CadDimensionSourceOverride::Dimtxsty(handle(code, data)?),
                    342 => CadDimensionSourceOverride::Dimblk(handle(code, data)?),
                    343 => CadDimensionSourceOverride::Dimblk1(handle(code, data)?),
                    344 => CadDimensionSourceOverride::Dimblk2(handle(code, data)?),
                    _ => {
                        return Err(unsupported(
                            &format!("DSTYLE/{code}"),
                            "dimvar is outside the qualified override grammar",
                        ))
                    }
                });
            }
        }
        if !leftover.is_empty() {
            residual.push(ExtendedDataRecord {
                application_name: record.application_name.clone(),
                values: leftover,
            });
        }
    }
    Ok(CadDimensionParsedOverrides { entries, residual })
}

pub(crate) fn project_source_overrides(
    entries: &[CadDimensionSourceOverride],
    source: &DimStyle,
    effective: &CadDimensionStyleValues,
) -> CadDimensionStyleOverrides {
    let mut values = CadDimensionStyleOverrides::default();
    macro_rules! leaf { ($($path:ident).+) => { values.$($path).+ = Some(effective.$($path).+.clone()); }; }
    for entry in entries {
        match entry {
            CadDimensionSourceOverride::Dimpost(_) => {
                leaf!(numbers.primary.prefix);
                leaf!(numbers.primary.suffix);
            }
            CadDimensionSourceOverride::Dimscale(_) => {
                leaf!(presentation_scale);
            }
            CadDimensionSourceOverride::Dimasz(_) => {
                leaf!(arrows.size);
            }
            CadDimensionSourceOverride::Dimexo(_) => {
                leaf!(extension_lines.first.offset);
                leaf!(extension_lines.second.offset);
            }
            CadDimensionSourceOverride::Dimexe(_) => {
                leaf!(extension_lines.first.extension);
                leaf!(extension_lines.second.extension);
            }
            CadDimensionSourceOverride::Dimrnd(_) => {
                leaf!(numbers.primary.rounding_increment);
            }
            CadDimensionSourceOverride::Dimdle(_) => {
                leaf!(dimension_line.extension);
            }
            CadDimensionSourceOverride::Dimfxl(_) => {
                if source.dimfxlon || matches!(entry, CadDimensionSourceOverride::Dimfxlon(_)) {
                    leaf!(extension_lines.first.length);
                    leaf!(extension_lines.second.length);
                }
            }
            CadDimensionSourceOverride::Dimtih(_) => {
                leaf!(text.inside_orientation);
            }
            CadDimensionSourceOverride::Dimtoh(_) => {
                leaf!(text.outside_orientation);
            }
            CadDimensionSourceOverride::Dimse1(_) => {
                leaf!(extension_lines.first.suppressed);
            }
            CadDimensionSourceOverride::Dimse2(_) => {
                leaf!(extension_lines.second.suppressed);
            }
            CadDimensionSourceOverride::Dimtad(_) => {
                leaf!(text.attachment);
            }
            CadDimensionSourceOverride::Dimzin(_) => {
                leaf!(numbers.primary.zero_suppression);
            }
            CadDimensionSourceOverride::Dimtxt(_) => {
                leaf!(text.height);
            }
            CadDimensionSourceOverride::Dimcen(_) => {
                leaf!(arrows.center_mark);
            }
            CadDimensionSourceOverride::Dimtsz(_) => {
                leaf!(arrows.tick_size);
            }
            CadDimensionSourceOverride::Dimlfac(_) => {
                leaf!(numbers.primary.measurement_factor);
                leaf!(numbers.primary.application);
            }
            CadDimensionSourceOverride::Dimtfac(_) => {
                leaf!(numbers.primary.fraction_scale);
                if let Some(a) = &effective.numbers.alternate {
                    values.numbers.alternate = Some(DimensionAlternateChoice::Enabled(a.clone()));
                }
            }
            CadDimensionSourceOverride::Dimgap(_) => {
                leaf!(text.gap);
            }
            CadDimensionSourceOverride::Dimtofl(_) => {
                leaf!(fit.keep_dimension_line_outside);
            }
            CadDimensionSourceOverride::Dimsah(_) => {
                leaf!(arrows.first);
                leaf!(arrows.second);
            }
            CadDimensionSourceOverride::Dimtix(_) => {
                leaf!(fit.force_text_inside);
            }
            CadDimensionSourceOverride::Dimclrd(_) => {
                leaf!(dimension_line.appearance.color);
            }
            CadDimensionSourceOverride::Dimclre(_) => {
                leaf!(extension_lines.first.appearance.color);
                leaf!(extension_lines.second.appearance.color);
            }
            CadDimensionSourceOverride::Dimclrt(_) => {
                leaf!(text.appearance.color);
            }
            CadDimensionSourceOverride::Dimdec(_) => {
                leaf!(numbers.primary.precision);
            }
            CadDimensionSourceOverride::Dimfrac(_) => {
                leaf!(numbers.primary.fraction_layout);
            }
            CadDimensionSourceOverride::Dimlunit(_) => {
                leaf!(numbers.primary.format);
            }
            CadDimensionSourceOverride::Dimdsep(_) => {
                leaf!(numbers.primary.decimal_separator);
            }
            CadDimensionSourceOverride::Dimtmove(_) => {
                leaf!(text.movement);
            }
            CadDimensionSourceOverride::Dimsd1(_) => {
                leaf!(dimension_line.suppress_first);
            }
            CadDimensionSourceOverride::Dimsd2(_) => {
                leaf!(dimension_line.suppress_second);
            }
            CadDimensionSourceOverride::Dimatfit(_) => {
                leaf!(fit.preferred_placement);
            }
            CadDimensionSourceOverride::Dimfxlon(_) => {
                if source.dimfxlon || matches!(entry, CadDimensionSourceOverride::Dimfxlon(_)) {
                    leaf!(extension_lines.first.length);
                    leaf!(extension_lines.second.length);
                }
            }
            CadDimensionSourceOverride::Dimtxsty(_) => {
                leaf!(text.style);
            }
            CadDimensionSourceOverride::Dimblk(_) => {
                if !source.dimsah {
                    leaf!(arrows.first);
                    leaf!(arrows.second);
                }
            }
            CadDimensionSourceOverride::Dimblk1(_) => {
                if source.dimsah {
                    leaf!(arrows.first);
                }
            }
            CadDimensionSourceOverride::Dimblk2(_) => {
                if source.dimsah {
                    leaf!(arrows.second);
                }
            }
        }
    }
    values
}
pub fn resolve_dimension_overrides(
    parsed: CadDimensionParsedOverrides,
    base: &DimStyle,
    doc: &CadDocument,
) -> Result<CadDimensionOverrides, CadDimensionPreparationError> {
    let mut source = base.clone();
    for entry in &parsed.entries {
        entry.apply_to(&mut source);
    }
    let prepared = prepare_style_from_cad(&source, doc)?;
    let mut values = project_source_overrides(&parsed.entries, &source, &prepared.values);
    if parsed
        .entries
        .iter()
        .any(|e| matches!(e, CadDimensionSourceOverride::Dimtxsty(_)))
        && crate::styles::effective_text_height(base, doc)? != prepared.values.text.height
    {
        values.text.height = Some(prepared.values.text.height);
    }
    Ok(CadDimensionOverrides {
        values,
        effective: prepared.values,
        residual: parsed.residual,
        losses: prepared.losses,
    })
}
