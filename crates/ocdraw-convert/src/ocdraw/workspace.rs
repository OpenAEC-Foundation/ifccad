//! Conversion of saved CAD view state through typed drawing values.
mod import;
use crate::source::{ExportAction, ExportDiagnostic, ExportDiagnosticSource, ExportLossReason};
use cadcodec::entities::EntityType;
use cadcodec::objects::ObjectType;
use cadcodec::tables::VPort;
use cadcodec::{CadDocument, Handle};
pub(super) use import::apply;
use ocdraw::ocdraw::{
    CoordinateFrame3, DrawingBuilder, DrawingClip as Clip, DrawingClipMode as FrontClipMode,
    DrawingClipMode as BackClipMode, DrawingGrid as WorkspaceGrid,
    DrawingGridStyle as WorkspaceGridStyle, DrawingIsometricPlane as IsometricPlane,
    DrawingModelWindow as ModelWindow, DrawingPaperCanvas as PaperCanvas,
    DrawingPaperContext as PaperActiveContext, DrawingProjection as ProjectionMode,
    DrawingRenderMode as ViewportRenderMode, DrawingSavedState, DrawingSnap as WorkspaceSnap,
    DrawingSnapStyle as WorkspaceSnapStyle, DrawingUcsSelection as UcsSelection,
    DrawingView as ViewDefinition, DrawingViewState, Point2, Point3, Vector3,
};
use std::collections::{BTreeMap, BTreeSet};
fn loss(diagnostics: &mut Vec<ExportDiagnostic>, name: &str) {
    diagnostics.push(ExportDiagnostic::loss(
        ExportDiagnosticSource::DocumentField { name: name.into() },
        ExportAction::Skipped,
        vec![ExportLossReason::UnsupportedSemantic { name: name.into() }],
    ));
}
fn point3(value: cadcodec::Vector3) -> Point3 {
    Point3::new(value.x, value.y, value.z)
}
fn vector3(value: cadcodec::Vector3) -> Vector3 {
    Vector3::new(value.x, value.y, value.z)
}
fn frame(
    origin: cadcodec::Vector3,
    x: cadcodec::Vector3,
    y: cadcodec::Vector3,
) -> Option<CoordinateFrame3> {
    CoordinateFrame3::try_new(point3(origin), vector3(x), vector3(y)).ok()
}
fn source_ucs(
    name: &str,
    origin: cadcodec::Vector3,
    x: cadcodec::Vector3,
    y: cadcodec::Vector3,
    names: &BTreeMap<String, u32>,
) -> Option<UcsSelection> {
    if !name.is_empty() {
        return names
            .get(&name.to_lowercase())
            .copied()
            .map(UcsSelection::Named);
    }
    let frame = frame(origin, x, y)?;
    Some(if frame == CoordinateFrame3::default() {
        UcsSelection::World
    } else {
        UcsSelection::Unnamed(frame)
    })
}
fn vport_ucs(vport: &VPort, handles: &BTreeMap<Handle, u32>) -> Option<UcsSelection> {
    if vport.named_ucs_handle != Handle::NULL {
        return handles
            .get(&vport.named_ucs_handle)
            .copied()
            .map(UcsSelection::Named);
    }
    source_ucs(
        "",
        vport.ucs_origin,
        vport.ucs_x_axis,
        vport.ucs_y_axis,
        &BTreeMap::new(),
    )
}

