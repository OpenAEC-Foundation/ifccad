use super::validation::{diagnostic, IfcdrEntityKind, IfcdrEvidence};
use super::*;
use crate::ifcdr::geometry::numeric::{exact, round_down, round_up};
use crate::ifcdr::{Bounds3d, PlanePlacement, Point3};
use std::collections::BTreeSet;

pub(super) fn frame_bounds(frame: ViewportFrame) -> Option<Bounds3d> {
    if !valid_point(frame.center)
        || !frame.width.is_finite()
        || !frame.height.is_finite()
        || frame.width <= 0.0
        || frame.height <= 0.0
    {
        return None;
    }
    let half_width = exact(frame.width) * exact(0.5);
    let half_height = exact(frame.height) * exact(0.5);
    let x = exact(frame.center.x());
    let y = exact(frame.center.y());
    Some(Bounds3d {
        min: Point3::new(
            round_down(&(x.clone() - &half_width)).ok()?,
            round_down(&(y.clone() - &half_height)).ok()?,
            0.0,
        ),
        max: Point3::new(
            round_up(&(x + half_width)).ok()?,
            round_up(&(y + half_height)).ok()?,
            0.0,
        ),
    })
}

pub(super) fn check_viewport<R: IfcdrResourceAccess>(
    r: &R,
    viewport: &IfcdrViewportRow,
    row: usize,
    evidence: &IfcdrEvidence,
    errors: &mut Vec<IfcdrDiagnostic>,
) {
    let mut reject = |property, message| {
        errors.push(diagnostic(
            r,
            IFCCAD_IFCDR_VIEWPORT_INVALID,
            "viewport",
            Some(row),
            property,
            message,
        ));
    };
    if !r.scopes().iter().any(|scope| {
        scope.id == viewport.entity.scope_id && scope.kind == IfcdrScopeKind::PaperSpace
    }) {
        reject("scopeId", "viewport owner must be a PaperSpace");
    }
    if !r
        .scopes()
        .iter()
        .any(|scope| scope.id == viewport.view_scope_id && scope.kind == IfcdrScopeKind::ModelSpace)
    {
        reject(
            "viewScopeId",
            "viewport must view this resource's ModelSpace",
        );
    }
    if frame_bounds(viewport.frame).is_none() {
        reject(
            "frame",
            "frame must have finite center and positive finite dimensions",
        );
    }
    let view = viewport.view;
    let direction = view.direction.components();
    let direction_norm = direction[0].hypot(direction[1]).hypot(direction[2]);
    if !valid_point(view.center)
        || !valid_point3(view.target)
        || !view.height.is_finite()
        || view.height <= 0.0
        || !view.twist.is_finite()
        || !direction_norm.is_finite()
        || direction_norm == 0.0
    {
        reject(
            "view",
            "view values must be finite, with nonzero direction and positive height",
        );
    }
    let valid_lens = match (view.projection, view.lens_length) {
        (ProjectionMode::Orthographic, None) => true,
        (ProjectionMode::Orthographic, Some(lens)) => lens.is_finite() && lens >= 0.0,
        (ProjectionMode::Perspective, Some(lens)) => lens.is_finite() && lens > 0.0,
        (ProjectionMode::Perspective, None) => false,
    };
    if !valid_lens {
        reject(
            "view",
            "lens length must be finite and nonnegative in Orthographic, positive in Perspective",
        );
    }
    let front = view.front_clip;
    let back = view.back_clip;
    if front.distance.is_some_and(|d| !d.is_finite())
        || (front.mode == FrontClipMode::AtDistance && front.distance.is_none())
        || back.distance.is_some_and(|d| !d.is_finite())
        || (back.mode == BackClipMode::AtDistance && back.distance.is_none())
    {
        reject(
            "view",
            "active clip distance is required and every stored distance must be finite",
        );
    }
    let front_position = match front.mode {
        FrontClipMode::Disabled => None,
        FrontClipMode::AtCamera => Some(direction_norm),
        FrontClipMode::AtDistance => front.distance,
    };
    if let (Some(front_position), BackClipMode::AtDistance, Some(back_position)) =
        (front_position, back.mode, back.distance)
    {
        if back_position >= front_position {
            reject("view", "active back clip must lie behind active front clip");
        }
    }
    if viewport.paper_clip.enabled && viewport.paper_clip.boundary_entity_id.is_none() {
        reject(
            "paperClip",
            "enabled paper clipping needs a boundary entity ID",
        );
    }
    if let Some(shading) = viewport.plot_shading_override {
        let dpi = shading.quality.dpi;
        if (shading.quality.mode == ShadedPlotQualityMode::Custom
            && !dpi.is_some_and(|value| (100..=32767).contains(&value)))
            || (shading.quality.mode != ShadedPlotQualityMode::Custom && dpi.is_some())
        {
            reject(
                "plotShadingOverride",
                "custom quality requires dpi 100..32767; other modes forbid dpi",
            );
        }
    }
    let mut seen = BTreeSet::new();
    for override_row in &viewport.layer_overrides {
        let patch_effective = override_row
            .appearance_override_id
            .and_then(|id| evidence.overrides.get(&id))
            .and_then(|row| r.overrides().get(*row))
            .is_some_and(|patch| {
                patch.color.is_some()
                    || patch.opacity.is_some()
                    || patch.ifcx_line_pattern.is_some()
                    || patch.line_weight.is_some()
            });
        if !seen.insert(override_row.layer_id)
            || !evidence.layers.contains_key(&override_row.layer_id)
            || override_row
                .appearance_override_id
                .is_some_and(|id| !evidence.overrides.contains_key(&id))
            || (!override_row.frozen && !patch_effective)
        {
            reject(
                "layerOverrides",
                "override must select one known layer and have an effective change",
            );
        }
    }
}

