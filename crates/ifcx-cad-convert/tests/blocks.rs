mod common;
use common::*;
use ifcx_cad_convert::*;
use ocdraw::ifcx_cad::*;

#[test]
fn nested_blocks_keep_shared_targets_and_order() {
    let source = nested([1., 2., 0.]);
    let cad = to_cad(&validated(&source)).unwrap();
    let back = from_cad(cad.document(), metadata()).unwrap();
    let target = back.validated_ifcx().document();
    assert_eq!(source.blocks.len(), target.blocks.len());
    for b in &source.blocks {
        let h = cad.mappings().blocks.cad_handle(b.id).unwrap();
        let id = back.mappings().blocks.ifcx_id(h).unwrap();
        let t = target.blocks.iter().find(|b| b.id == id).unwrap();
        assert_eq!(
            (&b.name, b.base_point, &b.insertion_unit),
            (&t.name, t.base_point, &t.insertion_unit)
        );
        assert_eq!(b.entities.len(), t.entities.len());
        compare(&b.entities, &t.entities, &cad, &back);
    }
    compare(&source.model.entities, &target.model.entities, &cad, &back);
}
fn compare(
    a: &[IfcxCadEntity],
    b: &[IfcxCadEntity],
    cad: &IfcxCadToCadOutcome,
    back: &CadToIfcxCadOutcome,
) {
    for (a, b) in a.iter().zip(b) {
        assert_eq!(
            cad.mappings().entities.cad_handle(a.id),
            back.mappings().entities.cad_handle(b.id)
        );
        assert_eq!(a.appearance, b.appearance);
        match (&a.kind, &b.kind) {
            (
                IfcxCadEntityKind::BlockInstance {
                    definition_id: a,
                    transform: x,
                },
                IfcxCadEntityKind::BlockInstance {
                    definition_id: b,
                    transform: y,
                },
            ) => {
                assert_eq!(x, y);
                assert_eq!(
                    cad.mappings().blocks.cad_handle(*a),
                    back.mappings().blocks.cad_handle(*b)
                );
            }
            (a, b) => assert_eq!(a, b),
        }
    }
}
#[test]
fn tiny_scale_clamping_is_rejected_for_empty_block() {
    let mut d = nested([0.; 3]);
    d.model.entities = vec![instance(1, 4, [0.; 3])];
    let IfcxCadEntityKind::BlockInstance { transform, .. } = &mut d.model.entities[0].kind else {
        panic!()
    };
    transform.scale[0] = 1e-14;
    assert!(
        matches!(to_cad(&validated(&d)),Err(IfcxCadConversionError::Unsupported(i)) if i.iter().any(|d|d.code=="scale-clamped"))
    );
}
#[test]
fn invalid_block_graphs_are_fatal_even_when_unused() {
    let c = to_cad(&validated(&nested([0.; 3])))
        .unwrap()
        .into_document();
    let mut missing = c.clone();
    let h = missing
        .block_records
        .get("*Model_Space")
        .unwrap()
        .entity_handles[0];
    let cadcodec::EntityType::Insert(i) = missing.get_entity_mut(h).unwrap() else {
        panic!()
    };
    i.block_name = "Missing".into();
    assert!(matches!(
        from_cad(&missing, metadata()),
        Err(IfcxCadConversionError::InvalidStructure(_))
    ));
    // Test the cycle independently of model occurrences.
    let mut d = nested([0.; 3]);
    d.model.entities.clear();
    let mut cycle = to_cad(&validated(&d)).unwrap().into_document();
    let mut i = cadcodec::entities::Insert::new("Outer", cadcodec::Vector3::ZERO);
    i.common.owner_handle = cycle.block_records.get("Inner").unwrap().handle;
    cycle.add_entity(cadcodec::EntityType::Insert(i)).unwrap();
    assert!(matches!(
        from_cad(&cycle, metadata()),
        Err(IfcxCadConversionError::InvalidStructure(_))
    ));
    let mut owners = c.clone();
    let h = owners.block_records.get("Outer").unwrap().entity_handles[0];
    owners
        .block_records
        .get_mut("Inner")
        .unwrap()
        .entity_handles
        .push(h);
    assert!(matches!(
        from_cad(&owners, metadata()),
        Err(IfcxCadConversionError::InvalidStructure(_))
    ));
    let mut names = c.clone();
    let mut b = names.block_records.get("Inner").unwrap().clone();
    b.handle = names.allocate_handle();
    names.block_records.add_allow_duplicate(b);
    assert!(matches!(
        from_cad(&names, metadata()),
        Err(IfcxCadConversionError::InvalidStructure(_))
    ));
}
#[test]
fn arrays_attributes_and_block_metadata_are_diagnosed() {
    for field in 0..5 {
        let mut c = to_cad(&validated(&nested([0.; 3])))
            .unwrap()
            .into_document();
        let h = c.block_records.get("*Model_Space").unwrap().entity_handles[0];
        if field < 2 {
            let cadcodec::EntityType::Insert(i) = c.get_entity_mut(h).unwrap() else {
                panic!()
            };
            if field == 0 {
                i.row_count = 2;
            } else {
                i.attributes.push(cadcodec::entities::AttributeEntity::new(
                    "tag".into(),
                    "value".into(),
                ));
            }
        } else if field == 4 {
            let mut dynamic = cadcodec::objects::DynamicBlockObject::new(
                "BLOCKVISIBILITYPARAMETER",
                "AcDbBlockVisibilityParameter",
            );
            dynamic.handle = c.allocate_handle();
            dynamic.owner = c.block_records.get("Inner").unwrap().handle;
            c.objects.insert(
                dynamic.handle,
                cadcodec::objects::ObjectType::DynamicBlock(dynamic),
            );
        } else {
            let b = c.block_records.get_mut("Inner").unwrap();
            if field == 2 {
                b.flags.is_xref = true;
            } else {
                b.description = "authored".into();
            }
        }
        assert!(matches!(
            from_cad(&c, metadata()),
            Err(IfcxCadConversionError::Unsupported(_))
        ));
    }
}
#[test]
fn large_ids_and_renumbered_handles_keep_relations() {
    let d = nested([0.; 3]);
    let original = to_cad(&validated(&d)).unwrap();
    let mut c = original.document().clone();
    let owner = c.header.model_space_block_handle;
    // Rebuild model entities under unrelated large handles in the same order.
    let handles = c
        .block_records
        .get("*Model_Space")
        .unwrap()
        .entity_handles
        .clone();
    c.block_records
        .get_mut("*Model_Space")
        .unwrap()
        .entity_handles
        .clear();
    for h in handles {
        let mut e = c.get_entity(h).unwrap().clone();
        c.remove_entity(h);
        e.common_mut().handle = cadcodec::Handle::new(h.value() + 9007199254740993);
        e.common_mut().owner_handle = owner;
        c.add_entity(e).unwrap();
    }
    let back = from_cad(&c, metadata()).unwrap();
    assert!(original
        .mappings()
        .entities
        .cad_handle(9007199254740993)
        .is_some());
    let entities = &back.validated_ifcx().document().model.entities;
    assert_eq!(entities.len(), 2);
    for e in entities {
        let h = back.mappings().entities.cad_handle(e.id).unwrap();
        assert!(h.value() > 9007199254740993);
        let IfcxCadEntityKind::BlockInstance { definition_id, .. } = e.kind else {
            panic!()
        };
        assert_eq!(
            back.mappings().blocks.cad_handle(definition_id),
            Some(c.block_records.get("Outer").unwrap().handle)
        );
    }
}
