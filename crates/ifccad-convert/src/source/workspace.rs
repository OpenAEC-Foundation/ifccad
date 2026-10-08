//! CAD workspace roles and residual fields; native IDs remain in the adapter.
use crate::IfccadDiagnostic;
use opencadcodec::objects::{Layout, ObjectType};
use opencadcodec::{CadDocument, EntityType, Handle};
pub(crate) fn overall_handle(doc: &CadDocument, layout: &Layout) -> Option<Handle> {
    let qualifies = |v: &opencadcodec::entities::Viewport| {
        v.common.owner_handle == layout.block_record
            && (v.id == 1
                || (doc.dwg_source_version.is_some()
                    && v.id == 0
                    && layout.viewport == v.common.handle))
    };
    if matches!(doc.get_entity(layout.viewport),Some(EntityType::Viewport(v)) if qualifies(v)) {
        return Some(layout.viewport);
    }
    let mut candidates = doc.entities().filter_map(|e| match e {
        EntityType::Viewport(v) if qualifies(v) => Some(v.common.handle),
        _ => None,
    });
    let first = candidates.next()?;
    candidates.next().is_none().then_some(first)
}
pub(crate) fn overall(doc: &CadDocument, view: &opencadcodec::entities::Viewport) -> bool {
    doc.objects.values().any(
        |o| matches!(o,ObjectType::Layout(l) if overall_handle(doc,l) == Some(view.common.handle)),
    )
}
pub(crate) fn vport_residual(v: &opencadcodec::VPort, issues: &mut Vec<IfccadDiagnostic>) {
    let mut b = opencadcodec::VPort::active();
    b.name = v.name.clone();
    crate::source::residual(
        v,
        &b,
        &[
            "handle",
            "name",
            "lower_left",
            "upper_right",
            "view_center",
            "snap_base",
            "snap_spacing",
            "grid_spacing",
            "view_direction",
            "view_target",
            "view_height",
            "aspect_ratio",
            "lens_length",
            "view_twist",
            "front_clip",
            "back_clip",
            "grid_on",
            "snap_on",
            "snap_style",
            "snap_isopair",
            "snap_rotation",
            "render_mode",
            "perspective",
            "front_clipping",
            "back_clipping",
            "front_clip_at_eye",
            "ucs_per_viewport",
            "ucs_origin",
            "ucs_x_axis",
            "ucs_y_axis",
            "ucs_elevation",
            "grid_flags",
            "grid_major",
            "named_ucs_handle",
        ],
        &format!("vport/{}", v.handle),
        issues,
    );
}
pub(crate) fn ucs_residual(u: &opencadcodec::Ucs, issues: &mut Vec<IfccadDiagnostic>) {
    crate::source::residual(
        u,
        &opencadcodec::Ucs::new(&u.name),
        &["handle", "name", "origin", "x_axis", "y_axis", "elevation"],
        &format!("ucs/{}", u.handle),
        issues,
    );
}
pub(crate) fn viewport_residual(
    v: &opencadcodec::entities::Viewport,
    issues: &mut Vec<IfccadDiagnostic>,
) {
    let b = opencadcodec::entities::Viewport::new();
    let mut r = v.clone();
    r.status.grid_on = b.status.grid_on;
    r.status.snap_on = b.status.snap_on;
    r.status.isometric_snap = b.status.isometric_snap;
    r.status.iso_pair_top = b.status.iso_pair_top;
    r.status.iso_pair_right = b.status.iso_pair_right;
    let mask = 1 | 2 | 4 | 16 | 0x4000 | 0x8000 | 0x10000 | 0x20000;
    r.status = opencadcodec::entities::ViewportStatusFlags::from_bits(
        (r.status.to_bits() & !mask) | (b.status.to_bits() & mask),
    );
    crate::source::residual(
        &r,
        &b,
        &[
            "common",
            "center",
            "width",
            "height",
            "id",
            "view_center",
            "view_target",
            "view_direction",
            "lens_length",
            "front_clip_z",
            "back_clip_z",
            "view_height",
            "twist_angle",
            "render_mode",
            "frozen_layers",
            "clip_boundary_handle",
            "snap_base",
            "snap_spacing",
            "grid_spacing",
            "snap_angle",
            "grid_flags",
            "grid_major",
            "ucs_per_viewport",
            "ucs_origin",
            "ucs_x_axis",
            "ucs_y_axis",
            "ucs_handle",
            "elevation",
        ],
        &format!("entity/{}", v.common.handle),
        issues,
    );
}

pub(crate) fn authored(doc: &CadDocument, view: &opencadcodec::entities::Viewport) -> bool {
    view.id >= 2 || (doc.dwg_source_version.is_some() && view.id == 0 && doc.objects.values().any(|o|matches!(o,ObjectType::Layout(l) if l.block_record == view.common.owner_handle && overall_handle(doc,l).is_some_and(|h|h != view.common.handle))))
}

pub(crate) fn canvas_residual(
    v: &opencadcodec::entities::Viewport,
    issues: &mut Vec<IfccadDiagnostic>,
) {
    viewport_residual(v, issues);
    let b = opencadcodec::entities::Viewport::new();
    let mask = 0x4000 | 0x8000 | 0x10000 | 0x20000;
    if (v.status.to_bits() ^ b.status.to_bits()) & mask != 0 {
        issues.push(crate::diagnostics::diagnostic(
            "source-field",
            format!("entity/{}.canvas.status", v.common.handle),
            "canvas on/off/locked/clipping display state has no native snapshot field",
        ));
    }
    for (field, changed) in [
        ("render_mode", v.render_mode != b.render_mode),
        ("frozen_layers", !v.frozen_layers.is_empty()),
        ("clip_boundary_handle", !v.clip_boundary_handle.is_null()),
    ] {
        if changed {
            issues.push(crate::diagnostics::diagnostic(
                "source-field",
                format!("entity/{}.canvas.{field}", v.common.handle),
                "canvas display property has no native snapshot field",
            ));
        }
    }
}
