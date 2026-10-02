use ocdraw::ocdraw::{load_drawing_bytes, DrawingGeometry, DrawingLoadStatus};
use serde_json::{json, Value};

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../conformance/next/ocdraw/valid/empty.ocdraw.json"
    ))
    .unwrap()
}

fn load(value: &Value) -> ocdraw::ocdraw::DrawingLoadOutcome {
    load_drawing_bytes(&serde_json::to_vec(value).unwrap())
}

#[test]
fn huge_stream_counts_are_rejected_before_row_validation() {
    const CHILD: &str = "OCDRAW_TEST_HUGE_STREAM_COUNT_CHILD";
    if std::env::var_os(CHILD).is_some() {
        for stream in ["pointStream", "planarPolylineStream", "viewportStream"] {
            let mut value = fixture();
            value["streams"] = json!({stream: {"count": u64::MAX}});
            let outcome = load(&value);
            assert_eq!(outcome.status(), DrawingLoadStatus::Invalid);
            assert!(outcome
                .diagnostics()
                .iter()
                .any(|d| d.code == "STREAM_COLUMN"));
        }
        let mut value = with_line();
        value["streams"]["lineStream"]["count"] = json!(u64::MAX);
        let outcome = load(&value);
        assert_eq!(outcome.status(), DrawingLoadStatus::Invalid);
        assert!(outcome
            .diagnostics()
            .iter()
            .any(|d| matches!(d.code, "COLUMN_COUNT" | "STREAM_COUNT")));
        value["streams"]["lineStream"]["count"] = json!(1.0);
        let outcome = load(&value);
        assert_eq!(outcome.status(), DrawingLoadStatus::Invalid);
        assert!(outcome
            .diagnostics()
            .iter()
            .any(|d| d.code == "STREAM_COUNT"));
        return;
    }

    // Isolate the reader so a regression cannot hang the complete test suite.
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "huge_stream_counts_are_rejected_before_row_validation",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "malformed-count reader subprocess failed");
            break;
        }
        if std::time::Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("reader did not reject tiny malformed streams within 10 seconds");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn with_line() -> Value {
    let mut value = fixture();
    value["linePatterns"] = json!([{"id":0,"name":"Continuous","pattern":[]}]);
    value["header"]["nextLinePatternId"] = json!(1);
    value["header"]["nextEntityId"] = json!(2);
    value["header"]["nextLayerId"] = json!(1);
    value["layers"] = json!([{
        "id": 0, "name": "0", "visible": true, "frozen": false, "locked": false,
        "plottable": true, "frozenInNewViewports": false,
        "color": {"rgb": [255, 255, 255]}, "opacity": 1.0,
        "linePatternId": 0, "lineWeight": 0.25
    }]);
    value["scopes"][0]["bounds"] = json!({
        "minX": 0.0, "minY": 0.0, "minZ": 0.0,
        "maxX": 2.0, "maxY": 2.0, "maxZ": 0.0
    });
    value["streams"] = json!({"lineStream": {
        "count": 1, "entityId": [1], "layerId": [0],
        "x1": [0.0], "y1": [0.0], "x2": [2.0], "y2": [2.0],
        "colorMode": ["Explicit"], "color": [{"rgb": [255, 0, 0]}]
    }});
    value["scopes"][0]["entities"] = json!([1]);
    value
}

#[test]
fn line_geometry_is_exposed_as_typed_drawing_meaning() {
    let read = load(&with_line());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let entities = read.validated_drawing().unwrap().geometric_entities();
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].id(), 1);
    assert_eq!(
        read.validated_drawing()
            .unwrap()
            .owner_scope_id(entities[0].id()),
        Some(0)
    );
    assert!(matches!(
        entities[0].geometry(),
        DrawingGeometry::Line { start, end }
            if *start == [0.0, 0.0, 0.0]
            && *end == [2.0, 2.0, 0.0]
    ));
}

