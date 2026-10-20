//! Report-only diagnosis using the production CAD conversion pipeline.
mod report;

use crate::*;
use opencadcodec::{CadDocument, EntityType, Handle};
use serde::Serialize;
use serde_json::{json, Value};
use std::{collections::BTreeMap, error::Error, io};

/// A check result, never a claim that all source semantics are represented.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CadAuditStatus {
    Passed,
    Failed,
    Blocked,
    Omitted,
    NotApplicable,
}

/// Exact CAD identity and source ownership, independent of generated native IDs.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CadAuditSource {
    pub handle: String,
    pub kind: String,
    pub owner_handle: String,
    pub owner_name: Option<String>,
    pub layout_name: Option<String>,
    pub layer: String,
}

/// A completed check. Errors retain both readable and typed-debug evidence.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CadAuditEvent {
    pub phase: String,
    pub status: CadAuditStatus,
    pub source: Option<CadAuditSource>,
    pub code: String,
    pub message: String,
    pub detail: Value,
}

/// Preparation coverage for one stored source entity.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CadAuditEntity {
    #[serde(flatten)]
    pub source: CadAuditSource,
    pub status: CadAuditStatus,
    pub reason: String,
}

/// A diagnostic inventory. Contains no partial IFCCAD output.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CadToIfccadAuditReport {
    pub version: u32,
    /// All scopes eligible for conversion were traversed. Unsupported scopes
    /// remain explicitly omitted; a traversal is not exhaustive validation.
    pub traversal_complete: bool,
    /// The normal logical conversion checks passed under the supplied policy.
    /// Encoding/readback and complete source fidelity are separate checks.
    pub conversion_ready: bool,
    pub entities: Vec<CadAuditEntity>,
    pub events: Vec<CadAuditEvent>,
    pub diagnostics: Vec<IfccadDiagnostic>,
}
impl CadToIfccadAuditReport {
    pub fn entity(&self, handle: Handle) -> Option<&CadAuditEntity> {
        let handle = handle.to_string();
        self.entities.iter().find(|e| e.source.handle == handle)
    }
    pub fn failures(&self) -> impl Iterator<Item = &CadAuditEvent> {
        self.events
            .iter()
            .filter(|e| e.status == CadAuditStatus::Failed)
    }
}

/// Diagnose without mutating or excluding source entities. Default production
/// options apply; the report distinguishes omissions from hard failures.
pub fn diagnose_cad_document(
    source: &CadDocument,
    metadata: IfccadTargetMetadata,
    options: CadToIfccadOptions,
) -> CadToIfccadAuditReport {
    diagnose_cad_document_with_progress(source, metadata, options, &mut |_| Ok(()))
        .expect("infallible audit observer")
}

/// As [`diagnose_cad_document`], streaming completed checks to a fallible observer.
/// Observer IO failure stops traversal and is returned, never reported as success.
pub fn diagnose_cad_document_with_progress(
    source: &CadDocument,
    metadata: IfccadTargetMetadata,
    options: CadToIfccadOptions,
    observer: &mut dyn FnMut(&CadAuditEvent) -> io::Result<()>,
) -> io::Result<CadToIfccadAuditReport> {
    let mut run = AuditRun::new(source, Some(observer));
    match crate::from_cad::convert(source, metadata, options, &mut run) {
        Ok(Some(output)) => {
            run.report.diagnostics = output.diagnostics;
            run.report.conversion_ready = true;
            run.stage(
                "finalization",
                CadAuditStatus::Passed,
                "logical-validation",
                "production logical validation passed",
            )
            .map_err(observer_error)?;
        }
        Ok(None) => {}
        Err(IfccadConversionError::AuditProgress(e)) => return Err(e),
        Err(error) => {
            let phase = run.phase;
            run.failure(phase, None, &error, None)
                .map_err(observer_error)?;
            run.stage("remaining-checks", CadAuditStatus::Blocked, "dependency-failed", "pipeline stopped at a shared prerequisite; unvisited entities and later checks are not certified")
                .map_err(observer_error)?;
        }
    }
    Ok(run.report)
}
fn observer_error(error: IfccadConversionError) -> io::Error {
    match error {
        IfccadConversionError::AuditProgress(e) => e,
        e => io::Error::other(e.to_string()),
    }
}

type AuditObserver<'a> = dyn FnMut(&CadAuditEvent) -> io::Result<()> + 'a;

