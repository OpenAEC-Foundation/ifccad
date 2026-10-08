use ocdraw::ocdraw::{
    load_ocdraw_bytes, AppearanceSelection, EntityAppearance, LayerDefinition, LineDefinition,
    OcdrawBuildOptions, OcdrawBuilder, OcdrawReadStatus, RgbColor,
};

#[test]
fn general_geometry_writer_preserves_placed_polyline_and_signed_block_transform() {
    use ocdraw::ocdraw::{
        BlockDefinition, BlockTransform, CoordinateFrame3, DrawingGeometry,
        GeometricEntityDefinition, Point3, Scale3, Vector3,
    };
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("placed", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(255, 255, 255),
            ocdraw::ocdraw::LinePatternId(0),
        ))
        .unwrap();
    let block = builder
        .add_block_definition(BlockDefinition::new("Shared"))
        .unwrap();
    let frame = CoordinateFrame3::try_new(
        Point3::new(10.0, 20.0, 30.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
    )
    .unwrap();
    let line = DrawingGeometry::PlanarPolyline {
        line_pattern_generation: ocdraw::ocdraw::LinePatternGeneration::PerSegment,

        placement: frame,
        vertices: vec![[0.0, 0.0, 0.25], [2.0, 3.0, 0.0]],
        closed: false,
    };
    builder
        .add_geometric_entity(GeometricEntityDefinition::new(block, layer, line.clone()))
        .unwrap();
    let transform = BlockTransform::try_new(
        CoordinateFrame3::default(),
        0.5,
        Scale3::new(-2.0, 3.0, 1.0),
    )
    .unwrap();
    builder
        .add_geometric_entity(GeometricEntityDefinition::new(
            0,
            layer,
            DrawingGeometry::BlockInstance {
                definition_scope_id: block,
                transform,
            },
        ))
        .unwrap();
    let encoded = builder.finish().unwrap();
    let read = load_ocdraw_bytes(encoded.bytes());
    assert_eq!(
        read.as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Valid,
        "{:?}",
        read.as_ref()
            .err()
            .map(|e| e.diagnostics())
            .unwrap_or_default()
    );
    let entities = read.as_ref().ok().unwrap().geometric_entities();
    assert!(entities.iter().any(|entity| entity.geometry() == &line));
    assert!(entities.iter().any(|entity| matches!(entity.geometry(), DrawingGeometry::BlockInstance {transform: stored, ..} if *stored == transform)));
}