#[test]
fn empty_drawing_opens_without_an_implicit_layer() {
    let outcome = load(&fixture());
    assert_eq!(
        outcome.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        outcome.diagnostics()
    );
    let drawing = outcome.validated_drawing().unwrap();
    assert_eq!(drawing.drawing_id(), "empty-drawing");
    assert_eq!(drawing.typed_layers().len(), 0);
    assert_eq!(drawing.typed_layouts().len(), 1);
    assert_eq!(drawing.plot_style_mode(), "colorDependent");
}

#[test]
fn malformed_plot_rectangles_fail_shared_layout_validation() {
    let mut value = fixture();
    value["layouts"][0]["plotSettings"] = json!({
        "media": {"unit":"mm","width":210.0,"height":297.0,
            "printableArea":{"minX":205.0,"minY":5.0,"maxX":5.0,"maxY":292.0},
            "rotation":"none"},
        "area":{"mode":"Extents"},
        "mapping":{"scale":{"mode":"FitToArea"},"placement":{"mode":"Centered"}},
        "output":{"shadedPlot":{"mode":"AsDisplayed","quality":{"mode":"Normal"}},"applyPlotStyles":false},
        "options":{"plotViewportBorders":false,"plotPaperSpaceLast":false,"hidePaperSpaceObjects":false,
            "plotLineWeights":true,"scaleLineWeights":false,"plotTransparency":false}
    });
    let result = load(&value);
    assert_eq!(result.status(), DrawingLoadStatus::Invalid);
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|item| item.code == "PLOT_RECT"),
        "{:?}",
        result.diagnostics()
    );
}

#[test]
fn drawing_view_state_requires_existing_model_window_and_named_ucs() {
    let mut value = fixture();
    value["drawingViewState"] = json!({
        "currentModelUcs": {"kind": "Named", "ucsId": 7},
        "activeModelWindowId": 3
    });
    let result = load(&value);
    assert_eq!(result.status(), DrawingLoadStatus::Invalid);
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|item| item.code == "MODEL_WINDOW_REF"),
        "{:?}",
        result.diagnostics()
    );
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|item| item.code == "UCS_REF"),
        "{:?}",
        result.diagnostics()
    );
    value["drawingViewState"]["currentModelUcs"] = json!({"kind": "Named"});
    assert!(load(&value)
        .diagnostics()
        .iter()
        .any(|item| item.code == "UCS_REF"));
    value["drawingViewState"]["currentModelUcs"] = json!({"kind": "World", "ucsId": 7});
    assert!(load(&value)
        .diagnostics()
        .iter()
        .any(|item| item.code == "UCS_CHOICE"));
    value["drawingViewState"]["currentModelUcs"] = json!({"kind": "Unnamed"});
    assert!(load(&value)
        .diagnostics()
        .iter()
        .any(|item| item.code == "UCS_CHOICE"));
}

fn with_model_window() -> Value {
    let mut value = fixture();
    value["drawingViewState"] = json!({
        "currentModelUcs": {"kind":"World"},
        "activeModelWindowId": 3
    });
    value["modelWindows"] = json!([{
        "modelWindowId": 3,
        "rectangle": {"minX":0.0,"minY":0.0,"maxX":1.0,"maxY":1.0},
        "view": {
            "center":{"x":0.0,"y":0.0},
            "target":{"x":0.0,"y":0.0,"z":0.0},
            "direction":{"x":0.0,"y":0.0,"z":1.0},
            "height":10.0,"twist":0.0,"projection":"Orthographic",
            "frontClip":{"mode":"Disabled"},"backClip":{"mode":"Disabled"}
        },
        "aspectRatio":1.0,"renderMode":"TwoDimensional",
        "grid":{"enabled":false,"spacing":{"x":1.0,"y":1.0},"style":"Lines",
            "majorLineFrequency":5,"beyondLimits":false,"adaptive":false,
            "subdivision":false,"followsWorkplane":false},
        "snap":{"enabled":false,"base":{"x":0.0,"y":0.0},
            "spacing":{"x":1.0,"y":1.0},"angle":0.0,
            "style":"Rectangular","isometricPlane":"Left"},
        "storedUcs":{"kind":"World"},"useStoredUcs":false
    }]);
    value
}

