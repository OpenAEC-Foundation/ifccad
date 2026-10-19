//! CAD definition allocation and reference resolution, independent of JSON.
use crate::source::{
    CadToOcdrawAction, CadToOcdrawDiagnostic, CadToOcdrawDiagnosticSource, CadToOcdrawLossReason,
};
use crate::CadToOcdrawError;
use crate::OcdrawToCadError;
use ocdraw::ocdraw::{
    AppearanceSelection, LinePatternDefinition, LinePatternId, OcdrawBuilder, OcdrawDocument,
};
use opencadcodec::{CadDocument, Handle};
use std::collections::BTreeMap;

pub(crate) struct ExportLinePatternMap {
    names: BTreeMap<String, LinePatternId>,
    handles: BTreeMap<Handle, String>,
}

pub(super) fn invalid(message: impl Into<String>) -> CadToOcdrawError {
    CadToOcdrawError::InvalidSourceStructure {
        problems: vec![
            crate::source::CadSourceStructureProblem::InconsistentRelationship {
                description: message.into(),
            },
        ],
    }
}

impl ExportLinePatternMap {
    /// Opaque common values may retain an unresolved reference. A known
    /// name/handle contradiction still violates source structure.
    pub(crate) fn resolve_preserved(
        &self,
        name: &str,
        handle: Option<Handle>,
    ) -> Result<Option<AppearanceSelection<LinePatternId>>, CadToOcdrawError> {
        let key = if name.is_empty() {
            "BYLAYER".into()
        } else {
            name.to_uppercase()
        };
        if let Some(handle) = handle.filter(|h| *h != Handle::NULL) {
            let Some(target) = self.handles.get(&handle) else {
                return Ok(None);
            };
            if target != &key {
                return Err(invalid("contradictory line pattern name and handle"));
            }
        }
        Ok(match key.as_str() {
            "BYLAYER" => Some(AppearanceSelection::ByLayer),
            "BYBLOCK" => Some(AppearanceSelection::ByBlock),
            _ => self
                .names
                .get(&key)
                .copied()
                .map(AppearanceSelection::Explicit),
        })
    }
    pub(crate) fn resolve(
        &self,
        name: &str,
        handle: Option<Handle>,
    ) -> Result<AppearanceSelection<LinePatternId>, CadToOcdrawError> {
        let key = if name.is_empty() {
            "BYLAYER".into()
        } else {
            name.to_uppercase()
        };
        if let Some(handle) = handle.filter(|h| *h != Handle::NULL) {
            let target = self
                .handles
                .get(&handle)
                .ok_or_else(|| invalid("missing line pattern handle"))?;
            if target != &key {
                return Err(invalid("contradictory line pattern name and handle"));
            }
        }
        match key.as_str() {
            "BYLAYER" => Ok(AppearanceSelection::ByLayer),
            "BYBLOCK" => Ok(AppearanceSelection::ByBlock),
            _ => self
                .names
                .get(&key)
                .copied()
                .map(AppearanceSelection::Explicit)
                .ok_or_else(|| invalid(format!("missing line pattern {name}"))),
        }
    }
    pub(crate) fn layer(&self, name: &str) -> Result<LinePatternId, CadToOcdrawError> {
        match self.resolve(name, None)? {
            AppearanceSelection::Explicit(id) => Ok(id),
            _ => Err(invalid("layer requires an explicit line pattern")),
        }
    }
}