fn model_window(vport: &VPort, id: u32, stored_ucs: UcsSelection) -> Option<ModelWindow> {
    let rect = [
        vport.lower_left.x,
        vport.lower_left.y,
        vport.upper_right.x,
        vport.upper_right.y,
    ];
    let render_mode = match vport.render_mode {
        cadcodec::entities::ViewportRenderMode::Wireframe2D => ViewportRenderMode::TwoDimensional,
        cadcodec::entities::ViewportRenderMode::Wireframe3D => ViewportRenderMode::Wireframe,
        cadcodec::entities::ViewportRenderMode::HiddenLine => ViewportRenderMode::HiddenLine,
        cadcodec::entities::ViewportRenderMode::FlatShaded => {
            ViewportRenderMode::FlatShadedWithoutEdges
        }
        cadcodec::entities::ViewportRenderMode::FlatShadedWithEdges => {
            ViewportRenderMode::FlatShadedWithEdges
        }
        cadcodec::entities::ViewportRenderMode::GouraudShaded => {
            ViewportRenderMode::SmoothShadedWithoutEdges
        }
        cadcodec::entities::ViewportRenderMode::GouraudShadedWithEdges => {
            ViewportRenderMode::SmoothShadedWithEdges
        }
    };
    let grid = WorkspaceGrid {
        enabled: vport.grid_on,
        spacing: Point2::new(vport.grid_spacing.x, vport.grid_spacing.y),
        style: WorkspaceGridStyle::Lines,
        major_line_frequency: u32::try_from(vport.grid_major).ok()?,
        beyond_limits: vport.grid_flags.beyond_limits,
        adaptive: vport.grid_flags.adaptive,
        subdivision: vport.grid_flags.subdivision,
        follows_workplane: vport.grid_flags.follow_dynamic,
    };
    let snap = WorkspaceSnap {
        enabled: vport.snap_on,
        base: Point2::new(vport.snap_base.x, vport.snap_base.y),
        spacing: Point2::new(vport.snap_spacing.x, vport.snap_spacing.y),
        angle: vport.snap_rotation,
        style: if vport.snap_style {
            WorkspaceSnapStyle::Isometric
        } else {
            WorkspaceSnapStyle::Rectangular
        },
        isometric_plane: match vport.snap_isopair {
            0 => IsometricPlane::Left,
            1 => IsometricPlane::Top,
            2 => IsometricPlane::Right,
            _ => return None,
        },
    };
    Some(ModelWindow {
        id,
        rectangle: rect,
        view: ViewDefinition {
            center: Point2::new(vport.view_center.x, vport.view_center.y),
            target: point3(vport.view_target),
            direction: vector3(vport.view_direction),
            height: vport.view_height,
            twist: vport.view_twist,
            projection: if vport.perspective {
                ProjectionMode::Perspective
            } else {
                ProjectionMode::Orthographic
            },
            lens_length: Some(vport.lens_length),
            front_clip: Clip {
                mode: if vport.front_clipping {
                    if vport.front_clip_at_eye {
                        FrontClipMode::AtCamera
                    } else {
                        FrontClipMode::AtDistance
                    }
                } else {
                    FrontClipMode::Disabled
                },
                distance: Some(vport.front_clip),
            },
            back_clip: Clip {
                mode: if vport.back_clipping {
                    BackClipMode::AtDistance
                } else {
                    BackClipMode::Disabled
                },
                distance: Some(vport.back_clip),
            },
        },
        aspect_ratio: vport.aspect_ratio,
        render_mode,
        grid,
        snap,
        stored_ucs,
        use_stored_ucs: vport.ucs_per_viewport,
    })
}

fn paper_canvas(
    viewport: &cadcodec::entities::Viewport,
    scope_id: u32,
    handles: &BTreeMap<Handle, u32>,
) -> Option<PaperCanvas> {
    if viewport.id != 1 || viewport.status.perspective {
        return None;
    }
    if viewport.status.snap_on && (viewport.snap_spacing.x <= 0.0 || viewport.snap_spacing.y <= 0.0)
    {
        return None;
    }
    let stored_ucs = if viewport.ucs_handle != Handle::NULL {
        UcsSelection::Named(*handles.get(&viewport.ucs_handle)?)
    } else {
        source_ucs(
            "",
            viewport.ucs_origin,
            viewport.ucs_x_axis,
            viewport.ucs_y_axis,
            &BTreeMap::new(),
        )?
    };
    let grid = WorkspaceGrid {
        enabled: viewport.status.grid_on,
        spacing: Point2::new(viewport.grid_spacing.x, viewport.grid_spacing.y),
        style: WorkspaceGridStyle::Lines,
        major_line_frequency: u32::try_from(viewport.grid_major).ok()?,
        beyond_limits: viewport.grid_flags.beyond_limits,
        adaptive: viewport.grid_flags.adaptive,
        subdivision: viewport.grid_flags.subdivision,
        follows_workplane: viewport.grid_flags.follow_dynamic,
    };
    let snap = WorkspaceSnap {
        enabled: viewport.status.snap_on,
        base: Point2::new(viewport.snap_base.x, viewport.snap_base.y),
        spacing: Point2::new(
            if viewport.snap_spacing.x > 0.0 {
                viewport.snap_spacing.x
            } else {
                1.0
            },
            if viewport.snap_spacing.y > 0.0 {
                viewport.snap_spacing.y
            } else {
                1.0
            },
        ),
        angle: viewport.snap_angle,
        style: if viewport.status.isometric_snap {
            WorkspaceSnapStyle::Isometric
        } else {
            WorkspaceSnapStyle::Rectangular
        },
        isometric_plane: if viewport.status.iso_pair_right {
            IsometricPlane::Right
        } else if viewport.status.iso_pair_top {
            IsometricPlane::Top
        } else {
            IsometricPlane::Left
        },
    };
    Some(PaperCanvas {
        scope_id,
        view: ViewDefinition {
            center: Point2::new(viewport.view_center.x, viewport.view_center.y),
            target: point3(viewport.view_target),
            direction: vector3(viewport.view_direction),
            height: viewport.view_height,
            twist: viewport.twist_angle,
            projection: ProjectionMode::Orthographic,
            lens_length: Some(viewport.lens_length),
            front_clip: Clip {
                mode: if viewport.status.front_clipping {
                    if viewport.status.front_clip_not_at_eye {
                        FrontClipMode::AtDistance
                    } else {
                        FrontClipMode::AtCamera
                    }
                } else {
                    FrontClipMode::Disabled
                },
                distance: Some(viewport.front_clip_z),
            },
            back_clip: Clip {
                mode: if viewport.status.back_clipping {
                    BackClipMode::AtDistance
                } else {
                    BackClipMode::Disabled
                },
                distance: Some(viewport.back_clip_z),
            },
        },
        grid,
        snap,
        stored_ucs,
        current_ucs: stored_ucs,
        active_context: PaperActiveContext::Canvas,
    })
}