pub(super) fn check_viewport_boundaries<R: IfcdrResourceAccess>(
    r: &R,
    evidence: &IfcdrEvidence,
    errors: &mut Vec<IfcdrDiagnostic>,
) {
    let mut claimed = BTreeSet::new();
    let polylines = r.polylines();
    for (row, viewport) in r.viewports().iter().enumerate() {
        let Some(id) = viewport.paper_clip.boundary_entity_id else {
            continue;
        };
        let Some(location) = evidence.entities.get(&id) else {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_VIEWPORT_INVALID,
                "viewport",
                Some(row),
                "paperClip",
                "boundary entity is missing",
            ));
            continue;
        };
        if !claimed.insert(id) || location.scope != viewport.entity.scope_id {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_VIEWPORT_INVALID,
                "viewport",
                Some(row),
                "paperClip",
                "boundary must be same-scope and unique to one viewport",
            ));
            continue;
        }
        if !viewport.paper_clip.enabled {
            continue;
        }
        if !matches!(location.kind, IfcdrEntityKind::PlanarPolyline) {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_VIEWPORT_INVALID,
                "viewport",
                Some(row),
                "paperClip",
                "active boundary must be a closed straight polyline",
            ));
            continue;
        }
        let Some(polyline) = polylines.get(location.row) else {
            continue;
        };
        let Some(frame) = frame_bounds(viewport.frame) else {
            continue;
        };
        let placement = polyline.placement();
        if !polyline.closed()
            || polyline.vertex_count() < 3
            || placement.origin.z() != 0.0
            || placement.x.z() != 0.0
            || placement.y.z() != 0.0
            || placement.validate().is_err()
        {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_VIEWPORT_INVALID,
                "viewport",
                Some(row),
                "paperClip",
                "active boundary requires a closed planar polyline with at least three vertices",
            ));
            continue;
        }
        let plane = PlanePlacement::from_validated_components(placement);
        let mut distinct = BTreeSet::new();
        let mut enclosed = true;
        for vertex in 0..polyline.vertex_count() {
            let Some(point) = polyline.vertex(vertex) else {
                enclosed = false;
                break;
            };
            let bits = |value: f64| if value == 0.0 { 0 } else { value.to_bits() };
            distinct.insert((bits(point.x()), bits(point.y())));
            enclosed &= plane.enclosed_by(point, frame);
        }
        if distinct.len() < 3 || !enclosed {
            errors.push(diagnostic(
                r,
                IFCCAD_IFCDR_VIEWPORT_INVALID,
                "viewport",
                Some(row),
                "paperClip",
                "active boundary needs three distinct vertices within the frame",
            ));
        }
    }
}
