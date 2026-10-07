use super::*;
use num_rational::BigRational;
use std::collections::{BTreeMap, BTreeSet};

/// A Model view owned by a Paper layout, using the ordinary entity identity.
#[derive(Clone, Debug, PartialEq)]
pub struct IfccadViewport {
    pub model_id: u64,
    pub frame: IfccadViewportFrame,
    pub view: IfccadViewportView,
    pub render_mode: IfccadViewportRenderMode,
    pub view_enabled: bool,
    pub view_locked: bool,
    pub visible: bool,
    pub paper_clip: IfccadViewportPaperClip,
    pub frozen_layers: Vec<u64>,
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
    IfccadReport::one(message)
}
fn exact(v: f64) -> BigRational {
    BigRational::from_float(v).expect("checked finite value")
}
fn square(v: &BigRational) -> BigRational {
    v * v
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
    let v = &viewport.view;
    if v.center
        .iter()
        .chain(v.target.iter())
        .chain(v.direction.iter())
        .any(|n| !n.is_finite())
        || !v.height.is_finite()
        || v.height <= 0.
        || !v.twist.is_finite()
    {
        return Err(problem(
            "viewport view needs finite coordinates and positive height",
        ));
    }
    let norm_squared: BigRational = v.direction.iter().map(|n| square(&exact(*n))).sum();
    if norm_squared <= exact(0.) || norm_squared > square(&exact(f64::MAX)) {
        return Err(problem("viewport direction needs a finite positive norm"));
    }
    if v.lens_length_mm.is_some_and(|n| !n.is_finite() || n < 0.)
        || (v.projection == IfccadViewportProjection::Perspective
            && !v.lens_length_mm.is_some_and(|n| n.is_finite() && n > 0.))
    {
        return Err(problem(
            "viewport projection has an invalid lens length in millimetres",
        ));
    }
    for (front, clip) in [(true, &v.front_clip), (false, &v.back_clip)] {
        if clip.distance.is_some_and(|n| !n.is_finite())
            || (clip.mode == IfccadViewportClipMode::AtDistance && clip.distance.is_none())
            || (!front && clip.mode == IfccadViewportClipMode::AtCamera)
        {
            return Err(problem(
                "viewport depth clipping has an invalid mode or distance",
            ));
        }
    }
    if v.front_clip.mode != IfccadViewportClipMode::Disabled
        && v.back_clip.mode != IfccadViewportClipMode::Disabled
    {
        let back = v.back_clip.distance.expect("validated active distance");
        let ordered = match v.front_clip.mode {
            IfccadViewportClipMode::AtCamera => back < 0. || square(&exact(back)) < norm_squared,
            IfccadViewportClipMode::AtDistance => {
                back < v.front_clip.distance.expect("validated active distance")
            }
            IfccadViewportClipMode::Disabled => unreachable!(),
        };
        if !ordered {
            return Err(problem(
                "viewport active back plane must be behind front plane",
            ));
        }
    }
    Ok(())
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
            let frozen: BTreeSet<_> = v.frozen_layers.iter().copied().collect();
            if frozen.len() != v.frozen_layers.len() || !frozen.is_subset(&layers) {
                return Err(problem(
                    "viewport frozen layers must be unique drawing-local references",
                ));
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
        result.map_err(|report: IfccadReport| IfccadReport {
            errors: report
                .errors
                .into_iter()
                .map(|message| format!("/cad/d{}/e{}: {message}", document.drawing_id, entity.id))
                .collect(),
        })?;
    }
    Ok(())
}
