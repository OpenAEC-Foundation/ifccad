//! Independent IFCCAD identities and owner binding around neutral CAD dimension values.
use crate::*;
use ocdraw::{dimension::*, ifccad::*};
use opencadcodec::{entities::Dimension, CadDocument, Handle};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
};

#[derive(Default)]
pub(crate) struct SourceDimensions {
    pub consumed_objects: BTreeSet<Handle>,
    pub consumed_styles: BTreeSet<Handle>,
    pub pending: BTreeMap<Handle, Vec<cad_dimension::CadDimensionPointReference>>,
}
fn fatal(
    location: impl Into<String>,
    source: cad_dimension::CadDimensionPreparationError,
) -> IfccadConversionError {
    IfccadConversionError::DimensionPreparation {
        location: location.into(),
        source,
    }
}
fn loss(
    location: impl Into<String>,
    message: impl Into<String>,
    issues: &mut Vec<IfccadDiagnostic>,
) {
    issues.push(crate::diagnostics::modification(
        "dimension-metadata",
        location,
        message,
    ));
}
fn from_color(c: &cad_dimension::CadColorValue) -> IfccadColor {
    match c {
        cad_dimension::CadColorValue::Foreground => IfccadColor::Foreground,
        cad_dimension::CadColorValue::Concrete {
            rgb,
            indexed,
            named,
        } => IfccadColor::Concrete {
            rgb: *rgb,
            indexed: indexed.clone(),
            named: named.clone(),
        },
    }
}
fn omission(
    location: impl Into<String>,
    message: impl Into<String>,
    issues: &mut Vec<IfccadDiagnostic>,
) {
    issues.push(crate::diagnostics::diagnostic(
        "dimension-metadata",
        location,
        message,
    ));
}
macro_rules! from_refs {
    ($v:expr,$doc:expr,$patterns:expr,$maps:expr) => {{
        let missing = RefCell::new(Vec::new());
        let maps = $maps;
        let doc = $doc;
        let patterns = $patterns;
        let result = $v.map_references(
            |h| {
                maps.text_styles
                    .ifccad_id(*h)
                    .map(IfccadTextStyleId)
                    .unwrap_or_else(|| {
                        missing.borrow_mut().push(format!("textStyle {h}"));
                        IfccadTextStyleId(0)
                    })
            },
            |h| {
                maps.blocks.ifccad_id(*h).unwrap_or_else(|| {
                    missing.borrow_mut().push(format!("block {h}"));
                    0
                })
            },
            from_color,
            |h| {
                doc.line_types
                    .iter()
                    .find(|p| p.handle == *h)
                    .and_then(|p| patterns.resolve(&p.name, Some(*h)).ok().flatten())
                    .unwrap_or_else(|| {
                        missing.borrow_mut().push(format!("linePattern {h}"));
                        IfccadLinePatternId(0)
                    })
            },
        );
        if !missing.borrow().is_empty() {
            return Err(IfccadConversionError::InvalidStructure(format!(
                "Dimension references not transferred: {:?}",
                missing.into_inner()
            )));
        }
        result
    }};
}
pub(crate) fn from_styles(
    doc: &CadDocument,
    patterns: &super::line_pattern::SourcePatterns,
    ids: &mut IfccadIdCounters,
    maps: &mut IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<(Vec<IfccadDimensionStyle>, SourceDimensions), IfccadConversionError> {
    let mut styles = Vec::new();
    let mut state = SourceDimensions::default();
    let baseline = CadDocument::new();
    let used = doc
        .entities()
        .filter_map(|e| {
            if let opencadcodec::EntityType::Dimension(d) = e {
                Some(d.base().style_name.to_lowercase())
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>();
    for s in doc.dim_styles.iter() {
        let mut comparable = s.clone();
        if let Some(original) = baseline.dim_styles.get(&s.name) {
            comparable.handle = original.handle;
            if comparable == *original && !used.contains(&s.name.to_lowercase()) {
                continue;
            }
        }
        let loc = format!("dimensionStyle/{}", s.handle);
        let p = match cad_dimension::prepare_style_from_cad(s, doc) {
            Ok(p) => p,
            Err(cad_dimension::CadDimensionPreparationError::Unsupported { field, detail }) => {
                omission(format!("{loc}/{field}"), detail, issues);
                continue;
            }
            Err(e) => return Err(fatal(&loc, e)),
        };
        if maps.text_styles.ifccad_id(p.values.text.style).is_none() {
            omission(
                &loc,
                "DimensionStyle font style was not represented; style omitted",
                issues,
            );
            continue;
        }
        let settings = from_refs!(&p.values, doc, patterns, &*maps);
        let id = ids.allocate_dimension_style_id()?;
        maps.dimension_styles.insert(id.0, s.handle);
        styles.push(IfccadDimensionStyle {
            id,
            name: s.name.clone(),
            settings,
            type_overrides: BTreeMap::new(),
        });
        state.consumed_styles.insert(s.handle);
        for l in p.losses {
            loss(format!("{loc}/{}", l.field), l.detail, issues);
        }
        if s.name
            .rsplit_once('$')
            .is_some_and(|(_, v)| matches!(v, "0" | "1"))
        {
            loss(&loc,"CAD type substyle retained as complete named style; parent hierarchy and sparse presence unqualified",issues);
        }
    }
    Ok((styles, state))
}
pub(crate) fn from_entity(
    d: &Dimension,
    doc: &CadDocument,
    patterns: &super::line_pattern::SourcePatterns,
    maps: &IfccadMappings,
    state: &mut SourceDimensions,
    geometry: &mut crate::geometry_context::GeometryContext,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<Option<IfccadEntityKind>, IfccadConversionError> {
    let loc = format!("entity/{}", d.base().common.handle);
    let p = match cad_dimension::prepare_dimension_from_cad(d, doc) {
        Ok(p) => p,
        Err(cad_dimension::CadDimensionPreparationError::Unsupported { field, detail }) => {
            omission(format!("{loc}/{field}"), detail, issues);
            return Ok(None);
        }
        Err(e) => return Err(fatal(&loc, e)),
    };
    let Some(style) = maps.dimension_styles.ifccad_id(p.style_handle) else {
        omission(
            &loc,
            "required DimensionStyle was not represented; entity omitted",
            issues,
        );
        return Ok(None);
    };
    let overrides = from_refs!(&p.overrides, doc, patterns, maps);
    let mut pair = cad_geometry_convert::GeometryPair {
        points: Vec::new(),
        curves: Vec::new(),
    };
    for p in p.pairs {
        pair.points.extend(p.points);
        pair.curves.extend(p.curves);
    }
    geometry.record(
        d.base().common.handle.value(),
        IfccadGeometryEntitySource::CadEntity {
            handle: d.base().common.handle,
            kind: "DIMENSION".into(),
        },
        pair,
        issues,
    )?;
    for l in p.losses {
        loss(format!("{loc}/{}", l.field), l.detail, issues);
    }
    state
        .consumed_objects
        .extend(p.consumed_association_objects);
    state
        .pending
        .insert(d.base().common.handle, p.pending_associations);
    Ok(Some(IfccadEntityKind::Dimension(Box::new(
        IfccadDimension {
            placement: super::geometry::placement(p.placement),
            style_id: IfccadDimensionStyleId(style),
            definition: p.definition,
            text_placement: p.text_placement,
            label: p.label,
            overrides,
            associations: Vec::new(),
        },
    ))))
}
pub(crate) fn bind(
    doc: &mut IfccadDocument,
    maps: &IfccadMappings,
    state: &SourceDimensions,
    issues: &mut Vec<IfccadDiagnostic>,
) {
    let mut owners = BTreeMap::new();
    for e in &doc.model.entities {
        owners.insert(e.id(), (0, doc.model.id));
    }
    for p in &doc.paper_layouts {
        for e in &p.entities {
            owners.insert(e.id(), (1, p.id));
        }
    }
    for b in &doc.blocks {
        for e in &b.entities {
            owners.insert(e.id(), (2, b.id));
        }
    }
    let eligible = doc
        .model
        .entities
        .iter()
        .chain(doc.paper_layouts.iter().flat_map(|p| p.entities.iter()))
        .chain(doc.blocks.iter().flat_map(|b| b.entities.iter()))
        .filter_map(IfccadEntity::as_native)
        .filter(|e| e.kind.as_shared_geometry().ok().flatten().is_some())
        .map(|e| e.id)
        .collect::<BTreeSet<_>>();
    for e in doc
        .model
        .entities
        .iter_mut()
        .chain(
            doc.paper_layouts
                .iter_mut()
                .flat_map(|p| p.entities.iter_mut()),
        )
        .chain(doc.blocks.iter_mut().flat_map(|b| b.entities.iter_mut()))
        .filter_map(IfccadEntity::as_native_mut)
    {
        let IfccadEntityKind::Dimension(d) = &mut e.kind else {
            continue;
        };
        let Some(h) = maps.entities.cad_handle(e.id) else {
            continue;
        };
        let Some(refs) = state.pending.get(&h) else {
            continue;
        };
        for r in refs {
            if let Some(id) = maps
                .entities
                .ifccad_id(r.source)
                .filter(|id| eligible.contains(id) && owners.get(id) == owners.get(&e.id))
            {
                d.associations.push(IfccadDimensionAssociation {
                    role: r.role,
                    source_entity_id: id,
                    selector: r.selector,
                });
            } else {
                loss(
                    format!("entity/{}/associations/{:?}", e.id, r.role),
                    "source not represented as same-owner geometry; definition retained",
                    issues,
                );
            }
        }
    }
}

pub(crate) struct TargetDimensionStyle {
    pub name: String,
    pub values: cad_dimension::CadDimensionStyleValues,
}
#[derive(Default)]
pub(crate) struct TargetDimensions {
    styles: BTreeMap<(IfccadDimensionStyleId, DimensionKind), TargetDimensionStyle>,
}
macro_rules! to_refs {
    ($v:expr,$maps:expr) => {{
        let missing = RefCell::new(Vec::new());
        let maps = $maps;
        let mapped = $v.map_references(
            |id| {
                maps.text_styles.cad_handle(id.0).unwrap_or_else(|| {
                    missing.borrow_mut().push(format!("textStyle {}", id.0));
                    Handle::NULL
                })
            },
            |id| {
                maps.blocks.cad_handle(*id).unwrap_or_else(|| {
                    missing.borrow_mut().push(format!("block {id}"));
                    Handle::NULL
                })
            },
            |c| match c {
                IfccadColor::Foreground => cad_dimension::CadColorValue::Foreground,
                IfccadColor::Concrete {
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
                maps.line_patterns.cad_handle(id.0).unwrap_or_else(|| {
                    missing.borrow_mut().push(format!("linePattern {}", id.0));
                    Handle::NULL
                })
            },
        );
        if !missing.borrow().is_empty() {
            return Err(cad_dimension::CadDimensionPreparationError::Unsupported {
                field: "references".into(),
                detail: format!("targets not represented: {:?}", missing.into_inner()),
            });
        }
        mapped
    }};
}
fn target_values(
    v: &IfccadDimensionSettings,
    maps: &IfccadMappings,
) -> Result<cad_dimension::CadDimensionStyleValues, cad_dimension::CadDimensionPreparationError> {
    Ok(to_refs!(v, maps))
}
fn target_overrides(
    v: &IfccadDimensionOverrides,
    maps: &IfccadMappings,
) -> Result<cad_dimension::CadDimensionStyleOverrides, cad_dimension::CadDimensionPreparationError>
{
    Ok(to_refs!(v, maps))
}
pub(crate) fn to_styles(
    doc: &IfccadDocument,
    target: &mut CadDocument,
    maps: &mut IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<TargetDimensions, IfccadConversionError> {
    let mut result = TargetDimensions::default();
    let mut names = doc
        .dimension_styles
        .iter()
        .map(|s| opencadcodec::tables::normalize_name(&s.name))
        .collect::<BTreeSet<_>>();
    if names.len() != doc.dimension_styles.len() {
        return Err(IfccadConversionError::InvalidStructure(
            "DimensionStyle lookup collision".into(),
        ));
    }
    for s in &doc.dimension_styles {
        let loc = format!("dimensionStyle/{}", s.id.0);
        let mut variants = vec![(None, s.name.clone(), s.settings.clone())];
        for (kind, over) in &s.type_overrides {
            let code = match kind {
                DimensionKind::Linear => "0",
                DimensionKind::Aligned => "1",
                _ => {
                    loss(
                        format!("{loc}/typeOverrides/{kind:?}"),
                        "future-kind type settings have no qualified CAD substyle",
                        issues,
                    );
                    continue;
                }
            };
            let name = format!("{}${code}", s.name);
            if !names.insert(opencadcodec::tables::normalize_name(&name)) {
                return Err(IfccadConversionError::InvalidStructure(
                    "DimensionStyle substyle name collision".into(),
                ));
            }
            let mut v = s.settings.clone();
            over.apply(&mut v);
            variants.push((Some(*kind), name, v));
            loss(format!("{loc}/typeOverrides/{kind:?}"),"native sparse type dependency materialized as complete CAD substyle; authored type presence unqualified on reimport",issues);
        }
        for (kind, name, v) in variants {
            let p = target_values(&v, maps).and_then(|values| {
                if target
                    .text_styles
                    .iter()
                    .find(|t| t.handle == values.text.style)
                    .is_some_and(|t| t.height != 0.0 && t.height != values.text.height)
                {
                    return Err(cad_dimension::CadDimensionPreparationError::Unsupported {
                        field: "text/style/fixedHeight".into(),
                        detail:
                            "fixed TextStyle height conflicts with the effective nominal Dimension height"
                                .into(),
                    });
                }
                cad_dimension::prepare_style_to_cad(&name, &values).map(|p| (values, p))
            });
            let (values, mut p) = match p {
                Ok(p) => p,
                Err(cad_dimension::CadDimensionPreparationError::Unsupported { field, detail }) => {
                    if let Some(kind) = kind {
                        result.styles.remove(&(s.id, kind));
                    }
                    omission(format!("{loc}/{field}"), detail, issues);
                    continue;
                }
                Err(e) => return Err(fatal(&loc, e)),
            };
            p.style.handle = target.allocate_handle();
            let handle = p.style.handle;
            p.style.dimtxsty = doc
                .text_styles
                .iter()
                .find(|t| t.id == v.text.style)
                .ok_or_else(|| {
                    IfccadConversionError::InvalidStructure("missing native dimension font".into())
                })?
                .name
                .clone();
            if let Some(existing) = target.dim_styles.get_mut(&name) {
                *existing = p.style;
            } else {
                target
                    .dim_styles
                    .add(p.style)
                    .map_err(|e| IfccadConversionError::CadConstruction(e.to_string()))?;
            }
            for l in p.losses {
                loss(format!("{loc}/{}", l.field), l.detail, issues);
            }
            if let Some(kind) = kind {
                result
                    .styles
                    .insert((s.id, kind), TargetDimensionStyle { name, values });
            } else {
                maps.dimension_styles.insert(s.id.0, handle);
                for kind in [DimensionKind::Linear, DimensionKind::Aligned] {
                    result.styles.insert(
                        (s.id, kind),
                        TargetDimensionStyle {
                            name: name.clone(),
                            values: values.clone(),
                        },
                    );
                }
            }
        }
    }
    Ok(result)
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn to_entity(
    d: &IfccadDimension,
    id: u64,
    doc: &CadDocument,
    maps: &IfccadMappings,
    styles: &TargetDimensions,
    geometry: &mut crate::geometry_context::GeometryContext,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<Option<opencadcodec::EntityType>, IfccadConversionError> {
    let loc = format!("entity/{id}");
    let Some(s) = styles.styles.get(&(d.style_id, d.definition.kind())) else {
        omission(
            &loc,
            "required DimensionStyle not represented; entity omitted",
            issues,
        );
        return Ok(None);
    };
    let p = (|| {
        let over = target_overrides(&d.overrides, maps)?;
        let font = over.text.style.unwrap_or(s.values.text.style);
        let height = over.text.height.unwrap_or(s.values.text.height);
        if doc
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
        let frame = d.placement.coordinate_frame().map_err(|e| {
            cad_dimension::CadDimensionPreparationError::Numeric {
                field: "placement".into(),
                detail: e.to_string(),
            }
        })?;
        let mut p = cad_dimension::prepare_dimension_to_cad(
            &d.definition,
            frame,
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
    let p = match p {
        Ok(p) => p,
        Err(cad_dimension::CadDimensionPreparationError::Unsupported { field, detail }) => {
            omission(format!("{loc}/{field}"), detail, issues);
            return Ok(None);
        }
        Err(e) => return Err(fatal(&loc, e)),
    };
    let mut pair = cad_geometry_convert::GeometryPair {
        points: Vec::new(),
        curves: Vec::new(),
    };
    for p in p.pairs {
        pair.points.extend(p.points);
        pair.curves.extend(p.curves);
    }
    geometry.record(
        id,
        IfccadGeometryEntitySource::NativeEntity {
            owner: geometry.owner,
            entity_id: id,
        },
        pair,
        issues,
    )?;
    for l in p.losses {
        loss(format!("{loc}/{}", l.field), l.detail, issues);
    }
    Ok(Some(opencadcodec::EntityType::Dimension(p.entity)))
}
pub(crate) fn to_associations(
    d: &IfccadDimension,
    id: u64,
    target: &mut CadDocument,
    maps: &IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<(), IfccadConversionError> {
    let Some(dimension) = maps.entities.cad_handle(id) else {
        return Ok(());
    };
    let mut refs = Vec::new();
    for r in &d.associations {
        if let Some(source) = maps.entities.cad_handle(r.source_entity_id) {
            refs.push(cad_dimension::CadDimensionPointReference {
                role: r.role,
                source,
                selector: r.selector,
            });
        } else {
            loss(
                format!("entity/{id}/associations/{:?}", r.role),
                "source omitted; relation detached, stored definition retained",
                issues,
            );
        }
    }
    if refs.is_empty() {
        return Ok(());
    }
    let a = match cad_dimension::encode_point_references(dimension, &refs, target) {
        Ok(a) => a,
        Err(cad_dimension::CadDimensionPreparationError::Unsupported { field, detail }) => {
            loss(format!("entity/{id}/{field}"), detail, issues);
            return Ok(());
        }
        Err(e) => return Err(fatal(format!("entity/{id}"), e)),
    };
    let mut o = opencadcodec::objects::AssociativeObject::new("DIMASSOC", "AcDbDimAssoc");
    o.handle = target.allocate_handle();
    o.owner = dimension;
    o.data = opencadcodec::objects::AssociativeData::DimensionAssociation(a);
    target
        .objects
        .insert(o.handle, opencadcodec::objects::ObjectType::Associative(o));
    loss(format!("entity/{id}/associations"),"DIMASSOC data constructed; CAD-host automatic update and dictionary/reactor activation unassessed",issues);
    Ok(())
}