pub(crate) fn export_line_patterns(
    source: &CadDocument,
    builder: &mut OcdrawBuilder,
    diagnostics: &mut Vec<CadToOcdrawDiagnostic>,
    styles: &super::text::SourceTextStyles,
    symbols: &super::symbol_source::ExportSymbolSourceMap,
) -> Result<ExportLinePatternMap, CadToOcdrawError> {
    let mut map = ExportLinePatternMap {
        names: BTreeMap::new(),
        handles: BTreeMap::new(),
    };
    for row in source.line_types.iter() {
        let key = row.name.to_uppercase();
        if row.handle != Handle::NULL && map.handles.insert(row.handle, key.clone()).is_some() {
            return Err(invalid("duplicate line pattern handle"));
        }
        if row.alignment != 'A'
            || !row.pattern_length.is_finite()
            || row.elements.iter().any(|e| !e.length.is_finite())
        {
            return Err(invalid("invalid CAD line pattern"));
        }
        if matches!(key.as_str(), "BYLAYER" | "BYBLOCK") {
            if !row.elements.is_empty()
                || row.pattern_length != 0.0
                || !row.description.is_empty()
                || row.xref_dependent
                || row.xref_block_record_handle != Handle::NULL
            {
                diagnostics.push(CadToOcdrawDiagnostic::loss(
                    CadToOcdrawDiagnosticSource::Table {
                        kind: format!("line_types/{}", row.name),
                    },
                    CadToOcdrawAction::PartiallyExported,
                    vec![CadToOcdrawLossReason::UnsupportedSemantic {
                        name: "modified inherited line pattern scaffolding".into(),
                    }],
                ));
            }
            continue;
        }
        let complex = row.is_complex();
        if complex && row.name.eq_ignore_ascii_case("Continuous") {
            return Err(invalid("Continuous cannot carry complex content"));
        }
        let raw = row
            .elements
            .iter()
            .map(|e| ocdraw::line_pattern::LinePatternElement {
                length: e.length,
                marker: e.complex.as_ref().map(|_| ()),
            })
            .collect::<Vec<_>>();
        ocdraw::line_pattern::validate_line_pattern_values(&raw)
            .map_err(|e| invalid(e.to_string()))?;
        let mut pattern = Vec::new();
        let mut fallback = false;
        for element in &row.elements {
            let marker = if let Some(value) = &element.complex {
                let style = source
                    .text_styles
                    .iter()
                    .find(|s| !value.style_handle.is_null() && s.handle == value.style_handle)
                    .ok_or_else(|| invalid("missing marker STYLE handle"))?;
                if style.is_shape_file != value.is_shape() {
                    return Err(invalid("marker STYLE has wrong role"));
                }
                let prepared = cad_presentation_convert::marker_values_from_cad(value);
                let prepared = match prepared {
                    Ok(v) => Some(v),
                    Err(cad_presentation_convert::CadLinePatternValueError::Unsupported(_)) => {
                        fallback = true;
                        None
                    }
                    Err(e) => return Err(invalid(e.to_string())),
                };
                if let Some(prepared) = prepared {
                    use ocdraw::{line_pattern::*, ocdraw::DrawingLinePatternMarker};
                    let transform = LinePatternTransform {
                        scale: prepared.scale,
                        offset: prepared.offset,
                        rotation: if prepared.absolute_rotation {
                            LinePatternRotation::Absolute(prepared.rotation)
                        } else {
                            LinePatternRotation::Relative(prepared.rotation)
                        },
                    };
                    match prepared.content {
                        cad_presentation_convert::CadLinePatternMarkerContent::Text(text) => {
                            match styles.handles.get(&value.style_handle) {
                                Some(id) => Some(DrawingLinePatternMarker::Text {
                                    text,
                                    text_style_id: *id,
                                    transform,
                                }),
                                None => {
                                    fallback = true;
                                    None
                                }
                            }
                        }
                        cad_presentation_convert::CadLinePatternMarkerContent::Shape(
                            shape_number,
                        ) => match symbols
                            .by_handle
                            .get(&value.style_handle)
                            .copied()
                            .flatten()
                        {
                            Some(id) => Some(DrawingLinePatternMarker::Symbol {
                                symbol_source_id: id,
                                shape_number,
                                transform,
                            }),
                            None => {
                                fallback = true;
                                None
                            }
                        },
                    }
                } else {
                    None
                }
            } else {
                None
            };
            pattern.push(ocdraw::line_pattern::LinePatternElement {
                length: element.length,
                marker,
            });
        }
        if fallback {
            pattern.clear();
        }
        let id = builder.add_line_pattern(LinePatternDefinition {
            name: row.name.clone(),
            description: (!row.description.is_empty()).then(|| row.description.clone()),
            pattern,
        })?;
        if map.names.insert(key, id).is_some() {
            return Err(invalid("duplicate CAD line pattern name"));
        }
        let mut reasons = Vec::new();
        if row.pattern_length != row.elements.iter().map(|e| e.length.abs()).sum::<f64>() {
            reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
                name: "inconsistent CAD cached pattern length replaced by element period".into(),
            });
        }
        if complex && fallback {
            if row.name.eq_ignore_ascii_case("Continuous") {
                return Err(invalid("Continuous cannot carry complex content"));
            }
            let mut text = false;
            let mut shapes = false;
            for e in &row.elements {
                if let Some(c) = &e.complex {
                    match c.content {
                        opencadcodec::tables::LineTypeComplexContent::Text { .. } => text = true,
                        opencadcodec::tables::LineTypeComplexContent::Shape { .. } => shapes = true,
                    }
                }
            }
            reasons.push(CadToOcdrawLossReason::ComplexLinePatternFallback {
                name: row.name.clone(),
                text,
                shapes,
            });
        }
        if row.xref_dependent || row.xref_block_record_handle != Handle::NULL {
            reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
                name: "line pattern xref provenance".into(),
            });
        }
        if !reasons.is_empty() {
            diagnostics.push(CadToOcdrawDiagnostic::loss(
                CadToOcdrawDiagnosticSource::Table {
                    kind: format!("line_types/{}", row.name),
                },
                CadToOcdrawAction::PartiallyExported,
                reasons,
            ));
        }
    }
    builder.set_line_pattern_scale(source.header.linetype_scale)?;
    Ok(map)
}

