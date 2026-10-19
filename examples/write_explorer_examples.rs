//! Generate the explorer's representative native drawings and strict-read them.
use ifccad::{geometry_kernel::*, workspace_kernel::*};
use serde_json::{json, Value};
use std::{error::Error, path::Path};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
#[path = "support/ifccad_consistency.rs"]
mod consistency;
#[path = "support/complex_line_patterns.rs"]
mod line_patterns;
#[path = "support/explorer_overview.rs"]
mod overview;

fn ifccad_overview() -> Result<Value> {
    // Retain the production-tested viewport construction; author a new model
    // inventory, patterns, nested blocks and source contributions around it.
    let mut d: Value = serde_json::from_slice(include_bytes!(
        "../tests/fixtures/ifccad/hello-viewports.ifcx"
    ))?;
    d["header"]["id"] = json!("explorer-ifccad-overview");
    d["header"]["author"] = json!("OpenAEC");
    d["header"]["timestamp"] = json!("2026-10-06T00:00:00Z");
    let all = d["data"].as_array_mut().unwrap();
    let drawing = all.iter_mut().find(|n| n["path"] == "/cad/d1").unwrap();
    drawing["children"]["block1"] = json!("/cad/d1/block/1");
    drawing["children"]["block2"] = json!("/cad/d1/block/2");
    drawing["children"]["axis"] = json!("/cad/d1/linePattern/7");
    drawing["children"]["unused"] = json!("/cad/d1/linePattern/8");
    drawing["attributes"]["ifccad::drawing"]["nextBlockId"] = json!(3);
    drawing["attributes"]["ifccad::drawing"]["nextLinePatternId"] = json!(9);
    let model = all
        .iter_mut()
        .find(|n| n["path"] == "/cad/d1/layout/1")
        .unwrap();
    model["children"] = json!({"0":"/cad/d1/e110","1":"/cad/d1/e111","2":"/cad/d1/e112","3":"/cad/d1/e113","4":"/cad/d1/e114"});
    let sheet = all
        .iter_mut()
        .find(|n| n["attributes"]["ifccad::layout"]["kind"] == "Paper")
        .unwrap();
    sheet["attributes"]["ifccad::layout"]["name"] = json!("A3 detail sheet");
    sheet["attributes"]["ifccad::layout"]["media"] = json!({"width":420,"height":297,"unit":"mm"});
    sheet["attributes"]["ifccad::layout"]["plotSettings"] = json!({
        "plotUnit":"mm",
        "page":{"printableArea":{"minX":0,"minY":0,"maxX":420,"maxY":297},"rotation":"none"},
        "area":{"mode":"Layout"},
        "mapping":{"scale":{"mode":"Fixed","outputLength":1,"scopeLength":1},
            "placement":{"mode":"Offset","reference":"Media","x":0,"y":0}},
        "output":{"shadedPlot":{"mode":"AsDisplayed","quality":{"mode":"Normal"}},"applyPlotStyles":false},
        "options":{"plotViewportBorders":false,"plotPaperSpaceLast":false,"hidePaperSpaceObjects":false,
            "plotLineWeights":false,"scaleLineWeights":false,"plotTransparency":false}
    });
    let appearance = json!({"color":{"mode":"ByLayer"},"linePattern":{"mode":"ByLayer"},"lineWeight":{"mode":"ByLayer"},"opacity":{"mode":"ByLayer"}});
    let entity = json!({"layer":"/cad/d1/layer/0","appearance":appearance});
    let placement = |x, y| json!({"origin":[x,y,0.],"xAxis":[1.,0.,0.],"yAxis":[0.,1.,0.]});
    for (path, attributes) in [
        (
            "/cad/d1/e110",
            json!({"ifccad::entity":entity,"ifccad::geom::lineSegment":{"start":[0,0,0],"end":[120,0,0]}}),
        ),
        (
            "/cad/d1/e111",
            json!({"ifccad::entity":entity,"ifccad::geom::placement":placement(30.,30.),"ifccad::geom::circle":{"radius":12}}),
        ),
        (
            "/cad/d1/e112",
            json!({"ifccad::entity":entity,"ifccad::geom::placement":placement(60.,20.),"ifccad::geom::planarPolyline":{"vertices":[[0,0],[20,0],[20,25]],"closed":false,"linePatternGeneration":"continuous"}}),
        ),
        (
            "/cad/d1/e113",
            json!({"ifccad::entity":entity,"ifccad::blockInstance":{"definition":"/cad/d1/block/2","transform":{"placement":placement(100.,50.),"rotation":0.3,"scale":[-1,1,1]}}}),
        ),
        (
            "/cad/d1/e114",
            json!({"ifccad::entity":entity,"ifccad::blockInstance":{"definition":"/cad/d1/block/1","transform":{"placement":placement(130.,50.),"rotation":0,"scale":[2,1,1]}}}),
        ),
        (
            "/cad/d1/e200",
            json!({"ifccad::entity":entity,"ifccad::geom::lineSegment":{"start":[2,0,0],"end":[12,8,0]}}),
        ),
        (
            "/cad/d1/e201",
            json!({"ifccad::entity":entity,"ifccad::blockInstance":{"definition":"/cad/d1/block/1","transform":{"placement":placement(0.,0.),"rotation":0,"scale":[1,1,1]}}}),
        ),
    ] {
        all.push(json!({"path":path,"attributes":attributes}));
    }
    all.push(json!({"path":"/cad/d1/block/1","children":{"0":"/cad/d1/e200"},"attributes":{"ifccad::blockDefinition":{"name":"Marker","basePoint":[2,0,0],"insertionUnit":"mm"}}}));
    all.push(json!({"path":"/cad/d1/block/2","children":{"0":"/cad/d1/e201"},"attributes":{"ifccad::blockDefinition":{"name":"Nested marker","basePoint":[0,0,0],"insertionUnit":"mm"}}}));
    all.push(json!({"path":"/cad/d1/linePattern/7","attributes":{"ifccad::linePattern":{"name":"Axis","description":"Dash and dot","pattern":[6,-2,0,-2]}}}));
    all.push(json!({"path":"/cad/d1/linePattern/8","attributes":{"ifccad::linePattern":{"name":"Unused reference","pattern":[]}}}));
    let layer = all
        .iter_mut()
        .find(|n| {
            n["attributes"]
                .get("ifccad::layer")
                .is_some_and(|l| l["name"] != "0")
        })
        .unwrap();
    layer["attributes"]["ifccad::layer"]["appearance"]["linePattern"] =
        json!("/cad/d1/linePattern/7");
    // Author the additional primitive tuples as millimetre values in this new
    // drawing; the source sample's document and coordinate unit are unchanged.
    let primitive_sample: Value = serde_json::from_slice(include_bytes!(
        "../tests/fixtures/ifccad/hello-geometry.ifcx"
    ))?;
    let mut next = 310;
    for original in primitive_sample["data"].as_array().unwrap() {
        if original["attributes"].as_object().is_some_and(|a| {
            [
                "ifccad::geom::point",
                "ifccad::geom::arc",
                "ifccad::geom::ellipse",
                "ifccad::geom::ellipseArc",
                "ifccad::geom::spatialPolyline",
            ]
            .iter()
            .any(|key| a.contains_key(*key))
        }) {
            let mut item = original.clone();
            item["path"] = json!(format!("/cad/d1/e{next}"));
            item["attributes"]["ifccad::entity"]["layer"] = json!("/cad/d1/layer/0");
            let model = all
                .iter_mut()
                .find(|n| n["path"] == "/cad/d1/layout/1")
                .unwrap();
            let index = model["children"].as_object().unwrap().len();
            model["children"][index.to_string()] = item["path"].clone();
            all.push(item);
            next += 1;
        }
    }
    all.iter_mut().find(|n| n["path"] == "/cad/d1").unwrap()["attributes"]["ifccad::drawing"]
        ["nextEntityId"] = json!(next);
    Ok(d)
}
fn prepare_ifccad(name: &str, value: &Value) -> Result<Vec<u8>> {
    let mut value = value.clone();
    // First author valid conservative envelopes, then ask the shared native
    // preparation to replace them. Updating only bounds retains source fragments.
    for node in value["data"].as_array_mut().unwrap() {
        if node["attributes"]["ifccad::layout"]["kind"] == "Model" {
            node["attributes"]["ifccad::layout"]["bounds"] =
                json!({"min":[-1e8,-1e8,-1e8],"max":[1e8,1e8,1e8]});
        }
        if node["attributes"].get("ifccad::blockDefinition").is_some() {
            let bound = if node["path"] == "/cad/d1/block/1" {
                1000.
            } else {
                10000.
            };
            node["attributes"]["ifccad::blockDefinition"]["bounds"] =
                json!({"min":[-bound,-bound,-bound],"max":[bound,bound,bound]});
        }
    }
    let initial = serde_json::to_vec(&value)?;
    let loaded = ifccad::load_ifccad_bytes(&initial, Default::default())
        .map_err(|e| format!("{name}: {}", e.report().errors.join("; ")))?;
    let mut document = loaded.document().clone();
    ifccad::recompute_ifccad_document_bounds(&mut document)?;
    let encoded = ifccad::encode_ifccad_document(&document)?;
    let prepared: Value = serde_json::from_slice(encoded.bytes())?;
    for node in value["data"].as_array_mut().unwrap() {
        if let Some(prepared) = prepared["data"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["path"] == node["path"])
        {
            for role in ["ifccad::layout", "ifccad::blockDefinition"] {
                if node["attributes"].get(role).is_some() {
                    for field in ["bounds", "boundsQuality"] {
                        if let Some(value) = prepared["attributes"][role].get(field) {
                            node["attributes"][role][field] = value.clone();
                        } else {
                            node["attributes"][role]
                                .as_object_mut()
                                .unwrap()
                                .remove(field);
                        }
                    }
                }
            }
        }
    }
    let bytes = consistency::normalize(&value)?;
    ifccad::load_ifccad_bytes(&bytes, Default::default())
        .map_err(|e| format!("{name}: {}", e.report().errors.join("; ")))?;
    Ok(bytes)
}
fn write_ifccad(root: &Path, name: &str, value: &Value) -> Result<()> {
    std::fs::write(root.join(name), prepare_ifccad(name, value)?)?;
    Ok(())
}

