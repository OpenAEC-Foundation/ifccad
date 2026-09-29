use ocdraw::drawing::{
    load_drawing_bytes, AppearanceSelection, DrawingBuilder, DrawingLoadStatus, DrawingOptions,
    EntityAppearance, LayerDefinition, LineDefinition, RgbColor,
};

#[test]
fn empty_and_populated_drawings_round_trip_through_the_production_reader() {
    let builder = DrawingBuilder::new(DrawingOptions::new("test-drawing", "unitless")).unwrap();
    let empty = builder.finish().unwrap();
    let read = load_drawing_bytes(empty.bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    assert!(read.validated_drawing().unwrap().layers().is_empty());

    let mut builder = DrawingBuilder::new(DrawingOptions::new("line-drawing", "mm")).unwrap();
    let layer_id = builder
        .add_layer(LayerDefinition::new("Red", RgbColor::new(255, 0, 0)))
        .unwrap();
    let mut line = LineDefinition::new(layer_id, [0.0, 0.0, 0.0], [2.0, 2.0, 0.0]);
    line.appearance = EntityAppearance {
        color: AppearanceSelection::ByLayer,
        opacity: AppearanceSelection::Explicit(0.5),
        line_pattern: AppearanceSelection::ByBlock,
        line_weight: AppearanceSelection::Explicit(0.35),
    };
    assert_eq!(builder.add_line(line).unwrap(), 1);
    let encoded = builder.finish().unwrap();
    let read = load_drawing_bytes(encoded.bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let drawing = read.validated_drawing().unwrap();
    assert_eq!(drawing.layers().len(), 1);
    assert_eq!(
        drawing.as_value()["streams"]["lineStream"]["opacityMode"][0],
        "Explicit"
    );
    assert_eq!(
        drawing.as_value()["streams"]["lineStream"]["linePatternMode"][0],
        "ByBlock"
    );
    assert_eq!(
        drawing.as_value()["streams"]["lineStream"]["lineWeight"][0],
        0.35
    );
}

#[test]
fn block_definition_and_instance_keep_distinct_owner_scopes() {
    use ocdraw::drawing::BlockInstanceDefinition;
    let mut builder = DrawingBuilder::new(DrawingOptions::new("blocks", "mm")).unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new("0", RgbColor::new(0, 0, 0)))
        .unwrap();
    let block_scope = builder.add_block_definition("Door").unwrap();
    let source = builder
        .add_line(
            LineDefinition::new(layer, [0.0, 0.0, 0.0], [2.0, 1.0, 0.0]).in_scope(block_scope),
        )
        .unwrap();
    let instance = builder
        .add_block_instance(BlockInstanceDefinition::new(
            layer,
            block_scope,
            [10.0, 20.0, 0.0],
        ))
        .unwrap();
    let encoded = builder.finish().unwrap();
    let read = load_drawing_bytes(encoded.bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let value = read.validated_drawing().unwrap().as_value();
    assert_eq!(value["blockDefinitions"][0]["scopeId"], block_scope);
    assert_eq!(value["streams"]["lineStream"]["scopeId"][0], block_scope);
    assert_eq!(value["streams"]["blockInstanceStream"]["scopeId"][0], 0);
    assert_eq!(
        value["streams"]["blockInstanceStream"]["definitionScopeId"][0],
        block_scope
    );
    assert_eq!(
        value["streams"]["entityOrderEntryStream"]["entityId"],
        serde_json::json!([instance, source])
    );
    assert_eq!(value["scopes"][0]["bounds"]["minX"], 10.0);
    assert_eq!(value["scopes"][0]["bounds"]["maxX"], 12.0);
}

#[test]
fn block_base_point_moves_definition_geometry_into_instance_scope() {
    use ocdraw::drawing::{BlockDefinition, BlockInstanceDefinition};
    let mut builder = DrawingBuilder::new(DrawingOptions::new("base", "mm")).unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new("0", RgbColor::new(0, 0, 0)))
        .unwrap();
    let mut definition = BlockDefinition::new("Offset");
    definition.base_point = [2.0, 0.0, 0.0];
    let scope = builder.add_block_definition(definition).unwrap();
    builder
        .add_line(LineDefinition::new(layer, [2.0, 0.0, 0.0], [4.0, 0.0, 0.0]).in_scope(scope))
        .unwrap();
    builder
        .add_block_instance(BlockInstanceDefinition::new(layer, scope, [10.0, 0.0, 0.0]))
        .unwrap();
    let encoded = builder.finish().unwrap();
    let value: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    assert_eq!(value["scopes"][0]["bounds"]["minX"], 10.0);
    assert_eq!(value["scopes"][0]["bounds"]["maxX"], 12.0);
}

#[test]
fn arc_is_written_with_scoped_bounds_and_strict_readback() {
    use ocdraw::drawing::ArcDefinition;
    let mut builder = DrawingBuilder::new(DrawingOptions::new("arc", "mm")).unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new("0", RgbColor::new(0, 0, 0)))
        .unwrap();
    let id = builder
        .add_arc(ArcDefinition::new(
            layer,
            [5.0, 5.0, 0.0],
            2.0,
            0.0,
            std::f64::consts::FRAC_PI_2,
        ))
        .unwrap();
    let encoded = builder.finish().unwrap();
    let read = load_drawing_bytes(encoded.bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let value = read.validated_drawing().unwrap().as_value();
    assert_eq!(value["streams"]["arcStream"]["entityId"][0], id);
    assert!(value["scopes"][0]["bounds"]["minX"].as_f64().unwrap() <= 5.0);
    assert!(value["scopes"][0]["bounds"]["maxY"].as_f64().unwrap() >= 7.0);
}

#[test]
fn full_and_partial_ellipses_have_typed_streams() {
    use ocdraw::drawing::EllipseDefinition;
    let mut builder = DrawingBuilder::new(DrawingOptions::new("ellipses", "mm")).unwrap();
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
    builder
        .add_ellipse_arc(
            EllipseDefinition::new(layer, [10.0, 0.0, 0.0], [0.0, 1.0, 0.0], 3.0, 1.0)
                .with_arc(0.0, std::f64::consts::PI),
        )
        .unwrap();
    let encoded = builder.finish().unwrap();
    let read = load_drawing_bytes(encoded.bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let value = read.validated_drawing().unwrap().as_value();
    assert_eq!(value["streams"]["ellipseStream"]["count"], 1);
    assert_eq!(value["streams"]["ellipseArcStream"]["count"], 1);
}

#[test]
fn planar_and_spatial_polylines_write_pooled_vertices() {
    use ocdraw::drawing::{PlanarPolylineDefinition, SpatialPolylineDefinition};
    let mut builder = DrawingBuilder::new(DrawingOptions::new("polylines", "mm")).unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new("0", RgbColor::new(0, 0, 0)))
        .unwrap();
    builder
        .add_planar_polyline(PlanarPolylineDefinition::new(
            layer,
            vec![[0.0, 0.0, 1.0], [2.0, 0.0, 0.0]],
            false,
        ))
        .unwrap();
    builder
        .add_spatial_polyline(SpatialPolylineDefinition::new(
            layer,
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
            false,
        ))
        .unwrap();
    let encoded = builder.finish().unwrap();
    let read = load_drawing_bytes(encoded.bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let value = read.validated_drawing().unwrap().as_value();
    assert_eq!(
        value["streams"]["planarPolylineStream"]["vertexCount"][0],
        2
    );
    assert_eq!(
        value["streams"]["planarPolylineStream"]["bulge"],
        serde_json::json!([1.0, 0.0])
    );
    assert_eq!(
        value["streams"]["spatialPolylineStream"]["z"],
        serde_json::json!([3.0, 6.0])
    );
}

#[test]
fn writer_and_reader_share_logical_definition_rules() {
    let mut builder = DrawingBuilder::new(DrawingOptions::new("logical", "mm")).unwrap();
    builder
        .add_layer(LayerDefinition::new("Walls", RgbColor::new(0, 0, 0)))
        .unwrap();
    builder
        .add_layer(LayerDefinition::new("walls", RgbColor::new(0, 0, 0)))
        .unwrap();
    assert!(builder.finish().is_err());

    let mut builder = DrawingBuilder::new(DrawingOptions::new("logical", "mm")).unwrap();
    builder
        .add_layer(LayerDefinition::new("Walls", RgbColor::new(0, 0, 0)))
        .unwrap();
    builder
        .add_layer(LayerDefinition::new("Roof", RgbColor::new(0, 0, 0)))
        .unwrap();
    let encoded = builder.finish().unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    value["layers"][1]["name"] = serde_json::json!("walls");
    let read = load_drawing_bytes(&serde_json::to_vec(&value).unwrap());
    assert_eq!(read.status(), DrawingLoadStatus::Invalid);
    assert!(read
        .diagnostics()
        .iter()
        .any(|item| item.code == "DUPLICATE_NAME"));
}

#[test]
fn point_display_is_drawing_local_and_roundtrips() {
    use ocdraw::drawing::{PointDisplay, PointGlyph, PointSize};
    let mut builder = DrawingBuilder::new(DrawingOptions::new("point-display", "mm")).unwrap();
    builder
        .set_point_display(PointDisplay {
            glyph: PointGlyph::Cross,
            circle: true,
            square: false,
            size: PointSize::ViewportPercent(5.0),
        })
        .unwrap();
    let encoded = builder.finish().unwrap();
    let read = load_drawing_bytes(encoded.bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let display = &read.validated_drawing().unwrap().as_value()["pointDisplay"];
    assert_eq!(display["form"]["glyph"], "cross");
    assert_eq!(display["size"]["kind"], "viewportPercent");
    assert_eq!(display["size"]["value"], 5.0);
}

#[test]
fn layout_limits_and_scaling_are_stored_in_the_layout() {
    use ocdraw::drawing::{LayoutRect, LayoutSettings};
    let mut builder = DrawingBuilder::new(DrawingOptions::new("layout-settings", "mm")).unwrap();
    let paper = builder.add_paper_layout("Sheet").unwrap();
    builder
        .set_layout_settings(
            paper,
            LayoutSettings {
                limits: Some(LayoutRect {
                    min_x: 1.0,
                    min_y: 2.0,
                    max_x: 100.0,
                    max_y: 200.0,
                }),
                limits_checking: true,
                paper_space_linetype_scaling: false,
                plot_settings: None,
            },
        )
        .unwrap();
    let encoded = builder.finish().unwrap();
    let read = load_drawing_bytes(encoded.bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let layout = &read.validated_drawing().unwrap().as_value()["layouts"][1];
    assert_eq!(layout["limits"]["maxX"], 100.0);
    assert_eq!(layout["limitsChecking"], true);
    assert_eq!(layout["paperSpaceLinetypeScaling"], false);
}

#[test]
fn layout_plot_settings_roundtrip_through_production_reader() {
    use ocdraw::drawing::{
        LayoutRect, LayoutSettings, PlotArea, PlotMapping, PlotMedia, PlotOptions, PlotOutput,
        PlotPlacement, PlotRotation, PlotScale, PlotSettings, PlotUnit, ShadedPlot, ShadedPlotMode,
        ShadedPlotQuality, ShadedPlotQualityMode,
    };
    let mut builder = DrawingBuilder::new(DrawingOptions::new("plot", "mm")).unwrap();
    let paper = builder.add_paper_layout("Sheet").unwrap();
    let settings = PlotSettings {
        media: PlotMedia {
            unit: PlotUnit::Millimetre,
            width: 210.0,
            height: 297.0,
            printable_area: LayoutRect {
                min_x: 5.0,
                min_y: 5.0,
                max_x: 205.0,
                max_y: 292.0,
            },
            rotation: PlotRotation::None,
            device_name: None,
            media_name: Some("A4".into()),
        },
        area: PlotArea::Layout,
        mapping: PlotMapping {
            scale: PlotScale::FitToArea,
            placement: PlotPlacement::Centered,
        },
        output: PlotOutput {
            shaded_plot: ShadedPlot {
                mode: ShadedPlotMode::AsDisplayed,
                quality: ShadedPlotQuality {
                    mode: ShadedPlotQualityMode::Normal,
                    dpi: None,
                },
            },
            apply_plot_styles: true,
            plot_style_table_name: None,
        },
        options: PlotOptions {
            plot_viewport_borders: false,
            plot_paper_space_last: false,
            hide_paper_space_objects: false,
            plot_line_weights: true,
            scale_line_weights: false,
            plot_transparency: false,
        },
    };
    builder
        .set_layout_settings(
            paper,
            LayoutSettings {
                plot_settings: Some(settings),
                ..LayoutSettings::default()
            },
        )
        .unwrap();
    let encoded = builder.finish().unwrap();
    let read = load_drawing_bytes(encoded.bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let plot = &read.validated_drawing().unwrap().as_value()["layouts"][1]["plotSettings"];
    assert_eq!(plot["media"]["mediaName"], "A4");
    assert_eq!(plot["mapping"]["scale"]["mode"], "FitToArea");
}

#[test]
fn unused_named_ucs_definition_is_drawing_local() {
    use ocdraw::drawing::{CoordinateFrame3, Point3, UcsDefinition, Vector3};
    let mut builder = DrawingBuilder::new(DrawingOptions::new("ucs", "mm")).unwrap();
    let frame = CoordinateFrame3::try_new(
        Point3::new(1.0, 2.0, 3.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
    )
    .unwrap();
    let id = builder
        .add_ucs_definition(UcsDefinition::new("Grid A", frame, 4.0))
        .unwrap();
    let encoded = builder.finish().unwrap();
    let read = load_drawing_bytes(encoded.bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    assert_eq!(
        read.validated_drawing().unwrap().as_value()["ucsDefinitions"][0]["ucsId"],
        id
    );
    assert_eq!(
        read.validated_drawing().unwrap().as_value()["ucsDefinitions"][0]["frame"]["origin"]["z"],
        3.0
    );
}

#[test]
fn paper_layout_and_drawing_workspace_are_local() {
    let mut builder = DrawingBuilder::new(DrawingOptions::new("paper-drawing", "mm")).unwrap();
    let unused_layer = builder
        .add_layer(LayerDefinition::new("Unused", RgbColor::new(0, 0, 0)))
        .unwrap();
    let paper = builder.add_paper_layout("A1").unwrap();
    builder.set_current_layer(unused_layer);
    builder.set_active_layout(paper);
    builder
        .add_line(
            LineDefinition::new(unused_layer, [1.0, 2.0, 0.0], [3.0, 4.0, 0.0]).in_scope(paper),
        )
        .unwrap();
    let drawing = builder.finish().unwrap();
    let read = load_drawing_bytes(drawing.bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let value = read.validated_drawing().unwrap().as_value();
    assert_eq!(value["layouts"].as_array().unwrap().len(), 2);
    assert_eq!(value["layouts"][1]["kind"], "paper");
    assert_eq!(
        value["drawingWorkspaceState"]["currentLayerId"],
        unused_layer
    );
    assert_eq!(value["drawingWorkspaceState"]["activeLayoutId"], paper);
    assert_eq!(value["streams"]["lineStream"]["scopeId"][0], paper);
    assert_eq!(value["scopes"][1]["bounds"]["maxX"], 3.0);
}

#[test]
fn authored_color_metadata_survives_readback() {
    use ocdraw::drawing::DrawingColor;
    let mut builder = DrawingBuilder::new(DrawingOptions::new("indexed", "mm")).unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new(
            "A",
            DrawingColor::rgb(255, 0, 0).with_indexed("ACI", 1),
        ))
        .unwrap();
    let mut line = LineDefinition::new(layer, [0.0, 0.0, 0.0], [1.0, 1.0, 0.0]);
    line.appearance.color =
        AppearanceSelection::Explicit(DrawingColor::rgb(0, 0, 255).with_named("Catalog", "Blue"));
    builder.add_line(line).unwrap();
    let encoded = builder.finish().unwrap();
    let read = load_drawing_bytes(encoded.bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let value = read.validated_drawing().unwrap().as_value();
    assert_eq!(value["layers"][0]["color"]["indexedColor"]["index"], 1);
    assert_eq!(
        value["streams"]["lineStream"]["color"][0]["namedColor"]["name"],
        "Blue"
    );
}

#[test]
fn writing_a_drawing_refuses_to_replace_an_existing_file() {
    let drawing = DrawingBuilder::new(DrawingOptions::new("file", "unitless"))
        .unwrap()
        .finish()
        .unwrap();
    let path = std::env::temp_dir().join(format!(
        "ocdraw-write-{}-{}.ocdraw.json",
        std::process::id(),
        std::thread::current().name().unwrap_or("writer")
    ));
    let _ = std::fs::remove_file(&path);
    drawing.write_file(&path).unwrap();
    assert!(drawing.write_file(&path).is_err());
    let opened = ocdraw::drawing::load_drawing_file(&path).unwrap();
    assert_eq!(opened.status(), DrawingLoadStatus::Valid);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn named_plot_style_mode_is_explicit() {
    let mut builder = DrawingBuilder::new(DrawingOptions::new("plot", "unitless")).unwrap();
    builder.set_plot_style_mode(ocdraw::drawing::PlotStyleMode::Named);
    let encoded = builder.finish().unwrap();
    let read = load_drawing_bytes(encoded.bytes());
    assert_eq!(read.status(), DrawingLoadStatus::Valid);
    assert_eq!(read.validated_drawing().unwrap().plot_style_mode(), "named");
}

#[test]
fn point_and_circle_share_entity_ids_and_draw_order_with_lines() {
    use ocdraw::drawing::{CircleDefinition, PointDefinition};
    let mut builder = DrawingBuilder::new(DrawingOptions::new("mixed", "mm")).unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new("0", RgbColor::new(255, 255, 255)))
        .unwrap();
    assert_eq!(
        builder
            .add_point(PointDefinition::new(layer, [0.0, 0.0, 0.0]))
            .unwrap(),
        1
    );
    assert_eq!(
        builder
            .add_line(LineDefinition::new(layer, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]))
            .unwrap(),
        2
    );
    assert_eq!(
        builder
            .add_circle(CircleDefinition::new(layer, [5.0, 0.0, 0.0], 2.0))
            .unwrap(),
        3
    );
    let encoded = builder.finish().unwrap();
    let read = load_drawing_bytes(encoded.bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let value = read.validated_drawing().unwrap().as_value();
    assert_eq!(
        value["streams"]["entityOrderEntryStream"]["entityId"],
        serde_json::json!([1, 2, 3])
    );
    assert!(value["scopes"][0]["bounds"]["maxX"].as_f64().unwrap() >= 7.0);
}