pub(crate) type ImportLinePatternMap = BTreeMap<LinePatternId, (String, Handle)>;
pub(crate) fn import_line_patterns(
    source: &OcdrawDocument,
    target: &mut CadDocument,
    styles: &super::text::TargetTextStyles,
    symbols: &BTreeMap<ocdraw::ocdraw::SymbolSourceId, Handle>,
    diagnostics: &mut Vec<crate::OcdrawToCadDiagnostic>,
) -> Result<ImportLinePatternMap, OcdrawToCadError> {
    let mut map = BTreeMap::new();
    for row in &source.line_patterns {
        let mut native = opencadcodec::LineType::new(&row.name);
        native.description = row.description.clone().unwrap_or_default();
        let mut fallback = false;
        for element in &row.pattern {
            let complex = if let Some(marker) = &element.marker {
                use cad_presentation_convert::*;
                use ocdraw::{line_pattern::LinePatternRotation, ocdraw::DrawingLinePatternMarker};
                let (content, handle, transform) = match marker {
                    DrawingLinePatternMarker::Text {
                        text,
                        text_style_id,
                        transform,
                    } => (
                        CadLinePatternMarkerContent::Text(text.clone()),
                        styles.handles.get(text_style_id),
                        transform,
                    ),
                    DrawingLinePatternMarker::Symbol {
                        symbol_source_id,
                        shape_number,
                        transform,
                    } => (
                        CadLinePatternMarkerContent::Shape(*shape_number),
                        symbols.get(symbol_source_id),
                        transform,
                    ),
                };
                if let Some(handle) = handle {
                    let (rotation, absolute_rotation) = match transform.rotation {
                        LinePatternRotation::Relative(v) => (v, false),
                        LinePatternRotation::Absolute(v) => (v, true),
                    };
                    match marker_values_to_cad(
                        &CadLinePatternMarkerValues {
                            content,
                            scale: transform.scale,
                            offset: transform.offset,
                            rotation,
                            absolute_rotation,
                        },
                        *handle,
                    ) {
                        Ok(v) => Some(v),
                        Err(CadLinePatternValueError::Unsupported(_)) => {
                            fallback = true;
                            None
                        }
                        Err(e) => return Err(OcdrawToCadError::Cad(e.to_string())),
                    }
                } else {
                    fallback = true;
                    None
                }
            } else {
                None
            };
            native.elements.push(opencadcodec::tables::LineTypeElement {
                length: element.length,
                complex,
            });
        }
        if fallback {
            native.elements.clear();
            diagnostics.push(crate::OcdrawToCadDiagnostic {code:"COMPLEX_LINE_PATTERN_FALLBACK",location:format!("/linePatterns/{}",row.id.0),message:"Unsupported active marker dependency: whole definition becomes named continuous".into()});
        }
        native.pattern_length = native.elements.iter().map(|v| v.length.abs()).sum();
        let handle = if let Some(existing) = target.line_types.get_mut(&row.name) {
            native.handle = existing.handle;
            *existing = native;
            existing.handle
        } else {
            let handle = target.allocate_handle();
            native.handle = handle;
            target
                .line_types
                .add(native)
                .map_err(|e| OcdrawToCadError::Cad(format!("line pattern: {e}")))?;
            handle
        };
        map.insert(row.id, (row.name.clone(), handle));
    }
    target.header.linetype_scale = source.line_pattern_scale;
    Ok(map)
}
