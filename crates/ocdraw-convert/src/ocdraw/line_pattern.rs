//! CAD definition allocation and reference resolution, independent of JSON.
use super::export::DirectExportError;
use super::import::DirectImportError;
use crate::source::{ExportAction, ExportDiagnostic, ExportDiagnosticSource, ExportLossReason};
use cadcodec::{CadDocument, Handle};
use ocdraw::ocdraw::{
    AppearanceSelection, DrawingBuilder, LinePatternDefinition, LinePatternId, OcdrawDocument,
};
use std::collections::BTreeMap;

pub(super) struct ExportLinePatternMap {
    names: BTreeMap<String, LinePatternId>,
    handles: BTreeMap<Handle, String>,
}

fn invalid(message: impl Into<String>) -> DirectExportError {
    DirectExportError::InvalidSourceStructure {
        problems: vec![
            crate::source::SourceStructureProblem::InconsistentRelationship {
                description: message.into(),
            },
        ],
    }
}

impl ExportLinePatternMap {
    pub(super) fn resolve(
        &self,
        name: &str,
        handle: Option<Handle>,
    ) -> Result<AppearanceSelection<LinePatternId>, DirectExportError> {
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
    pub(super) fn layer(&self, name: &str) -> Result<LinePatternId, DirectExportError> {
        match self.resolve(name, None)? {
            AppearanceSelection::Explicit(id) => Ok(id),
            _ => Err(invalid("layer requires an explicit line pattern")),
        }
    }
}

pub(super) fn export_line_patterns(
    source: &CadDocument,
    builder: &mut DrawingBuilder,
    diagnostics: &mut Vec<ExportDiagnostic>,
) -> Result<ExportLinePatternMap, DirectExportError> {
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
                diagnostics.push(ExportDiagnostic::loss(
                    ExportDiagnosticSource::Table {
                        kind: format!("line_types/{}", row.name),
                    },
                    ExportAction::PartiallyExported,
                    vec![ExportLossReason::UnsupportedSemantic {
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
                        cadcodec::tables::LineTypeComplexContent::Text { .. } => text = true,
                        cadcodec::tables::LineTypeComplexContent::Shape { .. } => shapes = true,
                    }
                }
            }
            reasons.push(ExportLossReason::ComplexLinePatternFallback {
                name: row.name.clone(),
                text,
                shapes,
            });
        }
        if row.xref_dependent || row.xref_block_record_handle != Handle::NULL {
            reasons.push(ExportLossReason::UnsupportedSemantic {
                name: "line pattern xref provenance".into(),
            });
        }
        if !reasons.is_empty() {
            diagnostics.push(ExportDiagnostic::loss(
                ExportDiagnosticSource::Table {
                    kind: format!("line_types/{}", row.name),
                },
                ExportAction::PartiallyExported,
                reasons,
            ));
        }
    }
    builder.set_line_pattern_scale(source.header.linetype_scale)?;
    Ok(map)
}

pub(super) type ImportLinePatternMap = BTreeMap<LinePatternId, (String, Handle)>;
pub(super) fn import_line_patterns(
    source: &OcdrawDocument,
    target: &mut CadDocument,
) -> Result<ImportLinePatternMap, DirectImportError> {
    let mut map = BTreeMap::new();
    for row in &source.line_patterns {
        let mut native = cadcodec::LineType::new(&row.name);
        native.description = row.description.clone().unwrap_or_default();
        native.elements = row
            .pattern
            .iter()
            .map(|v| cadcodec::tables::LineTypeElement {
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
                .map_err(|e| DirectImportError::Cad(format!("line pattern: {e}")))?;
            handle
        };
        map.insert(row.id, (row.name.clone(), handle));
    }
    target.header.linetype_scale = source.line_pattern_scale;
    Ok(map)
}
