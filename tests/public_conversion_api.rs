use ifccad::conformance::bundled_conformance_root;
use ifccad::ifcdr::{
    AppearanceId, EntityId, IfcdrEntityRef, IfcdrLengthUnit, IfcdrResourceRef, LayerId, Point2,
    ScopeId,
};
use ifccad::package::{
    load_directory_package, AppearanceProperty, DrawingLayoutKind, LinePatternRef,
};
use ifccad::ResourceId;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
struct EntityProjection {
    entity_id: EntityId,
    scope_id: ScopeId,
    layer_id: Option<LayerId>,
    appearance_id: Option<AppearanceId>,
    visible: Option<bool>,
    points: Vec<Point2>,
    unmodeled_schema: Option<String>,
}

#[allow(dead_code)]
fn project_ifcdr_entities(
    resource: IfcdrResourceRef<'_>,
    scope_id: ScopeId,
) -> (ResourceId, IfcdrLengthUnit, Vec<EntityProjection>) {
    let projected = resource
        .entities(scope_id)
        .map(|entity| match entity {
            IfcdrEntityRef::Line(line) => EntityProjection {
                entity_id: line.entity_id(),
                scope_id: line.scope_id(),
                layer_id: Some(line.layer_id()),
                appearance_id: Some(line.appearance_id()),
                visible: Some(line.visible()),
                points: vec![line.start(), line.end()],
                unmodeled_schema: None,
            },
            IfcdrEntityRef::Polyline(polyline) => EntityProjection {
                entity_id: polyline.entity_id(),
                scope_id: polyline.scope_id(),
                layer_id: Some(polyline.layer_id()),
                appearance_id: Some(polyline.appearance_id()),
                visible: Some(polyline.visible()),
                points: polyline.points().collect(),
                unmodeled_schema: None,
            },
            IfcdrEntityRef::Unmodeled(entity) => EntityProjection {
                entity_id: entity.entity_id(),
                scope_id: entity.scope_id(),
                layer_id: None,
                appearance_id: None,
                visible: None,
                points: Vec::new(),
                unmodeled_schema: Some(entity.schema_id().to_owned()),
            },
        })
        .collect();
    (resource.resource_id().clone(), resource.unit(), projected)
}

fn require_ifcdr_unit(unit: IfcdrLengthUnit) -> IfcdrLengthUnit {
    unit
}

#[test]
fn public_ifcdr_unit_retains_resource_context() {
    assert_eq!(
        require_ifcdr_unit(IfcdrLengthUnit::Millimetre),
        IfcdrLengthUnit::Millimetre
    );
}

#[test]
fn public_ifcdr_types_have_value_semantics() {
    fn require_copy<T: Copy>() {}
    fn require_eq<T: Eq>() {}

    require_copy::<EntityId>();
    require_copy::<ScopeId>();
    require_copy::<LayerId>();
    require_copy::<AppearanceId>();
    require_copy::<Point2>();
    require_eq::<EntityId>();
    require_eq::<ScopeId>();
    require_eq::<LayerId>();
    require_eq::<AppearanceId>();
}

#[test]
fn validated_package_exposes_converter_inputs_without_raw_json() {
    let root = bundled_conformance_root()
        .join("packages")
        .join("valid")
        .join("minimal-no-preservation");
    let outcome = load_directory_package(root).expect("inspect bundled package");
    let package = outcome
        .validated_package()
        .expect("fixture is strictly valid");

    assert_eq!(package.drawing_sets().count(), 1);
    assert_eq!(package.drawings().count(), 1);
    let drawing = package.drawings().next().expect("one drawing");
    assert_eq!(drawing.path(), "drawing-main");

    let representation = drawing.representation();
    assert_eq!(representation.path(), "representation-modelspace-main");
    assert_eq!(representation.role(), "modelspace");
    assert_eq!(
        representation.resource_id(),
        &ResourceId::new("geometry-modelspace-main").unwrap()
    );
    assert_eq!(representation.external_uri(), Some("drawing.ifcdr.json"));
    assert_eq!(
        representation.resource().resource_id(),
        representation.resource_id()
    );

    let layout = drawing.layouts().next().expect("one layout");
    assert_eq!(layout.name(), "Model");
    assert_eq!(layout.kind(), DrawingLayoutKind::Model);
    assert_eq!(layout.representation().path(), representation.path());
    let scope = layout.scope();
    assert_eq!(scope.name(), "ModelSpace");

    assert_eq!(representation.layers().len(), 2);
    let wall = representation
        .layer(LayerId::from(1))
        .expect("A-WALL layer");
    assert_eq!(wall.name(), "A-WALL");
    assert!(wall.visible());
    let wall_appearance = wall.appearance().expect("layer appearance");
    assert_eq!(wall_appearance.name(), "Dashed Red");

    let default_solid = representation
        .appearance(AppearanceId::from(2))
        .expect("default appearance binding");
    let AppearanceProperty::Explicit(color) = default_solid.color() else {
        panic!("default color must be explicit");
    };
    assert_eq!(color.rgb().components(), [0, 0, 0]);
    let indexed = color.indexed().expect("indexed identity");
    assert_eq!(indexed.system(), "ACI");
    assert_eq!(indexed.index(), 7);
    assert_eq!(default_solid.opacity(), AppearanceProperty::Explicit(1.0));
    assert_eq!(
        default_solid.line_pattern(),
        AppearanceProperty::Explicit(LinePatternRef::Name("continuous"))
    );
    assert_eq!(
        default_solid.line_weight(),
        AppearanceProperty::Explicit(0.25)
    );

    let dashed_red = representation
        .appearance(AppearanceId::from(3))
        .expect("red appearance binding");
    let AppearanceProperty::Explicit(color) = dashed_red.color() else {
        panic!("red color must be explicit");
    };
    assert_eq!(color.rgb().components(), [255, 0, 0]);
    let named = color.named().expect("named identity");
    assert_eq!(named.catalog(), "RAL");
    assert_eq!(named.name(), "Traffic red");

    let by_layer = representation
        .appearance(AppearanceId::from(0))
        .expect("ByLayer binding");
    assert!(matches!(by_layer.color(), AppearanceProperty::ByLayer));
    let by_block = representation
        .appearance(AppearanceId::from(1))
        .expect("ByBlock binding");
    assert!(matches!(
        by_block.line_weight(),
        AppearanceProperty::ByBlock
    ));

    let projections = project_ifcdr_entities(representation.resource(), scope.id()).2;
    assert_eq!(
        projections
            .iter()
            .map(|entity| entity.entity_id.get())
            .collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
    assert_eq!(
        representation
            .layer(projections[1].layer_id.expect("modeled layer"))
            .expect("resolved entity layer")
            .name(),
        "A-WALL"
    );
}