fn ifccad_workspace() -> Result<ifccad::IfccadDocument> {
    use ifccad::*;
    const BIG: u64 = 9007199254740993;
    let mut d = load_ifccad_bytes(
        include_bytes!("../tests/fixtures/ifccad/hello-viewports.ifcx"),
        Default::default(),
    )?
    .into_document();
    d.header.id = "explorer-ifccad-workspace".into();
    let grid = WorkspaceGrid {
        enabled: true,
        spacing: Point2::new(10., 10.),
        style: WorkspaceGridStyle::Dots,
        major_line_frequency: 5,
        beyond_limits: false,
        adaptive: true,
        subdivision: false,
        follows_workplane: true,
    };
    let snap = WorkspaceSnap {
        enabled: false,
        base: Point2::new(0., 0.),
        spacing: Point2::new(5., 5.),
        angle: 0.,
        style: WorkspaceSnapStyle::Rectangular,
        isometric_plane: WorkspaceIsometricPlane::Top,
    };
    let view = WorkspaceView {
        center: Point2::new(80., 40.),
        target: Point3::new(0., 0., 0.),
        direction: Vector3::new(0., 0., 1.),
        height: 140.,
        twist: 0.,
        projection: WorkspaceProjection::Orthographic,
        lens_length: Some(50.),
        front_clip: WorkspaceClip {
            mode: WorkspaceClipMode::Disabled,
            distance: Some(10.),
        },
        back_clip: WorkspaceClip {
            mode: WorkspaceClipMode::Disabled,
            distance: None,
        },
    };

    d.ucs_definitions.push(IfccadUcsDefinition {
        id: IfccadUcsId(BIG),
        name: "Reference plane".into(),
        frame: CoordinateFrame3::default(),
        elevation: 0.,
    });
    let mut grid = grid;
    grid.style = WorkspaceGridStyle::Lines;
    let mut snap = snap;
    snap.spacing = Point2::new(0., 0.);
    snap.angle = std::f64::consts::FRAC_PI_2;
    snap.style = WorkspaceSnapStyle::Isometric;
    snap.isometric_plane = WorkspaceIsometricPlane::Left;
    d.model_windows.push(IfccadModelWindow {
        id: IfccadModelWindowId(BIG),
        rectangle: [0., 0., 1., 1.],
        view,
        aspect_ratio: 1.5,
        render_mode: WorkspaceRenderMode::TwoDimensional,
        grid,
        snap,
        stored_ucs: IfccadUcsSelection::World,
        use_stored_ucs: false,
    });
    d.model_view_state = Some(IfccadModelViewState {
        current_model_ucs: Some(IfccadUcsSelection::Named(IfccadUcsId(BIG))),
        active_model_window_id: Some(IfccadModelWindowId(BIG)),
    });
    d.workspace_state = Some(IfccadDrawingWorkspaceState {
        current_layer_id: Some(d.layers[0].id),
        active_layout_id: Some(d.model.id),
    });
    d.id_counters.next_ucs_id = BIG + 1;
    d.id_counters.next_model_window_id = BIG + 1;
    for p in &mut d.paper_layouts {
        let mut canvas_grid = grid;
        canvas_grid.spacing = Point2::new(0.25, 0.5);
        p.canvas = Some(IfccadPaperCanvas {
            view,
            frame: Some(ifccad::workspace_kernel::WorkspaceCanvasFrame {
                center: Point3::new(100., -50., 3.),
                width: 20.,
                height: 10.,
            }),
            grid: canvas_grid,
            snap,
            stored_ucs: IfccadUcsSelection::World,
            use_stored_ucs: false,
            current_ucs: None,
            active_context: None,
        });
        for e in p
            .entities
            .iter_mut()
            .filter_map(IfccadEntity::as_native_mut)
        {
            if let IfccadEntityKind::Viewport(v) = &mut e.kind {
                let mut viewport_grid = grid;
                viewport_grid.spacing = Point2::new(2., 3.);
                v.workspace = Some(Box::new(IfccadViewportWorkspace {
                    grid: viewport_grid,
                    snap,
                    stored_ucs: IfccadUcsSelection::Unnamed(CoordinateFrame3::try_new(
                        Point3::new(2., 3., 0.),
                        Vector3::new(1., 0., 0.),
                        Vector3::new(0., 1., 0.),
                    )?),
                    use_stored_ucs: false,
                }));
            }
        }
    }
    validate_ifccad_document(&d)?;
    Ok(d)
}

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let root = args
        .next()
        .map(std::path::PathBuf::from)
        .ok_or("usage: write_explorer_examples <examples-directory> <test-fixtures-directory>")?;
    let fixtures = args
        .next()
        .map(std::path::PathBuf::from)
        .ok_or("test-fixtures-directory required")?;
    std::fs::create_dir_all(root.join("ifccad"))?;
    std::fs::create_dir_all(fixtures.join("ifccad"))?;
    let workspace = ifccad::encode_ifccad_document(&ifccad_workspace()?)?;
    ifccad::load_ifccad_bytes(workspace.bytes(), Default::default())?;
    std::fs::write(
        fixtures.join("ifccad/workspace-state.ifcx"),
        workspace.bytes(),
    )?;
    let overview = ifccad_overview()?;
    let native = ifccad::load_ifccad_bytes(
        &prepare_ifccad("overview.ifcx", &overview)?,
        Default::default(),
    )?
    .into_document();
    let native = line_patterns::ifccad(overview::ifccad(native, ifccad_workspace()?)?)?;
    let bytes = ifccad::encode_ifccad_document(&native)?;
    ifccad::load_ifccad_bytes(bytes.bytes(), Default::default())?;
    let mut curated: Value = serde_json::from_slice(bytes.bytes())?;
    let all = curated["data"].as_array_mut().unwrap();
    let entity = all
        .iter_mut()
        .find(|n| n["path"] == "/cad/d1/e1")
        .ok_or("overview line required")?;
    let geometry = entity["attributes"]
        .as_object_mut()
        .unwrap()
        .remove("ifccad::geom::lineSegment")
        .ok_or("line geometry required")?;
    all.push(json!({"path":"/cad/d1/e1","attributes":{"ifccad::geom::lineSegment":geometry,"example::note":"Geometry supplied by a second contribution"}}));
    all.push(json!({"path":"/reference/design-intent","children":{"line":"/cad/d1/e1","marker":"/cad/d1/block/1"},"inherits":{"reference":"/reference/shared-note"},"attributes":{"example::note":"References do not confer CAD ownership"}}));
    all.push(json!({"path":"/reference/shared-note","attributes":{"example::note":"Shared design information"}}));
    write_ifccad(&root.join("ifccad"), "overview.ifcx", &curated)?;
    let mut layouts = overview.clone();
    layouts["header"]["id"] = json!("explorer-ifccad-layouts-viewports");
    write_ifccad(&fixtures.join("ifccad"), "layouts-viewports.ifcx", &layouts)?;
    let mut fragments = overview;
    fragments["header"]["id"] = json!("explorer-ifccad-blocks-fragments");
    let all = fragments["data"].as_array_mut().unwrap();
    let n = all
        .iter_mut()
        .find(|n| n["path"] == "/cad/d1/e110")
        .unwrap();
    let geometry = n["attributes"]
        .as_object_mut()
        .unwrap()
        .remove("ifccad::geom::lineSegment")
        .unwrap();
    all.push(json!({"path":"/cad/d1/e110","attributes":{"ifccad::geom::lineSegment":geometry,"example::note":"Geometry supplied by a second source contribution"}}));
    all.push(json!({"path":"/reference/design-intent","children":{"line":"/cad/d1/e110","marker":"/cad/d1/block/1"},"inherits":{"reference":"/reference/shared-note"},"attributes":{"example::note":"Non-CAD references do not acquire CAD ownership"}}));
    all.push(json!({"path":"/reference/shared-note","attributes":{"example::note":"Shared design information"}}));
    write_ifccad(
        &fixtures.join("ifccad"),
        "blocks-and-fragments.ifcx",
        &fragments,
    )?;
    Ok(())
}
