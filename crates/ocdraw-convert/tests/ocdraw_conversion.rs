use ocdraw::ocdraw::{load_ocdraw_bytes, OcdrawReadStatus};
use ocdraw_convert::{
    cad_document_to_encoded_ocdraw, cad_document_to_encoded_ocdraw_with_id,
    ocdraw_source_to_cad_document, CadToOcdrawError, CadToOcdrawOptions, OcdrawLossPolicy,
    OcdrawToCadOptions,
};
use opencadcodec::{CadDocument, Circle, Color, EntityType, Line, Point, Vector3};

#[test]
fn direct_line_export_creates_a_standalone_drawing() {
    let mut source = CadDocument::new();
    let mut line = Line::from_coords(1.0, 2.0, 0.0, 3.0, 4.0, 0.0);
    line.common.color = Color::ByBlock;
    source.add_entity(EntityType::Line(line)).unwrap();
    let outcome = cad_document_to_encoded_ocdraw(&source, CadToOcdrawOptions::default()).unwrap();
    let read = load_ocdraw_bytes(outcome.encoded().bytes());
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
    assert_eq!(value["streams"]["lineStream"]["count"], 1);
    assert_eq!(value["streams"]["lineStream"]["colorMode"][0], "ByBlock");
    assert!(value["layers"]
        .as_array()
        .is_some_and(|layers| !layers.is_empty()));
    assert_eq!(value["drawingWorkspaceState"]["activeLayoutId"], 0);
    let imported =
        ocdraw_source_to_cad_document(read.as_ref().ok().unwrap(), OcdrawToCadOptions::default())
            .unwrap();
    let imported_line = imported
        .document()
        .entities()
        .find_map(|entity| match entity {
            EntityType::Line(line) => Some(line),
            _ => None,
        })
        .unwrap();
    assert_eq!(imported_line.start.x, 1.0);
    assert_eq!(imported_line.end.y, 4.0);
    assert_eq!(imported_line.common.color, Color::ByBlock);
}

#[test]
fn unsupported_cad_content_is_diagnosed_and_rejectable() {
    let mut source = CadDocument::new();
    source.header.project_name = "Unrepresented project name".into();
    let allow = cad_document_to_encoded_ocdraw(&source, CadToOcdrawOptions::default()).unwrap();
    assert!(!allow.diagnostics().is_empty());
    let reject = cad_document_to_encoded_ocdraw(
        &source,
        CadToOcdrawOptions {
            loss_policy: OcdrawLossPolicy::Reject,
            ..CadToOcdrawOptions::default()
        },
    );
    assert!(matches!(reject, Err(CadToOcdrawError::LossRejected { .. })));
}

#[test]
fn caller_can_assign_a_stable_drawing_identity() {
    let source = CadDocument::new();
    let outcome = cad_document_to_encoded_ocdraw_with_id(
        &source,
        "drawing-stable-42",
        CadToOcdrawOptions::default(),
    )
    .unwrap();
    let read = load_ocdraw_bytes(outcome.encoded().bytes());
    assert_eq!(
        read.as_ref().ok().unwrap().drawing_id(),
        "drawing-stable-42"
    );
}

#[test]
fn direct_import_preserves_paper_scope_and_mixed_appearance_modes() {
    use ocdraw::ocdraw::{
        AppearanceSelection, EntityAppearance, LayerDefinition, LineDefinition, OcdrawBuildOptions,
        OcdrawBuilder, RgbColor,
    };
    use opencadcodec::{LineWeight, Transparency};
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("paper", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(255, 255, 255),
            ocdraw::ocdraw::LinePatternId(0),
        ))
        .unwrap();
    let paper = builder.add_paper_layout("Sheet").unwrap();
    let mut line = LineDefinition::new(layer, [0.0, 0.0, 0.0], [2.0, 3.0, 0.0]).in_scope(paper);
    line.appearance = EntityAppearance {
        line_pattern_scale: 1.0,

        color: AppearanceSelection::ByBlock,
        opacity: AppearanceSelection::Explicit(0.5),
        line_pattern: AppearanceSelection::ByLayer,
        line_weight: AppearanceSelection::Explicit(0.25),
    };
    builder.add_line(line).unwrap();
    let encoded = builder.finish().unwrap();
    let read = load_ocdraw_bytes(encoded.bytes());
    let imported =
        ocdraw_source_to_cad_document(read.as_ref().ok().unwrap(), OcdrawToCadOptions::default())
            .unwrap();
    let line = imported
        .document()
        .entities()
        .find_map(|entity| match entity {
            EntityType::Line(line) => Some(line),
            _ => None,
        })
        .unwrap();
    assert_eq!(line.common.color, Color::ByBlock);
    assert_eq!(line.common.transparency, Transparency::Explicit(128));
    assert_eq!(line.common.line_weight, LineWeight::W0_25);
    assert!(imported.document().objects.values().any(|object| matches!(object,
        opencadcodec::objects::ObjectType::Layout(layout) if layout.name == "Sheet" && layout.block_record == line.common.owner_handle)));
}

#[test]
fn point_and_circle_roundtrip_through_direct_drawing() {
    let mut source = CadDocument::new();
    source
        .add_entity(EntityType::Point(Point::from_coords(4.0, 5.0, 0.0)))
        .unwrap();
    source
        .add_entity(EntityType::Circle(Circle::from_center_radius(
            Vector3::new(10.0, 20.0, 0.0),
            3.0,
        )))
        .unwrap();
    let exported = cad_document_to_encoded_ocdraw(&source, CadToOcdrawOptions::default()).unwrap();
    let read = load_ocdraw_bytes(exported.encoded().bytes());
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
    assert_eq!(value["streams"]["pointStream"]["count"], 1);
    assert_eq!(value["streams"]["circleStream"]["count"], 1);
    let imported =
        ocdraw_source_to_cad_document(read.as_ref().ok().unwrap(), OcdrawToCadOptions::default())
            .unwrap();
    assert!(imported.document().entities().any(|entity| matches!(entity,
        EntityType::Point(point) if point.location == Vector3::new(4.0,5.0,0.0))));
    assert!(imported.document().entities().any(|entity| matches!(entity,
        EntityType::Circle(circle) if circle.center == Vector3::new(10.0,20.0,0.0) && circle.radius == 3.0)));
}

