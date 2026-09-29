use super::decode::{
    num, point2_record, point_record, render_mode, rows, u32v, vector3_record, view_record,
};
use super::encode::{point2_json, point3_json, render_mode_code, vector3_json, view_json};
use crate::diagnostic::{PackageDiagnostic, PackageDiagnosticCategory, PackageDiagnosticSeverity};
use crate::ifcdr::geometry::CoordinateFrameComponents;
use crate::ifcdr::logical::*;
use crate::ifcdr::CoordinateFrame3;
use serde_json::{json, Value};

fn malformed(uri: &str, path: &str, message: &str) -> Box<PackageDiagnostic> {
    Box::new(PackageDiagnostic {
        category: PackageDiagnosticCategory::ContractViolation,
        code: "IFCCAD_IFCDR_STRUCTURE_INVALID".into(),
        severity: PackageDiagnosticSeverity::Error,
        resource_id: None,
        resource_uri: Some(uri.into()),
        location: Some(path.into()),
        context: Default::default(),
        message: message.into(),
    })
}

fn frame(v: &Value) -> CoordinateFrame3 {
    CoordinateFrame3::from_validated_components(CoordinateFrameComponents {
        origin: point_record(&v["origin"]),
        x: vector3_record(&v["X"]),
        y: vector3_record(&v["Y"]),
    })
}

fn choice(uri: &str, v: &Value, path: &str) -> Result<UcsSelection, Box<PackageDiagnostic>> {
    match (u32v(&v["kind"]), v.get("ucsId"), v.get("frame")) {
        (0, None, None) => Ok(UcsSelection::World),
        (1, Some(id), None) => Ok(UcsSelection::Named { ucs_id: u32v(id) }),
        (2, None, Some(value)) => Ok(UcsSelection::Unnamed {
            frame: frame(value),
        }),
        _ => Err(malformed(
            uri,
            path,
            "UCS choice must contain exactly its selected value",
        )),
    }
}

fn grid(v: &Value) -> WorkspaceGrid {
    WorkspaceGrid {
        enabled: v["enabled"].as_bool().unwrap(),
        spacing: point2_record(&v["spacing"]),
        style: match u32v(&v["style"]) {
            0 => WorkspaceGridStyle::Lines,
            1 => WorkspaceGridStyle::Dots,
            _ => unreachable!("physical grid style"),
        },
        major_line_frequency: u32v(&v["majorLineFrequency"]),
        beyond_limits: v["beyondLimits"].as_bool().unwrap(),
        adaptive: v["adaptive"].as_bool().unwrap(),
        subdivision: v["subdivision"].as_bool().unwrap(),
        follows_workplane: v["followsWorkplane"].as_bool().unwrap(),
    }
}

fn snap(v: &Value) -> WorkspaceSnap {
    WorkspaceSnap {
        enabled: v["enabled"].as_bool().unwrap(),
        base: point2_record(&v["base"]),
        spacing: point2_record(&v["spacing"]),
        angle: num(&v["angle"]),
        style: match u32v(&v["style"]) {
            0 => WorkspaceSnapStyle::Rectangular,
            1 => WorkspaceSnapStyle::Isometric,
            _ => unreachable!("physical snap style"),
        },
        isometric_plane: match u32v(&v["isometricPlane"]) {
            0 => IsometricPlane::Left,
            1 => IsometricPlane::Top,
            2 => IsometricPlane::Right,
            _ => unreachable!("physical isometric plane"),
        },
    }
}