#[test]
fn drawing_view_state_decodes_to_typed_workspace_records() {
    let value = with_model_window();
    let result = load(&value);
    assert_eq!(
        result.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        result.diagnostics()
    );
    let drawing = result.validated_drawing().unwrap();
    assert_eq!(drawing.view_state().unwrap().active_model_window_id, 3);
    assert_eq!(drawing.model_windows()[0].view.height, 10.0);
    assert!(drawing.has_unconverted_view_state());
}

fn with_viewport() -> Value {
    let mut value = fixture();
    value["linePatterns"] = json!([{"id":0,"name":"Continuous","pattern":[]}]);
    value["header"]["nextLinePatternId"] = json!(1);
    value["header"]["nextEntityId"] = json!(2);
    value["header"]["nextLayerId"] = json!(1);
    value["header"]["nextLayoutId"] = json!(2);
    value["layers"] = json!([{
        "id":0,"name":"0","visible":true,"frozen":false,"locked":false,
        "plottable":true,"frozenInNewViewports":false,
        "color":{"rgb":[255,255,255]},"opacity":1.0,
        "linePatternId":0,"lineWeight":0.25
    }]);
    value["layouts"].as_array_mut().unwrap().push(json!({
        "id":1,"scopeId":1,"kind":"paper","name":"Sheet","tabIndex":1
    }));
    value["scopes"].as_array_mut().unwrap().push(json!({
        "id":1,"kind":1,"entities":[1],
        "bounds":{"minX":-4.0,"minY":-3.0,"minZ":0.0,
                  "maxX":4.0,"maxY":3.0,"maxZ":0.0}
    }));
    value["streams"] = json!({
        "viewportStream":{
            "count":1,"entityId":[1],"viewScopeId":[0],"layerId":[0],
            "frame":[{"center":{"x":0.0,"y":0.0},"width":8.0,"height":6.0}],
            "view":[{"center":{"x":0.0,"y":0.0},
                "target":{"x":0.0,"y":0.0,"z":0.0},
                "direction":{"x":0.0,"y":0.0,"z":1.0},
                "height":10.0,"twist":0.0,"projection":"Orthographic",
                "frontClip":{"mode":"Disabled"},"backClip":{"mode":"Disabled"}}],
            "renderMode":["Wireframe"],"viewEnabled":[true],"viewLocked":[false],
            "paperClip":[{"enabled":false}],"plotShadingOverride":[null],
            "layerOverrideOffset":[0],"layerOverrideCount":[0]
        }
    });
    value
}

#[test]
fn clip_boundary_pattern_generation_does_not_change_clip_geometry() {
    let mut value = with_viewport();
    value["header"]["nextEntityId"] = json!(3);
    value["scopes"][1]["entities"] = json!([1, 2]);
    value["streams"]["viewportStream"]["paperClip"][0] =
        json!({"enabled":true,"boundaryEntityId":2});
    value["streams"]["planarPolylineStream"] = json!({
        "count":1,"entityId":[2],"layerId":[0],"closed":[true],
        "linePatternGeneration":["continuous"],"vertexOffset":[0],"vertexCount":[3],
        "x":[0.0,1.0,0.0],"y":[0.0,0.0,1.0],"bulge":[0.0,0.0,0.0]
    });
    let loaded = load(&value);
    assert_eq!(
        loaded.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        loaded.diagnostics()
    );
}

#[test]
fn paper_viewport_decodes_through_the_production_reader() {
    let mut value = with_viewport();
    let result = load(&value);
    assert_eq!(
        result.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        result.diagnostics()
    );
    let drawing = result.validated_drawing().unwrap();
    assert_eq!(drawing.viewports().len(), 1);
    assert_eq!(drawing.owner_scope_id(drawing.viewports()[0].id), Some(1));
    assert_eq!(drawing.viewports()[0].view_scope_id, 0);
    assert_eq!(drawing.scopes()[1].entities, vec![1]);
    value["streams"]["viewportStream"]["frame"][0]["width"] = json!(10.0);
    let outside = load(&value);
    assert_eq!(outside.status(), DrawingLoadStatus::Invalid);
    assert!(outside
        .diagnostics()
        .iter()
        .any(|item| item.code == "SCOPE_BOUNDS"));
}

