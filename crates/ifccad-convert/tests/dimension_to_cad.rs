use ifccad::*;
use ifccad_convert::*;
use opencadcodec::EntityType;
#[test]
fn full_width_native_dimension_relations_are_bound_after_allocation() {
    let read = load_ifccad_bytes(
        include_bytes!("../../../conformance/next/ifccad/valid/dimension-linear-aligned.ifcx"),
        Default::default(),
    )
    .unwrap();
    let result = ifccad_source_to_cad_document(&read, Default::default()).unwrap();
    assert_eq!(
        result
            .document()
            .entities()
            .filter(|e| matches!(e, EntityType::Dimension(_)))
            .count(),
        2
    );
    assert!(result
        .mappings()
        .entities
        .cad_handle(9007199254740993)
        .is_some());
    assert!(result
        .diagnostics()
        .iter()
        .any(|d| d.message.contains("picture")));
}

#[test]
fn sparse_native_dimension_containers_are_not_foreign_ifcx_data() {
    let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../conformance/next/ifccad/valid/dimension-linear-aligned.ifcx"
    ))
    .unwrap();
    for node in raw["data"].as_array_mut().unwrap() {
        let Some(a) = node
            .get_mut("attributes")
            .and_then(serde_json::Value::as_object_mut)
        else {
            continue;
        };
        if let Some(s) = a
            .get_mut("ifccad::dimensionStyle")
            .and_then(serde_json::Value::as_object_mut)
        {
            s.remove("typeOverrides");
        }
        if let Some(d) = a
            .get_mut("ifccad::dimension")
            .and_then(serde_json::Value::as_object_mut)
        {
            d.insert("overrides".into(), serde_json::json!({}));
        }
    }
    let read = load_ifccad_bytes(&serde_json::to_vec(&raw).unwrap(), Default::default()).unwrap();
    let result = ifccad_source_to_cad_document(&read, Default::default()).unwrap();
    assert!(!result
        .diagnostics()
        .iter()
        .any(|d| d.code == "foreign-ifcx"));
}

#[test]
fn independent_signed_nested_occurrence_excess_is_fatal() {
    let read = load_ifccad_bytes(
        include_bytes!("../../../conformance/next/ifccad/valid/dimension-linear-aligned.ifcx"),
        Default::default(),
    )
    .unwrap();
    let mut doc = read.document().clone();
    let mut leaf = doc
        .model
        .entities
        .iter()
        .filter_map(IfccadEntity::as_native)
        .find(|e| matches!(e.kind, IfccadEntityKind::Dimension(_)))
        .unwrap()
        .clone();
    if let IfccadEntityKind::Dimension(d) = &mut leaf.kind {
        d.associations.clear();
        d.placement.origin = [1e8, 0.0, 0.0];
        d.definition = ifccad::dimension::DimensionDefinition::Aligned {
            first_point: [0.0; 2],
            second_point: [3e-12, 0.0],
            dimension_line_point: [0.0, 1e-12],
            extension_direction_angle: None,
        };
    }
    let mut inner = leaf.clone();
    inner.id = 9007199254741010;
    inner.kind = IfccadEntityKind::BlockInstance {
        definition_id: 1,
        transform: IfccadBlockTransform {
            placement: IfccadPlacement {
                origin: [0.0; 3],
                x_axis: [1.0, 0.0, 0.0],
                y_axis: [0.0, 1.0, 0.0],
            },
            rotation: 0.0,
            scale: [-1e4, 2.0, 3.0],
        },
    };
    let mut root = inner.clone();
    root.id += 1;
    if let IfccadEntityKind::BlockInstance {
        definition_id,
        transform,
    } = &mut root.kind
    {
        *definition_id = 2;
        transform.scale = [1e4, -2.0, 3.0];
    }
    let block = |id, name: &str, entities| IfccadBlockDefinition {
        id,
        name: name.into(),
        description: String::new(),
        anonymous: false,
        explodable: true,
        uniform_scaling: false,
        insertion_unit: "mm".into(),
        base_point: [0.0; 3],
        entities,
        bounds: None,
        bounds_quality: None,
    };
    doc.model.entities = vec![IfccadEntity::Native(root)];
    doc.blocks = vec![
        block(1, "Inner", vec![IfccadEntity::Native(leaf)]),
        block(2, "Outer", vec![IfccadEntity::Native(inner)]),
    ];
    doc.id_counters.next_entity_id = 9007199254741012;
    doc.id_counters.next_block_id = 3;
    doc.model.bounds = None;
    doc.model.bounds_quality = None;
    recompute_ifccad_document_bounds(&mut doc).unwrap();
    for loss_policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        assert!(matches!(
            ifccad_document_to_cad_document(
                &doc,
                IfccadToCadOptions {
                    loss_policy,
                    ..Default::default()
                }
            ),
            Err(IfccadConversionError::Geometry(_))
        ));
    }
}
