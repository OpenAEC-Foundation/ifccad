use crate::{
    diagnostics::{diagnostic, modification},
    *,
};
use ifccad::*;
use opencadcodec::{CadDocument, Handle, LineType};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct SourcePatterns {
    records: BTreeMap<String, (Option<IfccadLinePatternId>, Handle)>,
}
fn invalid(message: impl Into<String>) -> IfccadConversionError {
    IfccadConversionError::InvalidStructure(message.into())
}

fn from_marker(
    doc: &CadDocument,
    value: &opencadcodec::tables::LineTypeComplexData,
    mappings: &IfccadMappings,
    symbols: &super::symbol_source::SourceSymbolSources,
) -> Result<Option<IfccadLinePatternMarker>, IfccadConversionError> {
    use cad_presentation_convert::*;
    use ifccad::line_pattern::*;
    let style = doc
        .text_styles
        .iter()
        .find(|s| !value.style_handle.is_null() && s.handle == value.style_handle)
        .ok_or_else(|| invalid("missing marker STYLE handle"))?;
    if style.is_shape_file != value.is_shape() {
        return Err(invalid("marker STYLE has wrong role"));
    }
    let value = match marker_values_from_cad(value) {
        Ok(v) => v,
        Err(CadLinePatternValueError::Unsupported(_)) => return Ok(None),
        Err(e) => return Err(invalid(e.to_string())),
    };
    let transform = LinePatternTransform {
        scale: value.scale,
        offset: value.offset,
        rotation: if value.upright {
            LinePatternRotation::Upright(value.rotation)
        } else if value.absolute_rotation {
            LinePatternRotation::Absolute(value.rotation)
        } else {
            LinePatternRotation::Relative(value.rotation)
        },
    };
    Ok(match value.content {
        CadLinePatternMarkerContent::Text(text) => mappings
            .text_styles
            .ifccad_id(style.handle)
            .map(|id| IfccadLinePatternMarker::Text {
                text,
                text_style_id: IfccadTextStyleId(id),
                transform,
            }),
        CadLinePatternMarkerContent::Shape(shape_number) => symbols
            .by_handle
            .get(&style.handle)
            .copied()
            .flatten()
            .map(|id| IfccadLinePatternMarker::Symbol {
                symbol_source_id: id,
                shape_number,
                transform,
            }),
    })
}

fn to_marker(
    marker: &IfccadLinePatternMarker,
    mappings: &IfccadMappings,
    symbols: &BTreeMap<IfccadSymbolSourceId, Handle>,
) -> Result<Option<opencadcodec::tables::LineTypeComplexData>, IfccadConversionError> {
    use cad_presentation_convert::*;
    use ifccad::line_pattern::*;
    let (content, handle, transform) = match marker {
        IfccadLinePatternMarker::Text {
            text,
            text_style_id,
            transform,
        } => (
            CadLinePatternMarkerContent::Text(text.clone()),
            mappings.text_styles.cad_handle(text_style_id.0),
            transform,
        ),
        IfccadLinePatternMarker::Symbol {
            symbol_source_id,
            shape_number,
            transform,
        } => (
            CadLinePatternMarkerContent::Shape(*shape_number),
            symbols.get(symbol_source_id).copied(),
            transform,
        ),
    };
    let Some(handle) = handle else {
        return Ok(None);
    };
    let (rotation, absolute_rotation, upright) = match transform.rotation {
        LinePatternRotation::Relative(v) => (v, false, false),
        LinePatternRotation::Absolute(v) => (v, true, false),
        LinePatternRotation::Upright(v) => (v, false, true),
    };
    match marker_values_to_cad(
        &CadLinePatternMarkerValues {
            content,
            scale: transform.scale,
            offset: transform.offset,
            rotation,
            absolute_rotation,
            upright,
        },
        handle,
    ) {
        Ok(v) => Ok(Some(v)),
        Err(CadLinePatternValueError::Unsupported(_)) => Ok(None),
        Err(e) => Err(invalid(e.to_string())),
    }
}
fn scale(v: f64) -> Result<(), IfccadConversionError> {
    if !v.is_finite() || v <= 0. {
        return Err(invalid("invalid drawing/entity line pattern scale"));
    }
    Ok(())
}
impl SourcePatterns {
    pub(crate) fn resolve(
        &self,
        name: &str,
        handle: Option<Handle>,
    ) -> Result<Option<IfccadLinePatternId>, IfccadConversionError> {
        let name = if name.is_empty() { "ByLayer" } else { name };
        let (id, h) = self
            .records
            .get(&opencadcodec::tables::normalize_name(name))
            .ok_or_else(|| invalid(format!("missing line pattern {name}")))?;
        if handle.is_some_and(|v| !v.is_null() && v != *h) {
            return Err(invalid(format!(
                "line pattern name/handle disagree: {name}"
            )));
        }
        Ok(*id)
    }
    pub(crate) fn layer(&self, name: &str) -> IfccadLinePatternId {
        self.resolve(name, None)
            .expect("checked layer reference")
            .expect("ordinary layer pattern")
    }
    pub(crate) fn entity(
        &self,
        c: &opencadcodec::entities::EntityCommon,
    ) -> IfccadMode<IfccadLinePatternId> {
        if c.linetype.is_empty() || c.linetype.eq_ignore_ascii_case("ByLayer") {
            IfccadMode::ByLayer
        } else if c.linetype.eq_ignore_ascii_case("ByBlock") {
            IfccadMode::ByBlock
        } else {
            IfccadMode::Explicit(
                self.resolve(&c.linetype, c.linetype_handle)
                    .expect("checked entity reference")
                    .unwrap(),
            )
        }
    }
}

