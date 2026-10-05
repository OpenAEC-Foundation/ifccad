use crate::{diagnostics::diagnostic, IfcxCadDiagnostic, IfcxCadMappings};
use ocdraw::ifcx_cad::*;
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
    mappings: &IfcxCadMappings,
    loc: &str,
    issues: &mut Vec<IfcxCadDiagnostic>,
) -> Option<IfcxCadViewport> {
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
        if let Some(id) = mappings.layers.ifcx_id(*handle) {
            frozen_layers.insert(id);
        } else {
            issues.push(diagnostic(
                "viewport-frozen-layer",
                format!("{loc}.frozenLayers/{handle}"),
                "unresolved frozen layer omitted",
            ));
        }
    }
    let result = IfcxCadViewport {
        model_id,
        frame: IfcxCadViewportFrame {
            center: [v.center.x, v.center.y],
            width: v.width,
            height: v.height,
        },
        view: IfcxCadViewportView {
            center: [v.view_center.x, v.view_center.y],
            target: crate::geometry::p(v.view_target),
            direction: crate::geometry::p(v.view_direction),
            height: v.view_height,
            twist: v.twist_angle,
            projection: if v.status.perspective {
                IfcxCadViewportProjection::Perspective
            } else {
                IfcxCadViewportProjection::Orthographic
            },
            lens_length_mm: Some(v.lens_length),
            front_clip: IfcxCadViewportDepthClip {
                mode: if !v.status.front_clipping {
                    IfcxCadViewportClipMode::Disabled
                } else if v.status.front_clip_not_at_eye {
                    IfcxCadViewportClipMode::AtDistance
                } else {
                    IfcxCadViewportClipMode::AtCamera
                },
                distance: Some(v.front_clip_z),
            },
            back_clip: IfcxCadViewportDepthClip {
                mode: if v.status.back_clipping {
                    IfcxCadViewportClipMode::AtDistance
                } else {
                    IfcxCadViewportClipMode::Disabled
                },
                distance: Some(v.back_clip_z),
            },
        },
        render_mode: match v.render_mode {
            ViewportRenderMode::Wireframe2D => IfcxCadViewportRenderMode::TwoDimensional,
            ViewportRenderMode::Wireframe3D => IfcxCadViewportRenderMode::Wireframe,
            ViewportRenderMode::HiddenLine => IfcxCadViewportRenderMode::HiddenLine,
            ViewportRenderMode::FlatShaded => IfcxCadViewportRenderMode::FlatShadedWithoutEdges,
            ViewportRenderMode::GouraudShaded => {
                IfcxCadViewportRenderMode::SmoothShadedWithoutEdges
            }
            ViewportRenderMode::FlatShadedWithEdges => {
                IfcxCadViewportRenderMode::FlatShadedWithEdges
            }
            ViewportRenderMode::GouraudShadedWithEdges => {
                IfcxCadViewportRenderMode::SmoothShadedWithEdges
            }
        },
        view_enabled: v.status.is_on && v.status.to_bits() & 0x20000 == 0,
        view_locked: v.status.locked,
        visible: !v.common.invisible,
        paper_clip: IfcxCadViewportPaperClip {
            enabled: v.status.to_bits() & 0x10000 != 0,
            boundary_entity_id: None,
        },
        frozen_layers: frozen_layers.into_iter().collect(),
    };
    if let Err(report) = validate_ifcx_cad_viewport_parameters(&result) {
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
    v: &IfcxCadViewport,
    id: i16,
    boundary: Handle,
    mappings: &IfcxCadMappings,
    loc: &str,
    issues: &mut Vec<IfcxCadDiagnostic>,
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
        if v.view.projection == IfcxCadViewportProjection::Perspective {
            1
        } else {
            0
        } | if v.view.front_clip.mode != IfcxCadViewportClipMode::Disabled {
            2
        } else {
            0
        } | if v.view.back_clip.mode != IfcxCadViewportClipMode::Disabled {
            4
        } else {
            0
        } | if v.view.front_clip.mode == IfcxCadViewportClipMode::AtDistance {
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
        IfcxCadViewportRenderMode::TwoDimensional => ViewportRenderMode::Wireframe2D,
        IfcxCadViewportRenderMode::Wireframe => ViewportRenderMode::Wireframe3D,
        IfcxCadViewportRenderMode::HiddenLine => ViewportRenderMode::HiddenLine,
        IfcxCadViewportRenderMode::FlatShadedWithoutEdges => ViewportRenderMode::FlatShaded,
        IfcxCadViewportRenderMode::SmoothShadedWithoutEdges => ViewportRenderMode::GouraudShaded,
        IfcxCadViewportRenderMode::FlatShadedWithEdges => ViewportRenderMode::FlatShadedWithEdges,
        IfcxCadViewportRenderMode::SmoothShadedWithEdges => {
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
