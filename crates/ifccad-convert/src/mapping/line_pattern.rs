use crate::{
    diagnostics::{diagnostic, modification},
    *,
};
use ocdraw::ifccad::*;
use opencadcodec::{CadDocument, Handle, LineType};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct SourcePatterns {
    records: BTreeMap<String, (Option<IfccadLinePatternId>, Handle)>,
}
fn invalid(message: impl Into<String>) -> IfccadConversionError {
    IfccadConversionError::InvalidStructure(message.into())
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
    ids: &mut IfccadIdCounters,
    issues: &mut Vec<IfccadDiagnostic>,
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
        let pattern = if p.is_complex() {
            let mut kinds = BTreeSet::new();
            for e in &p.elements {
                if let Some(c) = &e.complex {
                    kinds.insert(match c.content {
                        opencadcodec::tables::LineTypeComplexContent::Text { .. } => "text",
                        opencadcodec::tables::LineTypeComplexContent::Shape { .. } => "shape",
                    });
                }
            }
            issues.push(modification("line-pattern-complex",&loc,format!("entire complex pattern ({}) replaced with empty pattern; name, description and references retained",kinds.into_iter().collect::<Vec<_>>().join(", "))));
            Vec::new()
        } else {
            let lengths = p.elements.iter().map(|e| e.length).collect::<Vec<_>>();
            let period = lengths.iter().map(|v| v.abs()).sum::<f64>();
            if p.pattern_length != period {
                issues.push(modification(
                    "line-pattern-period",
                    &loc,
                    "declared pattern period replaced with derived element-length sum",
                ));
            }
            lengths
        };
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
        record.elements = p
            .pattern
            .iter()
            .map(|&length| opencadcodec::tables::LineTypeElement {
                length,
                complex: None,
            })
            .collect();
        record.pattern_length = p.pattern.iter().map(|v| v.abs()).sum();
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
            next_line_pattern_id: u64::MAX,
            ..Default::default()
        };
        let error = from_cad(&CadDocument::new(), &mut ids, &mut Vec::new())
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