#[test]
fn named_ucs_frame_must_have_independent_axes() {
    let mut value = fixture();
    value["ucsDefinitions"] = json!([{
        "ucsId": 0, "name": "Grid A", "elevation": 0.0,
        "frame": {
            "origin": {"x":0.0,"y":0.0,"z":0.0},
            "X": {"x":1.0,"y":0.0,"z":0.0},
            "Y": {"x":1.0,"y":0.0,"z":0.0}
        }
    }]);
    let result = load(&value);
    assert_eq!(result.status(), DrawingLoadStatus::Invalid);
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|item| item.code == "UCS_FRAME"),
        "{:?}",
        result.diagnostics()
    );
}

#[test]
fn version_and_structure_fail_separately() {
    let mut value = fixture();
    value["header"]["version"] = json!("0.2.0");
    assert_eq!(load(&value).status(), DrawingLoadStatus::UnsupportedVersion);
    value["header"]["version"] = json!("0.1.0");
    value["unexpected"] = json!(1);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
    let mut value = fixture();
    value["header"].as_object_mut().unwrap().remove("version");
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
}

#[test]
fn invalid_local_ids_and_appearance_are_rejected() {
    let mut value = fixture();
    value["layers"] = json!([
        {"id": 0, "name": "A", "visible": true, "frozen": false, "locked": false,
         "plottable": true, "frozenInNewViewports": false, "color": {"rgb": [255,255,255]},
         "opacity": 1.0, "linePatternId": 0, "lineWeight": 0.25},
        {"id": 0, "name": "B", "visible": true, "frozen": false, "locked": false,
         "plottable": true, "frozenInNewViewports": false, "color": {"rgb": [255,255,255]},
         "opacity": 1.0, "linePatternId": 0, "lineWeight": 0.25}
    ]);
    value["header"]["nextLayerId"] = json!(1);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);

    let mut value = fixture();
    value["scopes"][0]["kind"] = json!(1);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);

    let mut value = with_line();
    assert_eq!(
        load(&value).status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        load(&value).diagnostics()
    );
    value["streams"]["lineStream"]["colorMode"] = json!(["ByLayer"]);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
    let mut value = with_line();
    value["streams"]["lineStream"]
        .as_object_mut()
        .unwrap()
        .remove("colorMode");
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
}

#[test]
fn present_stream_columns_and_counts_are_checked() {
    let mut value = with_line();
    value["streams"]["lineStream"]["count"] = json!(2);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
    let mut value = with_line();
    value["streams"]["lineStream"]
        .as_object_mut()
        .unwrap()
        .remove("entityId");
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
}

#[test]
fn nested_unknown_core_fields_are_rejected() {
    let mut value = fixture();
    value["drawingViewState"] = json!({"madeUp": true});
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
    let mut value = with_line();
    value["streams"]["lineStream"]["madeUp"] = json!([42]);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
}

#[test]
fn order_scope_and_line_bounds_must_match_geometry() {
    let mut value = with_line();
    value["scopes"][0]["entities"][0] = json!(99);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);

    let mut value = with_line();
    value["scopes"][0]["bounds"]["maxX"] = json!(1.0);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);

    let mut value = with_line();
    value["scopes"][0]["bounds"]["minX"] = json!(100.0);
    let result = load(&value);
    assert_eq!(result.status(), DrawingLoadStatus::Invalid);
    assert!(result
        .diagnostics()
        .iter()
        .any(|item| { item.code == "SCOPE_BOUNDS" && item.location == "/scopes/0/bounds" }));

    let mut value = with_line();
    value["scopes"][0]["entities"] = json!([1, 1]);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
}

