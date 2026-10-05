use super::*;
use crate::ocdraw::{CoordinateFrame3, Point3, Vector3};
use num_rational::BigRational;
use std::collections::{BTreeMap, BTreeSet};

/// A Model view owned by a Paper layout, using the ordinary entity identity.
#[derive(Clone, Debug, PartialEq)]
pub struct IfcxCadViewport {
    pub model_id: u64,
    pub frame: IfcxCadViewportFrame,
    pub view: IfcxCadViewportView,
    pub render_mode: IfcxCadViewportRenderMode,
    pub view_enabled: bool,
    pub view_locked: bool,
    pub visible: bool,
    pub paper_clip: IfcxCadViewportPaperClip,
    pub frozen_layers: Vec<u64>,
}
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfcxCadViewportFrame {
    pub center: [f64; 2],
    pub width: f64,
    pub height: f64,
}
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfcxCadViewportView {
    pub center: [f64; 2],
    pub target: [f64; 3],
    /// Target-to-camera vector in Model units, retained without normalization.
    pub direction: [f64; 3],
    pub height: f64,
    pub twist: f64,
    pub projection: IfcxCadViewportProjection,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lens_length_mm: Option<f64>,
    pub front_clip: IfcxCadViewportDepthClip,
    pub back_clip: IfcxCadViewportDepthClip,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IfcxCadViewportProjection {
    Orthographic,
    Perspective,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IfcxCadViewportClipMode {
    Disabled,
    AtCamera,
    AtDistance,
}
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IfcxCadViewportDepthClip {
    pub mode: IfcxCadViewportClipMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance: Option<f64>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfcxCadViewportPaperClip {
    pub enabled: bool,
    pub boundary_entity_id: Option<u64>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IfcxCadViewportRenderMode {
    TwoDimensional,
    Wireframe,
    HiddenLine,
    FlatShadedWithoutEdges,
    FlatShadedWithEdges,
    SmoothShadedWithoutEdges,
    SmoothShadedWithEdges,
}

fn problem(message: &str) -> IfcxCadReport {
    IfcxCadReport::one(message)
}
fn exact(v: f64) -> BigRational {
    BigRational::from_float(v).expect("checked finite value")
}
fn square(v: &BigRational) -> BigRational {
    v * v
}

fn frame_bounds(
    frame: &IfcxCadViewportFrame,
) -> Result<[(BigRational, BigRational); 2], IfcxCadReport> {
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
pub fn validate_ifcx_cad_viewport_parameters(
    viewport: &IfcxCadViewport,
) -> Result<(), IfcxCadReport> {
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
        || (v.projection == IfcxCadViewportProjection::Perspective
            && !v.lens_length_mm.is_some_and(|n| n.is_finite() && n > 0.))
    {
        return Err(problem(
            "viewport projection has an invalid lens length in millimetres",
        ));
    }
    for (front, clip) in [(true, &v.front_clip), (false, &v.back_clip)] {
        if clip.distance.is_some_and(|n| !n.is_finite())
            || (clip.mode == IfcxCadViewportClipMode::AtDistance && clip.distance.is_none())
            || (!front && clip.mode == IfcxCadViewportClipMode::AtCamera)
        {
            return Err(problem(
                "viewport depth clipping has an invalid mode or distance",
            ));
        }
    }
    if v.front_clip.mode != IfcxCadViewportClipMode::Disabled
        && v.back_clip.mode != IfcxCadViewportClipMode::Disabled
    {
        let back = v.back_clip.distance.expect("validated active distance");
        let ordered = match v.front_clip.mode {
            IfcxCadViewportClipMode::AtCamera => back < 0. || square(&exact(back)) < norm_squared,
            IfcxCadViewportClipMode::AtDistance => {
                back < v.front_clip.distance.expect("validated active distance")
            }
            IfcxCadViewportClipMode::Disabled => unreachable!(),
        };
        if !ordered {
            return Err(problem(
                "viewport active back plane must be behind front plane",
            ));
        }
    }
    Ok(())
}

fn paper_placement(p: &IfcxCadPlacement) -> Result<(), IfcxCadReport> {
    let point = |v: [f64; 3]| Point3::new(v[0], v[1], v[2]);
    let vector = |v: [f64; 3]| Vector3::new(v[0], v[1], v[2]);
    CoordinateFrame3::try_new(point(p.origin), vector(p.x_axis), vector(p.y_axis))
        .map_err(|_| problem("viewport boundary has invalid placement"))?;
    if p.origin[2] != 0. || p.x_axis[2] != 0. || p.y_axis[2] != 0. {
        return Err(problem("viewport boundary must lie in Paper Z=0"));
    }
    Ok(())
}

fn circle_enclosed_by_viewport(
    frame: &IfcxCadViewportFrame,
    radius: f64,
    p: &IfcxCadPlacement,
) -> Result<bool, IfcxCadReport> {
    let bounds = frame_bounds(frame)?;
    paper_placement(p)?;
    if !radius.is_finite() || radius <= 0. {
        return Err(problem(
            "viewport circle boundary needs a positive finite radius",
        ));
    }
    let radius_squared = square(&exact(radius));
    for (i, (lo, hi)) in bounds.iter().enumerate() {
        let center = exact(p.origin[i]);
        if &center < lo || &center > hi {
            return Ok(false);
        }
        let amplitude_squared =
            &radius_squared * (square(&exact(p.x_axis[i])) + square(&exact(p.y_axis[i])));
        if amplitude_squared > square(&(&center - lo))
            || amplitude_squared > square(&(hi - &center))
        {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Validate the complete supported boundary shape, independently of identity/owner.
pub fn validate_ifcx_cad_viewport_boundary(
    frame: &IfcxCadViewportFrame,
    boundary: &IfcxCadEntityKind,
) -> Result<(), IfcxCadReport> {
    let bounds = frame_bounds(frame)?;
    match boundary {
        IfcxCadEntityKind::Circle { radius, placement } => {
            if !circle_enclosed_by_viewport(frame, *radius, placement)? {
                return Err(problem("viewport circle boundary extends outside frame"));
            }
        }
        IfcxCadEntityKind::PlanarPolyline {
            vertices,
            closed,
            placement,
            ..
        } => {
            paper_placement(placement)?;
            if !closed || vertices.iter().flatten().any(|n| !n.is_finite()) {
                return Err(problem(
                    "viewport polyline boundary must be closed and finite",
                ));
            }
            let bits = |n: f64| if n == 0. { 0 } else { n.to_bits() };
            let distinct: BTreeSet<_> = vertices.iter().map(|v| (bits(v[0]), bits(v[1]))).collect();
            if distinct.len() < 3 {
                return Err(problem(
                    "viewport polyline boundary needs three distinct vertices",
                ));
            }
            for vertex in vertices {
                for (i, (lo, hi)) in bounds.iter().enumerate() {
                    let n = exact(placement.origin[i])
                        + exact(placement.x_axis[i]) * exact(vertex[0])
                        + exact(placement.y_axis[i]) * exact(vertex[1]);
                    if &n < lo || &n > hi {
                        return Err(problem("viewport polyline boundary extends outside frame"));
                    }
                }
            }
        }
        _ => {
            return Err(problem(
                "viewport active boundary must be a closed planar polyline or circle",
            ))
        }
    }
    Ok(())
}

pub(super) fn validate_references(document: &IfcxCadDocument) -> Result<(), IfcxCadReport> {
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
            entities.insert(e.id, (paper, e));
        }
    }
    let layers: BTreeSet<_> = document.layers.iter().map(|l| l.id).collect();
    let mut claimed = BTreeSet::new();
    for (paper, entity) in entities.values() {
        let IfcxCadEntityKind::Viewport(v) = &entity.kind else {
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
                    || matches!(boundary.kind, IfcxCadEntityKind::Viewport(_))
                    || !claimed.insert(id)
                {
                    return Err(problem(
                        "viewport boundary must be non-viewport, same-Paper and exclusive",
                    ));
                }
                if v.paper_clip.enabled {
                    validate_ifcx_cad_viewport_boundary(&v.frame, &boundary.kind)?;
                }
            }
            Ok(())
        })();
        result.map_err(|report: IfcxCadReport| IfcxCadReport {
            errors: report
                .errors
                .into_iter()
                .map(|message| format!("/cad/d{}/e{}: {message}", document.drawing_id, entity.id))
                .collect(),
        })?;
    }
    Ok(())
}
