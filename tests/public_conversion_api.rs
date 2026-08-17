use ifccad::ifcdr::{
    AppearanceId, EntityId, IfccadLengthUnit, IfcdrEntityRef, IfcdrResourceRef, LayerId, Point2,
    ScopeId,
};

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
) -> (String, IfccadLengthUnit, Vec<EntityProjection>) {
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
    (
        resource.resource_id().to_owned(),
        resource.unit(),
        projected,
    )
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