pub(super) fn decode_workspace(
    uri: &str,
    root: &Value,
) -> Result<IfcdrWorkspace, Box<PackageDiagnostic>> {
    let drawing_view_state = root
        .get("drawingViewState")
        .map(|v| {
            Ok::<_, Box<PackageDiagnostic>>(DrawingViewState {
                current_model_ucs: choice(
                    uri,
                    &v["currentModelUcs"],
                    "/drawingViewState/currentModelUcs",
                )?,
                active_model_window_id: u32v(&v["activeModelWindowId"]),
            })
        })
        .transpose()?;
    let ucs_definitions = rows(root, "ucsDefinitionTable")
        .iter()
        .map(|v| UcsDefinition {
            ucs_id: u32v(&v["ucsId"]),
            name: v["name"].as_str().unwrap().into(),
            frame: frame(&v["frame"]),
            elevation: num(&v["elevation"]),
        })
        .collect();
    let model_windows = rows(root, "modelWindowTable")
        .iter()
        .enumerate()
        .map(|(row, v)| {
            let rect = &v["rectangle"];
            Ok::<_, Box<PackageDiagnostic>>(ModelWindow {
                model_window_id: u32v(&v["modelWindowId"]),
                rectangle: NormalizedRect2 {
                    min_x: num(&rect["minX"]),
                    min_y: num(&rect["minY"]),
                    max_x: num(&rect["maxX"]),
                    max_y: num(&rect["maxY"]),
                },
                view: view_record(&v["view"]),
                aspect_ratio: num(&v["aspectRatio"]),
                render_mode: render_mode(u32v(&v["renderMode"])),
                grid: grid(&v["grid"]),
                snap: snap(&v["snap"]),
                stored_ucs: choice(
                    uri,
                    &v["storedUcs"],
                    &format!("/modelWindowTable/{row}/storedUcs"),
                )?,
                use_stored_ucs: v["useStoredUcs"].as_bool().unwrap(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let paper_canvases = rows(root, "paperCanvasTable")
        .iter()
        .enumerate()
        .map(|(row, v)| {
            let active = &v["activeContext"];
            let active_context = match (u32v(&active["kind"]), active.get("viewportEntityId")) {
                (0, None) => PaperActiveContext::Canvas,
                (1, Some(id)) => PaperActiveContext::Viewport {
                    viewport_entity_id: id.as_u64().unwrap(),
                },
                _ => {
                    return Err(malformed(
                        uri,
                        &format!("/paperCanvasTable/{row}/activeContext"),
                        "paper active context must contain exactly its selected viewport",
                    ))
                }
            };
            Ok(PaperCanvas {
                scope_id: u32v(&v["scopeId"]),
                view: view_record(&v["view"]),
                grid: grid(&v["grid"]),
                snap: snap(&v["snap"]),
                stored_ucs: choice(
                    uri,
                    &v["storedUcs"],
                    &format!("/paperCanvasTable/{row}/storedUcs"),
                )?,
                current_ucs: choice(
                    uri,
                    &v["currentUcs"],
                    &format!("/paperCanvasTable/{row}/currentUcs"),
                )?,
                active_context,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let viewport_workspaces = rows(root, "viewportWorkspaceTable")
        .iter()
        .enumerate()
        .map(|(row, v)| {
            Ok::<_, Box<PackageDiagnostic>>(ViewportWorkspace {
                viewport_entity_id: v["viewportEntityId"].as_u64().unwrap(),
                grid: grid(&v["grid"]),
                snap: snap(&v["snap"]),
                stored_ucs: choice(
                    uri,
                    &v["storedUcs"],
                    &format!("/viewportWorkspaceTable/{row}/storedUcs"),
                )?,
                use_stored_ucs: v["useStoredUcs"].as_bool().unwrap(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(IfcdrWorkspace {
        drawing_view_state,
        ucs_definitions,
        model_windows,
        paper_canvases,
        viewport_workspaces,
    })
}

fn frame_json(value: CoordinateFrame3) -> Value {
    json!({"origin":point3_json(value.origin()),"X":vector3_json(value.x_axis()),
        "Y":vector3_json(value.y_axis())})
}
fn choice_json(value: UcsSelection) -> Value {
    match value {
        UcsSelection::World => json!({"kind":0}),
        UcsSelection::Named { ucs_id } => json!({"kind":1,"ucsId":ucs_id}),
        UcsSelection::Unnamed { frame } => json!({"kind":2,"frame":frame_json(frame)}),
    }
}
fn grid_json(value: WorkspaceGrid) -> Value {
    json!({"enabled":value.enabled,"spacing":point2_json(value.spacing),
        "style":match value.style { WorkspaceGridStyle::Lines=>0,WorkspaceGridStyle::Dots=>1 },
        "majorLineFrequency":value.major_line_frequency,"beyondLimits":value.beyond_limits,
        "adaptive":value.adaptive,"subdivision":value.subdivision,
        "followsWorkplane":value.follows_workplane})
}
fn snap_json(value: WorkspaceSnap) -> Value {
    json!({"enabled":value.enabled,"base":point2_json(value.base),
    "spacing":point2_json(value.spacing),"angle":value.angle,
    "style":match value.style { WorkspaceSnapStyle::Rectangular=>0,WorkspaceSnapStyle::Isometric=>1 },
    "isometricPlane":match value.isometric_plane {
        IsometricPlane::Left=>0,IsometricPlane::Top=>1,IsometricPlane::Right=>2
    }})
}
pub(super) fn encode_workspace(root: &mut Value, state: &IfcdrWorkspace) {
    if let Some(view) = state.drawing_view_state {
        root["drawingViewState"] = json!({"currentModelUcs":choice_json(view.current_model_ucs),
            "activeModelWindowId":view.active_model_window_id});
    }
    root["ucsDefinitionTable"] = json!(state
        .ucs_definitions
        .iter()
        .map(|u| json!({
            "ucsId":u.ucs_id,"name":u.name,"frame":frame_json(u.frame),"elevation":u.elevation
        }))
        .collect::<Vec<_>>());
    root["modelWindowTable"] = json!(state
        .model_windows
        .iter()
        .map(|w| json!({
            "modelWindowId":w.model_window_id,
            "rectangle":{"minX":w.rectangle.min_x,"minY":w.rectangle.min_y,
                "maxX":w.rectangle.max_x,"maxY":w.rectangle.max_y},
            "view":view_json(w.view),"aspectRatio":w.aspect_ratio,
            "renderMode":render_mode_code(w.render_mode),"grid":grid_json(w.grid),
            "snap":snap_json(w.snap),"storedUcs":choice_json(w.stored_ucs),
            "useStoredUcs":w.use_stored_ucs
        }))
        .collect::<Vec<_>>());
    root["paperCanvasTable"] = json!(state
        .paper_canvases
        .iter()
        .map(|c| json!({
            "scopeId":c.scope_id,"view":view_json(c.view),"grid":grid_json(c.grid),
            "snap":snap_json(c.snap),"storedUcs":choice_json(c.stored_ucs),
            "currentUcs":choice_json(c.current_ucs),"activeContext":match c.active_context {
                PaperActiveContext::Canvas=>json!({"kind":0}),
                PaperActiveContext::Viewport{viewport_entity_id}=>json!({"kind":1,
                    "viewportEntityId":viewport_entity_id})
            }
        }))
        .collect::<Vec<_>>());
    root["viewportWorkspaceTable"] = json!(state
        .viewport_workspaces
        .iter()
        .map(|v| json!({
            "viewportEntityId":v.viewport_entity_id,"grid":grid_json(v.grid),
            "snap":snap_json(v.snap),"storedUcs":choice_json(v.stored_ucs),
            "useStoredUcs":v.use_stored_ucs
        }))
        .collect::<Vec<_>>());
}
