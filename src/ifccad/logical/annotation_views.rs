use super::*;
use crate::geometry_kernel::hatch::{HatchBoundary2, HatchLineFamily};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IfccadAnnotationViewId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IfccadAnnotationViewTarget {
    PaperViewport { entity_id: u64 },
    ModelWindow { id: IfccadModelWindowId },
    StoredView { id: IfccadAnnotationViewId },
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadHatchContextPattern {
    pub families: Vec<HatchLineFamily>,
    pub scale: f64,
    pub orientation_reference: [f64; 3],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadHatchLoopClassification {
    pub external: bool,
    pub polyline: bool,
    pub derived: bool,
    pub text_box: bool,
    pub outermost: bool,
    pub not_closed: bool,
    pub self_intersecting: bool,
    pub text_island: bool,
    pub duplicate: bool,
    pub is_annotative: bool,
    pub does_not_support_scale: bool,
    pub force_annotation_visible: bool,
    pub orient_to_paper: bool,
    pub is_annotative_block: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadHatchContextBoundary {
    pub geometry: HatchBoundary2,
    pub source_entity_ids: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum IfccadHatchContextLoop {
    Shared {
        base_loop_index: u32,
        classification: IfccadHatchLoopClassification,
    },
    Local {
        classification: IfccadHatchLoopClassification,
        boundary: IfccadHatchContextBoundary,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadHatchAnnotationContext {
    pub pattern: IfccadHatchContextPattern,
    pub loops: Vec<IfccadHatchContextLoop>,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadHatchContextViewState {
    pub normal: [f64; 3],
    pub rotation: f64,
    pub evaluate: bool,
}

/// Leaf identity is local to one Hatch view collection, not its scale/target pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IfccadHatchAnnotationViewContextId(pub u64);

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadHatchAnnotationViewContext {
    pub id: IfccadHatchAnnotationViewContextId,
    pub scale_id: IfccadAnnotationScaleId,
    /// None retains an originally null source target, independently of scale.
    pub target: Option<IfccadAnnotationViewTarget>,
    pub presentation: IfccadHatchAnnotationContext,
    pub view_state: IfccadHatchContextViewState,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadHatchAnnotationViewContexts {
    pub next_context_id: u64,
    /// This collection's default; independent of the ordinary scale default.
    pub reference: Option<IfccadHatchAnnotationViewContextId>,
    pub contexts: Vec<IfccadHatchAnnotationViewContext>,
}

impl IfccadHatchAnnotationViewContexts {
    /// Reserve an exact local identity without changing any stored snapshots.
    pub fn allocate_context_id(
        &mut self,
    ) -> Result<IfccadHatchAnnotationViewContextId, IfccadReport> {
        let invalid = || {
            crate::ifccad::diagnostics::failure(
                "IFCCAD-ANNOTATION-002",
                "/ifccad::hatchAnnotationViews/nextContextId",
                "cannot allocate a positive Hatch view-context ID",
            )
        };
        let id = self.next_context_id;
        if id == 0 {
            return Err(invalid());
        }
        self.next_context_id = id.checked_add(1).ok_or_else(invalid)?;
        Ok(IfccadHatchAnnotationViewContextId(id))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IfccadHatchAnnotationViewRequest {
    ById(IfccadHatchAnnotationViewContextId),
    ByTarget {
        scale_id: IfccadAnnotationScaleId,
        target: IfccadAnnotationViewTarget,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IfccadHatchAnnotationViewSelection<'a> {
    Disabled,
    SelectionUnspecified,
    NoStoredContexts,
    Missing,
    Ambiguous,
    Available(&'a IfccadHatchAnnotationViewContext),
}

/// Select a saved view leaf without merging repeated selectors or applying layout.
/// Null-target leaves can be selected by ID, never by guessing an active viewport.
pub fn select_ifccad_hatch_annotation_view_context(
    collection: Option<&IfccadHatchAnnotationViewContexts>,
    enabled: bool,
    request: Option<IfccadHatchAnnotationViewRequest>,
) -> IfccadHatchAnnotationViewSelection<'_> {
    if !enabled {
        return IfccadHatchAnnotationViewSelection::Disabled;
    }
    let Some(request) = request else {
        return IfccadHatchAnnotationViewSelection::SelectionUnspecified;
    };
    let Some(collection) = collection.filter(|collection| !collection.contexts.is_empty()) else {
        return IfccadHatchAnnotationViewSelection::NoStoredContexts;
    };
    let mut matching = collection.contexts.iter().filter(|context| match request {
        IfccadHatchAnnotationViewRequest::ById(id) => context.id == id,
        IfccadHatchAnnotationViewRequest::ByTarget { scale_id, target } => {
            context.scale_id == scale_id && context.target == Some(target)
        }
    });
    match (matching.next(), matching.next()) {
        (None, _) => IfccadHatchAnnotationViewSelection::Missing,
        (Some(_), Some(_)) => IfccadHatchAnnotationViewSelection::Ambiguous,
        (Some(context), None) => IfccadHatchAnnotationViewSelection::Available(context),
    }
}

/// Bounded named-view camera needed by saved annotation view targets.
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadAnnotationStoredView {
    pub annotation_scale_id: Option<IfccadAnnotationScaleId>,
    pub id: IfccadAnnotationViewId,
    pub name: String,
    pub space: IfccadAnnotationViewSpace,
    pub width: f64,
    pub view: IfccadViewportView,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IfccadAnnotationViewSpace {
    Model,
    Paper,
}

pub(super) fn validate_stored_views(document: &IfccadDocument) -> Result<(), IfccadReport> {
    use std::collections::BTreeSet;
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for v in &document.annotation_views {
        let path = format!("/annotationView/{}", v.id.0);
        let invalid =
            |s: &str| crate::ifccad::diagnostics::failure("IFCCAD-ANNOTATION-007", &path, s);
        if !ids.insert(v.id)
            || v.name.trim().is_empty()
            || !names.insert(crate::names::name_key(&v.name))
            || !v.width.is_finite()
            || v.width <= 0.
        {
            return Err(invalid(
                "invalid stored annotation view identity or dimensions",
            ));
        }
        // Saved VIEW cameras are not the PaperCanvas window subset.
        crate::workspace_kernel::validate_view(
            &v.view.as_workspace_view(),
            crate::workspace_kernel::WorkspaceViewKind::Model,
        )
        .map_err(|e| {
            crate::ifccad::diagnostics::context(
                invalid(&e.to_string()),
                "IFCCAD-ANNOTATION-007",
                &format!("{path}/view/{}", e.field),
            )
        })?;
    }
    if document.id_counters.next_annotation_view_id == 0
        || document
            .annotation_views
            .iter()
            .any(|v| v.id.0 >= document.id_counters.next_annotation_view_id)
    {
        return Err(crate::ifccad::diagnostics::failure(
            "IFCCAD-ANNOTATION-002",
            "/allocation/nextAnnotationViewId",
            "stored-view watermark must exceed saved IDs",
        ));
    }
    Ok(())
}
