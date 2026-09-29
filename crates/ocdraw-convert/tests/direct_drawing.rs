use cadcodec::{CadDocument, Circle, Color, EntityType, Line, Point, Vector3};
use ocdraw::drawing::{load_drawing_bytes, DrawingLoadStatus};
use ocdraw_convert::{
    cad_document_to_drawing, cad_document_to_drawing_with_id, ocdraw_to_cad_document,
    ConversionLossPolicy, DirectExportError, ExportOptions, ImportOptions,
};

#[test]
fn direct_line_export_creates_a_standalone_drawing() {
    let mut source = CadDocument::new();
    let mut line = Line::from_coords(1.0, 2.0, 0.0, 3.0, 4.0, 0.0);
    line.common.color = Color::ByBlock;
    source.add_entity(EntityType::Line(line)).unwrap();
    let outcome = cad_document_to_drawing(&source, ExportOptions::default()).unwrap();
    let read = load_drawing_bytes(outcome.drawing().bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let value = read.validated_drawing().unwrap().as_value();
    assert_eq!(value["streams"]["lineStream"]["count"], 1);
    assert_eq!(value["streams"]["lineStream"]["colorMode"][0], "ByBlock");
    assert!(value["layers"]
        .as_array()
        .is_some_and(|layers| !layers.is_empty()));
    assert_eq!(value["drawingWorkspaceState"]["activeLayoutId"], 0);
    let imported =
        ocdraw_to_cad_document(read.validated_drawing().unwrap(), ImportOptions::default())
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
    let allow = cad_document_to_drawing(&source, ExportOptions::default()).unwrap();
    assert!(!allow.diagnostics().is_empty());
    let reject = cad_document_to_drawing(
        &source,
        ExportOptions {
            loss_policy: ConversionLossPolicy::Reject,
            ..ExportOptions::default()
        },
    );
    assert!(matches!(
        reject,
        Err(DirectExportError::LossRejected { .. })
    ));
}

#[test]
fn caller_can_assign_a_stable_drawing_identity() {
    let source = CadDocument::new();
    let outcome =
        cad_document_to_drawing_with_id(&source, "drawing-stable-42", ExportOptions::default())
            .unwrap();
    let read = load_drawing_bytes(outcome.drawing().bytes());
    assert_eq!(
        read.validated_drawing().unwrap().drawing_id(),
        "drawing-stable-42"
    );
}

#[test]
fn direct_import_preserves_paper_scope_and_mixed_appearance_modes() {
    use cadcodec::{LineWeight, Transparency};
    use ocdraw::drawing::{
        AppearanceSelection, DrawingBuilder, DrawingOptions, EntityAppearance, LayerDefinition,
        LineDefinition, RgbColor,
    };
    let mut builder = DrawingBuilder::new(DrawingOptions::new("paper", "mm")).unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new("0", RgbColor::new(255, 255, 255)))
        .unwrap();
    let paper = builder.add_paper_layout("Sheet").unwrap();
    let mut line = LineDefinition::new(layer, [0.0, 0.0, 0.0], [2.0, 3.0, 0.0]).in_scope(paper);
    line.appearance = EntityAppearance {
        color: AppearanceSelection::ByBlock,
        opacity: AppearanceSelection::Explicit(0.5),
        line_pattern: AppearanceSelection::ByLayer,
        line_weight: AppearanceSelection::Explicit(0.25),
    };
    builder.add_line(line).unwrap();
    let encoded = builder.finish().unwrap();
    let read = load_drawing_bytes(encoded.bytes());
    let imported =
        ocdraw_to_cad_document(read.validated_drawing().unwrap(), ImportOptions::default())
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
        cadcodec::objects::ObjectType::Layout(layout) if layout.name == "Sheet" && layout.block_record == line.common.owner_handle)));
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
    let exported = cad_document_to_drawing(&source, ExportOptions::default()).unwrap();
    let read = load_drawing_bytes(exported.drawing().bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let value = read.validated_drawing().unwrap().as_value();
    assert_eq!(value["streams"]["pointStream"]["count"], 1);
    assert_eq!(value["streams"]["circleStream"]["count"], 1);
    let imported =
        ocdraw_to_cad_document(read.validated_drawing().unwrap(), ImportOptions::default())
            .unwrap();
    assert!(imported.document().entities().any(|entity| matches!(entity,
        EntityType::Point(point) if point.location == Vector3::new(4.0,5.0,0.0))));
    assert!(imported.document().entities().any(|entity| matches!(entity,
        EntityType::Circle(circle) if circle.center == Vector3::new(10.0,20.0,0.0) && circle.radius == 3.0)));
}

