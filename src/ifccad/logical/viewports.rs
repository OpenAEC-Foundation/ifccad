use super::*;
use num_rational::BigRational;
use std::collections::{BTreeMap, BTreeSet};

/// A Model view owned by a Paper layout, using the ordinary entity identity.
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadViewport {
    /// Kept out of line so adding saved aids does not enlarge every entity variant.
    pub workspace: Option<Box<super::IfccadViewportWorkspace>>,
    pub model_id: u64,
    pub frame: IfccadViewportFrame,
    pub view: IfccadViewportView,
    pub render_mode: IfccadViewportRenderMode,
    pub view_enabled: bool,
    pub view_locked: bool,
    pub paper_clip: IfccadViewportPaperClip,
    pub layer_overrides: Vec<IfccadViewportLayerOverride>,
    pub plot_shading_override: Option<crate::plot_kernel::ShadedPlotMode>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfccadViewportLayerOverride {
    pub layer_id: u64,
    pub frozen: bool,
    pub color: Option<IfccadColor>,
    pub opacity: Option<f64>,
    pub line_pattern_id: Option<IfccadLinePatternId>,
    pub line_weight: Option<f64>,
}
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadViewportFrame {
    pub center: [f64; 2],
    pub width: f64,
    pub height: f64,
}
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadViewportView {
    pub center: [f64; 2],
    pub target: [f64; 3],
    /// Target-to-camera vector in Model units, retained without normalization.
    pub direction: [f64; 3],
    pub height: f64,
    pub twist: f64,
    pub projection: IfccadViewportProjection,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lens_length_mm: Option<f64>,
    pub front_clip: IfccadViewportDepthClip,
    pub back_clip: IfccadViewportDepthClip,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IfccadViewportProjection {
    Orthographic,
    Perspective,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IfccadViewportClipMode {
    Disabled,
    AtCamera,
    AtDistance,
}
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfccadViewportDepthClip {
    pub mode: IfccadViewportClipMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance: Option<f64>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadViewportPaperClip {
    pub enabled: bool,
    pub boundary_entity_id: Option<u64>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IfccadViewportRenderMode {
    TwoDimensional,
    Wireframe,
    HiddenLine,
    FlatShadedWithoutEdges,
    FlatShadedWithEdges,
    SmoothShadedWithoutEdges,
    SmoothShadedWithEdges,
}

fn problem(message: &str) -> IfccadReport {
    crate::ifccad::diagnostics::failure("IFCCAD-VIEWPORT-003", "/ifccad::viewport", message)
}
fn exact(v: f64) -> BigRational {
    BigRational::from_float(v).expect("checked finite value")
}

fn frame_bounds(
    frame: &IfccadViewportFrame,
) -> Result<[(BigRational, BigRational); 2], IfccadReport> {
    if frame.center.iter().any(|v| !v.is_finite())
        || !frame.width.is_finite()
        || frame.width <= 0.
        || !frame.height.is_finite()
        || frame.height <= 0.
    {
        return Err(problem(
            "viewport frame needs finite coordinates and positive dimensions",
        ));
    }
    let two = exact(2.);
    let bounds = std::array::from_fn(|i| {
        let c = exact(frame.center[i]);
        let half = exact([frame.width, frame.height][i]) / &two;
        (&c - &half, &c + &half)
    });
    let max = exact(f64::MAX);
    if bounds.iter().any(|(lo, hi)| lo < &(-&max) || hi > &max) {
        return Err(problem("viewport frame has no finite binary64 enclosure"));
    }
    Ok(bounds)
}

/// Validate viewport scalar parameters without resolving document references.
pub fn validate_ifccad_viewport_parameters(viewport: &IfccadViewport) -> Result<(), IfccadReport> {
    frame_bounds(&viewport.frame)?;
    crate::workspace_kernel::validate_view(
        &viewport.view.as_workspace_view(),
        crate::workspace_kernel::WorkspaceViewKind::Model,
    )
    .map_err(|error| {
        crate::ifccad::diagnostics::failure(
            "IFCCAD-VIEWPORT-002",
            &format!(
                "/ifccad::viewport/{}",
                error
                    .field
                    .replace("lensLength", "lensLengthMm")
                    .replace('.', "/")
            ),
            error.to_string(),
        )
    })
}

/// Validates active boundary geometry; ownership and exclusive references are document rules.
pub fn validate_ifccad_viewport_boundary(
    frame: &IfccadViewportFrame,
    boundary: &IfccadEntityKind,
) -> Result<(), IfccadReport> {
    let primitive = boundary
        .as_shared_geometry()?
        .ok_or_else(|| problem("unsupported active boundary family"))?;
    crate::geometry_kernel::validate_paper_boundary(
        crate::geometry_kernel::PaperFrame {
            center: frame.center,
            width: frame.width,
            height: frame.height,
        },
        primitive,
    )
    .map_err(|e| problem(&e.to_string()))
}

pub(super) fn validate_references(document: &IfccadDocument) -> Result<(), IfccadReport> {
    let mut entities = BTreeMap::new();
    for (paper, contents) in std::iter::once((None, document.model.entities.as_slice()))
        .chain(
            document
                .blocks
                .iter()
                .map(|b| (None, b.entities.as_slice())),
        )
        .chain(
            document
                .paper_layouts
                .iter()
                .map(|p| (Some(p.id), p.entities.as_slice())),
        )
    {
        for e in contents {
            entities.insert(e.id(), (paper, e));
        }
    }
    let layers: BTreeSet<_> = document.layers.iter().map(|l| l.id).collect();
    let mut claimed = BTreeSet::new();
    for (paper, entity) in entities.values() {
        let Some(entity) = entity.as_native() else {
            continue;
        };
        let IfccadEntityKind::Viewport(v) = &entity.kind else {
            continue;
        };
        let result = (|| {
            if paper.is_none() || v.model_id != document.model.id {
                return Err(problem(
                    "viewport requires Paper ownership and the drawing's Model target",
                ));
            }
            let overridden: BTreeSet<_> = v.layer_overrides.iter().map(|r| r.layer_id).collect();
            if overridden.len() != v.layer_overrides.len() || !overridden.is_subset(&layers) {
                return Err(problem(
                    "viewport layer overrides must be unique drawing-local references",
                ));
            }
            for row in &v.layer_overrides {
                if !row.frozen
                    && row.color.is_none()
                    && row.opacity.is_none()
                    && row.line_pattern_id.is_none()
                    && row.line_weight.is_none()
                {
                    return Err(problem(
                        "viewport layer override requires a frozen or appearance value",
                    ));
                }
                if row.color.as_ref().is_some_and(|c| !c.is_valid())
                    || row
                        .opacity
                        .is_some_and(|v| !v.is_finite() || !(0.0..=1.0).contains(&v))
                    || row.line_weight.is_some_and(|v| !v.is_finite() || v < 0.0)
                    || row
                        .line_pattern_id
                        .is_some_and(|id| !document.line_patterns.iter().any(|p| p.id == id))
                {
                    return Err(problem(
                        "invalid viewport layer override value or pattern reference",
                    ));
                }
            }
            if v.paper_clip.enabled && v.paper_clip.boundary_entity_id.is_none() {
                return Err(problem("viewport active Paper clip requires a boundary"));
            }
            if let Some(id) = v.paper_clip.boundary_entity_id {
                let (owner, boundary) = entities
                    .get(&id)
                    .ok_or_else(|| problem("viewport boundary is unresolved"))?;
                if owner != paper
                    || boundary
                        .as_native()
                        .is_some_and(|e| matches!(e.kind, IfccadEntityKind::Viewport(_)))
                    || !claimed.insert(id)
                {
                    return Err(problem(
                        "viewport boundary must be non-viewport, same-Paper and exclusive",
                    ));
                }
                if v.paper_clip.enabled {
                    let boundary = boundary
                        .as_native()
                        .ok_or_else(|| problem("opaque geometry cannot be an active clip"))?;
                    validate_ifccad_viewport_boundary(&v.frame, &boundary.kind)?;
                }
            }
            Ok(())
        })();
        result.map_err(|report: IfccadReport| {
            crate::ifccad::diagnostics::context(
                report,
                "IFCCAD-VIEWPORT-003",
                &format!("/cad/d{}/e{}", document.drawing_id, entity.id),
            )
        })?;
    }
    Ok(())
}

impl IfccadViewportView {
    pub fn as_workspace_view(&self) -> crate::workspace_kernel::WorkspaceView {
        use crate::geometry_kernel::{Point2, Point3, Vector3};
        use crate::workspace_kernel::*;
        let clip = |v: &IfccadViewportDepthClip| WorkspaceClip {
            mode: match v.mode {
                IfccadViewportClipMode::Disabled => WorkspaceClipMode::Disabled,
                IfccadViewportClipMode::AtCamera => WorkspaceClipMode::AtCamera,
                IfccadViewportClipMode::AtDistance => WorkspaceClipMode::AtDistance,
            },
            distance: v.distance,
        };
        WorkspaceView {
            center: Point2::new(self.center[0], self.center[1]),
            target: Point3::new(self.target[0], self.target[1], self.target[2]),
            direction: Vector3::new(self.direction[0], self.direction[1], self.direction[2]),
            height: self.height,
            twist: self.twist,
            projection: match self.projection {
                IfccadViewportProjection::Orthographic => WorkspaceProjection::Orthographic,
                IfccadViewportProjection::Perspective => WorkspaceProjection::Perspective,
            },
            lens_length: self.lens_length_mm,
            front_clip: clip(&self.front_clip),
            back_clip: clip(&self.back_clip),
        }
    }
}
