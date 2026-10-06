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

fn invalid(message: impl Into<String>) -> CadToOcdrawError {
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
        let pattern = if complex {
            vec![]
        } else {
            row.elements.iter().map(|e| e.length).collect()
        };
        let id = builder.add_line_pattern(LinePatternDefinition {
            name: row.name.clone(),
            description: (!row.description.is_empty()).then(|| row.description.clone()),
            pattern,
        })?;
        if map.names.insert(key, id).is_some() {
            return Err(invalid("duplicate CAD line pattern name"));
        }
        let mut reasons = Vec::new();
        if complex {
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
) -> Result<ImportLinePatternMap, OcdrawToCadError> {
    let mut map = BTreeMap::new();
    for row in &source.line_patterns {
        let mut native = opencadcodec::LineType::new(&row.name);
        native.description = row.description.clone().unwrap_or_default();
        native.elements = row
            .pattern
            .iter()
            .map(|v| opencadcodec::tables::LineTypeElement {
                length: *v,
                complex: None,
            })
            .collect();
        native.pattern_length = row.pattern.iter().map(|v| v.abs()).sum();
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