#[test]
fn planar_and_spatial_polylines_keep_vertices_and_order() {
    use cadcodec::{LwPolyline, Vector2};
    let mut source = CadDocument::new();
    let mut planar = LwPolyline::from_points(vec![Vector2::new(0.0, 0.0), Vector2::new(2.0, 0.0)]);
    planar.vertices[0].bulge = 1.0;
    planar.is_closed = false;
    source.add_entity(EntityType::LwPolyline(planar)).unwrap();
    let spatial = cadcodec::entities::Polyline3D::from_points(vec![
        Vector3::new(1.0, 2.0, 3.0),
        Vector3::new(4.0, 5.0, 6.0),
    ]);
    source.add_entity(EntityType::Polyline3D(spatial)).unwrap();
    let exported = cad_document_to_drawing(&source, ExportOptions::default()).unwrap();
    let read = load_drawing_bytes(exported.drawing().bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let drawing = read.validated_drawing().unwrap();
    assert_eq!(
        drawing.as_value()["streams"]["planarPolylineStream"]["bulge"],
        serde_json::json!([1.0, 0.0])
    );
    assert_eq!(
        drawing.as_value()["streams"]["spatialPolylineStream"]["z"],
        serde_json::json!([3.0, 6.0])
    );
    let imported = ocdraw_to_cad_document(drawing, ImportOptions::default()).unwrap();
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
    use cadcodec::Vector2;
    let mut source = CadDocument::new();
    let mut planar = cadcodec::entities::Polyline2D::new();
    planar.add_vertex(cadcodec::entities::Vertex2D::from_point(Vector2::new(
        0.0, 0.0,
    )));
    planar.add_vertex(cadcodec::entities::Vertex2D::from_point(Vector2::new(
        3.0, 4.0,
    )));
    source.add_entity(EntityType::Polyline2D(planar)).unwrap();
    let spatial = cadcodec::entities::Polyline::from_points(vec![
        Vector3::new(1.0, 2.0, 3.0),
        Vector3::new(4.0, 5.0, 6.0),
    ]);
    source.add_entity(EntityType::Polyline(spatial)).unwrap();
    let exported = cad_document_to_drawing(&source, ExportOptions::default()).unwrap();
    let read = load_drawing_bytes(exported.drawing().bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let drawing = read.validated_drawing().unwrap().as_value();
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
    let exported = cad_document_to_drawing(&source, ExportOptions::default()).unwrap();
    let read = load_drawing_bytes(exported.drawing().bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let drawing = read.validated_drawing().unwrap();
    assert_eq!(drawing.as_value()["pointDisplay"]["form"]["glyph"], "cross");
    let imported = ocdraw_to_cad_document(drawing, ImportOptions::default()).unwrap();
    assert_eq!(imported.document().header.point_display_mode, 35);
    assert_eq!(imported.document().header.point_display_size, -5.0);
}

#[test]
fn layout_limits_and_scaling_roundtrip_directly() {
    let mut source = CadDocument::new();
    source.add_layout("Sheet").unwrap();
    source.header.paper_space_linetype_scaling = false;
    for object in source.objects.values_mut() {
        if let cadcodec::objects::ObjectType::Layout(layout) = object {
            if layout.name == "Sheet" {
                layout.flags |= 2;
                layout.min_limits = (1.0, 2.0);
                layout.max_limits = (100.0, 200.0);
            }
        }
    }
    let exported = cad_document_to_drawing(&source, ExportOptions::default()).unwrap();
    let read = load_drawing_bytes(exported.drawing().bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let drawing = read.validated_drawing().unwrap();
    let sheet = drawing
        .layouts()
        .iter()
        .find(|layout| layout["name"] == "Sheet")
        .unwrap();
    assert_eq!(sheet["limits"]["maxY"], 200.0);
    assert_eq!(sheet["limitsChecking"], true);
    assert_eq!(sheet["paperSpaceLinetypeScaling"], false);
    let imported = ocdraw_to_cad_document(drawing, ImportOptions::default()).unwrap();
    assert!(!imported.document().header.paper_space_linetype_scaling);
    assert!(imported
        .document()
        .objects
        .values()
        .any(|object| matches!(object,
        cadcodec::objects::ObjectType::Layout(layout) if layout.name == "Sheet"
            && layout.flags & 2 != 0 && layout.min_limits == (1.0, 2.0)
            && layout.max_limits == (100.0, 200.0))));
}

#[test]
fn configured_layout_plot_settings_roundtrip_directly() {
    let mut source = CadDocument::new();
    source.add_layout("Sheet").unwrap();
    for object in source.objects.values_mut() {
        if let cadcodec::objects::ObjectType::Layout(layout) = object {
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
                layout.plot_flags.plot_centered = true;
                layout.paper_size = "A4".into();
            }
        }
    }
    let exported = cad_document_to_drawing(&source, ExportOptions::default()).unwrap();
    let read = load_drawing_bytes(exported.drawing().bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let drawing = read.validated_drawing().unwrap();
    let sheet = drawing
        .layouts()
        .iter()
        .find(|layout| layout["name"] == "Sheet")
        .unwrap();
    assert_eq!(sheet["plotSettings"]["media"]["width"], 210.0);
    assert_eq!(sheet["plotSettings"]["media"]["mediaName"], "A4");
    let imported = ocdraw_to_cad_document(drawing, ImportOptions::default()).unwrap();
    assert!(imported
        .document()
        .objects
        .values()
        .any(|object| matches!(object,
        cadcodec::objects::ObjectType::Layout(layout) if layout.name == "Sheet"
            && layout.paper_width == 210.0 && layout.paper_height == 297.0
            && layout.plot_margin_left == 5.0 && layout.plot_margin_top == 8.0
            && layout.paper_size == "A4")));
}

#[test]
fn unused_named_ucs_roundtrips_directly() {
    let mut source = CadDocument::new();
    let mut ucs = cadcodec::Ucs::new("Grid A");
    ucs.handle = source.allocate_handle();
    ucs.origin = Vector3::new(1.0, 2.0, 3.0);
    ucs.elevation = 4.0;
    source.ucss.add(ucs).unwrap();
    let exported = cad_document_to_drawing(&source, ExportOptions::default()).unwrap();
    assert!(!exported.diagnostics().iter().flat_map(|diagnostic| diagnostic.reasons()).any(|reason|
        matches!(reason, ocdraw_convert::ExportLossReason::UnsupportedTableRecords { kind, .. } if kind == "ucss")));
    let read = load_drawing_bytes(exported.drawing().bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let drawing = read.validated_drawing().unwrap();
    assert_eq!(drawing.as_value()["ucsDefinitions"][0]["name"], "Grid A");
    let imported = ocdraw_to_cad_document(drawing, ImportOptions::default()).unwrap();
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
    use ocdraw::drawing::{
        DrawingBuilder, DrawingOptions, LayerDefinition, PointDefinition, RgbColor,
    };
    let mut builder = DrawingBuilder::new(DrawingOptions::new("rotated", "mm")).unwrap();
    let layer = builder
        .add_layer(LayerDefinition::new("0", RgbColor::new(0, 0, 0)))
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
    let read = load_drawing_bytes(&bytes);
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let imported =
        ocdraw_to_cad_document(read.validated_drawing().unwrap(), ImportOptions::default())
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
    let mut record = cadcodec::BlockRecord::new("Door");
    record.handle = source.allocate_handle();
    let owner = record.handle;
    source.block_records.add(record).unwrap();
    let mut line = Line::from_coords(0.0, 0.0, 0.0, 2.0, 0.0, 0.0);
    line.common.owner_handle = owner;
    source.add_entity(EntityType::Line(line)).unwrap();
    source
        .add_entity(EntityType::Insert(cadcodec::entities::Insert::new(
            "Door",
            Vector3::new(10.0, 0.0, 0.0),
        )))
        .unwrap();
    let exported = cad_document_to_drawing(&source, ExportOptions::default()).unwrap();
    let read = load_drawing_bytes(exported.drawing().bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let value = read.validated_drawing().unwrap().as_value();
    assert_eq!(value["blockDefinitions"][0]["name"], "Door");
    assert_eq!(value["streams"]["blockInstanceStream"]["count"], 1);
    let imported =
        ocdraw_to_cad_document(read.validated_drawing().unwrap(), ImportOptions::default())
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
    source
        .add_entity(EntityType::Arc(cadcodec::Arc::from_center_radius_angles(
            Vector3::new(2.0, 3.0, 0.0),
            5.0,
            0.25,
            1.25,
        )))
        .unwrap();
    let exported = cad_document_to_drawing(&source, ExportOptions::default()).unwrap();
    let read = load_drawing_bytes(exported.drawing().bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    assert_eq!(
        read.validated_drawing().unwrap().as_value()["streams"]["arcStream"]["sweepParameter"][0],
        1.0
    );
    let imported =
        ocdraw_to_cad_document(read.validated_drawing().unwrap(), ImportOptions::default())
            .unwrap();
    assert!(imported.document().entities().any(|entity| matches!(entity,
        EntityType::Arc(arc) if arc.center == Vector3::new(2.0,3.0,0.0) && arc.radius == 5.0 && arc.start_angle == 0.25 && arc.end_angle == 1.25)));
}

#[test]
fn full_and_partial_ellipses_roundtrip_directly() {
    let mut source = CadDocument::new();
    source
        .add_entity(EntityType::Ellipse(cadcodec::Ellipse::from_center_axes(
            Vector3::new(1.0, 2.0, 0.0),
            Vector3::new(3.0, 0.0, 0.0),
            0.5,
        )))
        .unwrap();
    let mut partial = cadcodec::Ellipse::from_center_axes(
        Vector3::new(10.0, 2.0, 0.0),
        Vector3::new(0.0, 4.0, 0.0),
        0.25,
    );
    partial.start_parameter = 0.5;
    partial.end_parameter = 1.5;
    source.add_entity(EntityType::Ellipse(partial)).unwrap();
    let exported = cad_document_to_drawing(&source, ExportOptions::default()).unwrap();
    let read = load_drawing_bytes(exported.drawing().bytes());
    assert_eq!(
        read.status(),
        DrawingLoadStatus::Valid,
        "{:?}",
        read.diagnostics()
    );
    let value = read.validated_drawing().unwrap().as_value();
    assert_eq!(value["streams"]["ellipseStream"]["count"], 1);
    assert_eq!(value["streams"]["ellipseArcStream"]["count"], 1);
    let imported =
        ocdraw_to_cad_document(read.validated_drawing().unwrap(), ImportOptions::default())
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