#[test]
fn empty_and_populated_drawings_round_trip_through_the_production_reader() {
    let builder = OcdrawBuilder::new(OcdrawBuildOptions::new("test-drawing", "unitless")).unwrap();
    let empty = builder.finish().unwrap();
    let read = load_ocdraw_bytes(empty.bytes());
    assert_eq!(
        read.as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Valid,
        "{:?}",
        read.as_ref()
            .err()
            .map(|e| e.diagnostics())
            .unwrap_or_default()
    );
    assert!(read.as_ref().ok().unwrap().typed_layers().is_empty());

    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("line-drawing", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    let layer_id = builder
        .add_layer(LayerDefinition::new(
            "Red",
            RgbColor::new(255, 0, 0),
            ocdraw::ocdraw::LinePatternId(0),
        ))
        .unwrap();
    let mut line = LineDefinition::new(layer_id, [0.0, 0.0, 0.0], [2.0, 2.0, 0.0]);
    line.appearance = EntityAppearance {
        line_pattern_scale: 1.0,

        color: AppearanceSelection::ByLayer,
        opacity: AppearanceSelection::Explicit(0.5),
        line_pattern: AppearanceSelection::ByBlock,
        line_weight: AppearanceSelection::Explicit(0.35),
    };
    assert_eq!(builder.add_line(line).unwrap(), 1);
    let encoded = builder.finish().unwrap();
    let encoded_value: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    assert!(encoded_value.get("streamDirectory").is_none());
    assert!(encoded_value["streams"].get("lineStream").is_some());
    let read = load_ocdraw_bytes(encoded.bytes());
    assert_eq!(
        read.as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Valid,
        "{:?}",
        read.as_ref()
            .err()
            .map(|e| e.diagnostics())
            .unwrap_or_default()
    );
    let drawing = read.as_ref().ok().unwrap();
    assert_eq!(drawing.typed_layers().len(), 1);
    assert_eq!(drawing.typed_layouts()[0].name, "Model");
    assert_eq!(drawing.typed_layers()[0].name, "Red");
    assert_eq!(drawing.typed_layers()[0].color.rgb, [255, 0, 0]);
    assert!(matches!(
        drawing.geometric_entities()[0].appearance().opacity,
        AppearanceSelection::Explicit(value) if value == 0.5
    ));
    assert!(matches!(
        drawing.geometric_entities()[0].appearance().line_pattern,
        AppearanceSelection::ByBlock
    ));
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
    use ocdraw::ocdraw::BlockInstanceDefinition;
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("blocks", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(0, 0, 0),
            ocdraw::ocdraw::LinePatternId(0),
        ))
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
    let read = load_ocdraw_bytes(encoded.bytes());
    assert_eq!(
        read.as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Valid,
        "{:?}",
        read.as_ref()
            .err()
            .map(|e| e.diagnostics())
            .unwrap_or_default()
    );
    let value = read.as_ref().ok().unwrap().as_value();
    assert_eq!(value["blockDefinitions"][0]["scopeId"], block_scope);
    let definition = &read.as_ref().ok().unwrap().block_definitions()[0];
    assert_eq!(definition.scope_id, block_scope);
    assert_eq!(definition.name, "Door");
    let scopes = read.as_ref().ok().unwrap().scopes();
    assert_eq!(scopes[0].entities, vec![instance]);
    assert_eq!(scopes[1].entities, vec![source]);
    assert_eq!(scopes[0].kind, ocdraw::ocdraw::DrawingScopeKind::Model);
    assert_eq!(scopes[1].kind, ocdraw::ocdraw::DrawingScopeKind::Block);
    assert!(value["streams"]["lineStream"].get("scopeId").is_none());
    assert!(value["streams"]["blockInstanceStream"]
        .get("scopeId")
        .is_none());
    assert_eq!(
        value["streams"]["blockInstanceStream"]["definitionScopeId"][0],
        block_scope
    );
    assert_eq!(value["scopes"][1]["entities"], serde_json::json!([source]));
    assert!(value["scopes"][0]["bounds"]["minX"].as_f64().unwrap() <= 10.0);
    assert!(value["scopes"][0]["bounds"]["maxX"].as_f64().unwrap() >= 12.0);
}

#[test]
fn block_base_point_moves_definition_geometry_into_instance_scope() {
    use ocdraw::ocdraw::{BlockDefinition, BlockInstanceDefinition};
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("base", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(0, 0, 0),
            ocdraw::ocdraw::LinePatternId(0),
        ))
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
    assert!(value["scopes"][0]["bounds"]["minX"].as_f64().unwrap() <= 10.0);
    assert!(value["scopes"][0]["bounds"]["maxX"].as_f64().unwrap() >= 12.0);
}

#[test]
fn arc_is_written_with_scoped_bounds_and_strict_readback() {
    use ocdraw::ocdraw::ArcDefinition;
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("arc", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(0, 0, 0),
            ocdraw::ocdraw::LinePatternId(0),
        ))
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
    let read = load_ocdraw_bytes(encoded.bytes());
    assert_eq!(
        read.as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Valid,
        "{:?}",
        read.as_ref()
            .err()
            .map(|e| e.diagnostics())
            .unwrap_or_default()
    );
    let value = read.as_ref().ok().unwrap().as_value();
    assert_eq!(value["streams"]["arcStream"]["entityId"][0], id);
    assert!(value["scopes"][0]["bounds"]["minX"].as_f64().unwrap() <= 5.0);
    assert!(value["scopes"][0]["bounds"]["maxY"].as_f64().unwrap() >= 7.0);
}

