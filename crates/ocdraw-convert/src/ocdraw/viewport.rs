use crate::source::{ExportAction, ExportDiagnostic, ExportDiagnosticSource, ExportLossReason};
use cadcodec::{CadDocument, EntityType, Handle, Vector3};
use ocdraw::ocdraw::*;
use std::collections::BTreeMap;
pub(super) fn losses(
    viewport: &cadcodec::entities::Viewport,
    document: &CadDocument,
    mapping: &BTreeMap<Handle, u64>,
) -> Vec<ExportLossReason> {
    let mut reasons = Vec::new();
    if viewport.status.perspective {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "perspective viewport needs CAD fixture calibration".into(),
        });
    }
    if viewport.clip_boundary_handle != Handle::NULL {
        let supported = matches!(document.get_entity(viewport.clip_boundary_handle), Some(EntityType::LwPolyline(boundary))
            if boundary.common.owner_handle == viewport.common.owner_handle
                && boundary.is_closed && boundary.vertices.len() >= 3
                && boundary.vertices.iter().all(|vertex| vertex.bulge == 0.0)
                && boundary.elevation == 0.0 && boundary.normal == Vector3::UNIT_Z)
            && mapping.contains_key(&viewport.clip_boundary_handle);
        if !supported {
            reasons.push(ExportLossReason::UnsupportedSemantic {
                name: "active viewport clip boundary".into(),
            });
        }
    }
    if viewport.center.z != 0.0 || viewport.view_center.z != 0.0 {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "viewport paper or DCS z coordinate".into(),
        });
    }
    if viewport.width <= 0.0
        || viewport.height <= 0.0
        || viewport.view_height <= 0.0
        || ![
            viewport.center.x,
            viewport.center.y,
            viewport.width,
            viewport.height,
            viewport.view_center.x,
            viewport.view_center.y,
            viewport.view_target.x,
            viewport.view_target.y,
            viewport.view_target.z,
            viewport.view_direction.x,
            viewport.view_direction.y,
            viewport.view_direction.z,
            viewport.view_height,
            viewport.twist_angle,
            viewport.lens_length,
            viewport.front_clip_z,
            viewport.back_clip_z,
        ]
        .into_iter()
        .all(f64::is_finite)
    {
        reasons.push(ExportLossReason::NonFiniteCoordinate);
    }
    reasons
}

