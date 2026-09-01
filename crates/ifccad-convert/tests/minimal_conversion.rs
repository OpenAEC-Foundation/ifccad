use ifccad::conformance::bundled_conformance_root;
use ifccad::ifcdr::IfcdrEntityRef;
use ifccad::package::load_directory_package;
use ifccad_convert::cadcodec::{
    Color, DxfVersion, EntityType, LineWeight, Transparency, Vector2, Vector3,
};
use ifccad_convert::convert_drawing;

#[test]
fn converts_minimal_model_drawing_with_layers_and_entities() {
    let root = bundled_conformance_root()
        .join("packages")
        .join("valid")
        .join("minimal-no-preservation");
    let inspected = load_directory_package(root).expect("load bundled package");
    let package = inspected
        .validated_package()
        .expect("strictly valid package");
    let drawing = package.drawings().next().expect("one drawing");

    let outcome = convert_drawing(drawing).expect("convert drawing layers");

    assert_eq!(outcome.document().version, DxfVersion::AC1032);
    assert_eq!(outcome.document().header.insertion_units, 6);
    assert_eq!(outcome.document().header.measurement, 1);

    let layer_0 = outcome.document().layers.get("0").expect("layer 0");
    assert_eq!(layer_0.color, Color::Index(7));
    assert_eq!(layer_0.line_type, "Continuous");
    assert_eq!(layer_0.line_weight, LineWeight::W0_25);
    assert!(layer_0.is_visible());

    let walls = outcome
        .document()
        .layers
        .get("A-WALL")
        .expect("A-WALL layer");
    assert_eq!(walls.color, Color::from_rgb(255, 0, 0));
    assert_eq!(walls.color_name.as_deref(), Some("Traffic red"));
    assert_eq!(walls.book_name.as_deref(), Some("RAL"));
    assert_eq!(walls.line_type, "Dashed");
    assert_eq!(walls.line_weight, LineWeight::W0_18);
    assert!(walls.is_visible());
    assert!(outcome.document().line_types.contains("Dashed"));

    let entities = outcome.document().entities().collect::<Vec<_>>();
    assert_eq!(entities.len(), 4);
    let EntityType::Line(first_line) = entities[0] else {
        panic!("first entity must be a line");
    };
    assert_eq!(first_line.start, Vector3::new(0.0, 0.0, 0.0));
    assert_eq!(first_line.end, Vector3::new(5.0, 5.0, 0.0));
    assert_eq!(first_line.common.layer, "0");
    assert!(!first_line.common.invisible);

    let EntityType::Line(second_line) = entities[1] else {
        panic!("second entity must be a line");
    };
    assert_eq!(second_line.start, Vector3::new(10.0, 0.0, 0.0));
    assert_eq!(second_line.end, Vector3::new(10.0, 8.0, 0.0));
    assert_eq!(second_line.common.layer, "A-WALL");

    let EntityType::LwPolyline(closed_polyline) = entities[2] else {
        panic!("third entity must be a lightweight polyline");
    };
    assert!(closed_polyline.is_closed);
    assert_eq!(
        closed_polyline
            .vertices
            .iter()
            .map(|vertex| vertex.location)
            .collect::<Vec<_>>(),
        [
            Vector2::new(0.0, 0.0),
            Vector2::new(10.0, 0.0),
            Vector2::new(10.0, 5.0),
            Vector2::new(0.0, 5.0),
        ]
    );
    assert_eq!(closed_polyline.common.layer, "A-WALL");

    let EntityType::LwPolyline(open_polyline) = entities[3] else {
        panic!("fourth entity must be a lightweight polyline");
    };
    assert!(!open_polyline.is_closed);
    assert_eq!(
        open_polyline
            .vertices
            .iter()
            .map(|vertex| vertex.location)
            .collect::<Vec<_>>(),
        [
            Vector2::new(20.0, 10.0),
            Vector2::new(25.0, 15.0),
            Vector2::new(30.0, 10.0),
        ]
    );
    assert_eq!(open_polyline.common.layer, "0");
    assert_eq!(open_polyline.common.color, Color::from_rgb(255, 0, 0));
    assert_eq!(
        open_polyline.common.color_name.as_deref(),
        Some("RAL$Traffic red")
    );
    assert_eq!(open_polyline.common.linetype, "Dashed");
    assert_eq!(open_polyline.common.line_weight, LineWeight::W0_18);
    assert_eq!(open_polyline.common.transparency, Transparency::OPAQUE);

    let layout = drawing.layouts().next().expect("one layout");
    let source_ids = drawing
        .representation()
        .resource()
        .entities(layout.scope().id())
        .map(|entity| match entity {
            IfcdrEntityRef::Line(line) => line.entity_id(),
            IfcdrEntityRef::Polyline(polyline) => polyline.entity_id(),
            IfcdrEntityRef::Unmodeled(entity) => entity.entity_id(),
        })
        .collect::<Vec<_>>();
    assert_eq!(outcome.entity_mapping().len(), 4);
    for source_id in source_ids {
        let handle = outcome
            .entity_mapping()
            .target_handle(source_id)
            .expect("converted source ID");
        assert!(handle.is_valid());
        assert!(outcome.document().get_entity(handle).is_some());
    }
    assert!(outcome.diagnostics().is_empty());
}