#[test]
fn full_and_partial_ellipses_have_typed_streams() {
    use ocdraw::ocdraw::EllipseDefinition;
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("ellipses", "mm")).unwrap();
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
    builder
        .add_ellipse_arc(
            EllipseDefinition::new(layer, [10.0, 0.0, 0.0], [0.0, 1.0, 0.0], 3.0, 1.0)
                .with_arc(0.0, std::f64::consts::PI),
        )
        .unwrap();
    let encoded = builder.finish().unwrap();
    let read = load_ocdraw_bytes(encoded.bytes());
    assert_eq!(
        read.as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Valid,
        "{:?}",
        read.as_ref()
            .err()
            .map(|e| e.diagnostics())
            .unwrap_or_default()
    );
    let value = read.as_ref().ok().unwrap().as_value();
    assert_eq!(value["streams"]["ellipseStream"]["count"], 1);
    assert_eq!(value["streams"]["ellipseArcStream"]["count"], 1);
}

#[test]
fn planar_and_spatial_polylines_write_pooled_vertices() {
    use ocdraw::ocdraw::{PlanarPolylineDefinition, SpatialPolylineDefinition};
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("polylines", "mm")).unwrap();
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
    let read = load_ocdraw_bytes(encoded.bytes());
    assert_eq!(
        read.as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Valid,
        "{:?}",
        read.as_ref()
            .err()
            .map(|e| e.diagnostics())
            .unwrap_or_default()
    );
    let value = read.as_ref().ok().unwrap().as_value();
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
    assert!(value["streams"]["spatialPolylineStream"]
        .get("placement")
        .is_none());
    assert!(read
        .as_ref()
        .ok()
        .unwrap()
        .geometric_entities()
        .iter()
        .any(|entity| matches!(
            entity.geometry(),
            ocdraw::ocdraw::DrawingGeometry::SpatialPolyline { vertices, closed: false , ..}
                if vertices == &vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]
        )));
    let mut invalid = value.clone();
    invalid["streams"]["spatialPolylineStream"]["placement"] = serde_json::json!([{}]);
    let rejected = load_ocdraw_bytes(&serde_json::to_vec(&invalid).unwrap());
    assert_eq!(
        rejected
            .as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Invalid
    );
    assert!(rejected
        .as_ref()
        .err()
        .map(|e| e.diagnostics())
        .unwrap_or_default()
        .iter()
        .any(|item| item.code == "SCHEMA"));
}

