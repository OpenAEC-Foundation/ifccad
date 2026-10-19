#[path = "support/dimension.rs"]
mod support;
use ocdraw::{dimension::*, ocdraw::*};

#[test]
fn native_dimension_uses_independent_style_domain_and_shared_entity_identity() {
    let mut d = support::drawing();
    validate_ocdraw_document(&d).unwrap();
    assert_eq!(
        evaluate_dimension(&d.dimension_entities[0].definition).unwrap(),
        DimensionMeasurement::Distance(4000.0)
    );
    d.next_dimension_style_id = 0;
    assert!(validate_ocdraw_document(&d).is_err());
    d.next_dimension_style_id = 10;
    d.dimension_styles[0].id = OcdrawDimensionStyleId(10);
    assert!(validate_ocdraw_document(&d).is_err());
}

#[test]
fn source_edit_is_not_synchronization_and_detach_preserves_definition() {
    let mut d = support::drawing();
    let source = d.dimension_entities[0].id;
    d.dimension_entities[0]
        .associations
        .push(OcdrawDimensionAssociation {
            role: DimensionRole::FirstPoint,
            source_entity_id: source,
            selector: DimensionPointSelector::Start,
        });
    assert!(
        validate_ocdraw_document(&d).is_err(),
        "Dimension is not a supported point-source line"
    );
    d.dimension_entities[0].associations.clear();
    validate_ocdraw_document(&d).unwrap();
    assert_eq!(
        evaluate_dimension(&d.dimension_entities[0].definition).unwrap(),
        DimensionMeasurement::Distance(4000.0)
    );
}

#[test]
fn sparse_overrides_resolve_without_rewriting_authored_presence() {
    let mut d = support::drawing();
    d.dimension_entities[0].overrides.text.height = Some(2.5);
    d.dimension_styles[0].settings.text.height = 3.5;
    validate_ocdraw_document(&d).unwrap();
    let e = &d.dimension_entities[0];
    let style = &d.dimension_styles[0];
    assert_eq!(
        resolve_dimension_settings(
            &style.settings,
            e.definition.kind(),
            &style.type_overrides,
            &e.overrides
        )
        .text
        .height,
        2.5
    );
    assert_eq!(e.overrides.text.height, Some(2.5));
}

#[test]
fn duplicate_roles_and_unknown_styles_are_invalid() {
    let mut d = support::drawing();
    d.dimension_entities[0].style_id = OcdrawDimensionStyleId(77);
    assert!(validate_ocdraw_document(&d).is_err());
    d.dimension_entities[0].style_id = OcdrawDimensionStyleId(0);
    d.dimension_styles.push(d.dimension_styles[0].clone());
    assert!(validate_ocdraw_document(&d).is_err());
}

#[test]
fn forward_same_owner_relation_keeps_stored_points_after_source_edit() {
    let mut d = support::drawing();
    let layer = d.layers[0].id;
    d.geometric_entities.push(DrawingGeometricEntity {
        id: 2,
        layer_id: layer,
        visible: true,
        appearance: Default::default(),
        geometry: DrawingGeometry::Line {
            start: [0.0, 0.0, 0.0],
            end: [4000.0, 0.0, 0.0],
        },
    });
    d.scopes[0].entities.push(2);
    d.next_entity_id = 3;
    d.dimension_entities[0].associations = vec![
        OcdrawDimensionAssociation {
            role: DimensionRole::FirstPoint,
            source_entity_id: 2,
            selector: DimensionPointSelector::Start,
        },
        OcdrawDimensionAssociation {
            role: DimensionRole::SecondPoint,
            source_entity_id: 2,
            selector: DimensionPointSelector::End,
        },
    ];
    validate_ocdraw_document(&d).unwrap();
    if let DrawingGeometry::Line { end, .. } = &mut d.geometric_entities[0].geometry {
        *end = [4500.0, 0.0, 0.0];
    }
    validate_ocdraw_document(&d).unwrap();
    assert_eq!(
        evaluate_dimension(&d.dimension_entities[0].definition).unwrap(),
        DimensionMeasurement::Distance(4000.0)
    );
    let bytes = encode_ocdraw_document(&d).unwrap();
    let loaded = load_ocdraw_bytes(bytes.bytes()).unwrap();
    assert_eq!(
        loaded.dimension_entities()[0].associations,
        d.dimension_entities[0].associations
    );
    d.geometric_entities.clear();
    d.scopes[0].entities.retain(|id| *id != 2);
    assert!(validate_ocdraw_document(&d).is_err());
    d.dimension_entities[0].associations.clear();
    validate_ocdraw_document(&d).unwrap();
}

#[test]
fn builder_dimension_allocation_is_atomic_and_roundtrips() {
    use ocdraw::text::*;
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("dimension-builder", "mm")).unwrap();
    let p = b.ensure_continuous_line_pattern().unwrap();
    let l = b
        .add_layer(LayerDefinition::new(
            "Dimensions",
            RgbColor::new(1, 2, 3),
            p,
        ))
        .unwrap();
    let text = b
        .add_text_style(TextStyleDefinition {
            name: "Text".into(),
            properties: TextStyleProperties::new(FontRequest::family("Requested face")),
        })
        .unwrap();
    let style = b
        .add_dimension_style(DimensionStyleDefinition::new("Building", text))
        .unwrap();
    let definition = DimensionDefinition::Linear {
        first_point: [0.0, 0.0],
        second_point: [3.0, 4.0],
        dimension_line_point: [0.0, 5.0],
        direction_angle: 0.0,
        extension_direction_angle: None,
    };
    let mut bad = DimensionEntityDefinition::new(l, style, definition.clone());
    bad.overrides.text.height = Some(0.0);
    assert!(b.add_dimension(bad).is_err());
    assert_eq!(
        b.add_dimension(DimensionEntityDefinition::new(l, style, definition))
            .unwrap(),
        1
    );
    let bytes = b.finish().unwrap();
    let loaded = load_ocdraw_bytes(bytes.bytes()).unwrap();
    assert_eq!(loaded.dimension_entities().len(), 1);
}

#[test]
fn local_marker_render_cycles_are_rejected_even_in_unused_definitions() {
    let mut d = support::drawing();
    d.block_definitions.push(DrawingBlockDefinition {
        scope_id: 1,
        name: "Marker".into(),
        base_point: [0.0; 3],
        description: String::new(),
        anonymous: false,
        insertion_unit: "unitless".into(),
        explodable: true,
        uniform_scaling: false,
    });
    let mut scope = d.scopes[0].clone();
    scope.id = 1;
    scope.kind = DrawingScopeKind::Block;
    d.scopes[0].entities.clear();
    d.scopes.push(scope);
    d.dimension_styles[0].settings.arrows.first = DimensionMarker::LocalBlock { block: 1 };
    assert!(validate_ocdraw_document(&d)
        .unwrap_err()
        .diagnostics()
        .iter()
        .any(|e| e.code == "DIMENSION_BLOCK_CYCLE"));
}