#[test]
fn local_id_spaces_are_separate_and_watermarks_are_checked() {
    let mut value = with_line();
    value["layouts"][0]["id"] = json!(1);
    value["header"]["nextLayoutId"] = json!(2);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Valid);
    value["header"]["nextLayoutId"] = json!(1);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
    let mut value = with_line();
    value["header"]["nextEntityId"] = json!(1);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
}

#[test]
fn block_scope_requires_a_local_definition() {
    let mut value = fixture();
    value["scopes"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":1,"kind":2,"bounds":null,"entities":[]}));
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
    value["blockDefinitions"] = json!([{"scopeId":1,"name":"Chair"}]);
    assert_eq!(
        load(&value).status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        load(&value).diagnostics()
    );
    value["blockDefinitions"][0]["scopeId"] = json!(0);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
}

#[test]
fn viewport_layer_override_cannot_be_orphaned_or_target_a_missing_layer() {
    let mut value = with_line();
    value["streams"]["viewportLayerOverrideStream"] =
        json!({"count":1,"layerId":[0],"frozen":[true]});
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
    value["streams"]["viewportLayerOverrideStream"]["layerId"][0] = json!(99);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
}

#[test]
fn polyline_pool_ranges_and_scope_bounds_are_checked() {
    use ocdraw::ocdraw::{
        DrawingBuilder, DrawingOptions, LayerDefinition, PlanarPolylineDefinition, RgbColor,
    };
    let mut builder =
        DrawingBuilder::new(DrawingOptions::new("polyline-validation", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(0, 0, 0),
            ocdraw::ocdraw::LinePatternId(0),
        ))
        .unwrap();
    builder
        .add_planar_polyline(PlanarPolylineDefinition::new(
            layer,
            vec![[0.0, 0.0, -0.5], [2.0, 0.0, 0.0]],
            false,
        ))
        .unwrap();
    let bytes = builder.finish().unwrap();
    let mut value: Value = serde_json::from_slice(bytes.bytes()).unwrap();
    assert_eq!(load(&value).status(), DrawingLoadStatus::Valid);
    value["streams"]["planarPolylineStream"]["vertexOffset"][0] = json!(1);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
    value["streams"]["planarPolylineStream"]["vertexOffset"][0] = json!(0);
    value["streams"]["planarPolylineStream"]["bulge"][1] = json!(0.5);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Valid);
    value["streams"]["planarPolylineStream"]["bulge"][1] = json!(0.0);
    value["scopes"][0]["bounds"]["minY"] = json!(0.0);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
}

#[test]
fn point_and_circle_geometry_is_validated_against_scope_bounds() {
    use ocdraw::ocdraw::{
        CircleDefinition, DrawingBuilder, DrawingOptions, LayerDefinition, PointDefinition,
        RgbColor,
    };
    let mut builder = DrawingBuilder::new(DrawingOptions::new("geometry", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(0, 0, 0),
            ocdraw::ocdraw::LinePatternId(0),
        ))
        .unwrap();
    builder
        .add_point(PointDefinition::new(layer, [4.0, 5.0, 0.0]))
        .unwrap();
    builder
        .add_circle(CircleDefinition::new(layer, [10.0, 0.0, 0.0], 2.0))
        .unwrap();
    let encoded = builder.finish().unwrap();
    let mut value: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    value["streams"]["circleStream"]["radius"][0] = json!(0);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
    let mut value: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    value["scopes"][0]["bounds"]["maxX"] = json!(11.0);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
    let mut value: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    value["streams"]["pointStream"]["placement"][0]["origin"]["x"] = json!(100.0);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
}

#[test]
fn block_instance_definition_reference_must_target_a_block_scope() {
    use ocdraw::ocdraw::{
        BlockInstanceDefinition, DrawingBuilder, DrawingOptions, LayerDefinition, RgbColor,
    };
    let mut builder = DrawingBuilder::new(DrawingOptions::new("blocks", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(0, 0, 0),
            ocdraw::ocdraw::LinePatternId(0),
        ))
        .unwrap();
    let block = builder.add_block_definition("B").unwrap();
    builder
        .add_block_instance(BlockInstanceDefinition::new(layer, block, [0.0, 0.0, 0.0]))
        .unwrap();
    let encoded = builder.finish().unwrap();
    let mut value: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    value["streams"]["blockInstanceStream"]["definitionScopeId"][0] = json!(0);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
}

