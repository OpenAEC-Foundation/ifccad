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
    v.id == 1
        || (doc.dwg_source_version.is_some()
            && v.id == 0
            && doc.objects.values().any(|o| {
                matches!(o,opencadcodec::objects::ObjectType::Layout(l)
            if l.block_record==v.common.owner_handle && l.viewport==v.common.handle)
            }))
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
        model_id,
        frame: IfccadViewportFrame {
            center: [v.center.x, v.center.y],
            width: v.width,
            height: v.height,
        },
        view: IfccadViewportView {
            center: [v.view_center.x, v.view_center.y],
            target: crate::geometry::p(v.view_target),
            direction: crate::geometry::p(v.view_direction),
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
        visible: !v.common.invisible,
        paper_clip: IfccadViewportPaperClip {
            enabled: v.status.to_bits() & 0x10000 != 0,
            boundary_entity_id: None,
        },
        frozen_layers: frozen_layers.into_iter().collect(),
    };
    if let Err(report) = validate_ifccad_viewport_parameters(&result) {
        issues.push(diagnostic(
            "viewport-camera",
            loc,
            format!("unsupported or invalid viewport parameters: {report:?}"),
        ));
        return None;
    }
    let b = Viewport::new();
    let mut r = v.clone();
    r.common = b.common.clone();
    r.center = b.center;
    r.width = b.width;
    r.height = b.height;
    r.id = b.id;
    r.view_center = b.view_center;
    r.view_target = b.view_target;
    r.view_direction = b.view_direction;
    r.lens_length = b.lens_length;
    r.front_clip_z = b.front_clip_z;
    r.back_clip_z = b.back_clip_z;
    r.view_height = b.view_height;
    r.twist_angle = b.twist_angle;
    r.render_mode = b.render_mode;
    r.frozen_layers = b.frozen_layers.clone();
    r.clip_boundary_handle = b.clip_boundary_handle;
    let mask = 1
        | 2
        | 4
        | 0x4000
        | 0x8000
        | 0x10000
        | 0x20000
        | if v.status.front_clipping { 16 } else { 0 };
    r.status =
        ViewportStatusFlags::from_bits((v.status.to_bits() & !mask) | (b.status.to_bits() & mask));
    if r != b {
        issues.push(diagnostic(
            "viewport-state",
            loc,
            "unmapped viewport workspace, plot, off-screen or visual state omitted",
        ));
    }
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
    target.view_target = crate::geometry::v(v.view.target);
    target.view_direction = crate::geometry::v(v.view.direction);
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
    target.frozen_layers = v
        .frozen_layers
        .iter()
        .map(|id| mappings.layers.cad_handle(*id).expect("allocated layer"))
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
