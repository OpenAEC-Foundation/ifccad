use crate::ocdraw::logical::{
    DrawingClip, DrawingClipMode, DrawingGrid, DrawingGridStyle, DrawingIsometricPlane,
    DrawingModelWindow, DrawingPaperCanvas, DrawingPaperContext, DrawingProjection,
    DrawingRenderMode, DrawingSnap, DrawingSnapStyle, DrawingUcsSelection, DrawingView,
    DrawingViewState, DrawingViewportWorkspace,
};
use crate::ocdraw::{Point2, Point3, Vector3};
use serde_json::Value;

pub(crate) struct DecodedViewState {
    pub view_state: Option<DrawingViewState>,
    pub model_windows: Vec<DrawingModelWindow>,
    pub paper_canvases: Vec<DrawingPaperCanvas>,
    pub viewport_workspaces: Vec<DrawingViewportWorkspace>,
}

fn u32_value(value: &Value) -> Option<u32> {
    u32::try_from(value.as_u64()?).ok()
}

pub(super) fn point2(value: &Value) -> Option<Point2> {
    Some(Point2::new(
        value.get("x")?.as_f64()?,
        value.get("y")?.as_f64()?,
    ))
}

fn point3(value: &Value) -> Option<Point3> {
    Some(Point3::new(
        value.get("x")?.as_f64()?,
        value.get("y")?.as_f64()?,
        value.get("z")?.as_f64()?,
    ))
}

fn vector3(value: &Value) -> Option<Vector3> {
    Some(Vector3::new(
        value.get("x")?.as_f64()?,
        value.get("y")?.as_f64()?,
        value.get("z")?.as_f64()?,
    ))
}

fn choice(value: &Value) -> Option<DrawingUcsSelection> {
    match value.get("kind")?.as_str()? {
        "World" => Some(DrawingUcsSelection::World),
        "Named" => Some(DrawingUcsSelection::Named(u32_value(value.get("ucsId")?)?)),
        "Unnamed" => Some(DrawingUcsSelection::Unnamed(super::frame_from_json(
            value.get("frame")?,
        )?)),
        _ => None,
    }
}

fn clip(value: &Value) -> Option<DrawingClip> {
    let mode = match value.get("mode")?.as_str()? {
        "Disabled" => DrawingClipMode::Disabled,
        "AtCamera" => DrawingClipMode::AtCamera,
        "AtDistance" => DrawingClipMode::AtDistance,
        _ => return None,
    };
    Some(DrawingClip {
        mode,
        distance: value.get("distance").and_then(Value::as_f64),
    })
}

pub(super) fn view(value: &Value) -> Option<DrawingView> {
    Some(DrawingView {
        center: point2(value.get("center")?)?,
        target: point3(value.get("target")?)?,
        direction: vector3(value.get("direction")?)?,
        height: value.get("height")?.as_f64()?,
        twist: value.get("twist")?.as_f64()?,
        projection: match value.get("projection")?.as_str()? {
            "Orthographic" => DrawingProjection::Orthographic,
            "Perspective" => DrawingProjection::Perspective,
            _ => return None,
        },
        lens_length: value.get("lensLength").and_then(Value::as_f64),
        front_clip: clip(value.get("frontClip")?)?,
        back_clip: clip(value.get("backClip")?)?,
    })
}

fn grid(value: &Value) -> Option<DrawingGrid> {
    Some(DrawingGrid {
        enabled: value.get("enabled")?.as_bool()?,
        spacing: point2(value.get("spacing")?)?,
        style: match value.get("style")?.as_str()? {
            "Lines" => DrawingGridStyle::Lines,
            "Dots" => DrawingGridStyle::Dots,
            _ => return None,
        },
        major_line_frequency: u32_value(value.get("majorLineFrequency")?)?,
        beyond_limits: value.get("beyondLimits")?.as_bool()?,
        adaptive: value.get("adaptive")?.as_bool()?,
        subdivision: value.get("subdivision")?.as_bool()?,
        follows_workplane: value.get("followsWorkplane")?.as_bool()?,
    })
}

fn snap(value: &Value) -> Option<DrawingSnap> {
    Some(DrawingSnap {
        enabled: value.get("enabled")?.as_bool()?,
        base: point2(value.get("base")?)?,
        spacing: point2(value.get("spacing")?)?,
        angle: value.get("angle")?.as_f64()?,
        style: match value.get("style")?.as_str()? {
            "Rectangular" => DrawingSnapStyle::Rectangular,
            "Isometric" => DrawingSnapStyle::Isometric,
            _ => return None,
        },
        isometric_plane: match value.get("isometricPlane")?.as_str()? {
            "Left" => DrawingIsometricPlane::Left,
            "Top" => DrawingIsometricPlane::Top,
            "Right" => DrawingIsometricPlane::Right,
            _ => return None,
        },
    })
}

