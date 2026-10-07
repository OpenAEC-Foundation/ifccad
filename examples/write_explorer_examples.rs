//! Generate the explorer's representative native drawings and strict-read them.
use ocdraw::ocdraw::*;
use serde_json::{json, Value};
use std::{error::Error, path::Path};
type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn ocdraw_overview() -> Result<OcdrawDocument> {
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("explorer-ocdraw-overview", "mm"))?;
    let solid = b.ensure_continuous_line_pattern()?;
    let dash = b.add_line_pattern(LinePatternDefinition {
        name: "Axis".into(),
        description: Some("Long dash and dot".into()),
        pattern: vec![6., -2., 0., -2.],
    })?;
    b.add_line_pattern(LinePatternDefinition {
        name: "Unused reference".into(),
        description: Some("An unused definition stays in the document".into()),
        pattern: vec![],
    })?;
    let layer = b.add_layer(LayerDefinition::new(
        "Geometry",
        RgbColor::new(210, 119, 6),
        solid,
    ))?;
    b.add_layer(LayerDefinition::new(
        "Construction",
        RgbColor::new(90, 130, 160),
        dash,
    ))?;
    let inner = b.add_block_definition(BlockDefinition::new("Marker"))?;
    let outer = b.add_block_definition(BlockDefinition::new("Repeated markers"))?;
    let frame = |x, y, z| {
        CoordinateFrame3::try_new(
            Point3::new(x, y, z),
            Vector3::new(1., 0., 0.),
            Vector3::new(0., 1., 0.),
        )
    };
    b.add_line(LineDefinition::new(layer, [0., 0., 0.], [100., 0., 0.]))?;
    let geometries = vec![
        DrawingGeometry::Point {
            placement: frame(10., 20., 0.)?,
        },
        DrawingGeometry::Circle {
            placement: frame(30., 30., 0.)?,
            radius: 10.,
        },
        DrawingGeometry::Arc {
            placement: frame(65., 30., 0.)?,
            radius: 12.,
            start_parameter: 0.25,
            sweep_parameter: 2.5,
        },
        DrawingGeometry::Ellipse {
            placement: frame(100., 30., 0.)?,
            semi_major_radius: 16.,
            semi_minor_radius: 7.,
            arc: None,
        },
        DrawingGeometry::Ellipse {
            placement: frame(140., 30., 0.)?,
            semi_major_radius: 16.,
            semi_minor_radius: 7.,
            arc: Some((0.2, 3.0)),
        },
        DrawingGeometry::PlanarPolyline {
            placement: frame(0., 65., 0.)?,
            vertices: vec![[0., 0., 0.5], [25., 0., 0.], [25., 20., 0.]],
            closed: false,
            line_pattern_generation: LinePatternGeneration::Continuous,
        },
        DrawingGeometry::SpatialPolyline {
            vertices: vec![[60., 60., 0.], [80., 70., 15.], [100., 60., 0.]],
            closed: false,
            line_pattern_generation: LinePatternGeneration::PerSegment,
        },
    ];
    for g in geometries {
        b.add_geometric_entity(GeometricEntityDefinition::new(0, layer, g))?;
    }
    b.add_line(LineDefinition::new(layer, [2., 0., 0.], [12., 8., 0.]).in_scope(inner))?;
    b.add_geometric_entity(GeometricEntityDefinition::new(
        outer,
        layer,
        DrawingGeometry::BlockInstance {
            definition_scope_id: inner,
            transform: BlockTransform::try_new(frame(0., 0., 0.)?, 0., Scale3::new(1., 1., 1.))?,
        },
    ))?;
    for x in [130., 160.] {
        b.add_geometric_entity(GeometricEntityDefinition::new(
            0,
            layer,
            DrawingGeometry::BlockInstance {
                definition_scope_id: outer,
                transform: BlockTransform::try_new(
                    frame(x, 70., 0.)?,
                    0.3,
                    Scale3::new(-1., 1., 1.),
                )?,
            },
        ))?;
    }
    b.add_ucs_definition(UcsDefinition::new(
        "Detail plane",
        CoordinateFrame3::try_new(
            Point3::new(0., 0., 5.),
            Vector3::new(0., 1., 0.),
            Vector3::new(0., 0., 1.),
        )?,
        5.,
    ))?;
    b.set_current_layer(layer);
    b.set_active_layout(0);
    let mut d = b.build_document()?;
    d.block_definitions
        .iter_mut()
        .find(|v| v.scope_id == inner)
        .unwrap()
        .base_point = [2., 0., 0.];
    recompute_ocdraw_document_bounds(&mut d)?;
    Ok(d)
}
fn ocdraw_layouts() -> Result<OcdrawDocument> {
    let mut d = load_ocdraw_bytes(include_bytes!(
        "../conformance/next/ocdraw/valid/viewport-bulged-clip.ocdraw.json"
    ))?
    .into_document();
    d.drawing_id = "explorer-ocdraw-layouts-viewports".into();
    d.unit = "mm".into();
    for layout in &mut d.layouts {
        if layout.kind == DrawingLayoutKind::Paper {
            layout.name = "Detail sheet".into();
        }
    }
    let base = ocdraw_overview()?;
    let scope = d
        .scopes
        .iter_mut()
        .find(|s| s.kind == DrawingScopeKind::Model)
        .unwrap();
    for mut e in base
        .geometric_entities
        .into_iter()
        .filter(|e| !matches!(e.geometry, DrawingGeometry::BlockInstance { .. }))
        .take(8)
    {
        e.id = d.next_entity_id;
        d.next_entity_id += 1;
        e.layer_id = d.layers[0].id;
        scope.entities.push(e.id);
        d.geometric_entities.push(e);
    }
    recompute_ocdraw_document_bounds(&mut d)?;
    Ok(d)
}
fn ocdraw_state() -> Result<OcdrawDocument> {
    let mut d = ocdraw_overview()?;
    d.drawing_id = "explorer-ocdraw-state-storage".into();
    let grid = DrawingGrid {
        enabled: true,
        spacing: Point2::new(10., 10.),
        style: DrawingGridStyle::Dots,
        major_line_frequency: 5,
        beyond_limits: false,
        adaptive: true,
        subdivision: false,
        follows_workplane: true,
    };
    let snap = DrawingSnap {
        enabled: false,
        base: Point2::new(0., 0.),
        spacing: Point2::new(5., 5.),
        angle: 0.,
        style: DrawingSnapStyle::Rectangular,
        isometric_plane: DrawingIsometricPlane::Top,
    };
    let view = DrawingView {
        center: Point2::new(80., 40.),
        target: Point3::new(0., 0., 0.),
        direction: Vector3::new(0., 0., 1.),
        height: 140.,
        twist: 0.,
        projection: DrawingProjection::Orthographic,
        lens_length: Some(50.),
        front_clip: DrawingClip {
            mode: DrawingClipMode::Disabled,
            distance: Some(10.),
        },
        back_clip: DrawingClip {
            mode: DrawingClipMode::Disabled,
            distance: None,
        },
    };
    d.model_windows.push(DrawingModelWindow {
        id: 7,
        rectangle: [0., 0., 1., 1.],
        view,
        aspect_ratio: 1.5,
        render_mode: DrawingRenderMode::TwoDimensional,
        grid,
        snap,
        stored_ucs: DrawingUcsSelection::Named(d.ucs_definitions[0].id),
        use_stored_ucs: true,
    });
    d.view_state = Some(DrawingViewState {
        current_model_ucs: DrawingUcsSelection::Named(d.ucs_definitions[0].id),
        active_model_window_id: 7,
    });
    d.scopes
        .iter_mut()
        .find(|s| s.kind == DrawingScopeKind::Model)
        .unwrap()
        .entities
        .rotate_left(3);
    Ok(d)
}
fn ifccad_overview() -> Result<Value> {
    // Retain the production-tested viewport construction; author a new model
    // inventory, patterns, nested blocks and source contributions around it.
    let mut d: Value = serde_json::from_slice(include_bytes!("ifccad/hello-viewports.ifcx"))?;
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
        .find(|n| n["path"] == "/cad/d1/layout/42")
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
        .find(|n| n["path"] == "/cad/d1/layer/4")
        .unwrap();
    layer["attributes"]["ifccad::layer"]["appearance"]["linePattern"] =
        json!("/cad/d1/linePattern/7");
    // Author the additional primitive tuples as millimetre values in this new
    // drawing; the source sample's document and coordinate unit are unchanged.
    let primitive_sample: Value =
        serde_json::from_slice(include_bytes!("ifccad/hello-geometry.ifcx"))?;
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
    Ok(d)
}
fn write_ifccad(root: &Path, name: &str, value: &Value) -> Result<()> {
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
    let loaded = ocdraw::ifccad::load_ifccad_bytes(&initial, Default::default())
        .map_err(|e| format!("{name}: {}", e.report().errors.join("; ")))?;
    let mut document = loaded.document().clone();
    ocdraw::ifccad::recompute_ifccad_document_bounds(&mut document)?;
    let encoded = ocdraw::ifccad::encode_ifccad_document(&document)?;
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
                    node["attributes"][role]["bounds"] =
                        prepared["attributes"][role]["bounds"].clone();
                }
            }
        }
    }
    let bytes = serde_json::to_vec_pretty(&value)?;
    ocdraw::ifccad::load_ifccad_bytes(&bytes, Default::default())
        .map_err(|e| format!("{name}: {}", e.report().errors.join("; ")))?;
    std::fs::write(root.join(name), bytes)?;
    Ok(())
}
fn write_ocdraw(root: &Path, name: &str, d: &OcdrawDocument) -> Result<()> {
    let encoded = encode_ocdraw_document(d)?;
    load_ocdraw_bytes(encoded.bytes())?;
    std::fs::write(root.join(name), encoded.bytes())?;
    Ok(())
}
fn main() -> Result<()> {
    let root = std::env::args_os()
        .nth(1)
        .map(std::path::PathBuf::from)
        .ok_or("usage: write_explorer_examples <examples-directory>")?;
    std::fs::create_dir_all(root.join("ocdraw"))?;
    std::fs::create_dir_all(root.join("ifccad"))?;
    write_ocdraw(
        &root.join("ocdraw"),
        "overview.ocdraw.json",
        &ocdraw_overview()?,
    )?;
    write_ocdraw(
        &root.join("ocdraw"),
        "layouts-viewports.ocdraw.json",
        &ocdraw_layouts()?,
    )?;
    write_ocdraw(
        &root.join("ocdraw"),
        "state-and-storage.ocdraw.json",
        &ocdraw_state()?,
    )?;
    let overview = ifccad_overview()?;
    write_ifccad(&root.join("ifccad"), "overview.ifcx", &overview)?;
    let mut layouts = overview.clone();
    layouts["header"]["id"] = json!("explorer-ifccad-layouts-viewports");
    write_ifccad(&root.join("ifccad"), "layouts-viewports.ifcx", &layouts)?;
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
        &root.join("ifccad"),
        "blocks-and-fragments.ifcx",
        &fragments,
    )?;
    Ok(())
}