#[test]
fn planar_and_spatial_polylines_keep_vertices_and_order() {
    use opencadcodec::{LwPolyline, Vector2};
    let mut source = CadDocument::new();
    let mut planar =
        LwPolyline::from_points(vec![Vector2::new(-1.25, 2.5), Vector2::new(3.75, -4.5)]);
    planar.vertices[0].bulge = 1.0;
    planar.is_closed = false;
    source.add_entity(EntityType::LwPolyline(planar)).unwrap();
    let spatial = opencadcodec::entities::Polyline3D::from_points(vec![
        Vector3::new(-1.25, 2.5, -3.75),
        Vector3::new(4.5, -5.25, 6.125),
    ]);
    source.add_entity(EntityType::Polyline3D(spatial)).unwrap();
    let exported = cad_document_to_encoded_ocdraw(&source, CadToOcdrawOptions::default()).unwrap();
    let read = load_ocdraw_bytes(exported.encoded().bytes());
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
    assert_eq!(
        drawing.as_value()["streams"]["planarPolylineStream"]["bulge"],
        serde_json::json!([1.0, 0.0])
    );
    assert_eq!(
        drawing.as_value()["streams"]["spatialPolylineStream"]["z"],
        serde_json::json!([-3.75, 6.125])
    );
    let imported = ocdraw_source_to_cad_document(drawing, OcdrawToCadOptions::default()).unwrap();
    let kinds = imported
        .document()
        .entities()
        .filter_map(|entity| match entity {
            EntityType::LwPolyline(polyline) => Some(("planar", polyline.vertices.len())),
            EntityType::Polyline3D(polyline) => Some(("spatial", polyline.vertices.len())),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(kinds, vec![("planar", 2), ("spatial", 2)]);
}

#[test]
fn legacy_cad_polyline_variants_map_to_the_same_logical_families() {
    use opencadcodec::Vector2;
    let mut source = CadDocument::new();
    let mut planar = opencadcodec::entities::Polyline2D::new();
    planar.add_vertex(opencadcodec::entities::Vertex2D::from_point(Vector2::new(
        0.0, 0.0,
    )));
    planar.add_vertex(opencadcodec::entities::Vertex2D::from_point(Vector2::new(
        3.0, 4.0,
    )));
    source.add_entity(EntityType::Polyline2D(planar)).unwrap();
    let spatial = opencadcodec::entities::Polyline::from_points(vec![
        Vector3::new(1.0, 2.0, 3.0),
        Vector3::new(4.0, 5.0, 6.0),
    ]);
    source.add_entity(EntityType::Polyline(spatial)).unwrap();
    let exported = cad_document_to_encoded_ocdraw(&source, CadToOcdrawOptions::default()).unwrap();
    let read = load_ocdraw_bytes(exported.encoded().bytes());
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
    let drawing = read.as_ref().ok().unwrap().as_value();
    assert_eq!(
        drawing["streams"]["planarPolylineStream"]["x"],
        serde_json::json!([0.0, 3.0])
    );
    assert_eq!(
        drawing["streams"]["spatialPolylineStream"]["z"],
        serde_json::json!([3.0, 6.0])
    );
}

#[test]
fn point_display_header_state_roundtrips_directly() {
    let mut source = CadDocument::new();
    source.header.point_display_mode = 35;
    source.header.point_display_size = -5.0;
    let exported = cad_document_to_encoded_ocdraw(&source, CadToOcdrawOptions::default()).unwrap();
    let read = load_ocdraw_bytes(exported.encoded().bytes());
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
    assert_eq!(drawing.as_value()["pointDisplay"]["form"]["glyph"], "cross");
    let imported = ocdraw_source_to_cad_document(drawing, OcdrawToCadOptions::default()).unwrap();
    assert_eq!(imported.document().header.point_display_mode, 35);
    assert_eq!(imported.document().header.point_display_size, -5.0);
}

#[test]
fn layout_limits_and_scaling_roundtrip_directly() {
    let mut source = CadDocument::new();
    source.add_layout("Sheet").unwrap();
    source.header.paper_space_linetype_scaling = false;
    for object in source.objects.values_mut() {
        if let opencadcodec::objects::ObjectType::Layout(layout) = object {
            if layout.name == "Sheet" {
                layout.flags |= 2;
                layout.min_limits = (1.0, 2.0);
                layout.max_limits = (100.0, 200.0);
            }
        }
    }
    let exported = cad_document_to_encoded_ocdraw(&source, CadToOcdrawOptions::default()).unwrap();
    let read = load_ocdraw_bytes(exported.encoded().bytes());
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
    let sheet = drawing.as_value()["layouts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|layout| layout["name"] == "Sheet")
        .unwrap();
    assert_eq!(sheet["limits"]["maxY"], 200.0);
    assert_eq!(sheet["limitsChecking"], true);
    assert_eq!(sheet["paperSpaceLinetypeScaling"], false);
    let imported = ocdraw_source_to_cad_document(drawing, OcdrawToCadOptions::default()).unwrap();
    assert!(!imported.document().header.paper_space_linetype_scaling);
    assert!(imported
        .document()
        .objects
        .values()
        .any(|object| matches!(object,
        opencadcodec::objects::ObjectType::Layout(layout) if layout.name == "Sheet"
            && layout.flags & 2 != 0 && layout.min_limits == (1.0, 2.0)
            && layout.max_limits == (100.0, 200.0))));
}

#[test]
fn configured_layout_plot_settings_roundtrip_directly() {
    let mut source = CadDocument::new();
    source.add_layout("Sheet").unwrap();
    for object in source.objects.values_mut() {
        if let opencadcodec::objects::ObjectType::Layout(layout) = object {
            if layout.name == "Sheet" {
                layout.paper_width = 210.0;
                layout.paper_height = 297.0;
                layout.plot_margin_left = 5.0;
                layout.plot_margin_bottom = 6.0;
                layout.plot_margin_right = 7.0;
                layout.plot_margin_top = 8.0;
                layout.plot_type = 1;
                layout.plot_paper_units = 1;
                layout.plot_scale_type = 0;
                layout.plot_flags.use_standard_scale = true;
                layout.plot_flags.plot_centered = true;
                layout.paper_size = "A4".into();
            }
        }
    }
    let exported = cad_document_to_encoded_ocdraw(&source, CadToOcdrawOptions::default()).unwrap();
    let read = load_ocdraw_bytes(exported.encoded().bytes());
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
    let sheet = drawing.as_value()["layouts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|layout| layout["name"] == "Sheet")
        .unwrap();
    assert_eq!(sheet["media"]["width"], 210.0);
    assert_eq!(sheet["plotSettings"]["page"]["mediaName"], "A4");
    let imported = ocdraw_source_to_cad_document(drawing, OcdrawToCadOptions::default()).unwrap();
    assert!(imported
        .document()
        .objects
        .values()
        .any(|object| matches!(object,
        opencadcodec::objects::ObjectType::Layout(layout) if layout.name == "Sheet"
            && layout.paper_width == 210.0 && layout.paper_height == 297.0
            && layout.plot_margin_left == 5.0 && layout.plot_margin_top == 8.0
            && layout.paper_size == "A4")));
}

#[test]
fn unused_named_ucs_roundtrips_directly() {
    let mut source = CadDocument::new();
    let mut ucs = opencadcodec::Ucs::new("Grid A");
    ucs.handle = source.allocate_handle();
    ucs.origin = Vector3::new(1.0, 2.0, 3.0);
    ucs.elevation = 4.0;
    source.ucss.add(ucs).unwrap();
    let exported = cad_document_to_encoded_ocdraw(&source, CadToOcdrawOptions::default()).unwrap();
    assert!(!exported.diagnostics().iter().flat_map(|diagnostic| diagnostic.reasons()).any(|reason|
        matches!(reason, ocdraw_convert::CadToOcdrawLossReason::UnsupportedTableRecords { kind, .. } if kind == "ucss")));
    let read = load_ocdraw_bytes(exported.encoded().bytes());
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
    assert_eq!(drawing.as_value()["ucsDefinitions"][0]["name"], "Grid A");
    let imported = ocdraw_source_to_cad_document(drawing, OcdrawToCadOptions::default()).unwrap();
    let restored = imported
        .document()
        .ucss
        .iter()
        .find(|ucs| ucs.name == "Grid A")
        .unwrap();
    assert_eq!(restored.origin, Vector3::new(1.0, 2.0, 3.0));
    assert_eq!(restored.elevation, 4.0);
}

#[test]
fn rotated_point_frame_is_mapped_in_direct_import() {
    use ocdraw::ocdraw::{
        LayerDefinition, OcdrawBuildOptions, OcdrawBuilder, PointDefinition, RgbColor,
    };
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("rotated", "mm")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(0, 0, 0),
            ocdraw::ocdraw::LinePatternId(0),
        ))
        .unwrap();
    builder
        .add_point(PointDefinition::new(layer, [1.0, 2.0, 0.0]))
        .unwrap();
    let encoded = builder.finish().unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    value["streams"]["pointStream"]["placement"][0]["X"] =
        serde_json::json!({"x":0.0,"y":1.0,"z":0.0});
    value["streams"]["pointStream"]["placement"][0]["Y"] =
        serde_json::json!({"x":-1.0,"y":0.0,"z":0.0});
    let bytes = serde_json::to_vec(&value).unwrap();
    let read = load_ocdraw_bytes(&bytes);
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
    let imported =
        ocdraw_source_to_cad_document(read.as_ref().ok().unwrap(), OcdrawToCadOptions::default())
            .unwrap();
    let point = imported
        .document()
        .entities()
        .find_map(|entity| match entity {
            EntityType::Point(point) => Some(point),
            _ => None,
        })
        .unwrap();
    assert!((point.x_axis_angle - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
}

#[test]
fn shared_block_definition_and_instance_roundtrip_directly() {
    let mut source = CadDocument::new();
    let mut record = opencadcodec::BlockRecord::new("Door");
    record.handle = source.allocate_handle();
    let owner = record.handle;
    source.block_records.add(record).unwrap();
    let mut line = Line::from_coords(0.0, 0.0, 0.0, 2.0, 0.0, 0.0);
    line.common.owner_handle = owner;
    source.add_entity(EntityType::Line(line)).unwrap();
    source
        .add_entity(EntityType::Insert(opencadcodec::entities::Insert::new(
            "Door",
            Vector3::new(10.0, 0.0, 0.0),
        )))
        .unwrap();
    let exported = cad_document_to_encoded_ocdraw(&source, CadToOcdrawOptions::default()).unwrap();
    let read = load_ocdraw_bytes(exported.encoded().bytes());
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
    assert_eq!(value["blockDefinitions"][0]["name"], "Door");
    assert_eq!(value["streams"]["blockInstanceStream"]["count"], 1);
    let imported =
        ocdraw_source_to_cad_document(read.as_ref().ok().unwrap(), OcdrawToCadOptions::default())
            .unwrap();
    assert_eq!(
        imported
            .document()
            .entities_in_block("Door")
            .filter(|e| matches!(e, EntityType::Line(_)))
            .count(),
        1
    );
    assert!(imported.document().entities().any(
        |e| matches!(e, EntityType::Insert(i) if i.block_name == "Door" && i.insert_point.x == 10.0)
    ));
}

#[test]
fn arc_roundtrip_preserves_center_radius_and_sweep() {
    let mut source = CadDocument::new();
    source.header.insertion_units = 4;
    source
        .add_entity(EntityType::Arc(
            opencadcodec::Arc::from_center_radius_angles(
                Vector3::new(2.0, 3.0, 0.0),
                5.0,
                0.25,
                1.25,
            ),
        ))
        .unwrap();
    let exported = cad_document_to_encoded_ocdraw(&source, CadToOcdrawOptions::default()).unwrap();
    let read = load_ocdraw_bytes(exported.encoded().bytes());
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
        read.as_ref().ok().unwrap().as_value()["streams"]["arcStream"]["sweepParameter"][0],
        1.0
    );
    let imported =
        ocdraw_source_to_cad_document(read.as_ref().ok().unwrap(), OcdrawToCadOptions::default())
            .unwrap();
    assert!(imported.document().entities().any(|entity| matches!(entity,
        EntityType::Arc(arc) if arc.center == Vector3::new(2.0,3.0,0.0) && arc.radius == 5.0 && arc.start_angle == 0.25 && arc.end_angle == 1.25)));
}

#[test]
fn full_and_partial_ellipses_roundtrip_directly() {
    let mut source = CadDocument::new();
    source.header.insertion_units = 4;
    source
        .add_entity(EntityType::Ellipse(
            opencadcodec::Ellipse::from_center_axes(
                Vector3::new(1.0, 2.0, 0.0),
                Vector3::new(3.0, 0.0, 0.0),
                0.5,
            ),
        ))
        .unwrap();
    let mut partial = opencadcodec::Ellipse::from_center_axes(
        Vector3::new(10.0, 2.0, 0.0),
        Vector3::new(0.0, 4.0, 0.0),
        0.25,
    );
    partial.start_parameter = 0.5;
    partial.end_parameter = 1.5;
    source.add_entity(EntityType::Ellipse(partial)).unwrap();
    let exported = cad_document_to_encoded_ocdraw(&source, CadToOcdrawOptions::default()).unwrap();
    let read = load_ocdraw_bytes(exported.encoded().bytes());
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
    let imported =
        ocdraw_source_to_cad_document(read.as_ref().ok().unwrap(), OcdrawToCadOptions::default())
            .unwrap();
    assert_eq!(
        imported
            .document()
            .entities()
            .filter(|entity| matches!(entity, EntityType::Ellipse(_)))
            .count(),
        2
    );
}

#[test]
fn standalone_preserves_oblique_polyline_elevation_and_dormant_bulge() {
    let mut document = CadDocument::new();
    document.header.insertion_units = 4;
    let mut poly = opencadcodec::LwPolyline::from_points(vec![
        opencadcodec::Vector2::new(1.0, 2.0),
        opencadcodec::Vector2::new(4.0, 6.0),
    ]);
    poly.normal = Vector3::UNIT_X;
    poly.elevation = 7.0;
    poly.vertices[0].bulge = 0.25;
    poly.vertices[1].bulge = -0.5;
    document
        .add_entity(EntityType::LwPolyline(poly.clone()))
        .unwrap();
    let exported =
        cad_document_to_encoded_ocdraw(&document, CadToOcdrawOptions::default()).unwrap();
    let read = load_ocdraw_bytes(exported.encoded().bytes());
    let drawing = read.as_ref().expect("production readback");
    assert_eq!(
        drawing.geometric_entities().len(),
        1,
        "{:?}",
        exported.diagnostics()
    );
    let imported = ocdraw_source_to_cad_document(drawing, OcdrawToCadOptions::default()).unwrap();
    let target = imported
        .document()
        .entities()
        .find_map(|entity| match entity {
            EntityType::LwPolyline(poly) => Some(poly),
            _ => None,
        })
        .unwrap();
    assert_eq!(target.normal, poly.normal);
    assert_eq!(target.elevation, poly.elevation);
    assert!(!target.is_closed);
    for (a, b) in target.vertices.iter().zip(&poly.vertices) {
        assert_eq!(a.location, b.location);
        assert_eq!(a.bulge, b.bulge);
    }
}

#[test]
fn standalone_preserves_oblique_circular_geometry_and_point_orientation() {
    let mut document = CadDocument::new();
    document.header.insertion_units = 4;
    let mut circle = Circle::from_center_radius(Vector3::new(2.0, 3.0, 4.0), 5.0);
    circle.normal = Vector3::UNIT_X;
    document
        .add_entity(EntityType::Circle(circle.clone()))
        .unwrap();
    let mut point = Point::from_coords(7.0, 8.0, 9.0);
    point.normal = Vector3::UNIT_X;
    point.x_axis_angle = 0.7;
    document
        .add_entity(EntityType::Point(point.clone()))
        .unwrap();
    let exported =
        cad_document_to_encoded_ocdraw(&document, CadToOcdrawOptions::default()).unwrap();
    let read = load_ocdraw_bytes(exported.encoded().bytes());
    let drawing = read.as_ref().expect("production readback");
    assert_eq!(
        drawing.geometric_entities().len(),
        2,
        "{:?}",
        exported.diagnostics()
    );
    let imported = ocdraw_source_to_cad_document(drawing, OcdrawToCadOptions::default()).unwrap();
    let target = imported
        .document()
        .entities()
        .find_map(|entity| match entity {
            EntityType::Circle(circle) => Some(circle),
            _ => None,
        })
        .unwrap();
    assert_eq!(target.normal, circle.normal);
    assert_eq!(target.center, circle.center);
    assert_eq!(target.radius, circle.radius);
    let target = imported
        .document()
        .entities()
        .find_map(|entity| match entity {
            EntityType::Point(point) => Some(point),
            _ => None,
        })
        .unwrap();
    assert_eq!(target.normal, point.normal);
    assert_eq!(target.location, point.location);
    assert!((target.x_axis_angle - point.x_axis_angle).abs() < 1e-15);
}

#[test]
fn standalone_import_enforces_unitless_exactness_and_explicit_tolerance() {
    use ocdraw::ocdraw::{
        CoordinateFrame3, DrawingGeometry, GeometricEntityDefinition, LayerDefinition,
        OcdrawBuildOptions, OcdrawBuilder, Point3, RgbColor, Vector3 as NativeVector,
    };
    use ocdraw_convert::{OcdrawGeometryTolerance, OcdrawToCadError, OcdrawToleranceError};
    let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("unitless", "unitless")).unwrap();
    builder.ensure_continuous_line_pattern().unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new(
            "0",
            RgbColor::new(255, 255, 255),
            ocdraw::ocdraw::LinePatternId(0),
        ))
        .unwrap();
    let frame = CoordinateFrame3::try_new(
        Point3::new(9007199254740992.0, 0.0, 0.0),
        NativeVector::new(1.0, 0.0, 0.0),
        NativeVector::new(0.0, 1.0, 0.0),
    )
    .unwrap();
    builder
        .add_geometric_entity(GeometricEntityDefinition::new(
            0,
            layer,
            DrawingGeometry::PlanarPolyline {
                line_pattern_generation: ocdraw::ocdraw::LinePatternGeneration::PerSegment,

                placement: frame,
                vertices: vec![[1.0, 0.0, 0.0], [3.0, 2.0, 0.0]],
                closed: false,
            },
        ))
        .unwrap();
    let encoded = builder.finish().unwrap();
    let read = load_ocdraw_bytes(encoded.bytes());
    let drawing = read.as_ref().ok().unwrap();
    assert!(matches!(
        ocdraw_source_to_cad_document(drawing, OcdrawToCadOptions::default()),
        Err(OcdrawToCadError::Geometry(_))
    ));
    assert!(matches!(
        ocdraw_source_to_cad_document(
            drawing,
            OcdrawToCadOptions {
                geometry_tolerance: OcdrawGeometryTolerance::metres(1.0).unwrap(),
                ..OcdrawToCadOptions::default()
            }
        ),
        Err(OcdrawToCadError::GeometryTolerance(
            OcdrawToleranceError::PhysicalUnitRequired
        ))
    ));
    let imported = ocdraw_source_to_cad_document(
        drawing,
        OcdrawToCadOptions {
            geometry_tolerance: OcdrawGeometryTolerance::drawing_units(1.0).unwrap(),
            ..OcdrawToCadOptions::default()
        },
    )
    .unwrap();
    assert_eq!(
        imported
            .geometry_assessment()
            .domains()
            .iter()
            .find(|d| d.domain() == ocdraw_convert::OcdrawGeometryDomain::Drawing)
            .unwrap()
            .max_deviation_upper_bound(),
        1.0
    );
    assert_eq!(imported.geometry_assessment().rounded_entities(), 1);
    assert!(imported
        .diagnostics()
        .iter()
        .any(|d| d.code == "GEOMETRY_ROUNDED_WITHIN_TOLERANCE"));
}

#[test]
fn standalone_checks_scaled_block_occurrences_against_the_same_tolerance() {
    use ocdraw::ocdraw::{
        BlockDefinition, BlockTransform, CoordinateFrame3, DrawingGeometry,
        GeometricEntityDefinition, LayerDefinition, OcdrawBuildOptions, OcdrawBuilder, Point3,
        RgbColor, Scale3, Vector3 as NativeVector,
    };
    use ocdraw_convert::{OcdrawGeometryEntitySource, OcdrawGeometryTolerance, OcdrawToCadError};
    let mut builder =
        OcdrawBuilder::new(OcdrawBuildOptions::new("amplification", "unitless")).unwrap();
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
        Point3::new(9007199254740992.0, 0.0, 0.0),
        NativeVector::new(1.0, 0.0, 0.0),
        NativeVector::new(0.0, 1.0, 0.0),
    )
    .unwrap();
    builder
        .add_geometric_entity(GeometricEntityDefinition::new(
            block,
            layer,
            DrawingGeometry::PlanarPolyline {
                line_pattern_generation: ocdraw::ocdraw::LinePatternGeneration::PerSegment,

                placement: frame,
                vertices: vec![[1.0, 0.0, 0.0], [3.0, 2.0, 0.0]],
                closed: false,
            },
        ))
        .unwrap();
    let transform =
        BlockTransform::try_new(CoordinateFrame3::default(), 0.0, Scale3::new(4.0, 4.0, 1.0))
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
    let loaded = load_ocdraw_bytes(encoded.bytes());
    let drawing = loaded.as_ref().ok().unwrap();
    let options = OcdrawToCadOptions {
        geometry_tolerance: OcdrawGeometryTolerance::drawing_units(1.0).unwrap(),
        ..OcdrawToCadOptions::default()
    };
    let Err(OcdrawToCadError::Geometry(failure)) = ocdraw_source_to_cad_document(drawing, options)
    else {
        panic!("scaled local residual must fail")
    };
    assert!(matches!(
        failure.source,
        OcdrawGeometryEntitySource::BlockOccurrence { .. }
    ));
    assert_eq!(failure.deviation.unwrap().lower(), 4.0);
    let imported = ocdraw_source_to_cad_document(
        drawing,
        OcdrawToCadOptions {
            geometry_tolerance: OcdrawGeometryTolerance::drawing_units(4.0).unwrap(),
            ..OcdrawToCadOptions::default()
        },
    )
    .unwrap();
    assert_eq!(
        imported
            .geometry_assessment()
            .domains()
            .iter()
            .find(|d| d.domain() == ocdraw_convert::OcdrawGeometryDomain::Drawing)
            .unwrap()
            .max_deviation_upper_bound(),
        4.0
    );
}