#[test]
fn writer_and_reader_share_logical_definition_rules() {
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("logical", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    builder
        .add_layer(LayerDefinition::new(
            "Walls",
            RgbColor::new(0, 0, 0),
            ocdraw::ocdraw::LinePatternId(0),
        ))
        .unwrap();
    builder
        .add_layer(LayerDefinition::new(
            "walls",
            RgbColor::new(0, 0, 0),
            ocdraw::ocdraw::LinePatternId(0),
        ))
        .unwrap();
    assert!(builder.finish().is_err());

    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("logical", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    builder
        .add_layer(LayerDefinition::new(
            "Walls",
            RgbColor::new(0, 0, 0),
            ocdraw::ocdraw::LinePatternId(0),
        ))
        .unwrap();
    builder
        .add_layer(LayerDefinition::new(
            "Roof",
            RgbColor::new(0, 0, 0),
            ocdraw::ocdraw::LinePatternId(0),
        ))
        .unwrap();
    let encoded = builder.finish().unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    value["layers"][1]["name"] = serde_json::json!("walls");
    let read = load_ocdraw_bytes(&serde_json::to_vec(&value).unwrap());
    assert_eq!(
        read.as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Invalid
    );
    assert!(read
        .as_ref()
        .err()
        .map(|e| e.diagnostics())
        .unwrap_or_default()
        .iter()
        .any(|item| item.code == "DUPLICATE_NAME"));
}

#[test]
fn point_display_is_drawing_local_and_roundtrips() {
    use ocdraw::ocdraw::{PointDisplay, PointGlyph, PointSize};
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("point-display", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    builder
        .set_point_display(PointDisplay {
            glyph: PointGlyph::Cross,
            circle: true,
            square: false,
            size: PointSize::ViewportPercent(5.0),
        })
        .unwrap();
    let encoded = builder.finish().unwrap();
    let read = load_ocdraw_bytes(encoded.bytes());
    assert_eq!(
        read.as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Valid,
        "{:?}",
        read.as_ref()
            .err()
            .map(|e| e.diagnostics())
            .unwrap_or_default()
    );
    let display = &read.as_ref().ok().unwrap().as_value()["pointDisplay"];
    assert_eq!(display["form"]["glyph"], "cross");
    assert_eq!(display["size"]["kind"], "viewportPercent");
    assert_eq!(display["size"]["value"], 5.0);
    assert_eq!(
        read.as_ref().ok().unwrap().point_display(),
        Some(PointDisplay {
            glyph: PointGlyph::Cross,
            circle: true,
            square: false,
            size: PointSize::ViewportPercent(5.0),
        })
    );
}

#[test]
fn layout_limits_and_scaling_are_stored_in_the_layout() {
    use ocdraw::ocdraw::{LayoutRect, LayoutSettings};
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("layout-settings", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    let paper = builder.add_paper_layout("Sheet").unwrap();
    builder
        .set_layout_settings(
            paper,
            LayoutSettings {
                media: None,
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
    let read = load_ocdraw_bytes(encoded.bytes());
    assert_eq!(
        read.as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Valid,
        "{:?}",
        read.as_ref()
            .err()
            .map(|e| e.diagnostics())
            .unwrap_or_default()
    );
    let layout = &read.as_ref().ok().unwrap().as_value()["layouts"][1];
    assert_eq!(layout["limits"]["maxX"], 100.0);
    assert_eq!(layout["limitsChecking"], true);
    assert_eq!(layout["paperSpaceLinetypeScaling"], false);
}

#[test]
fn layout_plot_settings_roundtrip_through_production_reader() {
    use ocdraw::ocdraw::{
        LayoutMedia, LayoutRect, LayoutSettings, MediaUnit, PlotArea, PlotMapping, PlotOptions,
        PlotOutput, PlotPage, PlotPlacement, PlotRotation, PlotScale, PlotSettings, PlotUnit,
        ShadedPlot, ShadedPlotMode, ShadedPlotQuality, ShadedPlotQualityMode,
    };
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("plot", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    let paper = builder.add_paper_layout("Sheet").unwrap();
    let settings = PlotSettings {
        plot_unit: PlotUnit::Millimetre,
        page: PlotPage {
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
        area: PlotArea::Extents,
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
                media: Some(LayoutMedia {
                    unit: MediaUnit::Physical(
                        ocdraw::geometry_kernel::CoordinateLengthUnit::Millimetre,
                    ),
                    width: 210.,
                    height: 297.,
                }),
                plot_settings: Some(settings),
                ..LayoutSettings::default()
            },
        )
        .unwrap();
    let encoded = builder.finish().unwrap();
    let read = load_ocdraw_bytes(encoded.bytes());
    assert_eq!(
        read.as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Valid,
        "{:?}",
        read.as_ref()
            .err()
            .map(|e| e.diagnostics())
            .unwrap_or_default()
    );
    let plot = &read.as_ref().ok().unwrap().as_value()["layouts"][1]["plotSettings"];
    let typed_plot = read.as_ref().ok().unwrap().typed_layouts()[1]
        .settings
        .plot_settings
        .as_ref()
        .unwrap();
    assert_eq!(typed_plot.page.media_name.as_deref(), Some("A4"));
    assert!(matches!(typed_plot.mapping.scale, PlotScale::FitToArea));
    assert_eq!(plot["page"]["mediaName"], "A4");
    assert_eq!(plot["mapping"]["scale"]["mode"], "FitToArea");
}

#[test]
fn unused_named_ucs_definition_is_drawing_local() {
    use ocdraw::ocdraw::{CoordinateFrame3, Point3, UcsDefinition, Vector3};
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("ucs", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
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
    let read = load_ocdraw_bytes(encoded.bytes());
    assert_eq!(
        read.as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Valid,
        "{:?}",
        read.as_ref()
            .err()
            .map(|e| e.diagnostics())
            .unwrap_or_default()
    );
    assert_eq!(
        read.as_ref().ok().unwrap().as_value()["ucsDefinitions"][0]["ucsId"],
        id
    );
    assert_eq!(
        read.as_ref().ok().unwrap().as_value()["ucsDefinitions"][0]["frame"]["origin"]["z"],
        3.0
    );
    let saved = &read.as_ref().ok().unwrap().ucs_definitions()[0];
    assert_eq!(saved.id, id);
    assert_eq!(saved.definition, UcsDefinition::new("Grid A", frame, 4.0));
}

#[test]
fn paper_layout_and_drawing_workspace_are_local() {
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("paper-drawing", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    let unused_layer = builder
        .add_layer(LayerDefinition::new(
            "Unused",
            RgbColor::new(0, 0, 0),
            ocdraw::ocdraw::LinePatternId(0),
        ))
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
    let read = load_ocdraw_bytes(drawing.bytes());
    assert_eq!(
        read.as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Valid,
        "{:?}",
        read.as_ref()
            .err()
            .map(|e| e.diagnostics())
            .unwrap_or_default()
    );
    let value = read.as_ref().ok().unwrap().as_value();
    assert_eq!(value["layouts"].as_array().unwrap().len(), 2);
    assert_eq!(value["layouts"][1]["kind"], "paper");
    assert_eq!(
        value["drawingWorkspaceState"]["currentLayerId"],
        unused_layer
    );
    assert_eq!(value["drawingWorkspaceState"]["activeLayoutId"], paper);
    let workspace = read.as_ref().ok().unwrap().workspace_state().unwrap();
    assert_eq!(workspace.current_layer_id, Some(unused_layer));
    assert_eq!(workspace.active_layout_id, Some(paper));
    assert_eq!(read.as_ref().ok().unwrap().owner_scope_id(1), Some(paper));
    assert_eq!(value["scopes"][1]["bounds"]["maxX"], 3.0);
}

#[test]
fn authored_color_metadata_survives_readback() {
    use ocdraw::ocdraw::DrawingColor;
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("indexed", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new(
            "A",
            DrawingColor::rgb(255, 0, 0).with_indexed("ACI", 1),
            ocdraw::ocdraw::LinePatternId(0),
        ))
        .unwrap();
    let mut line = LineDefinition::new(layer, [0.0, 0.0, 0.0], [1.0, 1.0, 0.0]);
    line.appearance.color =
        AppearanceSelection::Explicit(DrawingColor::rgb(0, 0, 255).with_named("Catalog", "Blue"));
    builder.add_line(line).unwrap();
    let encoded = builder.finish().unwrap();
    let read = load_ocdraw_bytes(encoded.bytes());
    assert_eq!(
        read.as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Valid,
        "{:?}",
        read.as_ref()
            .err()
            .map(|e| e.diagnostics())
            .unwrap_or_default()
    );
    let value = read.as_ref().ok().unwrap().as_value();
    assert_eq!(value["layers"][0]["color"]["indexedColor"]["index"], 1);
    assert_eq!(
        value["streams"]["lineStream"]["color"][0]["namedColor"]["name"],
        "Blue"
    );
}

#[test]
fn writing_a_drawing_refuses_to_replace_an_existing_file() {
    let drawing = OcdrawBuilder::new(OcdrawBuildOptions::new("file", "unitless"))
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
    let opened = ocdraw::ocdraw::load_ocdraw_file(&path).unwrap();
    assert_eq!(
        opened.drawing_id(),
        load_ocdraw_bytes(drawing.bytes()).unwrap().drawing_id()
    );
    std::fs::remove_file(path).unwrap();
}

#[test]
fn named_plot_style_mode_is_explicit() {
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("plot", "unitless")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    builder.set_plot_style_mode(ocdraw::ocdraw::PlotStyleMode::Named);
    let encoded = builder.finish().unwrap();
    let read = load_ocdraw_bytes(encoded.bytes());
    assert_eq!(
        read.as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Valid
    );
    assert_eq!(read.as_ref().ok().unwrap().plot_style_mode(), "named");
    assert_eq!(
        read.as_ref().ok().unwrap().typed_plot_style_mode(),
        ocdraw::ocdraw::PlotStyleMode::Named
    );
    assert_eq!(read.as_ref().ok().unwrap().unit(), "unitless");
}

#[test]
fn point_and_circle_share_entity_ids_and_draw_order_with_lines() {
    use ocdraw::ocdraw::{CircleDefinition, DrawingGeometry, PointDefinition};
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("mixed", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(255, 255, 255),
            ocdraw::ocdraw::LinePatternId(0),
        ))
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
    let read = load_ocdraw_bytes(encoded.bytes());
    assert_eq!(
        read.as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Valid,
        "{:?}",
        read.as_ref()
            .err()
            .map(|e| e.diagnostics())
            .unwrap_or_default()
    );
    let typed = read.as_ref().ok().unwrap().geometric_entities();
    assert_eq!(
        typed.iter().map(|entity| entity.id()).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert!(matches!(typed[0].geometry(), DrawingGeometry::Point { .. }));
    assert!(matches!(typed[1].geometry(), DrawingGeometry::Line { .. }));
    assert!(matches!(
        typed[2].geometry(),
        DrawingGeometry::Circle { .. }
    ));
    let value = read.as_ref().ok().unwrap().as_value();
    assert_eq!(value["scopes"][0]["entities"], serde_json::json!([1, 2, 3]));
    assert!(value["scopes"][0]["bounds"]["maxX"].as_f64().unwrap() >= 7.0);
}

#[test]
fn transformed_block_bounds_enclose_occurrences_and_reject_false_parent_bounds() {
    use ocdraw::ocdraw::{
        BlockDefinition, BlockTransform, CoordinateFrame3, DrawingGeometry,
        GeometricEntityDefinition, Scale3,
    };
    let mut drawing = OcdrawBuilder::new(OcdrawBuildOptions::new("bounds", "mm")).unwrap();
    drawing.ensure_continuous_line_pattern().unwrap();
    let layer = drawing
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(255, 255, 255),
            ocdraw::ocdraw::LinePatternId(0),
        ))
        .unwrap();
    let block = drawing
        .add_block_definition(BlockDefinition::new("Shared"))
        .unwrap();
    drawing
        .add_line(LineDefinition::new(layer, [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]).in_scope(block))
        .unwrap();
    let transform = BlockTransform::try_new(
        CoordinateFrame3::default(),
        std::f64::consts::FRAC_PI_2,
        Scale3::new(-2.0, 3.0, 1.0),
    )
    .unwrap();
    drawing
        .add_geometric_entity(GeometricEntityDefinition::new(
            0,
            layer,
            DrawingGeometry::BlockInstance {
                definition_scope_id: block,
                transform,
            },
        ))
        .unwrap();
    let encoded = drawing.finish().unwrap();
    let read = load_ocdraw_bytes(encoded.bytes());
    let drawing = read.as_ref().ok().unwrap();
    let bounds = drawing.scopes()[0].bounds.unwrap();
    assert!(
        bounds.min().y() <= -20.0 && bounds.max().y() >= 0.0,
        "{bounds:?}"
    );
    let mut bad = drawing.as_value().clone();
    bad["scopes"][0]["bounds"] =
        serde_json::json!({"minX":-1.0,"minY":-1.0,"minZ":-1.0,"maxX":11.0,"maxY":1.0,"maxZ":1.0});
    let bad = load_ocdraw_bytes(&serde_json::to_vec(&bad).unwrap());
    assert_eq!(
        bad.as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Invalid
    );
    assert!(
        bad.as_ref()
            .err()
            .map(|e| e.diagnostics())
            .unwrap_or_default()
            .iter()
            .any(|d| d.code == "BLOCK_BOUNDS"),
        "{:?}",
        bad.as_ref()
            .err()
            .map(|e| e.diagnostics())
            .unwrap_or_default()
    );
}

#[test]
fn typed_saved_state_and_paper_viewport_have_production_readback() {
    use ocdraw::ocdraw::{
        DrawingClip, DrawingClipMode, DrawingGrid, DrawingGridStyle, DrawingIsometricPlane,
        DrawingModelWindow, DrawingProjection, DrawingRenderMode, DrawingSavedState, DrawingSnap,
        DrawingSnapStyle, DrawingUcsSelection, DrawingView, DrawingViewState, DrawingViewportFrame,
        Point2, Point3, Vector3, ViewportDefinition,
    };
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("workspace", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(255, 255, 255),
            ocdraw::ocdraw::LinePatternId(0),
        ))
        .unwrap();
    let paper = builder.add_paper_layout("Sheet").unwrap();
    let view = DrawingView {
        center: Point2::new(2.0, 3.0),
        target: Point3::new(4.0, 5.0, 6.0),
        direction: Vector3::new(0.0, 0.0, 1.0),
        height: 20.0,
        twist: 0.2,
        projection: DrawingProjection::Orthographic,
        lens_length: Some(0.0),
        front_clip: DrawingClip {
            mode: DrawingClipMode::Disabled,
            distance: Some(0.0),
        },
        back_clip: DrawingClip {
            mode: DrawingClipMode::Disabled,
            distance: Some(0.0),
        },
    };
    let grid = DrawingGrid {
        enabled: true,
        spacing: Point2::new(5.0, 6.0),
        style: DrawingGridStyle::Lines,
        major_line_frequency: 5,
        beyond_limits: false,
        adaptive: true,
        subdivision: false,
        follows_workplane: false,
    };
    let snap = DrawingSnap {
        enabled: false,
        base: Point2::new(0.0, 0.0),
        spacing: Point2::new(1.0, 1.0),
        angle: 0.0,
        style: DrawingSnapStyle::Rectangular,
        isometric_plane: DrawingIsometricPlane::Left,
    };
    let window = DrawingModelWindow {
        id: 7,
        rectangle: [0.0, 0.0, 1.0, 1.0],
        view,
        aspect_ratio: 1.0,
        render_mode: DrawingRenderMode::TwoDimensional,
        grid,
        snap,
        stored_ucs: DrawingUcsSelection::World,
        use_stored_ucs: false,
    };
    builder.set_saved_state(DrawingSavedState {
        view_state: Some(DrawingViewState {
            current_model_ucs: Some(DrawingUcsSelection::World),
            active_model_window_id: Some(7),
        }),
        model_windows: vec![window],
        ..DrawingSavedState::default()
    });
    let viewport = builder
        .add_viewport(ViewportDefinition::new(
            paper,
            layer,
            DrawingViewportFrame {
                center: Point2::new(30.0, 40.0),
                width: 50.0,
                height: 60.0,
            },
            view,
        ))
        .unwrap();
    let encoded = builder.finish().unwrap();
    let loaded = load_ocdraw_bytes(encoded.bytes());
    let drawing = loaded.as_ref().ok().unwrap();
    assert_eq!(drawing.model_windows(), &[window]);
    assert_eq!(
        drawing.view_state().unwrap().active_model_window_id,
        Some(7)
    );
    assert_eq!(drawing.viewports()[0].id, viewport);
    assert_eq!(
        drawing.owner_scope_id(drawing.viewports()[0].id),
        Some(paper)
    );
    assert_eq!(drawing.viewports()[0].view, view);
    assert_eq!(drawing.scopes()[paper as usize].entities, vec![viewport]);
}

#[test]
fn optional_placement_columns_preserve_every_row_regardless_of_first_row() {
    use ocdraw::ocdraw::{
        CoordinateFrame3, DrawingGeometry, GeometricEntityDefinition, Point3, Vector3,
    };
    let placed = CoordinateFrame3::try_new(
        Point3::new(0., 0., 8.),
        Vector3::new(1., 0., 0.),
        Vector3::new(0., 1., 0.),
    )
    .unwrap();
    for reverse in [false, true] {
        let mut builder =
            OcdrawBuilder::new(OcdrawBuildOptions::new("mixed-placements", "mm")).unwrap();
        builder.ensure_continuous_line_pattern().unwrap();
        let layer = builder
            .add_layer(LayerDefinition::new(
                "0",
                RgbColor::new(0, 0, 0),
                ocdraw::ocdraw::LinePatternId(0),
            ))
            .unwrap();
        let frames = if reverse {
            [placed, CoordinateFrame3::default()]
        } else {
            [CoordinateFrame3::default(), placed]
        };
        let mut expected = Vec::new();
        for placement in frames {
            for geometry in [
                DrawingGeometry::Point { placement },
                DrawingGeometry::Circle {
                    placement,
                    radius: 2.,
                },
                DrawingGeometry::Arc {
                    placement,
                    radius: 2.,
                    start_parameter: 0.,
                    sweep_parameter: 1.,
                },
                DrawingGeometry::Ellipse {
                    placement,
                    semi_major_radius: 3.,
                    semi_minor_radius: 2.,
                    arc: None,
                },
                DrawingGeometry::PlanarPolyline {
                    line_pattern_generation: ocdraw::ocdraw::LinePatternGeneration::PerSegment,

                    placement,
                    vertices: vec![[0., 0., 0.], [1., 1., 0.]],
                    closed: false,
                },
            ] {
                let id = builder
                    .add_geometric_entity(GeometricEntityDefinition::new(
                        0,
                        layer,
                        geometry.clone(),
                    ))
                    .unwrap();
                expected.push((id, geometry));
            }
        }
        let output = builder.finish().unwrap();
        let loaded = load_ocdraw_bytes(output.bytes());
        let drawing = loaded.as_ref().ok().unwrap();
        for (id, geometry) in expected {
            assert_eq!(
                drawing
                    .geometric_entities()
                    .iter()
                    .find(|e| e.id() == id)
                    .unwrap()
                    .geometry(),
                &geometry
            );
        }
    }
}

#[test]
fn scopes_store_authoritative_ordered_ownership_without_duplicate_columns() {
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("owners", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(255, 255, 255),
            ocdraw::ocdraw::LinePatternId(0),
        ))
        .unwrap();
    let a = builder
        .add_line(LineDefinition::new(layer, [0.0; 3], [1.0; 3]))
        .unwrap();
    let b = builder
        .add_point(ocdraw::ocdraw::PointDefinition::new(layer, [2.0; 3]))
        .unwrap();
    let bytes = builder.finish().unwrap();
    let read = load_ocdraw_bytes(bytes.bytes());
    assert_eq!(
        read.as_ref()
            .map(|_| OcdrawReadStatus::Valid)
            .unwrap_or_else(|e| e.status()),
        OcdrawReadStatus::Valid,
        "{:?}",
        read.as_ref()
            .err()
            .map(|e| e.diagnostics())
            .unwrap_or_default()
    );
    let value = read.as_ref().ok().unwrap().as_value();
    assert_eq!(value["scopes"][0]["entities"], serde_json::json!([a, b]));
    assert!(value["streams"].get("entityOrderStream").is_none());
    assert!(value["streams"].get("entityOrderEntryStream").is_none());
    for stream in value["streams"].as_object().unwrap().values() {
        assert!(stream.get("scopeId").is_none());
    }
}

#[test]
fn polyline_pools_accept_fractional_and_negative_coordinates() {
    use ocdraw::ocdraw::{CoordinateFrame3, DrawingGeometry, GeometricEntityDefinition};
    for geometry in [
        DrawingGeometry::PlanarPolyline {
            line_pattern_generation: ocdraw::ocdraw::LinePatternGeneration::PerSegment,

            placement: CoordinateFrame3::default(),
            vertices: vec![[-1.25, 2.5, 0.125], [3.75, -4.5, 0.0]],
            closed: false,
        },
        DrawingGeometry::SpatialPolyline {
            line_pattern_generation: ocdraw::ocdraw::LinePatternGeneration::PerSegment,

            vertices: vec![[-1.25, 2.5, -3.75], [4.5, -5.25, 6.125]],
            closed: false,
        },
    ] {
        let mut builder =
            OcdrawBuilder::new(OcdrawBuildOptions::new("fractional-polyline", "mm")).unwrap();
        builder.ensure_continuous_line_pattern().unwrap();
        let layer = builder
            .add_layer(LayerDefinition::new(
                "0",
                RgbColor::new(255, 255, 255),
                ocdraw::ocdraw::LinePatternId(0),
            ))
            .unwrap();
        builder
            .add_geometric_entity(GeometricEntityDefinition::new(0, layer, geometry.clone()))
            .unwrap();
        let encoded = builder.finish().unwrap();
        let loaded = load_ocdraw_bytes(encoded.bytes());
        assert_eq!(
            loaded.as_ref().ok().unwrap().geometric_entities()[0].geometry(),
            &geometry
        );
    }
}
