use crate::geometry_kernel::{CoordinateFrame3, Point2, Point3};
use crate::ifccad::*;
use crate::workspace_kernel::*;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

fn bad(message: impl Into<String>) -> IfccadReport {
    IfccadReport::one(message)
}
fn parse<T: serde::de::DeserializeOwned>(v: &Value) -> Result<T, IfccadReport> {
    serde_json::from_value(v.clone()).map_err(|e| bad(format!("invalid workspace value: {e}")))
}
fn attr<'a>(node: &'a Value, key: &str) -> Option<&'a Value> {
    node.get("attributes")?.get(key)
}
fn id(path: &str, prefix: &str) -> Result<u64, IfccadReport> {
    let suffix = path
        .strip_prefix(prefix)
        .ok_or_else(|| bad(format!("foreign workspace reference {path}")))?;
    let value = suffix
        .parse::<u64>()
        .map_err(|_| bad(format!("invalid workspace reference {path}")))?;
    if suffix != value.to_string() {
        return Err(bad(format!("noncanonical workspace reference {path}")));
    }
    Ok(value)
}
fn optional<T>(
    value: &Value,
    key: &str,
    f: impl FnOnce(&Value) -> Result<T, IfccadReport>,
) -> Result<Option<T>, IfccadReport> {
    value.get(key).map(f).transpose()
}
#[derive(Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum UcsWire {
    World,
    Named { ucs: String },
    Unnamed { frame: IfccadPlacement },
}
fn ucs(v: &Value, prefix: &str) -> Result<IfccadUcsSelection, IfccadReport> {
    match v.get("kind").and_then(Value::as_str) {
        Some("World") => fields(v, &["kind"])?,
        Some("Named") => fields(v, &["kind", "ucs"])?,
        Some("Unnamed") => fields(v, &["kind", "frame"])?,
        _ => return Err(bad("invalid UCS kind")),
    }
    Ok(match parse::<UcsWire>(v)? {
        UcsWire::World => IfccadUcsSelection::World,
        UcsWire::Named { ucs } => {
            IfccadUcsSelection::Named(IfccadUcsId(id(&ucs, &format!("{prefix}/ucs/"))?))
        }
        UcsWire::Unnamed { frame } => IfccadUcsSelection::Unnamed(frame.coordinate_frame()?),
    })
}
fn frame(v: CoordinateFrame3) -> Value {
    json!({"origin":v.origin().components(),"xAxis":v.x_axis().components(),"yAxis":v.y_axis().components()})
}
fn encode_ucs(v: IfccadUcsSelection, prefix: &str) -> Value {
    match v {
        IfccadUcsSelection::World => json!({"kind":"World"}),
        IfccadUcsSelection::Named(id) => {
            json!({"kind":"Named","ucs":format!("{prefix}/ucs/{}",id.0)})
        }
        IfccadUcsSelection::Unnamed(v) => json!({"kind":"Unnamed","frame":frame(v)}),
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GridWire {
    enabled: bool,
    spacing: [f64; 2],
    style: String,
    major_line_frequency: u32,
    beyond_limits: bool,
    adaptive: bool,
    subdivision: bool,
    follows_workplane: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SnapWire {
    enabled: bool,
    base: [f64; 2],
    spacing: [f64; 2],
    angle: f64,
    style: String,
    isometric_plane: String,
}
fn grid(v: &Value) -> Result<WorkspaceGrid, IfccadReport> {
    let v: GridWire = parse(v)?;
    Ok(WorkspaceGrid {
        enabled: v.enabled,
        spacing: Point2::new(v.spacing[0], v.spacing[1]),
        style: match v.style.as_str() {
            "Lines" => WorkspaceGridStyle::Lines,
            "Dots" => WorkspaceGridStyle::Dots,
            _ => return Err(bad("invalid grid style")),
        },
        major_line_frequency: v.major_line_frequency,
        beyond_limits: v.beyond_limits,
        adaptive: v.adaptive,
        subdivision: v.subdivision,
        follows_workplane: v.follows_workplane,
    })
}
fn snap(v: &Value) -> Result<WorkspaceSnap, IfccadReport> {
    let v: SnapWire = parse(v)?;
    Ok(WorkspaceSnap {
        enabled: v.enabled,
        base: Point2::new(v.base[0], v.base[1]),
        spacing: Point2::new(v.spacing[0], v.spacing[1]),
        angle: v.angle,
        style: match v.style.as_str() {
            "Rectangular" => WorkspaceSnapStyle::Rectangular,
            "Isometric" => WorkspaceSnapStyle::Isometric,
            _ => return Err(bad("invalid snap style")),
        },
        isometric_plane: match v.isometric_plane.as_str() {
            "Left" => WorkspaceIsometricPlane::Left,
            "Top" => WorkspaceIsometricPlane::Top,
            "Right" => WorkspaceIsometricPlane::Right,
            _ => return Err(bad("invalid isometric plane")),
        },
    })
}
fn encode_grid(v: WorkspaceGrid) -> Value {
    json!({"enabled":v.enabled,"spacing":[v.spacing.x(),v.spacing.y()],"style":format!("{:?}",v.style),"majorLineFrequency":v.major_line_frequency,"beyondLimits":v.beyond_limits,"adaptive":v.adaptive,"subdivision":v.subdivision,"followsWorkplane":v.follows_workplane})
}
fn encode_snap(v: WorkspaceSnap) -> Value {
    json!({"enabled":v.enabled,"base":[v.base.x(),v.base.y()],"spacing":[v.spacing.x(),v.spacing.y()],"angle":v.angle,"style":format!("{:?}",v.style),"isometricPlane":format!("{:?}",v.isometric_plane)})
}
fn view(v: &Value) -> Result<WorkspaceView, IfccadReport> {
    let v: IfccadViewportView = parse(v)?;
    Ok(v.as_workspace_view())
}
fn encode_view(v: WorkspaceView) -> Value {
    let clip = |v: WorkspaceClip| {
        let mut out = json!({"mode":format!("{:?}",v.mode)});
        if let Some(d) = v.distance {
            out["distance"] = json!(d);
        }
        out
    };
    let mut out = json!({"center":[v.center.x(),v.center.y()],"target":v.target.components(),"direction":v.direction.components(),"height":v.height,"twist":v.twist,"projection":format!("{:?}",v.projection),"frontClip":clip(v.front_clip),"backClip":clip(v.back_clip)});
    if let Some(lens) = v.lens_length {
        out["lensLengthMm"] = json!(lens);
    }
    out
}
fn fields(v: &Value, allowed: &[&str]) -> Result<(), IfccadReport> {
    let object = v
        .as_object()
        .ok_or_else(|| bad("workspace must be an object"))?;
    if let Some(key) = object.keys().find(|k| !allowed.contains(&k.as_str())) {
        return Err(bad(format!("unknown workspace field {key}")));
    }
    Ok(())
}
fn required<'a>(v: &'a Value, key: &str) -> Result<&'a Value, IfccadReport> {
    v.get(key)
        .ok_or_else(|| bad(format!("workspace missing {key}")))
}
fn activation(v: &Value) -> Result<bool, IfccadReport> {
    match v.get("useStoredUcs") {
        Some(v) => v.as_bool().ok_or_else(|| bad("invalid UCS activation")),
        None => Ok(true),
    }
}
fn aids(v: &Value, prefix: &str) -> Result<IfccadViewportWorkspace, IfccadReport> {
    Ok(IfccadViewportWorkspace {
        grid: grid(required(v, "grid")?)?,
        snap: snap(required(v, "snap")?)?,
        stored_ucs: ucs(required(v, "storedUcs")?, prefix)?,
        use_stored_ucs: activation(v)?,
    })
}
pub(super) fn decode_viewport(
    v: &Value,
    prefix: &str,
) -> Result<IfccadViewportWorkspace, IfccadReport> {
    fields(v, &["grid", "snap", "storedUcs", "useStoredUcs"])?;
    aids(v, prefix)
}
pub(super) fn encode_viewport(v: &IfccadViewportWorkspace, prefix: &str) -> Value {
    json!({"grid":encode_grid(v.grid),"snap":encode_snap(v.snap),"storedUcs":encode_ucs(v.stored_ucs,prefix),"useStoredUcs":v.use_stored_ucs})
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrameWire {
    center: [f64; 3],
    width: f64,
    height: f64,
}
#[derive(Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum ContextWire {
    Canvas,
    Viewport { viewport: String },
}
pub(super) fn decode_canvas(v: &Value, prefix: &str) -> Result<IfccadPaperCanvas, IfccadReport> {
    fields(
        v,
        &[
            "view",
            "frame",
            "grid",
            "snap",
            "storedUcs",
            "useStoredUcs",
            "currentUcs",
            "activeContext",
        ],
    )?;
    let a = aids(v, prefix)?;
    Ok(IfccadPaperCanvas {
        view: view(required(v, "view")?)?,
        grid: a.grid,
        snap: a.snap,
        stored_ucs: a.stored_ucs,
        use_stored_ucs: a.use_stored_ucs,
        frame: optional(v, "frame", |v| {
            let v: FrameWire = parse(v)?;
            Ok(WorkspaceCanvasFrame {
                center: Point3::new(v.center[0], v.center[1], v.center[2]),
                width: v.width,
                height: v.height,
            })
        })?,
        current_ucs: optional(v, "currentUcs", |v| ucs(v, prefix))?,
        active_context: optional(v, "activeContext", |v| {
            match v.get("kind").and_then(Value::as_str) {
                Some("Canvas") => fields(v, &["kind"])?,
                Some("Viewport") => fields(v, &["kind", "viewport"])?,
                _ => return Err(bad("invalid Paper context kind")),
            }
            Ok(match parse::<ContextWire>(v)? {
                ContextWire::Canvas => IfccadPaperContext::Canvas,
                ContextWire::Viewport { viewport } => {
                    IfccadPaperContext::Viewport(id(&viewport, &format!("{prefix}/e"))?)
                }
            })
        })?,
    })
}
pub(super) fn encode_canvas(v: &IfccadPaperCanvas, prefix: &str) -> Value {
    let mut out = json!({"view":encode_view(v.view),"grid":encode_grid(v.grid),"snap":encode_snap(v.snap),"storedUcs":encode_ucs(v.stored_ucs,prefix),"useStoredUcs":v.use_stored_ucs});
    if let Some(frame) = v.frame {
        out["frame"] =
            json!({"center":frame.center.components(),"width":frame.width,"height":frame.height});
    }
    if let Some(current) = v.current_ucs {
        out["currentUcs"] = encode_ucs(current, prefix);
    }
    if let Some(context) = v.active_context {
        out["activeContext"] = match context {
            IfccadPaperContext::Canvas => json!({"kind":"Canvas"}),
            IfccadPaperContext::Viewport(id) => {
                json!({"kind":"Viewport","viewport":format!("{prefix}/e{id}")})
            }
        };
    }
    out
}

pub(super) fn decode_document(
    raw: &Value,
    document: &mut IfccadDocument,
) -> Result<(), IfccadReport> {
    let prefix = format!("/cad/d{}", document.drawing_id);
    let nodes: BTreeMap<_, _> = raw["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| (n["path"].as_str().unwrap(), n))
        .collect();
    let drawing = nodes[prefix.as_str()];
    let has_workspace = nodes.values().any(|node| {
        [
            "ifccad::ucsDefinition",
            "ifccad::modelWindow",
            "ifccad::paperCanvas",
            "ifccad::viewportWorkspace",
            "ifccad::modelViewState",
            "ifccad::drawingWorkspace",
        ]
        .into_iter()
        .any(|key| attr(node, key).is_some())
    });
    if has_workspace
        && ["nextUcsId", "nextModelWindowId"]
            .into_iter()
            .any(|key| attr(drawing, "ifccad::drawing").unwrap().get(key).is_none())
    {
        return Err(bad("workspace requires explicit UCS/window watermarks"));
    }

    let declared: BTreeSet<_> = drawing["children"]
        .as_object()
        .unwrap()
        .values()
        .filter_map(Value::as_str)
        .collect();
    let mut windows = BTreeMap::new();
    for (&path, node) in &nodes {
        if let Some(v) = attr(node, "ifccad::ucsDefinition") {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Definition {
                name: String,
                frame: IfccadPlacement,
                elevation: f64,
            }
            let v: Definition = parse(v)?;
            if !declared.contains(path) {
                return Err(bad(format!("unlisted UCS {path}")));
            }
            document.ucs_definitions.push(IfccadUcsDefinition {
                id: IfccadUcsId(id(path, &format!("{prefix}/ucs/"))?),
                name: v.name,
                frame: v.frame.coordinate_frame()?,
                elevation: v.elevation,
            });
        }
        if let Some(v) = attr(node, "ifccad::modelWindow") {
            fields(
                v,
                &[
                    "rectangle",
                    "view",
                    "aspectRatio",
                    "renderMode",
                    "grid",
                    "snap",
                    "storedUcs",
                    "useStoredUcs",
                ],
            )?;
            if !declared.contains(path) {
                return Err(bad(format!("unlisted model window {path}")));
            }
            let a = aids(v, &prefix)?;
            let render = match required(v, "renderMode")?.as_str() {
                Some("TwoDimensional") => WorkspaceRenderMode::TwoDimensional,
                Some("Wireframe") => WorkspaceRenderMode::Wireframe,
                Some("HiddenLine") => WorkspaceRenderMode::HiddenLine,
                Some("FlatShadedWithoutEdges") => WorkspaceRenderMode::FlatShadedWithoutEdges,
                Some("FlatShadedWithEdges") => WorkspaceRenderMode::FlatShadedWithEdges,
                Some("SmoothShadedWithoutEdges") => WorkspaceRenderMode::SmoothShadedWithoutEdges,
                Some("SmoothShadedWithEdges") => WorkspaceRenderMode::SmoothShadedWithEdges,
                _ => return Err(bad("invalid workspace render mode")),
            };
            windows.insert(
                path.to_owned(),
                IfccadModelWindow {
                    id: IfccadModelWindowId(id(path, &format!("{prefix}/modelWindow/"))?),
                    rectangle: parse(required(v, "rectangle")?)?,
                    view: view(required(v, "view")?)?,
                    aspect_ratio: parse(required(v, "aspectRatio")?)?,
                    render_mode: render,
                    grid: a.grid,
                    snap: a.snap,
                    stored_ucs: a.stored_ucs,
                    use_stored_ucs: a.use_stored_ucs,
                },
            );
        }
        if let Some(v) = attr(node, "ifccad::paperCanvas") {
            let layout_id = id(path, &format!("{prefix}/layout/"))?;
            let paper = document
                .paper_layouts
                .iter_mut()
                .find(|p| p.id == layout_id)
                .ok_or_else(|| bad("canvas must belong to Paper layout"))?;
            paper.canvas = Some(decode_canvas(v, &prefix)?);
        }
        if let Some(v) = attr(node, "ifccad::viewportWorkspace") {
            let entity_id = id(path, &format!("{prefix}/e"))?;
            let viewport = document
                .paper_layouts
                .iter_mut()
                .flat_map(|p| &mut p.entities)
                .filter_map(IfccadEntity::as_native_mut)
                .find_map(|e| match &mut e.kind {
                    IfccadEntityKind::Viewport(viewport) if e.id == entity_id => Some(viewport),
                    _ => None,
                })
                .ok_or_else(|| bad("workspace must belong to authored Paper viewport"))?;
            viewport.workspace = Some(Box::new(decode_viewport(v, &prefix)?));
        }
        for key in ["ifccad::drawingWorkspace", "ifccad::modelViewState"] {
            if attr(node, key).is_some() && path != prefix {
                return Err(bad(format!("{key} belongs to drawing")));
            }
        }
    }
    let ordered: Vec<String> = match attr(drawing, "ifccad::drawing")
        .unwrap()
        .get("modelWindows")
    {
        Some(v) => parse(v)?,
        None => vec![],
    };
    for path in ordered {
        document.model_windows.push(
            windows
                .remove(&path)
                .ok_or_else(|| bad(format!("duplicate, foreign or missing window {path}")))?,
        );
    }
    if !windows.is_empty() {
        return Err(bad(
            "owned model windows must appear once in ordered reference list",
        ));
    }
    if let Some(v) = attr(drawing, "ifccad::drawingWorkspace") {
        fields(v, &["currentLayer", "activeLayout"])?;
        document.workspace_state = Some(IfccadDrawingWorkspaceState {
            current_layer_id: optional(v, "currentLayer", |v| {
                id(
                    v.as_str().ok_or_else(|| bad("invalid layer reference"))?,
                    &format!("{prefix}/layer/"),
                )
            })?,
            active_layout_id: optional(v, "activeLayout", |v| {
                id(
                    v.as_str().ok_or_else(|| bad("invalid layout reference"))?,
                    &format!("{prefix}/layout/"),
                )
            })?,
        });
    }
    if let Some(v) = attr(drawing, "ifccad::modelViewState") {
        fields(v, &["currentModelUcs", "activeModelWindow"])?;
        document.model_view_state = Some(IfccadModelViewState {
            current_model_ucs: optional(v, "currentModelUcs", |v| ucs(v, &prefix))?,
            active_model_window_id: optional(v, "activeModelWindow", |v| {
                Ok(IfccadModelWindowId(id(
                    v.as_str().ok_or_else(|| bad("invalid window reference"))?,
                    &format!("{prefix}/modelWindow/"),
                )?))
            })?,
        });
    }
    Ok(())
}

pub(super) fn encode_definitions(document: &IfccadDocument, prefix: &str) -> Vec<Value> {
    let mut nodes = Vec::new();
    for v in &document.ucs_definitions {
        nodes.push(json!({"path":format!("{prefix}/ucs/{}",v.id.0),"attributes":{"ifccad::ucsDefinition":{"name":v.name,"frame":frame(v.frame),"elevation":v.elevation}}}));
    }
    for v in &document.model_windows {
        nodes.push(json!({"path":format!("{prefix}/modelWindow/{}",v.id.0),"attributes":{"ifccad::modelWindow":{"rectangle":v.rectangle,"view":encode_view(v.view),"aspectRatio":v.aspect_ratio,"renderMode":format!("{:?}",v.render_mode),"grid":encode_grid(v.grid),"snap":encode_snap(v.snap),"storedUcs":encode_ucs(v.stored_ucs,prefix),"useStoredUcs":v.use_stored_ucs}}}));
    }
    nodes
}
pub(super) fn encode_selections(document: &IfccadDocument, prefix: &str) -> Value {
    let mut out = json!({});
    if let Some(v) = &document.workspace_state {
        let mut state = json!({});
        if let Some(id) = v.current_layer_id {
            state["currentLayer"] = json!(format!("{prefix}/layer/{id}"));
        }
        if let Some(id) = v.active_layout_id {
            state["activeLayout"] = json!(format!("{prefix}/layout/{id}"));
        }
        if !state.as_object().unwrap().is_empty() {
            out["ifccad::drawingWorkspace"] = state;
        }
    }
    if let Some(v) = &document.model_view_state {
        let mut state = json!({});
        if let Some(current) = v.current_model_ucs {
            state["currentModelUcs"] = encode_ucs(current, prefix);
        }
        if let Some(id) = v.active_model_window_id {
            state["activeModelWindow"] = json!(format!("{prefix}/modelWindow/{}", id.0));
        }
        if !state.as_object().unwrap().is_empty() {
            out["ifccad::modelViewState"] = state;
        }
    }
    out
}