pub(super) fn export(
    document: &CadDocument,
    drawing: &mut DrawingBuilder,
    paper_scopes: &BTreeMap<Handle, u32>,
    names: &BTreeMap<String, u32>,
    handles: &BTreeMap<Handle, u32>,
    diagnostics: &mut Vec<ExportDiagnostic>,
) -> BTreeSet<Handle> {
    let mut saved = DrawingSavedState::default();
    let mut mapped = BTreeSet::new();
    let Some(current_model_ucs) = source_ucs(
        &document.header.model_space_ucs_name,
        document.header.model_space_ucs_origin,
        document.header.model_space_ucs_x_axis,
        document.header.model_space_ucs_y_axis,
        names,
    ) else {
        loss(diagnostics, "header.model_space_ucs");
        return mapped;
    };
    for source in document
        .vports
        .iter()
        .filter(|v| v.name.eq_ignore_ascii_case("*active"))
    {
        let Some(stored) = vport_ucs(source, handles) else {
            loss(diagnostics, "VPORT UCS");
            return BTreeSet::new();
        };
        let Some(window) = model_window(source, saved.model_windows.len() as u32, stored) else {
            loss(diagnostics, "VPORT view/grid/snap");
            return BTreeSet::new();
        };
        mapped.insert(source.handle);
        saved.model_windows.push(window);
    }
    if saved.model_windows.len() > 1 {
        loss(
            diagnostics,
            "active tiled VPORT identity is not exposed by cadcodec",
        );
    }
    if let Some(active) = saved.model_windows.first_mut() {
        if active.use_stored_ucs && active.stored_ucs != current_model_ucs {
            loss(
                diagnostics,
                "active VPORT UCS differs from header current UCS",
            );
            active.use_stored_ucs = false;
        }
        saved.view_state = Some(DrawingViewState {
            current_model_ucs,
            active_model_window_id: active.id,
        });
    } else {
        loss(diagnostics, "active VPORT");
        return BTreeSet::new();
    }
    for layout in document.objects.values().filter_map(|object| {
        if let ObjectType::Layout(layout) = object {
            Some(layout)
        } else {
            None
        }
    }) {
        let Some(&scope) = paper_scopes.get(&layout.block_record) else {
            continue;
        };
        let Some(handle) = crate::source::overall_viewport_handle(document, layout) else {
            if layout.viewport!=Handle::NULL || !layout.viewports.is_empty() || document.entities().any(|entity|matches!(entity,EntityType::Viewport(v) if v.common.owner_handle==layout.block_record)) {loss(diagnostics,"paper overall viewport identity unavailable");}
            continue;
        };
        let Some(EntityType::Viewport(viewport)) = document.get_entity(handle) else {
            continue;
        };
        let default = cadcodec::entities::Viewport::new();
        if viewport.off_screen {
            loss(diagnostics, "paper canvas off-screen state");
        }
        if viewport.center != default.center
            || viewport.width != default.width
            || viewport.height != default.height
        {
            loss(diagnostics, "paper canvas screen frame");
        }
        if !viewport.status.snap_on
            && (viewport.snap_spacing.x <= 0.0 || viewport.snap_spacing.y <= 0.0)
        {
            loss(
                diagnostics,
                "disabled paper snap spacing normalized to valid defaults",
            );
        }
        if let Some(canvas) = paper_canvas(viewport, scope, handles) {
            saved.paper_canvases.push(canvas);
        } else {
            loss(
                diagnostics,
                "paper layout overall VIEWPORT view/grid/snap/UCS",
            );
        }
    }
    drawing.set_saved_state(saved);
    mapped
}
