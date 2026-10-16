//! OCDraw-owned identities around neutral CAD dimension preparation.
use crate::*;
use ocdraw::{dimension::DimensionKind, ocdraw::*};
use opencadcodec::{entities::Dimension, CadDocument, Handle};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
};

pub(crate) struct SourceDimensionStyles {
    pub handles: BTreeMap<Handle, OcdrawDimensionStyleId>,
    pub consumed: BTreeSet<Handle>,
}
pub(crate) struct SourceDimensionIndex<'a> {
    pub document: &'a CadDocument,
    pub text: &'a super::text::SourceTextStyles,
    pub blocks: &'a BTreeMap<Handle, u32>,
    pub patterns: &'a super::line_pattern::ExportLinePatternMap,
}
impl SourceDimensionIndex<'_> {
    fn text(&self, h: &Handle) -> Option<OcdrawTextStyleId> {
        self.text.handles.get(h).copied()
    }
    fn pattern(&self, h: &Handle) -> Option<LinePatternId> {
        let name = &self
            .document
            .line_types
            .iter()
            .find(|p| p.handle == *h)?
            .name;
        match self.patterns.resolve(name, Some(*h)).ok()? {
            AppearanceSelection::Explicit(id) => Some(id),
            _ => None,
        }
    }
}
fn color(c: &cad_dimension::CadColorValue) -> DrawingColor {
    match c {
        cad_dimension::CadColorValue::Foreground => DrawingColor::Foreground,
        cad_dimension::CadColorValue::Concrete {
            rgb,
            indexed,
            named,
        } => DrawingColor::Concrete {
            rgb: *rgb,
            indexed: indexed.clone(),
            named: named.clone(),
        },
    }
}
// Missing references only produce private provisional values; the result is discarded on error.
macro_rules! from_refs {
    ($v:expr,$idx:expr) => {{
        let missing = RefCell::new(Vec::new());
        let idx = $idx;
        let mapped = $v.map_references(
            |h| {
                idx.text(h).unwrap_or_else(|| {
                    missing.borrow_mut().push(format!("textStyle {h}"));
                    OcdrawTextStyleId(0)
                })
            },
            |h| {
                idx.blocks.get(h).copied().unwrap_or_else(|| {
                    missing.borrow_mut().push(format!("block {h}"));
                    0
                })
            },
            color,
            |h| {
                idx.pattern(h).unwrap_or_else(|| {
                    missing.borrow_mut().push(format!("linePattern {h}"));
                    LinePatternId(0)
                })
            },
        );
        if !missing.borrow().is_empty() {
            return Err(CadToOcdrawError::DrawingBuild(OcdrawBuildError::Invalid(
                format!(
                    "Dimension references unavailable: {:?}",
                    missing.into_inner()
                ),
            )));
        }
        mapped
    }};
}
fn source_loss(
    source: CadToOcdrawDiagnosticSource,
    action: CadToOcdrawAction,
    message: impl Into<String>,
    diagnostics: &mut Vec<CadToOcdrawDiagnostic>,
) {
    diagnostics.push(CadToOcdrawDiagnostic::loss(
        source,
        action,
        vec![CadToOcdrawLossReason::UnsupportedSemantic {
            name: message.into(),
        }],
    ));
}
pub(crate) fn from_styles(
    idx: &SourceDimensionIndex<'_>,
    builder: &mut OcdrawBuilder,
    diagnostics: &mut Vec<CadToOcdrawDiagnostic>,
) -> Result<SourceDimensionStyles, CadToOcdrawError> {
    let mut map = SourceDimensionStyles {
        handles: BTreeMap::new(),
        consumed: BTreeSet::new(),
    };
    let baseline = CadDocument::new();
    let used = idx
        .document
        .entities()
        .filter_map(|e| {
            if let opencadcodec::EntityType::Dimension(d) = e {
                Some(d.base().style_name.to_lowercase())
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>();
    for style in idx.document.dim_styles.iter() {
        let mut comparable = style.clone();
        if let Some(original) = baseline.dim_styles.get(&style.name) {
            comparable.handle = original.handle;
            if comparable == *original && !used.contains(&style.name.to_lowercase()) {
                continue;
            }
        }
        let source = CadToOcdrawDiagnosticSource::Table {
            kind: format!("dim_styles/{}", style.name),
        };
        let p = match cad_dimension::prepare_style_from_cad(style, idx.document) {
            Ok(p) => p,
            Err(cad_dimension::CadDimensionPreparationError::Unsupported { field, detail }) => {
                source_loss(
                    source,
                    CadToOcdrawAction::Skipped,
                    format!("DimensionStyle {field}: {detail}"),
                    diagnostics,
                );
                continue;
            }
            Err(source) => {
                return Err(CadToOcdrawError::DimensionPreparation {
                    handle: style.handle,
                    source,
                })
            }
        };
        if !idx.text.handles.contains_key(&p.values.text.style) {
            source_loss(
                source,
                CadToOcdrawAction::Skipped,
                "DimensionStyle text style was not converted",
                diagnostics,
            );
            continue;
        }
        let settings = from_refs!(&p.values, idx);
        let id = builder.add_dimension_style(DimensionStyleDefinition {
            name: style.name.clone(),
            settings,
            type_overrides: BTreeMap::new(),
        })?;
        if map.handles.insert(style.handle, id).is_some() {
            return Err(
                OcdrawBuildError::Invalid("duplicate CAD DimensionStyle handle".into()).into(),
            );
        }
        map.consumed.insert(style.handle);
        if style
            .name
            .rsplit_once('$')
            .is_some_and(|(_, code)| matches!(code, "0" | "1"))
        {
            source_loss(source.clone(),CadToOcdrawAction::PartiallyExported,"CAD type substyle retained as a complete named native style; parent hierarchy and sparse authored presence are unqualified",diagnostics);
        }
        for l in p.losses {
            source_loss(
                source.clone(),
                CadToOcdrawAction::PartiallyExported,
                format!("DimensionStyle {}: {}", l.field, l.detail),
                diagnostics,
            );
        }
    }
    Ok(map)
}
pub(crate) struct PreparedDimension {
    pub definition: DimensionEntityDefinition,
    pub references: Vec<cad_dimension::CadDimensionPointReference>,
    pub consumed: BTreeSet<Handle>,
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn from_entity(
    d: &Dimension,
    idx: &SourceDimensionIndex<'_>,
    styles: &SourceDimensionStyles,
    scope_id: u32,
    layer_id: u32,
    appearance: EntityAppearance,
    state: &mut super::geometry::ExchangeState<Handle>,
    partial: &mut Vec<CadToOcdrawLossReason>,
) -> Result<Option<PreparedDimension>, CadToOcdrawError> {
    let p = match cad_dimension::prepare_dimension_from_cad(d, idx.document) {
        Ok(p) => p,
        Err(cad_dimension::CadDimensionPreparationError::Unsupported { field, detail }) => {
            partial.push(CadToOcdrawLossReason::UnsupportedSemantic {
                name: format!("Dimension {field}: {detail}"),
            });
            return Ok(None);
        }
        Err(source) => {
            return Err(CadToOcdrawError::DimensionPreparation {
                handle: d.base().common.handle,
                source,
            })
        }
    };
    let Some(&style_id) = styles.handles.get(&p.style_handle) else {
        partial.push(CadToOcdrawLossReason::MissingTarget {
            kind: "DimensionStyle".into(),
            identifier: p.style_handle.to_string(),
        });
        return Ok(None);
    };
    let overrides = from_refs!(&p.overrides, idx);
    let mut pair = cad_geometry_convert::GeometryPair {
        points: Vec::new(),
        curves: Vec::new(),
    };
    for p in p.pairs {
        pair.points.extend(p.points);
        pair.curves.extend(p.curves);
    }
    let bound = state
        .record_geometry(
            d.base().common.handle,
            OcdrawGeometryEntitySource::CadEntity {
                handle: d.base().common.handle,
                kind: "DIMENSION".into(),
            },
            pair,
        )
        .map_err(CadToOcdrawError::Geometry)?;
    if bound > 0.0 {
        partial.push(CadToOcdrawLossReason::GeometryRoundedWithinTolerance {
            max_deviation_upper_bound: bound,
        });
    }
    partial.extend(
        p.losses
            .into_iter()
            .map(|l| CadToOcdrawLossReason::UnsupportedSemantic {
                name: format!("Dimension {}: {}", l.field, l.detail),
            }),
    );
    Ok(Some(PreparedDimension {
        definition: DimensionEntityDefinition {
            scope_id,
            layer_id,
            visible: !d.base().common.invisible,
            appearance,
            placement: p.placement,
            style_id,
            definition: p.definition,
            text_placement: p.text_placement,
            label: p.label,
            overrides,
            associations: Vec::new(),
        },
        references: p.pending_associations,
        consumed: p.consumed_association_objects.into_iter().collect(),
    }))
}

pub(crate) struct TargetDimensionStyle {
    pub name: String,
    pub values: cad_dimension::CadDimensionStyleValues,
}
pub(crate) struct TargetDimensionStyles {
    pub styles: BTreeMap<(OcdrawDimensionStyleId, DimensionKind), TargetDimensionStyle>,
    pub handles: BTreeMap<OcdrawDimensionStyleId, Handle>,
}
pub(crate) struct TargetDimensionIndex<'a> {
    pub text: &'a super::text::TargetTextStyles,
    pub blocks: &'a BTreeMap<u64, Handle>,
    pub patterns: &'a super::line_pattern::ImportLinePatternMap,
}
macro_rules! to_refs {
    ($v:expr,$idx:expr) => {{
        let missing = RefCell::new(Vec::new());
        let idx = $idx;
        let mapped = $v.map_references(
            |id| {
                idx.text.handles.get(id).copied().unwrap_or_else(|| {
                    missing.borrow_mut().push(format!("textStyle {}", id.0));
                    Handle::NULL
                })
            },
            |id| {
                idx.blocks.get(&u64::from(*id)).copied().unwrap_or_else(|| {
                    missing.borrow_mut().push(format!("block {id}"));
                    Handle::NULL
                })
            },
            |c| match c {
                DrawingColor::Foreground => cad_dimension::CadColorValue::Foreground,
                DrawingColor::Concrete {
                    rgb,
                    indexed,
                    named,
                } => cad_dimension::CadColorValue::Concrete {
                    rgb: *rgb,
                    indexed: indexed.clone(),
                    named: named.clone(),
                },
            },
            |id| {
                idx.patterns.get(id).map(|p| p.1).unwrap_or_else(|| {
                    missing.borrow_mut().push(format!("linePattern {}", id.0));
                    Handle::NULL
                })
            },
        );
        if !missing.borrow().is_empty() {
            return Err(cad_dimension::CadDimensionPreparationError::Unsupported {
                field: "references".into(),
                detail: format!("targets not transferred: {:?}", missing.into_inner()),
            });
        }
        mapped
    }};
}
fn target_values(
    v: &OcdrawDimensionSettings,
    idx: &TargetDimensionIndex<'_>,
) -> Result<cad_dimension::CadDimensionStyleValues, cad_dimension::CadDimensionPreparationError> {
    Ok(to_refs!(v, idx))
}
fn target_overrides(
    v: &OcdrawDimensionOverrides,
    idx: &TargetDimensionIndex<'_>,
) -> Result<cad_dimension::CadDimensionStyleOverrides, cad_dimension::CadDimensionPreparationError>
{
    Ok(to_refs!(v, idx))
}
fn target_loss(
    location: impl Into<String>,
    message: impl Into<String>,
    diagnostics: &mut Vec<OcdrawToCadDiagnostic>,
) {
    diagnostics.push(crate::to_cad::diagnostic(
        "DIMENSION_METADATA",
        location,
        message,
    ));
}
pub(crate) fn to_styles(
    source: &OcdrawDocument,
    target: &mut CadDocument,
    idx: &TargetDimensionIndex<'_>,
    diagnostics: &mut Vec<OcdrawToCadDiagnostic>,
) -> Result<TargetDimensionStyles, OcdrawToCadError> {
    let mut map = TargetDimensionStyles {
        styles: BTreeMap::new(),
        handles: BTreeMap::new(),
    };
    let mut names = source
        .dimension_styles
        .iter()
        .map(|s| opencadcodec::tables::normalize_name(&s.name))
        .collect::<BTreeSet<_>>();
    if names.len() != source.dimension_styles.len() {
        return Err(OcdrawToCadError::Cad(
            "DimensionStyle names collide in CAD lookup".into(),
        ));
    }
    for (i, s) in source.dimension_styles.iter().enumerate() {
        let location = format!("/dimensionStyles/{i}");
        let mut variants = vec![(None, s.name.clone(), s.settings.clone())];
        for (kind, over) in &s.type_overrides {
            let code = match kind {
                DimensionKind::Linear => "0",
                DimensionKind::Aligned => "1",
                _ => {
                    target_loss(
                        format!("{location}/typeOverrides/{kind:?}"),
                        "unused future-kind settings have no qualified CAD substyle mapping",
                        diagnostics,
                    );
                    continue;
                }
            };
            let name = format!("{}${code}", s.name);
            if !names.insert(opencadcodec::tables::normalize_name(&name)) {
                return Err(OcdrawToCadError::Cad(
                    "DimensionStyle substyle name collision".into(),
                ));
            }
            let mut v = s.settings.clone();
            over.apply(&mut v);
            variants.push((Some(*kind), name, v));
            target_loss(format!("{location}/typeOverrides/{kind:?}"),"native sparse type-style dependency materialized as a complete CAD substyle; authored type override presence unqualified on reimport",diagnostics);
        }
        for (kind, name, v) in variants {
            let prepared = target_values(&v, idx).and_then(|values| {
                if target
                    .text_styles
                    .iter()
                    .find(|t| t.handle == values.text.style)
                    .is_some_and(|t| t.height != 0.0 && t.height != values.text.height)
                {
                    return Err(cad_dimension::CadDimensionPreparationError::Unsupported {
                        field: "text/style/fixedHeight".into(),
                        detail: "fixed TextStyle height conflicts with the effective nominal Dimension height"
                            .into(),
                    });
                }
                cad_dimension::prepare_style_to_cad(&name, &values).map(|p| (values, p))
            });
            let (values, mut p) = match prepared {
                Ok(p) => p,
                Err(cad_dimension::CadDimensionPreparationError::Unsupported { field, detail }) => {
                    if let Some(kind) = kind {
                        map.styles.remove(&(s.id, kind));
                    }
                    target_loss(format!("{location}/{field}"), detail, diagnostics);
                    continue;
                }
                Err(source) => {
                    return Err(OcdrawToCadError::DimensionPreparation {
                        entity_id: 0,
                        source,
                    })
                }
            };
            p.style.handle = target.allocate_handle();
            p.style.dimtxsty = idx
                .text
                .names
                .get(&v.text.style)
                .and_then(|v| v.clone())
                .ok_or_else(|| {
                    OcdrawToCadError::Cad("converted Dimension TextStyle has no name".into())
                })?;
            let handle = p.style.handle;
            if target.dim_styles.get(&name).is_some() {
                *target.dim_styles.get_mut(&name).expect("existing style") = p.style;
            } else {
                target
                    .dim_styles
                    .add(p.style)
                    .map_err(|e| OcdrawToCadError::Cad(format!("DimensionStyle: {e}")))?;
            }
            for l in p.losses {
                target_loss(format!("{location}/{}", l.field), l.detail, diagnostics);
            }
            let value = TargetDimensionStyle {
                name: name.clone(),
                values,
            };
            if let Some(kind) = kind {
                map.styles.insert((s.id, kind), value);
            } else {
                map.handles.insert(s.id, handle);
                for kind in [DimensionKind::Linear, DimensionKind::Aligned] {
                    map.styles.insert(
                        (s.id, kind),
                        TargetDimensionStyle {
                            name: name.clone(),
                            values: value.values.clone(),
                        },
                    );
                }
            }
        }
    }
    Ok(map)
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn to_entity(
    d: &DrawingDimensionEntity,
    owner: u32,
    styles: &TargetDimensionStyles,
    idx: &TargetDimensionIndex<'_>,
    target: &CadDocument,
    state: &mut super::geometry::ExchangeState<u64>,
    diagnostics: &mut Vec<OcdrawToCadDiagnostic>,
) -> Result<Option<opencadcodec::EntityType>, OcdrawToCadError> {
    let location = format!("/entities/{}", d.id);
    let Some(s) = styles.styles.get(&(d.style_id, d.definition.kind())) else {
        target_loss(
            location,
            "DimensionStyle not transferred; dimension skipped",
            diagnostics,
        );
        return Ok(None);
    };
    let prepared = (|| {
        let over = target_overrides(&d.overrides, idx)?;
        let font = over.text.style.unwrap_or(s.values.text.style);
        let height = over.text.height.unwrap_or(s.values.text.height);
        if target
            .text_styles
            .iter()
            .find(|t| t.handle == font)
            .is_some_and(|t| t.height != 0.0 && t.height != height)
        {
            return Err(cad_dimension::CadDimensionPreparationError::Unsupported {
                field: "text/style/fixedHeight".into(),
                detail:
                    "fixed TextStyle height conflicts with the effective nominal Dimension height"
                        .into(),
            });
        }
        let mut p = cad_dimension::prepare_dimension_to_cad(
            &d.definition,
            d.placement,
            &d.text_placement,
            &d.label,
        )?;
        p.entity.base_mut().style_name = s.name.clone();
        p.losses
            .extend(cad_dimension::apply_entity_overrides_to_cad(
                &mut p.entity,
                &over,
                &s.values,
            )?);
        Ok::<_, cad_dimension::CadDimensionPreparationError>(p)
    })();
    let p = match prepared {
        Ok(p) => p,
        Err(cad_dimension::CadDimensionPreparationError::Unsupported { field, detail }) => {
            target_loss(format!("{location}/{field}"), detail, diagnostics);
            return Ok(None);
        }
        Err(source) => {
            return Err(OcdrawToCadError::DimensionPreparation {
                entity_id: d.id,
                source,
            })
        }
    };
    let mut pair = cad_geometry_convert::GeometryPair {
        points: Vec::new(),
        curves: Vec::new(),
    };
    for p in p.pairs {
        pair.points.extend(p.points);
        pair.curves.extend(p.curves);
    }
    let bound = state
        .record_geometry(
            d.id,
            OcdrawGeometryEntitySource::DrawingEntity {
                scope_id: owner,
                entity_id: d.id,
            },
            pair,
        )
        .map_err(OcdrawToCadError::Geometry)?;
    if bound > 0.0 {
        diagnostics.push(crate::to_cad::diagnostic(
            "GEOMETRY_ROUNDED_WITHIN_TOLERANCE",
            &location,
            format!("Dimension anchor deviation is at most {bound} coordinate units"),
        ));
    }
    for l in p.losses {
        target_loss(format!("{location}/{}", l.field), l.detail, diagnostics);
    }
    Ok(Some(opencadcodec::EntityType::Dimension(Box::new(
        p.entity,
    ))))
}
pub(crate) fn to_associations(
    d: &DrawingDimensionEntity,
    dimension: Handle,
    mapping: &BTreeMap<u64, Handle>,
    target: &mut CadDocument,
    diagnostics: &mut Vec<OcdrawToCadDiagnostic>,
) -> Result<(), OcdrawToCadError> {
    let mut refs = Vec::new();
    for r in &d.associations {
        if let Some(source) = mapping.get(&r.source_entity_id) {
            refs.push(cad_dimension::CadDimensionPointReference {
                role: r.role,
                source: *source,
                selector: r.selector,
            });
        } else {
            target_loss(
                format!("/entities/{}/associations/{:?}", d.id, r.role),
                "source not materialized; relation detached, stored definition retained",
                diagnostics,
            );
        }
    }
    if refs.is_empty() {
        return Ok(());
    }
    let a = match cad_dimension::encode_point_references(dimension, &refs, target) {
        Ok(a) => a,
        Err(cad_dimension::CadDimensionPreparationError::Unsupported { field, detail }) => {
            target_loss(format!("/entities/{}/{}", d.id, field), detail, diagnostics);
            return Ok(());
        }
        Err(source) => {
            return Err(OcdrawToCadError::DimensionPreparation {
                entity_id: d.id,
                source,
            })
        }
    };
    let mut o = opencadcodec::objects::AssociativeObject::new("DIMASSOC", "AcDbDimAssoc");
    o.handle = target.allocate_handle();
    o.owner = dimension;
    o.data = opencadcodec::objects::AssociativeData::DimensionAssociation(a);
    target
        .objects
        .insert(o.handle, opencadcodec::objects::ObjectType::Associative(o));
    target_loss(format!("/entities/{}/associations",d.id),"DIMASSOC data constructed; CAD-host automatic update and dictionary/reactor activation are unassessed",diagnostics);
    Ok(())
}