pub(super) fn deferred_losses(viewport: &cadcodec::entities::Viewport) -> Vec<ExportLossReason> {
    let baseline = cadcodec::entities::Viewport::new();
    let mut reasons = Vec::new();
    let mut status = viewport.status;
    status.is_on = baseline.status.is_on;
    status.locked = baseline.status.locked;
    status.perspective = baseline.status.perspective;
    status.front_clipping = baseline.status.front_clipping;
    status.back_clipping = baseline.status.back_clipping;
    status.front_clip_not_at_eye = baseline.status.front_clip_not_at_eye;
    if status != baseline.status
        || viewport.snap_base != baseline.snap_base
        || viewport.snap_spacing != baseline.snap_spacing
        || viewport.grid_spacing != baseline.grid_spacing
        || viewport.snap_angle != baseline.snap_angle
        || viewport.circle_sides != baseline.circle_sides
        || viewport.grid_flags != baseline.grid_flags
        || viewport.grid_major != baseline.grid_major
    {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "viewport workspace snap/grid/display state".into(),
        });
    }
    if viewport.ucs_at_origin != baseline.ucs_at_origin
        || viewport.ucs_per_viewport != baseline.ucs_per_viewport
        || viewport.ucs_icon_visible != baseline.ucs_icon_visible
        || viewport.ucs_origin != baseline.ucs_origin
        || viewport.ucs_x_axis != baseline.ucs_x_axis
        || viewport.ucs_y_axis != baseline.ucs_y_axis
        || viewport.ucs_handle != Handle::NULL
        || viewport.base_ucs_handle != Handle::NULL
        || viewport.ucs_ortho_type != baseline.ucs_ortho_type
        || viewport.elevation != baseline.elevation
    {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "viewport UCS state".into(),
        });
    }
    if !viewport.style_sheet.is_empty()
        || viewport.shade_plot_mode != baseline.shade_plot_mode
        || viewport.background_handle != Handle::NULL
        || viewport.shade_plot_handle != Handle::NULL
        || viewport.visual_style_handle != Handle::NULL
        || viewport.sun_handle != Handle::NULL
        || viewport.default_lighting != baseline.default_lighting
        || viewport.default_lighting_type != baseline.default_lighting_type
        || viewport.brightness != baseline.brightness
        || viewport.contrast != baseline.contrast
        || viewport.ambient_color != baseline.ambient_color
    {
        reasons.push(ExportLossReason::UnsupportedSemantic {
            name: "viewport visual and plot state".into(),
        });
    }
    reasons
}
fn render_from_cad(mode: cadcodec::entities::ViewportRenderMode) -> DrawingRenderMode {
    use cadcodec::entities::ViewportRenderMode as Cad;
    match mode {
        Cad::Wireframe2D => DrawingRenderMode::TwoDimensional,
        Cad::Wireframe3D => DrawingRenderMode::Wireframe,
        Cad::HiddenLine => DrawingRenderMode::HiddenLine,
        Cad::FlatShaded => DrawingRenderMode::FlatShadedWithoutEdges,
        Cad::FlatShadedWithEdges => DrawingRenderMode::FlatShadedWithEdges,
        Cad::GouraudShaded => DrawingRenderMode::SmoothShadedWithoutEdges,
        Cad::GouraudShadedWithEdges => DrawingRenderMode::SmoothShadedWithEdges,
    }
}
fn render_to_cad(mode: DrawingRenderMode) -> cadcodec::entities::ViewportRenderMode {
    use cadcodec::entities::ViewportRenderMode as Cad;
    match mode {
        DrawingRenderMode::TwoDimensional => Cad::Wireframe2D,
        DrawingRenderMode::Wireframe => Cad::Wireframe3D,
        DrawingRenderMode::HiddenLine => Cad::HiddenLine,
        DrawingRenderMode::FlatShadedWithoutEdges => Cad::FlatShaded,
        DrawingRenderMode::FlatShadedWithEdges => Cad::FlatShadedWithEdges,
        DrawingRenderMode::SmoothShadedWithoutEdges => Cad::GouraudShaded,
        DrawingRenderMode::SmoothShadedWithEdges => Cad::GouraudShadedWithEdges,
    }
}
pub(super) struct SourceIndex<'a> {
    pub document: &'a CadDocument,
    pub layers: &'a BTreeMap<String, u32>,
    pub entities: &'a BTreeMap<Handle, u64>,
}
pub(super) fn from_cad(
    source: &cadcodec::entities::Viewport,
    scope: u32,
    layer: u32,
    appearance: EntityAppearance,
    index: &SourceIndex<'_>,
    diagnostics: &mut Vec<ExportDiagnostic>,
) -> ViewportDefinition {
    let SourceIndex {
        document,
        layers,
        entities: mapping,
    } = index;
    let view = DrawingView {
        center: Point2::new(source.view_center.x, source.view_center.y),
        target: Point3::new(
            source.view_target.x,
            source.view_target.y,
            source.view_target.z,
        ),
        direction: ocdraw::ocdraw::Vector3::new(
            source.view_direction.x,
            source.view_direction.y,
            source.view_direction.z,
        ),
        height: source.view_height,
        twist: source.twist_angle,
        projection: DrawingProjection::Orthographic,
        lens_length: Some(source.lens_length),
        front_clip: DrawingClip {
            mode: if source.status.front_clipping {
                if source.status.front_clip_not_at_eye {
                    DrawingClipMode::AtDistance
                } else {
                    DrawingClipMode::AtCamera
                }
            } else {
                DrawingClipMode::Disabled
            },
            distance: Some(source.front_clip_z),
        },
        back_clip: DrawingClip {
            mode: if source.status.back_clipping {
                DrawingClipMode::AtDistance
            } else {
                DrawingClipMode::Disabled
            },
            distance: Some(source.back_clip_z),
        },
    };
    let mut target = ViewportDefinition::new(
        scope,
        layer,
        DrawingViewportFrame {
            center: Point2::new(source.center.x, source.center.y),
            width: source.width,
            height: source.height,
        },
        view,
    );
    target.render_mode = render_from_cad(source.render_mode);
    target.view_enabled = source.status.is_on;
    target.view_locked = source.status.locked;
    target.appearance = appearance;
    target.visible = !source.common.invisible;
    target.paper_clip = DrawingPaperClip {
        enabled: source.clip_boundary_handle != Handle::NULL,
        boundary_entity_id: mapping.get(&source.clip_boundary_handle).copied(),
    };
    for handle in &source.frozen_layers {
        if let Some(id) = document
            .layers
            .iter()
            .find(|l| l.handle == *handle)
            .and_then(|l| layers.get(&l.name.to_lowercase()))
        {
            target.layer_overrides.push(DrawingViewportLayerOverride {
                layer_id: *id,
                frozen: true,
                color: None,
                opacity: None,
                line_pattern: None,
                line_weight: None,
            });
        } else {
            diagnostics.push(ExportDiagnostic::loss(
                ExportDiagnosticSource::Entity {
                    handle: source.common.handle,
                    kind: "VIEWPORT".into(),
                },
                ExportAction::PartiallyExported,
                vec![ExportLossReason::MissingTarget {
                    kind: "viewport frozen layer".into(),
                    identifier: handle.to_string(),
                }],
            ));
        }
    }
    target
}
pub(super) fn to_cad(
    source: &DrawingViewport,
    document: &CadDocument,
    layers: &BTreeMap<u64, String>,
    mapping: &BTreeMap<u64, Handle>,
    diagnostics: &mut Vec<super::import::DirectImportDiagnostic>,
) -> Option<cadcodec::entities::Viewport> {
    let location = format!("/entities/{}", source.id);
    if source.view.projection == DrawingProjection::Perspective {
        diagnostics.push(super::import::diagnostic(
            "VIEWPORT",
            &location,
            "perspective viewport requires CAD fixture calibration",
        ));
        return None;
    }
    let clip = if source.paper_clip.enabled {
        let Some(handle) = source
            .paper_clip
            .boundary_entity_id
            .and_then(|id| mapping.get(&id))
            .copied()
        else {
            diagnostics.push(super::import::diagnostic(
                "VIEWPORT",
                &location,
                "clip boundary could not be mapped",
            ));
            return None;
        };
        handle
    } else {
        Handle::NULL
    };
    let frame = source.frame;
    let view = source.view;
    let mut target = cadcodec::entities::Viewport::new();
    target.center = Vector3::new(frame.center.x(), frame.center.y(), 0.0);
    target.width = frame.width;
    target.height = frame.height;
    target.view_center = Vector3::new(view.center.x(), view.center.y(), 0.0);
    target.view_target = Vector3::new(view.target.x(), view.target.y(), view.target.z());
    target.view_direction =
        Vector3::new(view.direction.x(), view.direction.y(), view.direction.z());
    target.view_height = view.height;
    target.twist_angle = view.twist;
    target.lens_length = view.lens_length.unwrap_or(target.lens_length);
    target.status.front_clipping = view.front_clip.mode != DrawingClipMode::Disabled;
    target.status.front_clip_not_at_eye = view.front_clip.mode == DrawingClipMode::AtDistance;
    target.front_clip_z = view.front_clip.distance.unwrap_or(0.0);
    target.status.back_clipping = view.back_clip.mode != DrawingClipMode::Disabled;
    target.back_clip_z = view.back_clip.distance.unwrap_or(0.0);
    target.clip_boundary_handle = clip;
    target.status.is_on = source.view_enabled;
    target.status.locked = source.view_locked;
    target.render_mode = render_to_cad(source.render_mode);
    target.id = document
        .entities()
        .filter_map(|e| {
            if let EntityType::Viewport(v) = e {
                Some(v.id)
            } else {
                None
            }
        })
        .max()
        .unwrap_or(1)
        .checked_add(1)?;
    for entry in &source.layer_overrides {
        if entry.frozen {
            if let Some(layer) = layers
                .get(&u64::from(entry.layer_id))
                .and_then(|name| document.layers.get(name))
            {
                target.frozen_layers.push(layer.handle);
            }
        }
        if entry.color.is_some()
            || entry.opacity.is_some()
            || entry.line_pattern.is_some()
            || entry.line_weight.is_some()
        {
            diagnostics.push(super::import::diagnostic(
                "VIEWPORT",
                &location,
                "viewport appearance override is not represented by CAD",
            ));
        }
    }
    if source.plot_shading_override.is_some() {
        diagnostics.push(super::import::diagnostic(
            "VIEWPORT",
            &location,
            "viewport plot-shading quality override is not represented by CAD",
        ));
    }
    Some(target)
}