#[test]
fn standalone_roundtrips_rotated_mirrored_oblique_instances_and_shared_contents() {
    let mut document = CadDocument::new();
    document.header.insertion_units = 4;
    let mut block = opencadcodec::BlockRecord::new("Shared");
    block.handle = document.allocate_handle();
    block.base_point = Vector3::new(2.0, 3.0, 4.0);
    let owner = block.handle;
    document.block_records.add(block).unwrap();
    let mut line = Line::from_coords(2.0, 3.0, 4.0, 4.0, 6.0, 8.0);
    line.common.owner_handle = owner;
    document.add_entity(EntityType::Line(line)).unwrap();
    for x in [10.0, 20.0] {
        let mut instance = opencadcodec::entities::Insert::new("Shared", Vector3::new(x, 5.0, 7.0));
        instance.normal = Vector3::UNIT_X;
        instance.rotation = 0.4;
        instance.set_x_scale(-2.0);
        instance.set_y_scale(3.0);
        instance.set_z_scale(0.5);
        document.add_entity(EntityType::Insert(instance)).unwrap();
    }
    let exported =
        cad_document_to_encoded_ocdraw(&document, CadToOcdrawOptions::default()).unwrap();
    let loaded = load_ocdraw_bytes(exported.encoded().bytes());
    let drawing = loaded.as_ref().ok().unwrap();
    assert_eq!(drawing.block_definitions().len(), 1);
    assert_eq!(drawing.geometric_entities().len(), 3);
    let imported = ocdraw_source_to_cad_document(drawing, OcdrawToCadOptions::default()).unwrap();
    let source = document.entities().filter_map(|e| {
        if let EntityType::Insert(i) = e {
            Some(i)
        } else {
            None
        }
    });
    let targets = imported
        .document()
        .entities()
        .filter_map(|e| {
            if let EntityType::Insert(i) = e {
                Some(i)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(targets.len(), 2);
    for (a, b) in source.zip(targets) {
        assert_eq!(a.normal, b.normal);
        assert_eq!(a.insert_point, b.insert_point);
        assert_eq!(a.rotation, b.rotation);
        assert_eq!(
            [a.x_scale(), a.y_scale(), a.z_scale()],
            [b.x_scale(), b.y_scale(), b.z_scale()]
        );
        assert_eq!(
            a.get_transform().apply(Vector3::new(2.0, 3.0, 4.0)),
            b.get_transform().apply(Vector3::new(2.0, 3.0, 4.0))
        );
    }
    assert_eq!(
        imported
            .document()
            .entities_in_block("Shared")
            .filter(|e| matches!(e, EntityType::Line(_)))
            .count(),
        1
    );
}

#[test]
fn standalone_preserves_active_model_window_and_named_current_ucs() {
    let mut document = CadDocument::new();
    let mut named = opencadcodec::Ucs::new("Survey");
    named.handle = document.allocate_handle();
    named.origin = Vector3::new(10.0, 20.0, 30.0);
    document.ucss.add(named).unwrap();
    document.header.model_space_ucs_name = "Survey".into();
    document.header.model_space_ucs_origin = Vector3::new(10.0, 20.0, 30.0);
    let vport = document.vports.iter_mut().next().unwrap();
    vport.view_center = opencadcodec::Vector2::new(3.0, 4.0);
    vport.view_height = 27.0;
    vport.view_twist = 0.4;
    vport.grid_on = true;
    vport.grid_spacing = opencadcodec::Vector2::new(5.0, 7.0);
    vport.snap_on = true;
    vport.snap_spacing = opencadcodec::Vector2::new(2.0, 3.0);
    let exported =
        cad_document_to_encoded_ocdraw(&document, CadToOcdrawOptions::default()).unwrap();
    let loaded = load_ocdraw_bytes(exported.encoded().bytes());
    let drawing = loaded.as_ref().ok().unwrap();
    assert_eq!(drawing.model_windows().len(), 1);
    assert_eq!(drawing.model_windows()[0].view.height, 27.0);
    let imported = ocdraw_source_to_cad_document(drawing, OcdrawToCadOptions::default()).unwrap();
    assert_eq!(imported.document().header.model_space_ucs_name, "Survey");
    let target = imported.document().vports.iter().next().unwrap();
    assert_eq!(target.view_center, opencadcodec::Vector2::new(3.0, 4.0));
    assert_eq!(target.view_height, 27.0);
    assert_eq!(target.view_twist, 0.4);
    assert!(target.grid_on);
    assert_eq!(target.grid_spacing, opencadcodec::Vector2::new(5.0, 7.0));
    assert!(target.snap_on);
    assert_eq!(target.snap_spacing, opencadcodec::Vector2::new(2.0, 3.0));
}

#[test]
fn standalone_preserves_paper_viewports_canvas_and_mixed_draw_order() {
    let mut document = CadDocument::new();
    let sheet = document.add_layout("Sheet").unwrap();
    let opencadcodec::objects::ObjectType::Layout(layout) = document.objects.get(&sheet).unwrap()
    else {
        panic!()
    };
    document.header.show_model_space = false;
    document.header.paper_space_block_handle = layout.block_record;
    let canvas = layout.viewport;
    let EntityType::Viewport(overall) = document.get_entity_mut(canvas).unwrap() else {
        panic!()
    };
    overall.view_height = 35.0;
    document
        .add_entity_to_layout(
            EntityType::Line(Line::from_coords(1.0, 0.0, 0.0, 2.0, 0.0, 0.0)),
            "Sheet",
        )
        .unwrap();
    let mut viewport = opencadcodec::entities::Viewport::new();
    viewport.id = 2;
    viewport.center = Vector3::new(20.0, 30.0, 0.0);
    viewport.width = 40.0;
    viewport.height = 50.0;
    viewport.view_height = 60.0;
    viewport.status.locked = true;
    viewport.lens_length = 0.0;
    viewport
        .frozen_layers
        .push(document.layers.get("0").unwrap().handle);
    document
        .add_entity_to_layout(EntityType::Viewport(viewport), "Sheet")
        .unwrap();
    document
        .add_entity_to_layout(
            EntityType::Line(Line::from_coords(3.0, 0.0, 0.0, 4.0, 0.0, 0.0)),
            "Sheet",
        )
        .unwrap();
    let exported =
        cad_document_to_encoded_ocdraw(&document, CadToOcdrawOptions::default()).unwrap();
    let loaded = load_ocdraw_bytes(exported.encoded().bytes());
    let drawing = loaded.as_ref().ok().unwrap();
    assert_eq!(drawing.viewports().len(), 1, "{:?}", exported.diagnostics());
    assert_eq!(drawing.viewports()[0].view.height, 60.0);
    assert_eq!(drawing.viewports()[0].view.lens_length, Some(0.0));
    assert_eq!(
        drawing
            .paper_canvases()
            .iter()
            .find(|c| c.scope_id == drawing.owner_scope_id(drawing.viewports()[0].id).unwrap())
            .unwrap()
            .view
            .height,
        35.0
    );
    let imported = ocdraw_source_to_cad_document(drawing, OcdrawToCadOptions::default()).unwrap();
    let overall = imported
        .document()
        .entities()
        .find_map(|entity| match entity {
            EntityType::Viewport(viewport) if viewport.id == 1 => Some(viewport),
            _ => None,
        })
        .unwrap();
    assert_eq!(overall.view_height, 35.0);
    assert!(!imported.document().header.show_model_space);
    assert_eq!(
        imported.document().header.paper_space_block_handle,
        overall.common.owner_handle
    );
    let target = imported
        .document()
        .entities()
        .find_map(|e| {
            if let EntityType::Viewport(v) = e {
                if v.id > 1 {
                    Some(v)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(target.view_height, 60.0);
    assert_eq!(target.lens_length, 0.0);
    assert!(target.status.locked);
    assert_eq!(
        target.frozen_layers,
        vec![imported.document().layers.get("0").unwrap().handle]
    );
    let scope = drawing
        .scopes()
        .iter()
        .find(|s| s.id == drawing.owner_scope_id(drawing.viewports()[0].id).unwrap())
        .unwrap();
    let kinds = scope
        .entities
        .iter()
        .map(|id| {
            imported
                .document()
                .get_entity(imported.entity_mapping()[id])
                .unwrap()
                .as_entity()
                .entity_type()
        })
        .collect::<Vec<_>>();
    assert_eq!(kinds, vec!["LINE", "VIEWPORT", "LINE"]);
}

#[test]
fn continuous_linetype_names_use_cad_case_insensitive_semantics() {
    let mut source = CadDocument::new();
    source.layers.get_mut("0").unwrap().line_type = "CONTINUOUS".into();
    let mut line = opencadcodec::Line::from_coords(0., 0., 0., 1., 0., 0.);
    line.common.linetype = "continuous".into();
    source
        .add_entity(opencadcodec::EntityType::Line(line))
        .unwrap();
    let exported = cad_document_to_encoded_ocdraw(&source, CadToOcdrawOptions::default()).unwrap();
    assert!(
        exported.diagnostics().is_empty(),
        "{:?}",
        exported.diagnostics()
    );
    let readback = load_ocdraw_bytes(exported.encoded().bytes());
    assert_eq!(
        readback.as_ref().ok().unwrap().typed_layers()[0].line_pattern_id,
        ocdraw::ocdraw::LinePatternId(0)
    );
}

#[test]
fn authoritative_scope_order_survives_direct_dxf_and_dwg_exchange() {
    use ocdraw::ocdraw::{DrawingGeometry, ValidatedOcdraw};
    use std::{collections::BTreeMap, io::Cursor};
    fn projection(drawing: &ValidatedOcdraw) -> BTreeMap<String, Vec<String>> {
        let geometry = drawing
            .geometric_entities()
            .iter()
            .map(|e| (e.id(), e))
            .collect::<BTreeMap<_, _>>();
        drawing
            .scopes()
            .iter()
            .map(|scope| {
                let name = drawing
                    .typed_layouts()
                    .iter()
                    .find(|l| l.scope_id == scope.id)
                    .map(|l| l.name.clone())
                    .or_else(|| {
                        drawing
                            .block_definitions()
                            .iter()
                            .find(|b| b.scope_id == scope.id)
                            .map(|b| b.name.clone())
                    })
                    .unwrap();
                let records = scope
                    .entities
                    .iter()
                    .map(|id| match geometry[id].geometry() {
                        DrawingGeometry::Line { start, end } => format!("Line:{start:?}:{end:?}"),
                        DrawingGeometry::BlockInstance {
                            definition_scope_id,
                            transform,
                        } => format!(
                            "Insert:{}:{transform:?}",
                            drawing
                                .block_definitions()
                                .iter()
                                .find(|b| b.scope_id == *definition_scope_id)
                                .unwrap()
                                .name
                        ),
                        _ => panic!("unexpected fixture family"),
                    })
                    .collect();
                (name, records)
            })
            .collect()
    }
    let bytes = include_bytes!("../../../conformance/next/ocdraw/valid/ordered-scopes.ocdraw.json");
    let read = load_ocdraw_bytes(bytes);
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
    let expected = projection(drawing);
    let target = ocdraw_source_to_cad_document(drawing, OcdrawToCadOptions::default()).unwrap();
    for (format, document) in [
        ("CadDocument", target.document().clone()),
        (
            "DXF",
            opencadcodec::DxfReader::from_reader(Cursor::new(
                opencadcodec::DxfWriter::new(target.document())
                    .write_to_vec()
                    .unwrap(),
            ))
            .unwrap()
            .read()
            .unwrap(),
        ),
        (
            "DWG",
            opencadcodec::DwgReader::from_stream(Cursor::new(
                opencadcodec::DwgWriter::write_to_vec(target.document()).unwrap(),
            ))
            .read()
            .unwrap(),
        ),
    ] {
        let output = cad_document_to_encoded_ocdraw(&document, CadToOcdrawOptions::default())
            .unwrap_or_else(|e| panic!("{format}: {e:?}"));
        let returned = load_ocdraw_bytes(output.encoded().bytes());
        assert_eq!(
            returned
                .as_ref()
                .map(|_| OcdrawReadStatus::Valid)
                .unwrap_or_else(|e| e.status()),
            OcdrawReadStatus::Valid,
            "{:?}",
            returned
                .as_ref()
                .err()
                .map(|e| e.diagnostics())
                .unwrap_or_default()
        );
        assert_eq!(projection(returned.as_ref().ok().unwrap()), expected);
    }
}

#[test]
fn paper_layouts_match_the_source_without_bootstrap_layouts() {
    use ocdraw::ocdraw::{
        LayerDefinition, LineDefinition, OcdrawBuildOptions, OcdrawBuilder, RgbColor,
    };
    use opencadcodec::objects::ObjectType;
    use std::io::Cursor;
    for paper_names in [
        vec![],
        vec!["Sheet"],
        vec!["Layout1"],
        vec!["Sheet", "Layout1"],
    ] {
        let mut builder = OcdrawBuilder::new(OcdrawBuildOptions::new("layouts", "mm")).unwrap();
        builder.ensure_continuous_line_pattern().unwrap();
        let layer = builder
            .add_layer(LayerDefinition::new(
                "0",
                RgbColor::new(255, 255, 255),
                ocdraw::ocdraw::LinePatternId(0),
            ))
            .unwrap();
        builder
            .add_line(LineDefinition::new(
                layer,
                [-2.0, 0.0, 0.0],
                [-1.0, 0.0, 0.0],
            ))
            .unwrap();
        for (index, name) in paper_names.iter().enumerate() {
            let scope = builder.add_paper_layout(*name).unwrap();
            builder
                .add_line(
                    LineDefinition::new(
                        layer,
                        [index as f64, 0.0, 0.0],
                        [index as f64 + 1.0, 2.0, 0.0],
                    )
                    .in_scope(scope),
                )
                .unwrap();
        }
        let encoded = builder.finish().unwrap();
        let mut value: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
        value["layouts"].as_array_mut().unwrap().reverse();
        let loaded = load_ocdraw_bytes(&serde_json::to_vec(&value).unwrap());
        let imported = ocdraw_source_to_cad_document(
            loaded.as_ref().ok().unwrap(),
            OcdrawToCadOptions::default(),
        )
        .unwrap();
        for (format, document) in [
            ("CadDocument", imported.document().clone()),
            (
                "DXF",
                opencadcodec::DxfReader::from_reader(Cursor::new(
                    opencadcodec::DxfWriter::new(imported.document())
                        .write_to_vec()
                        .unwrap(),
                ))
                .unwrap()
                .read()
                .unwrap(),
            ),
            (
                "DWG",
                opencadcodec::DwgReader::from_stream(Cursor::new(
                    opencadcodec::DwgWriter::write_to_vec(imported.document()).unwrap(),
                ))
                .read()
                .unwrap(),
            ),
        ] {
            let mut layouts = document
                .objects
                .values()
                .filter_map(|object| match object {
                    ObjectType::Layout(layout) => Some(layout),
                    _ => None,
                })
                .collect::<Vec<_>>();
            layouts.sort_by_key(|layout| layout.tab_order);
            let names = layouts.iter().map(|l| l.name.as_str()).collect::<Vec<_>>();
            let expected = std::iter::once("Model")
                .chain(paper_names.iter().copied())
                .collect::<Vec<_>>();
            assert_eq!(names, expected, "{format}");
            let Some(ObjectType::Dictionary(dictionary)) = document
                .objects
                .get(&document.header.acad_layout_dict_handle)
            else {
                panic!("missing layout dictionary: {format}");
            };
            assert_eq!(dictionary.entries.len(), layouts.len(), "{format}");
            for layout in &layouts {
                assert!(
                    dictionary
                        .entries
                        .iter()
                        .any(|(name, handle)| name == &layout.name && *handle == layout.handle),
                    "{format}: {}",
                    layout.name
                );
            }
            for (index, name) in paper_names.iter().enumerate() {
                let layout = layouts.iter().find(|l| &l.name == name).unwrap();
                let lines = document
                    .entities()
                    .filter_map(|entity| match entity {
                        EntityType::Line(line)
                            if line.common.owner_handle == layout.block_record =>
                        {
                            Some(line)
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                assert_eq!(lines.len(), 1, "{format}: {name}");
                assert_eq!(lines[0].start, Vector3::new(index as f64, 0.0, 0.0));
            }
            if paper_names.len() > 1 && format == "DWG" {
                let record = document.block_records.get("*Paper_Space0").unwrap();
                let Some(EntityType::Block(marker)) =
                    document.get_entity(record.block_entity_handle)
                else {
                    panic!("missing paper marker");
                };
                assert_eq!(marker.name, record.name);
                // The name is fixed; DWG still gives the extra paper marker
                // the primary paper record as owner. Keep rejecting the conflict.
                assert_ne!(marker.common.owner_handle, record.handle);
                assert!(matches!(
                    cad_document_to_encoded_ocdraw(&document, CadToOcdrawOptions::default()),
                    Err(CadToOcdrawError::InvalidSourceStructure { .. })
                ));
            } else {
                let returned = cad_document_to_encoded_ocdraw(
                    &document,
                    CadToOcdrawOptions {
                        loss_policy: OcdrawLossPolicy::Reject,
                        ..CadToOcdrawOptions::default()
                    },
                )
                .unwrap();
                let readback = load_ocdraw_bytes(returned.encoded().bytes());
                let drawing = readback.as_ref().ok().unwrap();
                assert!(
                    drawing.block_definitions().is_empty(),
                    "{format}: reserved paper block became a definition"
                );
                assert_eq!(
                    drawing.scopes().len(),
                    drawing.typed_layouts().len(),
                    "{format}"
                );
                assert_eq!(
                    drawing
                        .typed_layouts()
                        .iter()
                        .map(|l| l.name.as_str())
                        .collect::<Vec<_>>(),
                    expected,
                    "{format}"
                );
            }
        }
    }
}

#[test]
fn fractional_planar_polyline_survives_actual_dxf_and_dwg_exchange() {
    use std::io::Cursor;
    let bytes =
        include_bytes!("../../../conformance/next/ocdraw/valid/fractional-polylines.ocdraw.json");
    let mut value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    value["streams"]
        .as_object_mut()
        .unwrap()
        .remove("spatialPolylineStream");
    value["scopes"][0]["entities"] = serde_json::json!([1]);
    let loaded = load_ocdraw_bytes(&serde_json::to_vec(&value).unwrap());
    let drawing = loaded.as_ref().ok().unwrap();
    let expected = drawing
        .geometric_entities()
        .iter()
        .map(|entity| entity.geometry().clone())
        .collect::<Vec<_>>();
    let imported = ocdraw_source_to_cad_document(drawing, OcdrawToCadOptions::default()).unwrap();
    for (format, document) in [
        (
            "DXF",
            opencadcodec::DxfReader::from_reader(Cursor::new(
                opencadcodec::DxfWriter::new(imported.document())
                    .write_to_vec()
                    .unwrap(),
            ))
            .unwrap()
            .read()
            .unwrap(),
        ),
        (
            "DWG",
            opencadcodec::DwgReader::from_stream(Cursor::new(
                opencadcodec::DwgWriter::write_to_vec(imported.document()).unwrap(),
            ))
            .read()
            .unwrap(),
        ),
    ] {
        let exported =
            cad_document_to_encoded_ocdraw(&document, CadToOcdrawOptions::default()).unwrap();
        let returned = load_ocdraw_bytes(exported.encoded().bytes());
        assert_eq!(
            returned
                .as_ref()
                .map(|_| OcdrawReadStatus::Valid)
                .unwrap_or_else(|e| e.status()),
            OcdrawReadStatus::Valid,
            "{format}: {:?}",
            returned
                .as_ref()
                .err()
                .map(|e| e.diagnostics())
                .unwrap_or_default()
        );
        let actual = returned
            .as_ref()
            .ok()
            .unwrap()
            .geometric_entities()
            .iter()
            .map(|entity| entity.geometry().clone())
            .collect::<Vec<_>>();
        assert_eq!(
            actual,
            expected,
            "{format}: kinds {:?}; diagnostics {:?}",
            document
                .entities()
                .map(|e| e.as_entity().entity_type())
                .collect::<Vec<_>>(),
            exported.diagnostics()
        );
    }
}

#[test]
fn reserved_paper_block_with_authored_metadata_is_not_discarded() {
    use ocdraw::ocdraw::{OcdrawBuildOptions, OcdrawBuilder};
    let encoded = OcdrawBuilder::new(OcdrawBuildOptions::new("model-only", "mm"))
        .unwrap()
        .finish()
        .unwrap();
    let loaded = load_ocdraw_bytes(encoded.bytes());
    let mut document =
        ocdraw_source_to_cad_document(loaded.as_ref().ok().unwrap(), OcdrawToCadOptions::default())
            .unwrap()
            .into_document();
    document
        .block_records
        .get_mut("*Paper_Space")
        .unwrap()
        .description = "Authored metadata".into();
    let exported =
        cad_document_to_encoded_ocdraw(&document, CadToOcdrawOptions::default()).unwrap();
    let returned = load_ocdraw_bytes(exported.encoded().bytes());
    let blocks = returned.as_ref().ok().unwrap().block_definitions();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].description, "Authored metadata");
}