#[test]
fn arc_sweep_and_bounds_are_validated() {
    use ocdraw::ocdraw::{
        ArcDefinition, DrawingBuilder, DrawingOptions, LayerDefinition, RgbColor,
    };
    let mut builder = DrawingBuilder::new(DrawingOptions::new("arc", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(0, 0, 0),
            ocdraw::ocdraw::LinePatternId(0),
        ))
        .unwrap();
    builder
        .add_arc(ArcDefinition::new(
            layer,
            [0.0, 0.0, 0.0],
            2.0,
            0.0,
            std::f64::consts::FRAC_PI_2,
        ))
        .unwrap();
    let encoded = builder.finish().unwrap();
    let mut value: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    value["streams"]["arcStream"]["sweepParameter"][0] = json!(0.0);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
    let mut value: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    value["scopes"][0]["bounds"]["maxY"] = json!(1.0);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
}

#[test]
fn ellipse_axes_and_radii_are_validated() {
    use ocdraw::ocdraw::{
        DrawingBuilder, DrawingOptions, EllipseDefinition, LayerDefinition, RgbColor,
    };
    let mut builder = DrawingBuilder::new(DrawingOptions::new("ellipse", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(0, 0, 0),
            ocdraw::ocdraw::LinePatternId(0),
        ))
        .unwrap();
    builder
        .add_ellipse(EllipseDefinition::new(
            layer,
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            3.0,
            2.0,
        ))
        .unwrap();
    let encoded = builder.finish().unwrap();
    let mut value: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    value["streams"]["ellipseStream"]["semiMinorRadius"][0] = json!(4.0);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
    let mut value: Value = serde_json::from_slice(encoded.bytes()).unwrap();
    value["streams"]["ellipseStream"]["placement"][0]["X"]["x"] = json!(0.0);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
}

#[test]
fn stream_names_and_mapping_suffice_without_a_document_directory() {
    for mut value in [fixture(), with_line()] {
        value.as_object_mut().unwrap().remove("streamDirectory");
        let read = load(&value);
        assert_eq!(
            read.status(),
            DrawingLoadStatus::Valid,
            "{:?}",
            read.diagnostics()
        );
    }
}

#[test]
fn absent_unused_streams_do_not_relax_validation_of_present_streams() {
    let valid = with_line();
    assert_eq!(valid["streams"].as_object().unwrap().len(), 1);
    assert_eq!(load(&valid).status(), DrawingLoadStatus::Valid);
    for (field, value, code) in [
        ("x2", json!([]), "COLUMN_COUNT"),
        ("layerId", json!([99]), "ENTITY_REF"),
    ] {
        let mut drawing = valid.clone();
        drawing["streams"]["lineStream"][field] = value;
        let read = load(&drawing);
        assert_eq!(read.status(), DrawingLoadStatus::Invalid);
        assert!(
            read.diagnostics().iter().any(|d| d.code == code),
            "{:?}",
            read.diagnostics()
        );
    }
    let mut obsolete = valid.clone();
    obsolete["streamDirectory"] = json!({"version":"ocdraw.streamDirectory.v1","streams":[]});
    assert!(load(&obsolete)
        .diagnostics()
        .iter()
        .any(|d| d.code == "SCHEMA"));
    let mut unknown = valid;
    unknown["streams"]["inventedStream"] = json!({"count":0});
    assert!(load(&unknown)
        .diagnostics()
        .iter()
        .any(|d| d.code == "SCHEMA"));
}

