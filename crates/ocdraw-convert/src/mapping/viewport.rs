use crate::source::{
    CadToOcdrawAction, CadToOcdrawDiagnostic, CadToOcdrawDiagnosticSource, CadToOcdrawLossReason,
};
use ocdraw::ocdraw::*;
use opencadcodec::{CadDocument, EntityType, Handle, Vector3};
use std::collections::BTreeMap;
pub(crate) fn losses(viewport: &opencadcodec::entities::Viewport) -> Vec<CadToOcdrawLossReason> {
    let mut reasons = Vec::new();
    if ocdraw::workspace_kernel::validate_view(
        &view_from_cad(viewport),
        ocdraw::workspace_kernel::WorkspaceViewKind::Model,
    )
    .is_err()
    {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "invalid viewport lens, direction or clip planes".into(),
        });
    }
    if viewport.center.z != 0.0 || viewport.view_center.z != 0.0 {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
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
        reasons.push(CadToOcdrawLossReason::NonFiniteCoordinate);
    }
    reasons
}

pub(crate) fn deferred_losses(
    viewport: &opencadcodec::entities::Viewport,
) -> Vec<CadToOcdrawLossReason> {
    let baseline = opencadcodec::entities::Viewport::new();
    let mut reasons = Vec::new();
    let mut status = viewport.status;
    // Clipping activation is mapped independently of boundary presence.
    status = opencadcodec::entities::ViewportStatusFlags::from_bits(status.to_bits() & !0x10000);
    if viewport.off_screen {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "viewport off-screen state".into(),
        });
    }
    status.is_on = baseline.status.is_on;
    status.is_off = baseline.status.is_off;
    status.grid_on = baseline.status.grid_on;
    status.snap_on = baseline.status.snap_on;
    status.isometric_snap = baseline.status.isometric_snap;
    status.iso_pair_top = baseline.status.iso_pair_top;
    status.iso_pair_right = baseline.status.iso_pair_right;
    status.locked = baseline.status.locked;
    status.perspective = baseline.status.perspective;
    status.front_clipping = baseline.status.front_clipping;
    status.back_clipping = baseline.status.back_clipping;
    status.front_clip_not_at_eye = baseline.status.front_clip_not_at_eye;
    if status != baseline.status || viewport.circle_sides != baseline.circle_sides {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "viewport display status/circleSides".into(),
        });
    }
    if viewport.ucs_at_origin != baseline.ucs_at_origin
        || viewport.ucs_icon_visible != baseline.ucs_icon_visible
        || viewport.base_ucs_handle != Handle::NULL
        || viewport.ucs_ortho_type != baseline.ucs_ortho_type
    {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "viewport UCS icon/base/orthographic state".into(),
        });
    }
    if !viewport.style_sheet.is_empty()
        || !(0..=3).contains(&viewport.shade_plot_mode)
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
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {
            name: "viewport visual and plot state".into(),
        });
    }
    reasons
}
pub(crate) fn deferred_canvas_losses(
    viewport: &opencadcodec::entities::Viewport,
) -> Vec<CadToOcdrawLossReason> {
    let mut reasons = deferred_losses(viewport);
    let baseline = opencadcodec::entities::Viewport::new();
    let mask = 0x4000 | 0x8000 | 0x10000 | 0x20000;
    if (viewport.status.to_bits() ^ baseline.status.to_bits()) & mask != 0
        || viewport.shade_plot_mode != baseline.shade_plot_mode
        || viewport.render_mode != baseline.render_mode
        || !viewport.frozen_layers.is_empty()
        || !viewport.clip_boundary_handle.is_null()
    {
        reasons.push(CadToOcdrawLossReason::UnsupportedSemantic {name:"canvas display status/renderMode/frozenLayers/clipBoundary has no native snapshot field".into()});
    }
    reasons.extend(crate::source::direct_common_losses(&viewport.common));
    reasons
}
fn render_from_cad(mode: opencadcodec::entities::ViewportRenderMode) -> DrawingRenderMode {
    use opencadcodec::entities::ViewportRenderMode as Cad;
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
fn render_to_cad(mode: DrawingRenderMode) -> opencadcodec::entities::ViewportRenderMode {
    use opencadcodec::entities::ViewportRenderMode as Cad;
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
fn view_from_cad(source: &opencadcodec::entities::Viewport) -> DrawingView {
    DrawingView {
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
        projection: if source.status.perspective {
            DrawingProjection::Perspective
        } else {
            DrawingProjection::Orthographic
        },
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
    }
}
pub(crate) struct SourceIndex<'a> {
    pub document: &'a CadDocument,
    pub layers: &'a BTreeMap<String, u32>,
}
pub(crate) fn from_cad(
    source: &opencadcodec::entities::Viewport,
    scope: u32,
    layer: u32,
    appearance: EntityAppearance,
    index: &SourceIndex<'_>,
    diagnostics: &mut Vec<CadToOcdrawDiagnostic>,
) -> ViewportDefinition {
    let SourceIndex { document, layers } = index;
    let view = view_from_cad(source);
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
    target.plot_shading_override = match source.shade_plot_mode {
        0 => None,
        1 => Some(ocdraw::plot_kernel::ShadedPlotMode::Wireframe),
        2 => Some(ocdraw::plot_kernel::ShadedPlotMode::Hidden),
        3 => Some(ocdraw::plot_kernel::ShadedPlotMode::Rendered),
        _ => None,
    };
    target.view_enabled = source.is_on();
    target.view_locked = source.status.locked;
    target.appearance = appearance;
    target.visible = !source.common.invisible;
    target.paper_clip.enabled = source.status.to_bits() & 0x10000 != 0;
    // Clip dependencies are resolved from converted geometry during ordered emission.
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
                line_pattern_id: None,
                line_weight: None,
            });
        } else {
            diagnostics.push(CadToOcdrawDiagnostic::loss(
                CadToOcdrawDiagnosticSource::Entity {
                    handle: source.common.handle,
                    kind: "VIEWPORT".into(),
                },
                CadToOcdrawAction::PartiallyExported,
                vec![CadToOcdrawLossReason::MissingTarget {
                    kind: "viewport frozen layer".into(),
                    identifier: handle.to_string(),
                }],
            ));
        }
    }
    target
}
pub(crate) fn to_cad(
    source: &DrawingViewport,
    document: &CadDocument,
    layers: &BTreeMap<u64, String>,
    diagnostics: &mut Vec<crate::OcdrawToCadDiagnostic>,
) -> Option<opencadcodec::entities::Viewport> {
    let location = format!("/entities/{}", source.id);
    let frame = source.frame;
    let view = source.view;
    let mut target = opencadcodec::entities::Viewport::new();
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
    target.status.perspective = view.projection == DrawingProjection::Perspective;
    target.status.front_clipping = view.front_clip.mode != DrawingClipMode::Disabled;
    target.status.front_clip_not_at_eye = view.front_clip.mode == DrawingClipMode::AtDistance;
    target.front_clip_z = view.front_clip.distance.unwrap_or(0.0);
    target.status.back_clipping = view.back_clip.mode != DrawingClipMode::Disabled;
    target.back_clip_z = view.back_clip.distance.unwrap_or(0.0);
    target.clip_boundary_handle = Handle::NULL;
    if source.view_enabled {
        target.turn_on();
    } else {
        target.turn_off();
    }
    target.status.locked = source.view_locked;
    target.status = opencadcodec::entities::ViewportStatusFlags::from_bits(
        target.status.to_bits()
            | if source.paper_clip.enabled {
                0x10000
            } else {
                0
            },
    );
    target.render_mode = render_to_cad(source.render_mode);
    target.shade_plot_mode = match source
        .plot_shading_override
        .unwrap_or(ocdraw::plot_kernel::ShadedPlotMode::AsDisplayed)
    {
        ocdraw::plot_kernel::ShadedPlotMode::AsDisplayed => 0,
        ocdraw::plot_kernel::ShadedPlotMode::Wireframe => 1,
        ocdraw::plot_kernel::ShadedPlotMode::Hidden => 2,
        ocdraw::plot_kernel::ShadedPlotMode::Rendered => 3,
    };
    let next_number = document
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
        .checked_add(1);
    let Some(number) = next_number else {
        diagnostics.push(crate::to_cad::diagnostic(
            "VIEWPORT",
            &location,
            "CAD viewport number exceeds i16; viewport omitted",
        ));
        return None;
    };
    target.id = number;
    for entry in &source.layer_overrides {
        if entry.frozen {
            if let Some(layer) = layers
                .get(&u64::from(entry.layer_id))
                .and_then(|name| document.layers.get(name))
            {
                target.frozen_layers.push(layer.handle);
            }
        }
    }
    Some(target)
}
