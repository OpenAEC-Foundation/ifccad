//! IFCCAD text identities and independent CAD conversion adapters.
use crate::*;
use ocdraw::{ifccad::*, text::*};
use opencadcodec::{CadDocument, EntityType};
use std::collections::BTreeMap;

fn skipped(loc: &str, error: impl std::fmt::Display, issues: &mut Vec<IfccadDiagnostic>) {
    issues.push(crate::diagnostics::diagnostic(
        "text-skipped",
        loc,
        error.to_string(),
    ));
}
fn fatal(loc: &str, error: cad_text::CadTextError) -> IfccadConversionError {
    IfccadConversionError::TextPreparation {
        location: loc.into(),
        source: error,
    }
}
fn unsupported(e: &cad_text::CadTextError) -> bool {
    matches!(
        e,
        cad_text::CadTextError::Unsupported(_) | cad_text::CadTextError::Markup(_)
    )
}

fn disabled_annotation(r: &opencadcodec::xdata::ExtendedDataRecord) -> bool {
    use opencadcodec::xdata::XDataValue as X;
    matches!(r.values.as_slice(),[X::String(name),X::ControlString(open),X::Integer16(1),X::Integer16(0),X::ControlString(close)] if name=="AnnotativeData"&&open=="{"&&close=="}")
}
pub(crate) fn from_styles(
    source: &CadDocument,
    ids: &mut IfccadIdCounters,
    mappings: &mut IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<Vec<IfccadTextStyle>, IfccadConversionError> {
    let mut styles = Vec::new();
    let mut names = BTreeMap::new();
    for s in source.text_styles.iter() {
        let loc = format!("textStyle/{}", s.handle);
        if !s.is_shape_file
            && (s.name.is_empty()
                || s.name.contains('\0')
                || !s.height.is_finite()
                || s.height < 0.
                || !s.last_height.is_finite()
                || s.last_height < 0.
                || !s.width_factor.is_finite()
                || s.width_factor <= 0.
                || !s.oblique_angle.is_finite()
                || s.oblique_angle.abs() >= std::f64::consts::FRAC_PI_2)
        {
            return Err(IfccadConversionError::InvalidStructure(format!(
                "{loc} invalid ordinary style name or metrics"
            )));
        }
        if !s.is_shape_file
            && names
                .insert(opencadcodec::tables::normalize_name(&s.name), s.handle)
                .is_some()
        {
            return Err(IfccadConversionError::InvalidStructure(
                "duplicate CAD text style lookup name".into(),
            ));
        }
        match cad_text::import_text_style(s) {
            Ok(p) => {
                let id = ids.allocate_text_style_id()?;
                mappings.text_styles.insert(id.0, s.handle);
                styles.push(IfccadTextStyle {
                    id,
                    name: p.name,
                    properties: p.properties,
                });
            }
            Err(e) if unsupported(&e) => skipped(&loc, e, issues),
            Err(e) => return Err(fatal(&loc, e)),
        }
    }
    Ok(styles)
}
pub(crate) fn to_styles(
    d: &IfccadDocument,
    target: &mut CadDocument,
    mappings: &mut IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<(), IfccadConversionError> {
    let mut names = BTreeMap::new();
    for s in &d.text_styles {
        if names
            .insert(opencadcodec::tables::normalize_name(&s.name), s.id)
            .is_some()
        {
            return Err(IfccadConversionError::InvalidStructure(
                "target CAD text style lookup collision".into(),
            ));
        }
    }
    for s in &d.text_styles {
        let loc = format!("textStyle/{}", s.id.0);
        let mut value = match cad_text::prepare_text_style_to_cad(&s.name, &s.properties) {
            Ok(v) => v,
            Err(e) if unsupported(&e) => {
                skipped(&loc, e, issues);
                continue;
            }
            Err(e) => return Err(fatal(&loc, e)),
        };
        if s.properties.last_used_height.is_none() {
            issues.push(crate::diagnostics::modification(
                "text-style-default",
                &loc,
                "absent last-used height becomes the mandatory CAD default 2.5",
            ));
        }
        value.handle = target
            .text_styles
            .get(&s.name)
            .map(|s| s.handle)
            .unwrap_or_else(|| target.allocate_handle());
        let handle = value.handle;
        if let Some(existing) = target.text_styles.get_mut(&s.name) {
            *existing = value;
        } else {
            target
                .text_styles
                .add(value)
                .map_err(IfccadConversionError::CadConstruction)?;
        }
        mappings.text_styles.insert(s.id.0, handle);
    }
    Ok(())
}
fn from_color(c: &cad_text::MarkupColor) -> Result<IfccadColor, cad_text::CadTextError> {
    let source = match *c {
        cad_text::MarkupColor::Rgb { red, green, blue } => {
            opencadcodec::Color::from_rgb(red, green, blue)
        }
        cad_text::MarkupColor::Index(i) => opencadcodec::Color::Index(
            u8::try_from(i).map_err(|_| cad_text::CadTextError::Unsupported("inline ACI range"))?,
        ),
    };
    let value = cad_presentation_convert::explicit_color_from_cad(source)
        .map_err(|_| cad_text::CadTextError::Unsupported("unresolved inline colour"))?;
    Ok(IfccadColor {
        rgb: value.rgb,
        indexed: value.indexed,
        named: value.named,
    })
}
fn to_color(c: &IfccadColor) -> Result<cad_text::MarkupColor, cad_text::CadTextError> {
    let mapped = cad_presentation_convert::color_to_cad(&cad_presentation_convert::CadColorValue {
        rgb: c.rgb,
        indexed: c.indexed.clone(),
        named: c.named.clone(),
    })
    .map_err(|_| cad_text::CadTextError::Unsupported("invalid inline color identity"))?;
    if !mapped.losses.is_empty() || mapped.named.is_some() {
        return Err(cad_text::CadTextError::Unsupported(
            "inline/background color metadata cannot be represented",
        ));
    }
    match mapped.color {
        opencadcodec::Color::Index(index) => Ok(cad_text::MarkupColor::Index(u16::from(index))),
        opencadcodec::Color::Rgb {
            r: red,
            g: green,
            b: blue,
        } => Ok(cad_text::MarkupColor::Rgb { red, green, blue }),
        _ => Err(cad_text::CadTextError::Unsupported("inline color mode")),
    }
}
pub(crate) fn from_entity(
    source: &CadDocument,
    e: &EntityType,
    mappings: &IfccadMappings,
    geometry: &mut crate::geometry_context::GeometryContext,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<Option<IfccadEntityKind>, IfccadConversionError> {
    let loc = format!("entity/{}", e.common().handle);
    let name = match e {
        EntityType::Text(t) => &t.style,
        EntityType::MText(t) => &t.style,
        _ => unreachable!(),
    };
    let key = opencadcodec::tables::normalize_name(name);
    let source_style = source
        .text_styles
        .iter()
        .find(|s| !s.is_shape_file && opencadcodec::tables::normalize_name(&s.name) == key)
        .or_else(|| {
            source
                .text_styles
                .iter()
                .find(|s| opencadcodec::tables::normalize_name(&s.name) == key)
        })
        .ok_or_else(|| {
            IfccadConversionError::InvalidStructure(format!("{loc} missing text style {name}"))
        })?;
    // Annotation scale contexts are outside the bounded text profile.
    if e.common().extended_data.records().iter().any(|r| {
        r.application_name.eq_ignore_ascii_case("AcadAnnotative") && !disabled_annotation(r)
    }) {
        skipped(&loc, "annotation scale context", issues);
        return Ok(None);
    }
    let Some(id) = mappings.text_styles.ifccad_id(source_style.handle) else {
        skipped(&loc, "unsupported text style", issues);
        return Ok(None);
    };
    let style_id = IfccadTextStyleId(id);
    let colour = |c: &cad_text::MarkupColor| from_color(c);
    let prepared = (|| -> Result<_, cad_text::CadTextError> {
        Ok(match e {
            EntityType::Text(t) => {
                let p = cad_text::prepare_text_from_cad(t)?;
                (
                    IfccadEntityKind::Text(IfccadText {
                        style_id,
                        placement: crate::mapping::geometry::placement(p.placement),
                        rotation: p.rotation,
                        backward: p.backward,
                        upside_down: p.upside_down,
                        layout: p.layout,
                        oblique_angle: p.oblique_angle,
                        thickness: p.thickness,
                        content: p.content,
                    }),
                    p.pair,
                    p.normal_normalized,
                    Vec::new(),
                )
            }
            EntityType::MText(t) => {
                let p = cad_text::prepare_mtext_from_cad(t)?;
                (
                    IfccadEntityKind::MText(Box::new(IfccadMText {
                        style_id,
                        placement: crate::mapping::geometry::placement(p.placement),
                        rotation: p.rotation,
                        backward: false,
                        upside_down: false,
                        height: p.height,
                        attachment: p.attachment,
                        flow: p.flow,
                        wrap_width: p.wrap_width,
                        columns: p.columns,
                        background: p
                            .background
                            .as_ref()
                            .map(|b| map_background(b, &colour))
                            .transpose()?,
                        character_format: map_character(&p.content.character_format, &colour)?,
                        paragraph_format: p.content.paragraph_format,
                        content: map_content(&p.content.content, &colour)?,
                    })),
                    p.pair,
                    p.normal_normalized,
                    p.issues,
                )
            }
            _ => unreachable!(),
        })
    })();
    let (kind, pair, normalized, changes) = match prepared {
        Ok(p) => p,
        Err(e) if unsupported(&e) => {
            skipped(&loc, e, issues);
            return Ok(None);
        }
        Err(e) => return Err(fatal(&loc, e)),
    };
    geometry.record(
        e.common().handle.value(),
        IfccadGeometryEntitySource::CadEntity {
            handle: e.common().handle,
            kind: e.as_entity().entity_type().into(),
        },
        pair,
        issues,
    )?;
    if normalized {
        issues.push(crate::diagnostics::modification(
            "source-normal-normalized",
            &loc,
            "text normal magnitude normalized",
        ));
    }
    for change in changes {
        issues.push(crate::diagnostics::modification(
            "text-dependency",
            &loc,
            format!("{change:?}"),
        ));
    }
    Ok(Some(kind))
}
pub(crate) fn to_entity(
    d: &IfccadDocument,
    e: &IfccadNativeEntity,
    mappings: &IfccadMappings,
    geometry: &mut crate::geometry_context::GeometryContext,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<Option<EntityType>, IfccadConversionError> {
    let loc = format!("entity/{}", e.id);
    let id = match &e.kind {
        IfccadEntityKind::Text(t) => t.style_id,
        IfccadEntityKind::MText(t) => t.style_id,
        _ => unreachable!(),
    };
    if mappings.text_styles.cad_handle(id.0).is_none() {
        skipped(&loc, "unsupported target text style", issues);
        return Ok(None);
    }
    let name = &d
        .text_styles
        .iter()
        .find(|s| s.id == id)
        .expect("validated style")
        .name;
    let prepared = (|| -> Result<_, cad_text::CadTextError> {
        match &e.kind {
            IfccadEntityKind::Text(t) => cad_text::prepare_text_to_cad(&cad_text::TextCadInput {
                style_name: name,
                placement: t.placement.coordinate_frame().map_err(|_| {
                    cad_text::CadTextError::InvalidSource("invalid native text frame")
                })?,
                rotation: t.rotation,
                backward: t.backward,
                upside_down: t.upside_down,
                layout: t.layout,
                oblique_angle: t.oblique_angle,
                thickness: t.thickness,
                content: &t.content,
            }),
            IfccadEntityKind::MText(t) => {
                cad_text::prepare_mtext_to_cad(&cad_text::MTextCadInput {
                    style_name: name,
                    placement: t.placement.coordinate_frame().map_err(|_| {
                        cad_text::CadTextError::InvalidSource("invalid native MText frame")
                    })?,
                    rotation: t.rotation,
                    backward: t.backward,
                    upside_down: t.upside_down,
                    height: t.height,
                    attachment: t.attachment,
                    flow: t.flow,
                    wrap_width: t.wrap_width,
                    columns: t.columns.as_ref(),
                    background: map_background_opt(t.background.as_ref())?.as_ref(),
                    content: &map_content(&t.content, &|c: &IfccadColor| to_color(c))?,
                    character_format: &map_character(&t.character_format, &|c: &IfccadColor| {
                        to_color(c)
                    })?,
                    paragraph_format: &t.paragraph_format,
                })
            }
            _ => unreachable!(),
        }
    })();
    let p = match prepared {
        Ok(p) => p,
        Err(e) if unsupported(&e) => {
            skipped(&loc, e, issues);
            return Ok(None);
        }
        Err(e) => return Err(fatal(&loc, e)),
    };
    geometry.record(
        e.id,
        IfccadGeometryEntitySource::NativeEntity {
            owner: geometry.owner,
            entity_id: e.id,
        },
        p.pair,
        issues,
    )?;
    for change in p.issues {
        issues.push(crate::diagnostics::modification(
            "text-dependency",
            &loc,
            format!("{change:?}"),
        ));
    }
    if p.parameterization_changed {
        issues.push(crate::diagnostics::modification(
            "parameterization-changed",
            &loc,
            "text placement converted to the CAD coordinate convention",
        ));
    }
    Ok(Some(p.entity))
}
fn map_background_opt(
    v: Option<&MTextBackground<IfccadColor>>,
) -> Result<Option<MTextBackground<cad_text::MarkupColor>>, cad_text::CadTextError> {
    v.map(|b| map_background(b, &|c: &IfccadColor| to_color(c)))
        .transpose()
}
fn map_character<C, D>(
    v: &CharacterFormat<C>,
    map: &impl Fn(&C) -> Result<D, cad_text::CadTextError>,
) -> Result<CharacterFormat<D>, cad_text::CadTextError> {
    Ok(CharacterFormat {
        font: v.font.clone(),
        height: v.height,
        width_factor: v.width_factor,
        tracking: v.tracking,
        oblique_angle: v.oblique_angle,
        underline: v.underline,
        overline: v.overline,
        strike_through: v.strike_through,
        position: v.position,
        color: v
            .color
            .as_ref()
            .map(|c| {
                Ok::<_, cad_text::CadTextError>(match c {
                    TextColor::Entity => TextColor::Entity,
                    TextColor::ByLayer => TextColor::ByLayer,
                    TextColor::ByBlock => TextColor::ByBlock,
                    TextColor::Explicit(c) => TextColor::Explicit(map(c)?),
                })
            })
            .transpose()?,
    })
}
fn map_content<C, D>(
    content: &[MTextParagraph<C>],
    map: &impl Fn(&C) -> Result<D, cad_text::CadTextError>,
) -> Result<Vec<MTextParagraph<D>>, cad_text::CadTextError> {
    content
        .iter()
        .map(|p| {
            Ok(MTextParagraph {
                paragraph_format: p.paragraph_format.clone(),
                character_format: map_character(&p.character_format, map)?,
                inlines: p
                    .inlines
                    .iter()
                    .map(|v| {
                        Ok(match v {
                            MTextInline::Run {
                                text,
                                character_format,
                            } => MTextInline::Run {
                                text: text.clone(),
                                character_format: map_character(character_format, map)?,
                            },
                            MTextInline::Tab => MTextInline::Tab,
                            MTextInline::LineBreak => MTextInline::LineBreak,
                            MTextInline::ColumnBreak => MTextInline::ColumnBreak,
                            MTextInline::Stack(s) => MTextInline::Stack(TextStack {
                                stack_kind: s.stack_kind,
                                upper: s.upper.clone(),
                                lower: s.lower.clone(),
                                alignment: s.alignment,
                                text_scale: s.text_scale,
                                separator: s.separator,
                                character_format: map_character(&s.character_format, map)?,
                            }),
                        })
                    })
                    .collect::<Result<Vec<_>, cad_text::CadTextError>>()?,
            })
        })
        .collect()
}
fn map_background<C, D>(
    v: &MTextBackground<C>,
    map: &impl Fn(&C) -> Result<D, cad_text::CadTextError>,
) -> Result<MTextBackground<D>, cad_text::CadTextError> {
    Ok(MTextBackground {
        fill: match &v.fill {
            MTextFill::None => MTextFill::None,
            MTextFill::Canvas => MTextFill::Canvas,
            MTextFill::Color(c) => MTextFill::Color(map(c)?),
        },
        padding: v.padding,
        opacity: v.opacity,
        frame: v.frame,
    })
}
