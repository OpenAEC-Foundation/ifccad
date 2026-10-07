//! OCDraw identities and loss policy around the shared CAD text preparation.
use crate::*;
use ocdraw::{ocdraw::*, text::*};
use opencadcodec::{CadDocument, EntityType, Handle};
use std::collections::BTreeMap;

pub(crate) struct SourceTextStyles {
    pub names: BTreeMap<String, Option<OcdrawTextStyleId>>,
    pub handles: BTreeMap<Handle, OcdrawTextStyleId>,
}
pub(crate) struct TargetTextStyles {
    pub names: BTreeMap<OcdrawTextStyleId, Option<String>>,
    pub handles: BTreeMap<OcdrawTextStyleId, Handle>,
}
fn skipped(
    source: CadToOcdrawDiagnosticSource,
    message: impl Into<String>,
    diagnostics: &mut Vec<CadToOcdrawDiagnostic>,
) {
    diagnostics.push(CadToOcdrawDiagnostic::loss(
        source,
        CadToOcdrawAction::Skipped,
        vec![CadToOcdrawLossReason::UnsupportedSemantic {
            name: message.into(),
        }],
    ));
}
fn unsupported(error: &cad_text::CadTextError) -> bool {
    matches!(
        error,
        cad_text::CadTextError::Unsupported(_) | cad_text::CadTextError::Markup(_)
    )
}
pub(crate) fn from_styles(
    source: &CadDocument,
    builder: &mut OcdrawBuilder,
    diagnostics: &mut Vec<CadToOcdrawDiagnostic>,
) -> Result<SourceTextStyles, CadToOcdrawError> {
    let mut map = SourceTextStyles {
        names: BTreeMap::new(),
        handles: BTreeMap::new(),
    };
    let mut seen_handles = std::collections::BTreeSet::new();
    for style in source.text_styles.iter() {
        if style.handle != Handle::NULL && !seen_handles.insert(style.handle) {
            return Err(OcdrawBuildError::Invalid("duplicate CAD text style handle".into()).into());
        }
        let key = style.name.to_lowercase();
        if map.names.contains_key(&key) {
            return Err(
                OcdrawBuildError::Invalid("duplicate CAD text style lookup name".into()).into(),
            );
        }
        let prepared = cad_text::import_text_style(style);
        let id = match prepared {
            Ok(p) => Some(builder.add_text_style(TextStyleDefinition {
                name: p.name,
                properties: p.properties,
            })?),
            Err(error)
                if unsupported(&error)
                    || matches!(&error,cad_text::CadTextError::Values(e) if e.code==TextValueErrorCode::InvalidFont) =>
            {
                skipped(
                    CadToOcdrawDiagnosticSource::Table {
                        kind: format!("text_styles/{}", style.name),
                    },
                    error.to_string(),
                    diagnostics,
                );
                None
            }
            Err(source) => {
                return Err(CadToOcdrawError::TextPreparation {
                    handle: style.handle,
                    source,
                })
            }
        };
        map.names.insert(key, id);
        if let Some(id) = id.filter(|_| style.handle != Handle::NULL) {
            if map.handles.insert(style.handle, id).is_some() {
                return Err(
                    OcdrawBuildError::Invalid("duplicate CAD text style handle".into()).into(),
                );
            }
        }
    }
    Ok(map)
}
pub(crate) fn to_styles(
    source: &OcdrawDocument,
    target: &mut CadDocument,
    diagnostics: &mut Vec<OcdrawToCadDiagnostic>,
) -> Result<TargetTextStyles, OcdrawToCadError> {
    let mut map = TargetTextStyles {
        names: BTreeMap::new(),
        handles: BTreeMap::new(),
    };
    let mut keys = std::collections::BTreeSet::new();
    for s in &source.text_styles {
        if !keys.insert(s.name.to_lowercase()) {
            return Err(OcdrawToCadError::Cad(
                "text styles collide in CAD lookup".into(),
            ));
        }
    }
    for (i, s) in source.text_styles.iter().enumerate() {
        let mut style = match cad_text::prepare_text_style_to_cad(&s.name, &s.properties) {
            Ok(style) => style,
            Err(error) if unsupported(&error) => {
                diagnostics.push(OcdrawToCadDiagnostic {
                    code: "TEXT_STYLE_UNSUPPORTED",
                    location: format!("/textStyles/{i}"),
                    message: error.to_string(),
                });
                map.names.insert(s.id, None);
                continue;
            }
            Err(source) => {
                return Err(OcdrawToCadError::TextPreparation {
                    entity_id: 0,
                    source,
                })
            }
        };
        if s.properties.last_used_height.is_none() {
            diagnostics.push(OcdrawToCadDiagnostic { code:"TEXT_STYLE_LAST_HEIGHT_DEFAULTED", location:format!("/textStyles/{i}/lastUsedHeight"), message:"CAD requires a last-used height value; absent native history becomes the CAD default 2.5".into() });
        }
        style.handle = target
            .text_styles
            .get(&s.name)
            .map(|s| s.handle)
            .unwrap_or_else(|| target.allocate_handle());
        let handle = style.handle;
        if let Some(existing) = target.text_styles.get_mut(&s.name) {
            *existing = style;
        } else {
            target
                .text_styles
                .add(style)
                .map_err(|e| OcdrawToCadError::Cad(e.to_string()))?;
        }
        map.names.insert(s.id, Some(s.name.clone()));
        map.handles.insert(s.id, handle);
    }
    Ok(map)
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
fn from_color(c: &cad_text::MarkupColor) -> Result<DrawingColor, cad_text::CadTextError> {
    Ok(match *c {
        cad_text::MarkupColor::Rgb { red, green, blue } => DrawingColor::rgb(red, green, blue),
        cad_text::MarkupColor::Index(index) => {
            let index = u8::try_from(index).map_err(|_| {
                cad_text::CadTextError::Unsupported("ACI color outside qualified range")
            })?;
            let (r, g, b) = opencadcodec::Color::Index(index).rgb().ok_or(
                cad_text::CadTextError::Unsupported("unresolved inline ACI color"),
            )?;
            DrawingColor::rgb(r, g, b).with_indexed("ACI", u64::from(index))
        }
    })
}
fn to_color(c: &DrawingColor) -> Result<cad_text::MarkupColor, cad_text::CadTextError> {
    if c.named.is_some() {
        return Err(cad_text::CadTextError::Unsupported(
            "named inline/background color metadata",
        ));
    }
    if let Some((system, index)) = &c.indexed {
        if system.eq_ignore_ascii_case("ACI")
            && (1..=255).contains(index)
            && opencadcodec::Color::Index(*index as u8).rgb()
                == Some((c.rgb[0], c.rgb[1], c.rgb[2]))
        {
            return Ok(cad_text::MarkupColor::Index(*index as u16));
        }
        return Err(cad_text::CadTextError::Unsupported(
            "inline/background indexed color metadata or contradictory RGB fallback",
        ));
    }
    Ok(cad_text::MarkupColor::Rgb {
        red: c.rgb[0],
        green: c.rgb[1],
        blue: c.rgb[2],
    })
}

pub(crate) fn from_entity(
    entity: &EntityType,
    scope_id: u32,
    layer_id: u32,
    appearance: EntityAppearance,
    styles: &SourceTextStyles,
    state: &mut crate::geometry_context::GeometryContext<Handle>,
    diagnostics: &mut Vec<CadToOcdrawDiagnostic>,
) -> Result<Option<crate::from_cad::prepared_entities::PreparedCadEntityValue>, CadToOcdrawError> {
    use crate::from_cad::prepared_entities::PreparedCadEntityValue as Prepared;
    let handle = entity.common().handle;
    let identity = OcdrawGeometryEntitySource::CadEntity {
        handle,
        kind: entity.as_entity().entity_type().into(),
    };
    let location = CadToOcdrawDiagnosticSource::Entity {
        handle,
        kind: entity.as_entity().entity_type().into(),
    };
    if entity
        .common()
        .extended_data
        .records()
        .iter()
        .any(|r| r.application_name.eq_ignore_ascii_case("AcadAnnotative"))
    {
        skipped(location, "annotative text context", diagnostics);
        return Ok(None);
    }
    let name = match entity {
        EntityType::Text(t) => &t.style,
        EntityType::MText(t) => &t.style,
        _ => unreachable!("text route"),
    };
    let style_id = match styles.names.get(&name.to_lowercase()) {
        Some(Some(id)) => *id,
        Some(None) => {
            skipped(
                location,
                "text depends on an unsupported style",
                diagnostics,
            );
            return Ok(None);
        }
        None => {
            return Err(CadToOcdrawError::TextPreparation {
                handle,
                source: cad_text::CadTextError::InvalidSource(
                    "unresolved drawing-local CAD text style",
                ),
            })
        }
    };
    let prepared: Result<
        (
            Prepared,
            cad_geometry_convert::GeometryPair,
            bool,
            Vec<cad_text::CadTextIssue>,
        ),
        cad_text::CadTextError,
    > = (|| {
        Ok(match entity {
            EntityType::Text(t) => {
                let p = cad_text::prepare_text_from_cad(t)?;
                (
                    Prepared::Text(TextEntityDefinition {
                        scope_id,
                        layer_id,
                        visible: !t.common.invisible,
                        appearance,
                        style_id,
                        placement: p.placement,
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
                    Prepared::MText(Box::new(MTextEntityDefinition {
                        scope_id,
                        layer_id,
                        visible: !t.common.invisible,
                        appearance,
                        style_id,
                        placement: p.placement,
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
                            .map(|b| map_background(b, &from_color))
                            .transpose()?,
                        character_format: map_character(&p.content.character_format, &from_color)?,
                        paragraph_format: p.content.paragraph_format,
                        content: map_content(&p.content.content, &from_color)?,
                    })),
                    p.pair,
                    p.normal_normalized,
                    p.issues,
                )
            }
            _ => unreachable!("text route"),
        })
    })();
    let (value, pair, normalized, issues) = match prepared {
        Ok(p) => p,
        Err(error) if unsupported(&error) => {
            skipped(location, error.to_string(), diagnostics);
            return Ok(None);
        }
        Err(source) => return Err(CadToOcdrawError::TextPreparation { handle, source }),
    };
    let bound = state
        .record_geometry(handle, identity, pair)
        .map_err(CadToOcdrawError::Geometry)?;
    let mut reasons = issues
        .into_iter()
        .map(|issue| CadToOcdrawLossReason::UnsupportedSemantic {
            name: format!("text dependency change: {issue:?}"),
        })
        .collect::<Vec<_>>();
    if normalized {
        reasons.push(CadToOcdrawLossReason::SourceNormalNormalized);
    }
    if bound > 0. {
        reasons.push(CadToOcdrawLossReason::GeometryRoundedWithinTolerance {
            max_deviation_upper_bound: bound,
        });
    }
    if !reasons.is_empty() {
        diagnostics.push(CadToOcdrawDiagnostic::loss(
            location,
            CadToOcdrawAction::PartiallyExported,
            reasons,
        ));
    }
    Ok(Some(value))
}

pub(crate) fn to_entity(
    text: Option<&DrawingTextEntity>,
    mtext: Option<&DrawingMTextEntity>,
    owner: u32,
    styles: &TargetTextStyles,
    state: &mut crate::geometry_context::GeometryContext<u64>,
    diagnostics: &mut Vec<OcdrawToCadDiagnostic>,
) -> Result<Option<EntityType>, OcdrawToCadError> {
    let (id, style_id) = if let Some(t) = text {
        (t.id, t.style_id)
    } else {
        let t = mtext.expect("text route");
        (t.id, t.style_id)
    };
    let Some(Some(name)) = styles.names.get(&style_id) else {
        diagnostics.push(OcdrawToCadDiagnostic {
            code: "TEXT_UNSUPPORTED",
            location: format!("/entities/{id}"),
            message: "text depends on an unsupported style".into(),
        });
        return Ok(None);
    };
    let prepared = (|| {
        if let Some(t) = text {
            return cad_text::prepare_text_to_cad(&cad_text::TextCadInput {
                style_name: name,
                placement: t.placement,
                rotation: t.rotation,
                backward: t.backward,
                upside_down: t.upside_down,
                layout: t.layout,
                oblique_angle: t.oblique_angle,
                thickness: t.thickness,
                content: &t.content,
            });
        }
        let t = mtext.expect("MText route");
        let content = map_content(&t.content, &to_color)?;
        let character = map_character(&t.character_format, &to_color)?;
        let background = t
            .background
            .as_ref()
            .map(|b| map_background(b, &to_color))
            .transpose()?;
        cad_text::prepare_mtext_to_cad(&cad_text::MTextCadInput {
            style_name: name,
            placement: t.placement,
            rotation: t.rotation,
            backward: t.backward,
            upside_down: t.upside_down,
            height: t.height,
            attachment: t.attachment,
            flow: t.flow,
            wrap_width: t.wrap_width,
            columns: t.columns.as_ref(),
            background: background.as_ref(),
            content: &content,
            character_format: &character,
            paragraph_format: &t.paragraph_format,
        })
    })();
    let p = match prepared {
        Ok(p) => p,
        Err(e) if unsupported(&e) => {
            diagnostics.push(OcdrawToCadDiagnostic {
                code: "TEXT_UNSUPPORTED",
                location: format!("/entities/{id}"),
                message: e.to_string(),
            });
            return Ok(None);
        }
        Err(source) => {
            return Err(OcdrawToCadError::TextPreparation {
                entity_id: id,
                source,
            })
        }
    };
    let bound = state
        .record_geometry(
            id,
            OcdrawGeometryEntitySource::DrawingEntity {
                scope_id: owner,
                entity_id: id,
            },
            p.pair,
        )
        .map_err(OcdrawToCadError::Geometry)?;
    for issue in p.issues {
        diagnostics.push(OcdrawToCadDiagnostic {
            code: "TEXT_DEPENDENCY_CHANGED",
            location: format!("/entities/{id}"),
            message: format!("{issue:?}"),
        });
    }
    if bound > 0. {
        diagnostics.push(OcdrawToCadDiagnostic{code:"GEOMETRY_ROUNDED_WITHIN_TOLERANCE",location:format!("/entities/{id}"),message:format!("text anchor/baseline residual is at most {bound} scope units; glyph geometry is unassessed")});
    }
    if p.parameterization_changed {
        diagnostics.push(OcdrawToCadDiagnostic {
            code: "PARAMETERIZATION_CHANGED",
            location: format!("/entities/{id}"),
            message: "text placement was converted to the CAD coordinate convention".into(),
        });
    }
    Ok(Some(p.entity))
}
