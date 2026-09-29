use ocdraw::drawing::{load_drawing_bytes, DrawingLoadStatus};
use serde_json::{json, Value};

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../conformance/next/ocdraw/valid/empty.ocdraw.json"
    ))
    .unwrap()
}

fn load(value: &Value) -> ocdraw::drawing::DrawingLoadOutcome {
    load_drawing_bytes(&serde_json::to_vec(value).unwrap())
}

fn with_line() -> Value {
    let mut value = fixture();
    value["header"]["nextEntityId"] = json!(2);
    value["header"]["nextLayerId"] = json!(1);
    value["layers"] = json!([{
        "id": 0, "name": "0", "visible": true, "frozen": false, "locked": false,
        "plottable": true, "frozenInNewViewports": false,
        "color": {"rgb": [255, 255, 255]}, "opacity": 1.0,
        "linePattern": "Continuous", "lineWeight": 0.25
    }]);
    value["scopes"][0]["bounds"] = json!({
        "minX": 0.0, "minY": 0.0, "minZ": 0.0,
        "maxX": 2.0, "maxY": 2.0, "maxZ": 0.0
    });
    value["streamDirectory"]["streams"] = json!([{
        "name": "line", "schema": "ocdraw.line.v3", "role": "object", "count": 1,
        "columns": ["entityId", "scopeId", "layerId", "x1", "y1", "x2", "y2", "colorMode", "color"]
    }, {
        "name": "entityOrder", "schema": "ocdraw.entityOrder.v1", "role": "order", "count": 1,
        "columns": ["scopeId", "entryOffset", "entryCount"], "children": ["entityOrderEntry"]
    }, {
        "name": "entityOrderEntry", "schema": "ocdraw.entityOrderEntry.v1", "role": "child", "count": 1,
        "columns": ["entityId"], "parent": "entityOrder"
    }]);
    value["streams"] = json!({"lineStream": {
        "count": 1, "entityId": [1], "scopeId": [0], "layerId": [0],
        "x1": [0.0], "y1": [0.0], "x2": [2.0], "y2": [2.0],
        "colorMode": ["Explicit"], "color": [{"rgb": [255, 0, 0]}]
    }, "entityOrderStream": {"count": 1, "scopeId": [0], "entryOffset": [0], "entryCount": [1]},
       "entityOrderEntryStream": {"count": 1, "entityId": [1]}});
    value
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
    assert_eq!(drawing.layers().len(), 0);
    assert_eq!(drawing.layouts().len(), 1);
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
         "opacity": 1.0, "linePattern": "Continuous", "lineWeight": 0.25},
        {"id": 0, "name": "B", "visible": true, "frozen": false, "locked": false,
         "plottable": true, "frozenInNewViewports": false, "color": {"rgb": [255,255,255]},
         "opacity": 1.0, "linePattern": "Continuous", "lineWeight": 0.25}
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
    value["streamDirectory"]["streams"][0]["columns"] =
        json!(["entityId", "scopeId", "layerId", "x1", "y1", "x2", "y2", "color"]);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
}

#[test]
fn stream_directory_and_required_columns_are_checked() {
    let mut value = with_line();
    value["streamDirectory"]["streams"][0]["count"] = json!(2);
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
    value["streams"]["entityOrderStream"]["scopeId"][0] = json!(99);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);

    let mut value = with_line();
    value["scopes"][0]["bounds"]["maxX"] = json!(1.0);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);

    let mut value = with_line();
    value["streams"]["entityOrderStream"]["entryCount"][0] = json!(2);
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
        .push(json!({"id":1,"kind":2,"bounds":null}));
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
    value["streamDirectory"]["streams"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "name":"viewportLayerOverride", "schema":"ocdraw.viewportLayerOverride.v1",
            "role":"child", "count":1, "columns":["layerId","frozen"], "parent":"viewport"
        }));
    value["streams"]["viewportLayerOverrideStream"] =
        json!({"count":1,"layerId":[0],"frozen":[true]});
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
    value["streams"]["viewportLayerOverrideStream"]["layerId"][0] = json!(99);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
}

#[test]
fn polyline_pool_ranges_and_scope_bounds_are_checked() {
    use ocdraw::drawing::{
        DrawingBuilder, DrawingOptions, LayerDefinition, PlanarPolylineDefinition, RgbColor,
    };
    let mut builder =
        DrawingBuilder::new(DrawingOptions::new("polyline-validation", "mm")).unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new("0", RgbColor::new(0, 0, 0)))
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
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
    value["streams"]["planarPolylineStream"]["bulge"][1] = json!(0.0);
    value["scopes"][0]["bounds"]["minY"] = json!(0.0);
    assert_eq!(load(&value).status(), DrawingLoadStatus::Invalid);
}

#[test]
fn point_and_circle_geometry_is_validated_against_scope_bounds() {
    use ocdraw::drawing::{
        CircleDefinition, DrawingBuilder, DrawingOptions, LayerDefinition, PointDefinition,
        RgbColor,
    };
    let mut builder = DrawingBuilder::new(DrawingOptions::new("geometry", "mm")).unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new("0", RgbColor::new(0, 0, 0)))
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
    use ocdraw::drawing::{
        BlockInstanceDefinition, DrawingBuilder, DrawingOptions, LayerDefinition, RgbColor,
    };
    let mut builder = DrawingBuilder::new(DrawingOptions::new("blocks", "mm")).unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new("0", RgbColor::new(0, 0, 0)))
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
    use ocdraw::drawing::{
        ArcDefinition, DrawingBuilder, DrawingOptions, LayerDefinition, RgbColor,
    };
    let mut builder = DrawingBuilder::new(DrawingOptions::new("arc", "mm")).unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new("0", RgbColor::new(0, 0, 0)))
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
    use ocdraw::drawing::{
        DrawingBuilder, DrawingOptions, EllipseDefinition, LayerDefinition, RgbColor,
    };
    let mut builder = DrawingBuilder::new(DrawingOptions::new("ellipse", "mm")).unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new("0", RgbColor::new(0, 0, 0)))
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