#[test]
fn view_and_workspace_cross_record_rules_survive_the_logical_model_migration() {
    let mut viewport = with_viewport();
    viewport["streams"]["viewportStream"]["view"][0]["direction"] = json!({"x":0.,"y":0.,"z":0.});
    assert_eq!(load(&viewport).status(), DrawingLoadStatus::Invalid);
    let mut clipping = with_viewport();
    clipping["streams"]["viewportStream"]["paperClip"][0] =
        json!({"enabled":false,"boundaryEntityId":99});
    assert_eq!(load(&clipping).status(), DrawingLoadStatus::Invalid);
    let mut grid = with_model_window();
    grid["modelWindows"][0]["grid"]["majorLineFrequency"] = json!(0);
    assert_eq!(load(&grid).status(), DrawingLoadStatus::Invalid);
    let mut conflict = with_model_window();
    conflict["modelWindows"][0]["useStoredUcs"] = json!(true);
    conflict["modelWindows"][0]["storedUcs"] = json!({"kind":"Unnamed","frame":{"origin":{"x":1.,"y":0.,"z":0.},"X":{"x":1.,"y":0.,"z":0.},"Y":{"x":0.,"y":1.,"z":0.}}});
    assert_eq!(load(&conflict).status(), DrawingLoadStatus::Invalid);
    let mut missing = with_model_window();
    missing.as_object_mut().unwrap().remove("drawingViewState");
    assert_eq!(load(&missing).status(), DrawingLoadStatus::Invalid);
}

#[test]
fn null_placement_rows_follow_the_registered_logical_default() {
    for (kind, fields) in [
        ("point", json!({})),
        ("circle", json!({"radius":[1.]})),
        (
            "arc",
            json!({"radius":[1.],"startParameter":[0.],"sweepParameter":[1.]}),
        ),
        (
            "ellipse",
            json!({"semiMajorRadius":[2.],"semiMinorRadius":[1.]}),
        ),
        (
            "ellipseArc",
            json!({"semiMajorRadius":[2.],"semiMinorRadius":[1.],"startParameter":[0.],"sweepParameter":[1.]}),
        ),
        (
            "planarPolyline",
            json!({"closed":[false],"vertexOffset":[0],"vertexCount":[2],"x":[0.,1.],"y":[0.,1.],"bulge":[0.,0.]}),
        ),
    ] {
        let mut value = with_line();
        value["scopes"][0]["bounds"] =
            json!({"minX":-10.,"minY":-10.,"minZ":-10.,"maxX":10.,"maxY":10.,"maxZ":10.});
        let mut stream = json!({"count":1,"entityId":[1],"layerId":[0],"placement":[null]});
        stream
            .as_object_mut()
            .unwrap()
            .extend(fields.as_object().unwrap().clone());
        value["streams"] = json!({format!("{kind}Stream"):stream});
        let read = load(&value);
        assert_eq!(
            read.status(),
            DrawingLoadStatus::Valid,
            "{kind}: {:?}",
            read.diagnostics()
        );
        let geometry = read.validated_drawing().unwrap().geometric_entities()[0].geometry();
        let placement = match geometry {
            DrawingGeometry::Point { placement }
            | DrawingGeometry::Circle { placement, .. }
            | DrawingGeometry::Arc { placement, .. }
            | DrawingGeometry::Ellipse { placement, .. }
            | DrawingGeometry::PlanarPolyline { placement, .. } => placement,
            _ => panic!("unexpected family"),
        };
        assert_eq!(*placement, ocdraw::ocdraw::CoordinateFrame3::default());
    }
}

#[test]
fn viewport_enclosure_does_not_round_away_a_small_frame_at_large_coordinates() {
    let mut value = with_viewport();
    let center = 1e16_f64;
    value["streams"]["viewportStream"]["frame"][0]["center"]["x"] = json!(center);
    value["streams"]["viewportStream"]["frame"][0]["width"] = json!(1.);
    value["scopes"][1]["bounds"]["minX"] = json!(center);
    value["scopes"][1]["bounds"]["maxX"] = json!(center);
    assert!(load(&value)
        .diagnostics()
        .iter()
        .any(|d| d.code == "SCOPE_BOUNDS"));
    value["scopes"][1]["bounds"]["minX"] = json!(center.next_down());
    value["scopes"][1]["bounds"]["maxX"] = json!(center.next_up());
    assert_eq!(
        load(&value).status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        load(&value).diagnostics()
    );
}