pub(crate) fn from_cad(
    doc: &CadDocument,
    preserve_splines: bool,
    ids: &mut IfccadIdCounters,
    issues: &mut Vec<IfccadDiagnostic>,
    mappings: &IfccadMappings,
    symbols: &super::symbol_source::SourceSymbolSources,
) -> Result<(Vec<IfccadLinePattern>, SourcePatterns), IfccadConversionError> {
    scale(doc.header.linetype_scale)?;
    let mut definitions = Vec::new();
    let mut records = BTreeMap::new();
    for p in doc.line_types.iter() {
        let key = opencadcodec::tables::normalize_name(&p.name);
        if records.contains_key(&key) {
            return Err(invalid("duplicate line pattern lookup name"));
        }
        if p.alignment != 'A'
            || !p.pattern_length.is_finite()
            || p.pattern_length < 0.
            || !p
                .elements
                .iter()
                .map(|e| e.length.abs())
                .sum::<f64>()
                .is_finite()
            || p.elements.iter().any(|e| {
                !e.length.is_finite()
                    || e.complex.as_ref().is_some_and(|c| {
                        !c.scale.is_finite()
                            || !c.rotation.is_finite()
                            || c.offset.iter().any(|v| !v.is_finite())
                    })
            })
        {
            return Err(invalid(format!(
                "invalid line pattern values/alignment: {}",
                p.name
            )));
        }
        let loc = format!("linePattern/{}", p.name);
        if key == "BYLAYER" || key == "BYBLOCK" {
            if !p.elements.is_empty() || p.pattern_length != 0. {
                return Err(invalid("contradictory special line pattern record"));
            }
            let mut expected = if key == "BYLAYER" {
                LineType::by_layer()
            } else {
                LineType::by_block()
            };
            expected.handle = p.handle;
            expected.name = p.name.clone();
            if &expected != p {
                issues.push(diagnostic(
                    "line-pattern-special",
                    &loc,
                    "modified selection scaffold metadata omitted",
                ));
            }
            records.insert(key, (None, p.handle));
            continue;
        }
        if key == "CONTINUOUS" && !p.elements.is_empty() {
            return Err(invalid("nonempty Continuous record"));
        }
        let raw = p
            .elements
            .iter()
            .map(|e| ifccad::line_pattern::LinePatternElement {
                length: e.length,
                marker: e.complex.as_ref().map(|_| ()),
            })
            .collect::<Vec<_>>();
        ifccad::line_pattern::validate_line_pattern_values(&raw)
            .map_err(|e| invalid(e.to_string()))?;
        let mut pattern = Vec::new();
        let mut fallback = false;
        for element in &p.elements {
            let marker = if let Some(c) = &element.complex {
                let marker = from_marker(doc, c, mappings, symbols)?;
                if marker.is_none() {
                    fallback = true;
                }
                marker
            } else {
                None
            };
            pattern.push(ifccad::line_pattern::LinePatternElement {
                length: element.length,
                marker,
            });
        }
        if fallback {
            pattern.clear();
            issues.push(modification("line-pattern-complex",&loc,"unsupported active marker/style/source dependency: entire definition replaced with named continuous"));
        }
        if p.pattern_length != p.elements.iter().map(|e| e.length.abs()).sum::<f64>() {
            issues.push(modification(
                "line-pattern-period",
                &loc,
                "declared pattern period replaced with derived element-length sum",
            ));
        }
        if p.xref_dependent || !p.xref_block_record_handle.is_null() {
            issues.push(diagnostic(
                "line-pattern-provenance",
                &loc,
                "external line pattern provenance omitted",
            ));
        }
        let id = ids
            .allocate_line_pattern_id()
            .map_err(IfccadConversionError::IdAllocation)?;
        definitions.push(IfccadLinePattern {
            id,
            name: p.name.clone(),
            description: (!p.description.is_empty()).then(|| p.description.clone()),
            pattern,
        });
        records.insert(key, (Some(id), p.handle));
    }
    validate_ifccad_line_patterns(&definitions).map_err(|e| invalid(format!("{e:?}")))?;
    let lookup = SourcePatterns { records };
    for l in doc.layers.iter() {
        if lookup.resolve(&l.line_type, None)?.is_none() {
            return Err(invalid("layer requires ordinary line pattern"));
        }
    }
    for e in doc.entities() {
        if preserve_splines && matches!(e, opencadcodec::EntityType::Spline(_)) {
            continue;
        }
        scale(e.common().linetype_scale)?;
        lookup.resolve(&e.common().linetype, e.common().linetype_handle)?;
    }
    Ok((definitions, lookup))
}