pub(super) fn render_mode(value: &Value) -> Option<DrawingRenderMode> {
    Some(match value.as_str()? {
        "TwoDimensional" => DrawingRenderMode::TwoDimensional,
        "Wireframe" => DrawingRenderMode::Wireframe,
        "HiddenLine" => DrawingRenderMode::HiddenLine,
        "FlatShadedWithoutEdges" => DrawingRenderMode::FlatShadedWithoutEdges,
        "FlatShadedWithEdges" => DrawingRenderMode::FlatShadedWithEdges,
        "SmoothShadedWithoutEdges" => DrawingRenderMode::SmoothShadedWithoutEdges,
        "SmoothShadedWithEdges" => DrawingRenderMode::SmoothShadedWithEdges,
        _ => return None,
    })
}

fn rows<'a>(root: &'a Value, key: &str) -> Option<&'a [Value]> {
    match root.get(key) {
        Some(value) => Some(value.as_array()?.as_slice()),
        None => Some(&[]),
    }
}

fn optional<T>(
    value: &Value,
    key: &str,
    decode: impl FnOnce(&Value) -> Option<T>,
) -> Option<Option<T>> {
    match value.get(key) {
        Some(v) => Some(Some(decode(v)?)),
        None => Some(None),
    }
}
fn canvas_frame(value: &Value) -> Option<crate::workspace_kernel::WorkspaceCanvasFrame> {
    Some(crate::workspace_kernel::WorkspaceCanvasFrame {
        center: point3(value.get("center")?)?,
        width: value.get("width")?.as_f64()?,
        height: value.get("height")?.as_f64()?,
    })
}

pub(crate) fn decode_view_state(root: &Value) -> Option<DecodedViewState> {
    let view_state = match root.get("drawingViewState") {
        Some(value) => Some(DrawingViewState {
            current_model_ucs: optional(value, "currentModelUcs", choice)?,
            active_model_window_id: optional(value, "activeModelWindowId", u32_value)?,
        }),
        None => None,
    };
    let model_windows = rows(root, "modelWindows")?
        .iter()
        .map(|value| {
            let rectangle = value.get("rectangle")?;
            Some(DrawingModelWindow {
                id: u32_value(value.get("modelWindowId")?)?,
                rectangle: [
                    rectangle.get("minX")?.as_f64()?,
                    rectangle.get("minY")?.as_f64()?,
                    rectangle.get("maxX")?.as_f64()?,
                    rectangle.get("maxY")?.as_f64()?,
                ],
                view: view(value.get("view")?)?,
                aspect_ratio: value.get("aspectRatio")?.as_f64()?,
                render_mode: render_mode(value.get("renderMode")?)?,
                grid: grid(value.get("grid")?)?,
                snap: snap(value.get("snap")?)?,
                stored_ucs: choice(value.get("storedUcs")?)?,
                use_stored_ucs: value.get("useStoredUcs")?.as_bool()?,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    let paper_canvases = rows(root, "paperCanvases")?
        .iter()
        .map(|value| {
            let active_context = optional(value, "activeContext", |active| {
                match active.get("kind")?.as_str()? {
                    "Canvas" => Some(DrawingPaperContext::Canvas),
                    "Viewport" => Some(DrawingPaperContext::Viewport(
                        active.get("viewportEntityId")?.as_u64()?,
                    )),
                    _ => None,
                }
            })?;
            Some(DrawingPaperCanvas {
                frame: optional(value, "frame", canvas_frame)?,
                use_stored_ucs: match value.get("useStoredUcs") {
                    Some(v) => v.as_bool()?,
                    None => true,
                },
                scope_id: u32_value(value.get("scopeId")?)?,
                view: view(value.get("view")?)?,
                grid: grid(value.get("grid")?)?,
                snap: snap(value.get("snap")?)?,
                stored_ucs: choice(value.get("storedUcs")?)?,
                current_ucs: optional(value, "currentUcs", choice)?,
                active_context,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    let viewport_workspaces = rows(root, "viewportWorkspaces")?
        .iter()
        .map(|value| {
            Some(DrawingViewportWorkspace {
                viewport_entity_id: value.get("viewportEntityId")?.as_u64()?,
                grid: grid(value.get("grid")?)?,
                snap: snap(value.get("snap")?)?,
                stored_ucs: choice(value.get("storedUcs")?)?,
                use_stored_ucs: value.get("useStoredUcs")?.as_bool()?,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(DecodedViewState {
        view_state,
        model_windows,
        paper_canvases,
        viewport_workspaces,
    })
}
