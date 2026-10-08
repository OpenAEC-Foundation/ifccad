use crate::{diagnostics::diagnostic, IfccadDiagnostic, IfccadMappings};
use ocdraw::ifccad::*;
use opencadcodec::entities::{Viewport, ViewportRenderMode, ViewportStatusFlags};
use opencadcodec::{CadDocument, Handle};

pub(crate) fn codec_supports_clipping() -> bool {
    ViewportStatusFlags::from_bits(0x30000).to_bits() == 0x30000
}

pub(crate) fn checked_viewport_number(authored_index: usize) -> Option<i16> {
    i16::try_from(authored_index.checked_add(2)?).ok()
}

pub(crate) fn overall_canvas(doc: &CadDocument, v: &Viewport) -> bool {
    crate::source::workspace::overall(doc, v)
}

pub(crate) fn from_cad(
    v: &Viewport,
    model_id: u64,
    mappings: &IfccadMappings,
    loc: &str,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Option<IfccadViewport> {
    if !codec_supports_clipping() {
        issues.push(diagnostic(
            "viewport-codec",
            loc,
            "viewport conversion requires the explicit codec clipping repair",
        ));
        return None;
    }
    if v.center.z != 0. || v.view_center.z != 0. {
        issues.push(diagnostic(
            "viewport-camera",
            loc,
            "nonzero frame or DCS elevation cannot be represented",
        ));
        return None;
    }
    let mut frozen_layers = std::collections::BTreeSet::new();
    for handle in &v.frozen_layers {
        if let Some(id) = mappings.layers.ifccad_id(*handle) {
            frozen_layers.insert(id);
        } else {
            issues.push(diagnostic(
                "viewport-frozen-layer",
                format!("{loc}.frozenLayers/{handle}"),
                "unresolved frozen layer omitted",
            ));
        }
    }
    let result = IfccadViewport {
        workspace: None,

        model_id,
        frame: IfccadViewportFrame {
            center: [v.center.x, v.center.y],
            width: v.width,
            height: v.height,
        },
        view: IfccadViewportView {
            center: [v.view_center.x, v.view_center.y],
            target: crate::mapping::geometry::p(v.view_target),
            direction: crate::mapping::geometry::p(v.view_direction),
            height: v.view_height,
            twist: v.twist_angle,
            projection: if v.status.perspective {
                IfccadViewportProjection::Perspective
            } else {
                IfccadViewportProjection::Orthographic
            },
            lens_length_mm: Some(v.lens_length),
            front_clip: IfccadViewportDepthClip {
                mode: if !v.status.front_clipping {
                    IfccadViewportClipMode::Disabled
                } else if v.status.front_clip_not_at_eye {
                    IfccadViewportClipMode::AtDistance
                } else {
                    IfccadViewportClipMode::AtCamera
                },
                distance: Some(v.front_clip_z),
            },
            back_clip: IfccadViewportDepthClip {
                mode: if v.status.back_clipping {
                    IfccadViewportClipMode::AtDistance
                } else {
                    IfccadViewportClipMode::Disabled
                },
                distance: Some(v.back_clip_z),
            },
        },
        render_mode: match v.render_mode {
            ViewportRenderMode::Wireframe2D => IfccadViewportRenderMode::TwoDimensional,
            ViewportRenderMode::Wireframe3D => IfccadViewportRenderMode::Wireframe,
            ViewportRenderMode::HiddenLine => IfccadViewportRenderMode::HiddenLine,
            ViewportRenderMode::FlatShaded => IfccadViewportRenderMode::FlatShadedWithoutEdges,
            ViewportRenderMode::GouraudShaded => IfccadViewportRenderMode::SmoothShadedWithoutEdges,
            ViewportRenderMode::FlatShadedWithEdges => {
                IfccadViewportRenderMode::FlatShadedWithEdges
            }
            ViewportRenderMode::GouraudShadedWithEdges => {
                IfccadViewportRenderMode::SmoothShadedWithEdges
            }
        },
        view_enabled: v.status.is_on && v.status.to_bits() & 0x20000 == 0,
        view_locked: v.status.locked,
        paper_clip: IfccadViewportPaperClip {
            enabled: v.status.to_bits() & 0x10000 != 0,
            boundary_entity_id: None,
        },
        layer_overrides: frozen_layers
            .into_iter()
            .map(|layer_id| IfccadViewportLayerOverride {
                layer_id,
                frozen: true,
                color: None,
                opacity: None,
                line_pattern_id: None,
                line_weight: None,
            })
            .collect(),
        plot_shading_override: match v.shade_plot_mode {
            0 => None,
            1 => Some(ocdraw::plot_kernel::ShadedPlotMode::Wireframe),
            2 => Some(ocdraw::plot_kernel::ShadedPlotMode::Hidden),
            3 => Some(ocdraw::plot_kernel::ShadedPlotMode::Rendered),
            _ => None, // The source residual diagnoses unsupported plot modes.
        },
    };
    if let Err(report) = validate_ifccad_viewport_parameters(&result) {
        issues.push(diagnostic(
            "viewport-camera",
            loc,
            format!("unsupported or invalid viewport parameters: {report:?}"),
        ));
        return None;
    }
    crate::source::workspace::viewport_residual(v, issues);
    Some(result)
}

pub(crate) fn to_cad(
    v: &IfccadViewport,
    id: i16,
    boundary: Handle,
    mappings: &IfccadMappings,
    loc: &str,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Option<Viewport> {
    if !codec_supports_clipping() {
        issues.push(diagnostic(
            "viewport-codec",
            loc,
            "viewport conversion requires the explicit codec clipping repair",
        ));
        return None;
    }
    let mut target = Viewport::new();
    target.id = id;
    target.center = opencadcodec::Vector3::new(v.frame.center[0], v.frame.center[1], 0.);
    target.width = v.frame.width;
    target.height = v.frame.height;
    target.view_center = opencadcodec::Vector3::new(v.view.center[0], v.view.center[1], 0.);
    target.view_target = crate::mapping::geometry::v(v.view.target);
    target.view_direction = crate::mapping::geometry::v(v.view.direction);
    target.view_height = v.view.height;
    target.twist_angle = v.view.twist;
    if let Some(lens) = v.view.lens_length_mm {
        target.lens_length = lens;
    }
    if let Some(distance) = v.view.front_clip.distance {
        target.front_clip_z = distance;
    }
    if let Some(distance) = v.view.back_clip.distance {
        target.back_clip_z = distance;
    }
    target.status = ViewportStatusFlags::from_bits(
        if v.view.projection == IfccadViewportProjection::Perspective {
            1
        } else {
            0
        } | if v.view.front_clip.mode != IfccadViewportClipMode::Disabled {
            2
        } else {
            0
        } | if v.view.back_clip.mode != IfccadViewportClipMode::Disabled {
            4
        } else {
            0
        } | if v.view.front_clip.mode == IfccadViewportClipMode::AtDistance {
            16
        } else {
            0
        } | if v.view_locked { 0x4000 } else { 0 }
            | 0x8000
            | if v.view_enabled { 0 } else { 0x20000 }
            | if v.paper_clip.enabled { 0x10000 } else { 0 },
    );
    target.clip_boundary_handle = boundary;
    target.shade_plot_mode = match v
        .plot_shading_override
        .unwrap_or(ocdraw::plot_kernel::ShadedPlotMode::AsDisplayed)
    {
        ocdraw::plot_kernel::ShadedPlotMode::AsDisplayed => 0,
        ocdraw::plot_kernel::ShadedPlotMode::Wireframe => 1,
        ocdraw::plot_kernel::ShadedPlotMode::Hidden => 2,
        ocdraw::plot_kernel::ShadedPlotMode::Rendered => 3,
    };
    target.frozen_layers = v
        .layer_overrides
        .iter()
        .filter(|row| row.frozen)
        .map(|row| {
            mappings
                .layers
                .cad_handle(row.layer_id)
                .expect("allocated layer")
        })
        .collect();
    target.render_mode = match v.render_mode {
        IfccadViewportRenderMode::TwoDimensional => ViewportRenderMode::Wireframe2D,
        IfccadViewportRenderMode::Wireframe => ViewportRenderMode::Wireframe3D,
        IfccadViewportRenderMode::HiddenLine => ViewportRenderMode::HiddenLine,
        IfccadViewportRenderMode::FlatShadedWithoutEdges => ViewportRenderMode::FlatShaded,
        IfccadViewportRenderMode::SmoothShadedWithoutEdges => ViewportRenderMode::GouraudShaded,
        IfccadViewportRenderMode::FlatShadedWithEdges => ViewportRenderMode::FlatShadedWithEdges,
        IfccadViewportRenderMode::SmoothShadedWithEdges => {
            ViewportRenderMode::GouraudShadedWithEdges
        }
    };
    Some(target)
}

#[cfg(test)]
mod tests {
    #[test]
    fn viewport_number_range_is_checked() {
        assert_eq!(super::checked_viewport_number(0), Some(2));
        assert_eq!(super::checked_viewport_number(32765), Some(i16::MAX));
        assert_eq!(super::checked_viewport_number(32766), None);
        assert_eq!(super::checked_viewport_number(usize::MAX), None);
    }
}