pub(crate) fn allocate(
    doc: &mut CadDocument,
    defs: &[IfccadLinePattern],
    map: &mut IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
    symbols: &BTreeMap<IfccadSymbolSourceId, Handle>,
) -> Result<(), IfccadConversionError> {
    let mut keys = BTreeSet::new();
    for p in defs {
        if !keys.insert(opencadcodec::tables::normalize_name(&p.name)) {
            return Err(invalid("target line pattern lookup collision"));
        }
    }
    if !keys.contains("CONTINUOUS") {
        issues.push(modification("line-pattern-scaffold","drawing.linePatterns","Continuous definition added for the CAD target scaffold; native reader does not synthesize it"));
    }
    for p in defs {
        let mut record = LineType::new(&p.name);
        record.handle = doc
            .line_types
            .get(&p.name)
            .map(|r| r.handle)
            .unwrap_or_else(|| doc.allocate_handle());
        record.description = p.description.clone().unwrap_or_default();
        let mut fallback = false;
        for element in &p.pattern {
            let complex = if let Some(marker) = &element.marker {
                let result = to_marker(marker, map, symbols)?;
                if result.is_none() {
                    fallback = true;
                }
                result
            } else {
                None
            };
            record.elements.push(opencadcodec::tables::LineTypeElement {
                length: element.length,
                complex,
            });
        }
        if fallback {
            record.elements.clear();
            issues.push(modification("line-pattern-complex",format!("linePattern/{}",p.id.0),"unsupported active marker dependency: entire definition replaced with named continuous"));
        }
        record.pattern_length = record.elements.iter().map(|e| e.length.abs()).sum();
        map.line_patterns.insert(p.id.0, record.handle);
        doc.line_types.add_or_replace(record);
    }
    Ok(())
}
pub(crate) fn target(
    doc: &CadDocument,
    map: &IfccadMappings,
    id: IfccadLinePatternId,
) -> (String, Handle) {
    let h = map
        .line_patterns
        .cad_handle(id.0)
        .expect("allocated pattern");
    (
        doc.line_types
            .iter()
            .find(|p| p.handle == h)
            .unwrap()
            .name
            .clone(),
        h,
    )
}

#[cfg(test)]
mod error_tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn pattern_allocation_exhaustion_retains_domain_and_source() {
        let mut ids = IfccadIdCounters {
            next_symbol_source_id: 1,
            next_preservation_record_id: 1,
            next_line_pattern_id: u64::MAX,
            ..Default::default()
        };
        let error = from_cad(
            &CadDocument::new(),
            false,
            &mut ids,
            &mut Vec::new(),
            &IfccadMappings::default(),
            &super::super::symbol_source::SourceSymbolSources {
                by_handle: BTreeMap::new(),
            },
        )
        .err()
        .expect("exhausted pattern IDs");
        let IfccadConversionError::IdAllocation(source) = &error else {
            panic!("expected typed pattern allocation failure");
        };
        assert_eq!(source.domain, IfccadIdDomain::LinePattern);
        assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<IfccadIdAllocationError>(),
            Some(source)
        );
        assert_eq!(ids.next_line_pattern_id, u64::MAX);
    }
}
