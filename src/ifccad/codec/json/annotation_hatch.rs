//! Saved Hatch scale and view presentations share the existing boundary/pattern values.
use super::hatch_values::BoundaryValue;
use super::*;
use crate::geometry_kernel::hatch::{HatchFill, HatchLinePattern};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Pattern {
    families: Value,
    scale: f64,
    orientation_reference: [f64; 3],
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Boundary {
    geometry: BoundaryValue,
    #[serde(default)]
    source_entity_ids: Vec<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum Loop {
    Shared {
        base_loop_index: u32,
        classification: IfccadHatchLoopClassification,
    },
    Local {
        classification: IfccadHatchLoopClassification,
        boundary: Boundary,
    },
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Presentation {
    pattern: Pattern,
    loops: Vec<Loop>,
}
fn failure(path: &str, message: impl ToString) -> IfccadReport {
    crate::ifccad::diagnostics::failure("IFCCAD-ANNOTATION-006", path, message.to_string())
}
fn reference(value: &str, prefix: &str, domain: &str, path: &str) -> Result<u64, IfccadReport> {
    let stem = format!("{prefix}/{domain}");
    let suffix = value
        .strip_prefix(&stem)
        .ok_or_else(|| failure(path, "misplaced annotation reference"))?;
    let id = suffix
        .parse::<u64>()
        .map_err(|_| failure(path, "invalid annotation reference"))?;
    if suffix != id.to_string() {
        return Err(failure(path, "noncanonical annotation reference"));
    }
    Ok(id)
}
impl Presentation {
    fn decode(
        self,
        prefix: &str,
        path: &str,
    ) -> Result<IfccadHatchAnnotationContext, IfccadReport> {
        let fill = super::hatch_pattern::decode(
            &json!({"kind":"linePattern","families":self.pattern.families}),
        )
        .map_err(|e| failure(path, e))?;
        let HatchFill::LinePattern(pattern) = fill else {
            unreachable!("requested line pattern")
        };
        let loops = self
            .loops
            .into_iter()
            .map(|l| {
                Ok(match l {
                    Loop::Shared {
                        base_loop_index,
                        classification,
                    } => IfccadHatchContextLoop::Shared {
                        base_loop_index,
                        classification,
                    },
                    Loop::Local {
                        classification,
                        boundary,
                    } => IfccadHatchContextLoop::Local {
                        classification,
                        boundary: IfccadHatchContextBoundary {
                            geometry: boundary.geometry.into(),
                            source_entity_ids: boundary
                                .source_entity_ids
                                .iter()
                                .map(|id| reference(id, prefix, "e", path))
                                .collect::<Result<_, _>>()?,
                        },
                    },
                })
            })
            .collect::<Result<_, IfccadReport>>()?;
        Ok(IfccadHatchAnnotationContext {
            pattern: IfccadHatchContextPattern {
                families: pattern.families,
                scale: self.pattern.scale,
                orientation_reference: self.pattern.orientation_reference,
            },
            loops,
        })
    }
    fn encode(p: IfccadHatchAnnotationContext, prefix: &str) -> Self {
        let fill = HatchFill::LinePattern(HatchLinePattern {
            name: None,
            description: None,
            origin: [0.; 2],
            rotation: 0.,
            scale: 1.,
            families: p.pattern.families,
        });
        let mut wire = super::hatch_pattern::encode(&fill);
        Self {
            pattern: Pattern {
                families: wire["families"].take(),
                scale: p.pattern.scale,
                orientation_reference: p.pattern.orientation_reference,
            },
            loops: p
                .loops
                .into_iter()
                .map(|l| match l {
                    IfccadHatchContextLoop::Shared {
                        base_loop_index,
                        classification,
                    } => Loop::Shared {
                        base_loop_index,
                        classification,
                    },
                    IfccadHatchContextLoop::Local {
                        classification,
                        boundary,
                    } => Loop::Local {
                        classification,
                        boundary: Boundary {
                            geometry: boundary.geometry.into(),
                            source_entity_ids: boundary
                                .source_entity_ids
                                .iter()
                                .map(|id| format!("{prefix}/e{id}"))
                                .collect(),
                        },
                    },
                })
                .collect(),
        }
    }
}
pub(super) fn decode_annotation(
    value: &Value,
    prefix: &str,
    path: &str,
) -> Result<Box<IfccadAnnotation<IfccadHatchAnnotationContext>>, IfccadReport> {
    let a = *super::annotation::decode::<Presentation>(value, prefix, path)?;
    Ok(Box::new(IfccadAnnotation {
        orientation: a.orientation,
        mode: a.mode,
        reference: a.reference,
        contexts: a
            .contexts
            .into_iter()
            .map(|c| {
                Ok(IfccadAnnotationContext {
                    key: c.key,
                    presentation: c.presentation.map(|p| p.decode(prefix, path)).transpose()?,
                })
            })
            .collect::<Result<_, IfccadReport>>()?,
    }))
}
#[derive(Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum Target {
    PaperViewport { entity_id: String },
    ModelWindow { id: String },
    StoredView { id: String },
}
impl Target {
    fn decode(self, prefix: &str, path: &str) -> Result<IfccadAnnotationViewTarget, IfccadReport> {
        Ok(match self {
            Self::PaperViewport { entity_id } => IfccadAnnotationViewTarget::PaperViewport {
                entity_id: reference(&entity_id, prefix, "e", path)?,
            },
            Self::ModelWindow { id } => IfccadAnnotationViewTarget::ModelWindow {
                id: IfccadModelWindowId(reference(&id, prefix, "modelWindow/", path)?),
            },
            Self::StoredView { id } => IfccadAnnotationViewTarget::StoredView {
                id: IfccadAnnotationViewId(reference(&id, prefix, "annotationView/", path)?),
            },
        })
    }
    fn encode(value: IfccadAnnotationViewTarget, prefix: &str) -> Self {
        match value {
            IfccadAnnotationViewTarget::PaperViewport { entity_id } => Self::PaperViewport {
                entity_id: format!("{prefix}/e{entity_id}"),
            },
            IfccadAnnotationViewTarget::ModelWindow { id } => Self::ModelWindow {
                id: format!("{prefix}/modelWindow/{}", id.0),
            },
            IfccadAnnotationViewTarget::StoredView { id } => Self::StoredView {
                id: format!("{prefix}/annotationView/{}", id.0),
            },
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ViewContext {
    id: u64,
    scale_id: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::dimension::present_option"
    )]
    target: Option<Target>,
    presentation: Presentation,
    view_state: IfccadHatchContextViewState,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Views {
    next_context_id: u64,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::dimension::present_option"
    )]
    reference: Option<u64>,
    contexts: Vec<ViewContext>,
}
pub(super) fn attach_views(
    kind: &mut IfccadEntityKind,
    value: &Value,
    prefix: &str,
    path: &str,
) -> Result<(), IfccadReport> {
    let IfccadEntityKind::Hatch(h) = kind else {
        return Err(failure(path, "Hatch view collection requires Hatch owner"));
    };
    super::supplemental::validate_value("ifccad::hatchAnnotationViews", value, path)?;
    let a: Views = serde_json::from_value(value.clone()).map_err(|e| failure(path, e))?;
    h.annotation_view_contexts = Some(Box::new(IfccadHatchAnnotationViewContexts {
        next_context_id: a.next_context_id,
        reference: a.reference.map(IfccadHatchAnnotationViewContextId),
        contexts: a
            .contexts
            .into_iter()
            .map(|c| {
                Ok(IfccadHatchAnnotationViewContext {
                    id: IfccadHatchAnnotationViewContextId(c.id),
                    scale_id: IfccadAnnotationScaleId(reference(
                        &c.scale_id,
                        prefix,
                        "annotationScale/",
                        path,
                    )?),
                    target: c.target.map(|v| v.decode(prefix, path)).transpose()?,
                    presentation: c.presentation.decode(prefix, path)?,
                    view_state: c.view_state,
                })
            })
            .collect::<Result<_, IfccadReport>>()?,
    }));
    Ok(())
}
pub(super) fn encode_attrs(
    h: &IfccadHatch,
    prefix: &str,
    attrs: &mut serde_json::Map<String, Value>,
) {
    if let Some(a) = &h.annotation {
        let a = IfccadAnnotation {
            orientation: a.orientation,
            mode: a.mode,
            reference: a.reference,
            contexts: a
                .contexts
                .iter()
                .map(|c| IfccadAnnotationContext {
                    key: c.key,
                    presentation: c
                        .presentation
                        .clone()
                        .map(|p| Presentation::encode(p, prefix)),
                })
                .collect(),
        };
        attrs.insert(
            "ifccad::annotation".into(),
            super::annotation::encode(&a, prefix),
        );
    }
    if let Some(a) = &h.annotation_view_contexts {
        let a = Views {
            next_context_id: a.next_context_id,
            reference: a.reference.map(|id| id.0),
            contexts: a
                .contexts
                .iter()
                .map(|c| ViewContext {
                    id: c.id.0,
                    scale_id: format!("{prefix}/annotationScale/{}", c.scale_id.0),
                    target: c.target.map(|t| Target::encode(t, prefix)),
                    presentation: Presentation::encode(c.presentation.clone(), prefix),
                    view_state: c.view_state,
                })
                .collect(),
        };
        attrs.insert(
            "ifccad::hatchAnnotationViews".into(),
            serde_json::to_value(a).expect("validated Hatch views"),
        );
    }
}