pub(crate) struct AuditRun<'a> {
    pub enabled: bool,
    pub partial: bool,
    rejected: std::collections::BTreeSet<Handle>,
    pub phase: &'static str,
    pub report: CadToIfccadAuditReport,
    index: BTreeMap<Handle, usize>,
    observer: Option<&'a mut AuditObserver<'a>>,
}
impl AuditRun<'_> {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            partial: false,
            rejected: Default::default(),
            phase: "source-context",
            index: BTreeMap::new(),
            observer: None,
            report: CadToIfccadAuditReport {
                version: 1,
                traversal_complete: false,
                conversion_ready: false,
                entities: vec![],
                events: vec![],
                diagnostics: vec![],
            },
        }
    }
    fn new<'a>(source: &CadDocument, observer: Option<&'a mut AuditObserver<'a>>) -> AuditRun<'a> {
        let mut entities = Vec::new();
        let mut index = BTreeMap::new();
        for entity in source.entities() {
            let c = entity.common();
            let marker = matches!(entity, EntityType::Block(_) | EntityType::BlockEnd(_));
            index.insert(c.handle, entities.len());
            entities.push(CadAuditEntity {
                source: CadAuditSource {
                    handle: c.handle.to_string(),
                    kind: entity.as_entity().entity_type().into(),
                    owner_handle: c.owner_handle.to_string(),
                    owner_name: source
                        .block_records
                        .iter()
                        .find(|b| b.handle == c.owner_handle)
                        .map(|b| b.name.clone()),
                    layout_name: source.objects.values().find_map(|o| match o {
                        opencadcodec::objects::ObjectType::Layout(l)
                            if l.block_record == c.owner_handle =>
                        {
                            Some(l.name.clone())
                        }
                        _ => None,
                    }),
                    layer: c.layer.clone(),
                },
                status: if marker {
                    CadAuditStatus::NotApplicable
                } else {
                    CadAuditStatus::Blocked
                },
                reason: if marker {
                    "structural marker: checked by source-context, no drawable preparation"
                } else {
                    "prerequisite not completed or scope not reached"
                }
                .into(),
            });
        }
        AuditRun {
            enabled: observer.is_some(),
            partial: false,
            rejected: Default::default(),
            phase: "source-context",
            index,
            observer,
            report: CadToIfccadAuditReport {
                version: 1,
                traversal_complete: false,
                conversion_ready: false,
                entities,
                events: vec![],
                diagnostics: vec![],
            },
        }
    }
    pub fn entity_failed(&self, handle: Handle) -> bool {
        if self.partial {
            return self.rejected.contains(&handle);
        }
        self.enabled
            && self
                .index
                .get(&handle)
                .is_some_and(|i| self.report.entities[*i].status == CadAuditStatus::Failed)
    }
    pub fn partial(source: &CadDocument) -> Self {
        let mut run = Self::new(source, None);
        run.enabled = true;
        run.partial = true;
        run
    }
    pub fn partial_report(&self) -> Option<CadPartialOpenReport> {
        self.partial.then(|| CadPartialOpenReport {
            entities: self
                .report
                .entities
                .iter()
                .filter(|e| e.status == CadAuditStatus::Omitted)
                .cloned()
                .collect(),
            errors: self
                .report
                .events
                .iter()
                .filter(|e| e.detail.get("debug").is_some())
                .cloned()
                .collect(),
        })
    }
    pub fn validate_entities(&mut self, source: &CadDocument) -> Result<(), IfccadConversionError> {
        for entity in source.entities() {
            if let Err(error) = crate::source::entities::validate_source(entity) {
                self.failure(
                    "entity-preparation",
                    Some(entity.common().handle),
                    &error,
                    None,
                )?;
            }
        }
        Ok(())
    }
    pub fn source(&self, handle: Handle) -> Option<CadAuditSource> {
        self.index
            .get(&handle)
            .map(|i| self.report.entities[*i].source.clone())
    }
    fn emit(&mut self, event: CadAuditEvent) -> Result<(), IfccadConversionError> {
        if self.enabled {
            if self.partial && event.status != CadAuditStatus::Omitted {
                return Ok(());
            }
            if let Some(observer) = &mut self.observer {
                observer(&event).map_err(IfccadConversionError::AuditProgress)?;
            }
            self.report.events.push(event);
        }
        Ok(())
    }
    pub fn stage(
        &mut self,
        phase: &str,
        status: CadAuditStatus,
        code: &str,
        message: &str,
    ) -> Result<(), IfccadConversionError> {
        self.emit(CadAuditEvent {
            phase: phase.into(),
            status,
            source: None,
            code: code.into(),
            message: message.into(),
            detail: Value::Null,
        })
    }
    pub fn entity_finished(
        &mut self,
        handle: Handle,
        status: CadAuditStatus,
        reason: &str,
    ) -> Result<(), IfccadConversionError> {
        if !self.enabled {
            return Ok(());
        }
        let entry = &mut self.report.entities[self.index[&handle]];
        // A failure may not be overwritten by a later scope completion.
        if entry.status == CadAuditStatus::Failed || self.rejected.contains(&handle) {
            return Ok(());
        }
        entry.status = status;
        entry.reason = reason.into();
        self.emit(CadAuditEvent {
            phase: "entity-preparation".into(),
            status,
            source: self.source(handle),
            code: "entity-preparation".into(),
            message: reason.into(),
            detail: Value::Null,
        })
    }
    pub fn failure(
        &mut self,
        phase: &str,
        handle: Option<Handle>,
        error: &IfccadConversionError,
        preparation: Option<cad_geometry_convert::CadPreparationError>,
    ) -> Result<(), IfccadConversionError> {
        if !self.enabled {
            return Ok(());
        }
        let status = if self.partial {
            CadAuditStatus::Omitted
        } else {
            CadAuditStatus::Failed
        };
        if self.partial {
            if phase != "entity-preparation" && phase != "block-occurrence" {
                return Err(IfccadConversionError::InvalidStructure(
                    "partial recovery requires a located entity transaction".into(),
                ));
            }
            if let Some(h) = handle {
                self.rejected.insert(h);
            }
        }
        if let Some(index) = handle
            .filter(|_| phase == "entity-preparation" || self.partial)
            .and_then(|h| self.index.get(&h))
        {
            self.report.entities[*index].status = status;
            self.report.entities[*index].reason = error.to_string();
        }
        let (code, mut detail) = error_detail(error);
        if let Some(preparation) = preparation {
            detail["preparationCause"] = json!(format!("{preparation:?}"));
        }
        detail["debug"] = json!(format!("{error:?}"));
        let mut chain = vec![];
        let mut cause = error.source();
        while let Some(e) = cause {
            chain.push(e.to_string());
            cause = e.source();
        }
        detail["causes"] = json!(chain);
        self.emit(CadAuditEvent {
            phase: phase.into(),
            status,
            source: handle.and_then(|h| self.source(h)),
            code,
            message: error.to_string(),
            detail,
        })
    }
    pub fn failed(&self) -> bool {
        self.report.failures().next().is_some()
    }

    pub fn finish_scope(
        &mut self,
        handles: &[Handle],
        mappings: &IfccadMappings,
    ) -> Result<(), IfccadConversionError> {
        if !self.enabled {
            return Ok(());
        }
        for h in handles {
            if self.report.entities[self.index[h]].status != CadAuditStatus::Blocked {
                continue;
            }
            if mappings.entities.ifccad_id(*h).is_some() {
                self.entity_finished(
                    *h,
                    CadAuditStatus::Passed,
                    "prepared by production conversion",
                )?;
            } else {
                self.entity_finished(
                    *h,
                    CadAuditStatus::Omitted,
                    "not represented by the current conversion profile; consult loss diagnostics",
                )?;
            }
        }
        Ok(())
    }

    pub fn assess_occurrences(
        &mut self,
        source: &CadDocument,
        drawing: &ifccad::IfccadDocument,
        mappings: &IfccadMappings,
        geometry: &mut crate::geometry_context::GeometryContext,
        issues: &mut Vec<IfccadDiagnostic>,
    ) -> Result<(), IfccadConversionError> {
        use std::collections::BTreeSet;
        let members = source
            .block_records
            .iter()
            .map(|b| {
                (
                    b.handle.value(),
                    b.entity_handles
                        .iter()
                        .map(|h| h.value())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        // Use original membership, not the incomplete target membership: a failed
        // descendant must never disappear from a successful occurrence proof.
        let failed_definitions = source
            .block_records
            .iter()
            .filter(|b| !b.is_model_space() && !b.is_paper_space())
            .filter(|b| {
                b.entity_handles.iter().any(|h| {
                    self.index.get(h).is_some_and(|i| {
                        matches!(
                            self.report.entities[*i].status,
                            CadAuditStatus::Failed | CadAuditStatus::Blocked
                        )
                    })
                })
            })
            .map(|b| b.handle.value())
            .collect::<BTreeSet<_>>();
        let affected = geometry
            .state
            .affected_instances(&members, &failed_definitions);
        let blocked_roots = affected
            .values()
            .flatten()
            .copied()
            .collect::<BTreeSet<_>>();
        let scopes = std::iter::once((
            IfccadGeometryOwner::ModelLayout(drawing.model.id),
            &drawing.model.entities,
        ))
        .chain(
            drawing
                .blocks
                .iter()
                .map(|b| (IfccadGeometryOwner::BlockDefinition(b.id), &b.entities)),
        )
        .chain(
            drawing
                .paper_layouts
                .iter()
                .map(|p| (IfccadGeometryOwner::PaperLayout(p.id), &p.entities)),
        );
        for (owner, entities) in scopes {
            geometry.select(owner);
            for e in entities {
                if !e.as_native().is_some_and(|n| {
                    matches!(n.kind, ifccad::IfccadEntityKind::BlockInstance { .. })
                }) {
                    continue;
                }
                let h = mappings
                    .entities
                    .cad_handle(e.id())
                    .expect("prepared root mapping");
                if blocked_roots.contains(&h.value()) {
                    self.emit(CadAuditEvent {
                        phase: "block-occurrence".into(),
                        status: CadAuditStatus::Blocked,
                        source: self.source(h),
                        code: "dependency-failed".into(),
                        message: "referenced definition contains a failed or unvisited entity, including through nested blocks".into(),
                        detail: json!({"failedDefinitions": affected.iter()
                            .filter(|(_,roots)|roots.contains(&h.value()))
                            .map(|(key,_)|Handle::new(*key).to_string()).collect::<Vec<_>>()})
                    })?;
                    continue;
                }
                match geometry.assess_roots(&members, &[h.value()], issues) {
                    Ok(()) => self.emit(CadAuditEvent {
                        phase: "block-occurrence".into(),
                        status: CadAuditStatus::Passed,
                        source: self.source(h),
                        code: "occurrence-accuracy".into(),
                        message: "production occurrence check passed for represented content"
                            .into(),
                        detail: Value::Null,
                    })?,
                    Err(error) => self.failure("block-occurrence", Some(h), &error, None)?,
                }
            }
        }
        Ok(())
    }
}

fn error_detail(error: &IfccadConversionError) -> (String, Value) {
    use IfccadConversionError as E;
    match error {
        E::Geometry(g) => {
            let f = &g.failure;
            (
                format!("Geometry/{:?}/{:?}", f.stage, f.reason),
                json!({
                    "domain": format!("{:?}", g.domain), "coordinateMeaning": format!("{:?}", g.coordinate_meaning),
                    "reason": format!("{:?}", f.reason), "stage": format!("{:?}", f.stage), "vertexIndex": f.vertex_index,
                    "requestedTolerance": format!("{:?}", f.requested_tolerance),
                    "resolvedTolerance": f.resolved_tolerance.map(|t| json!({"lower":t.lower(),"upper":t.upper()})),
                    "deviation": f.deviation.map(|d| json!({"lower":d.lower(),"upper":d.upper()})),
                    "geometrySource": format!("{:?}", f.source),
                }),
            )
        }
        E::HatchPreparation {
            location,
            source: cad_geometry_convert::hatch::CadHatchPreparationError::Boundary(e),
        } => {
            let mut detail = json!({"location":location,"loopIndex":e.loop_index,"edgeIndex":e.edge_index,"reason":format!("{:?}",e.reason)});
            let reason = if let ifccad::geometry_kernel::hatch::HatchFailureReason::GapExceeded {
                lower,
                upper,
                limit,
            } = e.reason
            {
                detail["gap"] = json!({"lower":lower,"upper":upper,"limit":limit});
                "GapExceeded"
            } else {
                "Boundary"
            };
            (format!("HatchPreparation/{reason}"), detail)
        }
        E::DimensionPreparation { .. } => ("DimensionPreparation".into(), json!({})),
        E::HatchPreparation { .. } => ("HatchPreparation".into(), json!({})),
        E::TextPreparation { .. } => ("TextPreparation".into(), json!({})),
        E::InvalidStructure(_) => ("InvalidStructure".into(), json!({})),
        E::Unsupported(_) => ("Unsupported".into(), json!({})),
        E::CoreValidation(_) => ("CoreValidation".into(), json!({})),
        E::PaperTolerance { .. } => ("PaperTolerance".into(), json!({})),
        E::Tolerance(_) | E::HatchTolerance(_) => ("Tolerance".into(), json!({})),
        E::AuditProgress(_) => ("AuditProgress".into(), json!({})),
        _ => ("SharedPreparation".into(), json!({})),
    }
}
