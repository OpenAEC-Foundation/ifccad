mod common;
use common::*;
use ifccad_convert::*;
use ocdraw::ifccad::*;

#[test]
fn nested_blocks_keep_shared_targets_and_order() {
    let source = nested([1., 2., 0.]);
    let cad = to_cad(&validated(&source)).unwrap();
    let back = from_cad(cad.document(), metadata()).unwrap();
    let target = back.validated_source().document();
    assert_eq!(source.blocks.len(), target.blocks.len());
    for b in &source.blocks {
        let h = cad.mappings().blocks.cad_handle(b.id).unwrap();
        let id = back.mappings().blocks.ifccad_id(h).unwrap();
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
    a: &[IfccadEntity],
    b: &[IfccadEntity],
    cad: &IfccadToCadOutcome,
    back: &CadToEncodedIfccadOutcome,
) {
    for (a, b) in a.iter().zip(b) {
        assert_eq!(
            cad.mappings().entities.cad_handle(a.id()),
            back.mappings().entities.cad_handle(b.id())
        );
        assert_appearance_mapping(
            &a.as_native().unwrap().appearance,
            &b.as_native().unwrap().appearance,
            cad.mappings(),
            back.mappings(),
        );
        match (&a.as_native().unwrap().kind, &b.as_native().unwrap().kind) {
            (
                IfccadEntityKind::BlockInstance {
                    definition_id: a,
                    transform: x,
                },
                IfccadEntityKind::BlockInstance {
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
    let IfccadEntityKind::BlockInstance { transform, .. } =
        &mut d.model.entities[0].as_native_mut().unwrap().kind
    else {
        panic!()
    };
    transform.scale[0] = 1e-14;
    assert!(
        matches!(to_cad(&validated(&d)),Err(IfccadConversionError::Unsupported(i)) if i.iter().any(|d|d.code=="scale-clamped"))
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
    let opencadcodec::EntityType::Insert(i) = missing.get_entity_mut(h).unwrap() else {
        panic!()
    };
    i.block_name = "Missing".into();
    assert!(matches!(
        from_cad(&missing, metadata()),
        Err(IfccadConversionError::InvalidStructure(_))
    ));
    // Test the cycle independently of model occurrences.
    let mut d = nested([0.; 3]);
    d.model.entities.clear();
    let mut cycle = to_cad(&validated(&d)).unwrap().into_document();
    let mut i = opencadcodec::entities::Insert::new("Outer", opencadcodec::Vector3::ZERO);
    i.common.owner_handle = cycle.block_records.get("Inner").unwrap().handle;
    cycle
        .add_entity(opencadcodec::EntityType::Insert(i))
        .unwrap();
    assert!(matches!(
        from_cad(&cycle, metadata()),
        Err(IfccadConversionError::InvalidStructure(_))
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
        Err(IfccadConversionError::InvalidStructure(_))
    ));
    let mut names = c.clone();
    let mut b = names.block_records.get("Inner").unwrap().clone();
    b.handle = names.allocate_handle();
    names.block_records.add_allow_duplicate(b);
    assert!(matches!(
        from_cad(&names, metadata()),
        Err(IfccadConversionError::InvalidStructure(_))
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
            let opencadcodec::EntityType::Insert(i) = c.get_entity_mut(h).unwrap() else {
                panic!()
            };
            if field == 0 {
                i.row_count = 2;
            } else {
                i.attributes
                    .push(opencadcodec::entities::AttributeEntity::new(
                        "tag".into(),
                        "value".into(),
                    ));
            }
        } else if field == 4 {
            let mut dynamic = opencadcodec::objects::DynamicBlockObject::new(
                "BLOCKVISIBILITYPARAMETER",
                "AcDbBlockVisibilityParameter",
            );
            dynamic.handle = c.allocate_handle();
            dynamic.owner = c.block_records.get("Inner").unwrap().handle;
            c.objects.insert(
                dynamic.handle,
                opencadcodec::objects::ObjectType::DynamicBlock(dynamic),
            );
        } else {
            let b = c.block_records.get_mut("Inner").unwrap();
            if field == 2 {
                b.flags.is_xref = true;
            } else {
                b.preview_data = vec![1, 2, 3];
            }
        }
        assert!(matches!(
            from_cad(&c, metadata()),
            Err(IfccadConversionError::Unsupported(_))
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
        e.common_mut().handle = opencadcodec::Handle::new(h.value() + 9007199254740993);
        e.common_mut().owner_handle = owner;
        c.add_entity(e).unwrap();
    }
    let back = from_cad(&c, metadata()).unwrap();
    assert!(original
        .mappings()
        .entities
        .cad_handle(9007199254740993)
        .is_some());
    let entities = &back.validated_source().document().model.entities;
    assert_eq!(entities.len(), 2);
    for e in entities {
        let h = back.mappings().entities.cad_handle(e.id()).unwrap();
        assert!(h.value() > 9007199254740993);
        let IfccadEntityKind::BlockInstance { definition_id, .. } = e.as_native().unwrap().kind
        else {
            panic!()
        };
        assert_eq!(
            back.mappings().blocks.cad_handle(definition_id),
            Some(c.block_records.get("Outer").unwrap().handle)
        );
    }
}

#[test]
fn definition_metadata_and_anonymous_name_survive_memory_dxf_dwg() {
    use opencadcodec::{DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType};
    use serde_json::{json, Value};
    use std::io::Cursor;
    let mut source = ifccad_document_to_cad_document(&nested([1., 2., 0.]), Default::default())
        .unwrap()
        .into_document();
    let marker_handle = {
        let mut block = source.block_records.remove("Inner").unwrap();
        block.flags.anonymous = true;
        block.explodable = false;
        block.scale_uniformly = true;
        block.description = "Reusable anonymous definition".into();
        block.name = "*U42".into();
        let handle = block.block_entity_handle;
        source.block_records.add(block).unwrap();
        handle
    };
    let EntityType::Block(marker) = source.get_entity_mut(marker_handle).unwrap() else {
        panic!()
    };
    marker.name = "*U42".into();
    marker.description = "Reusable anonymous definition".into();
    let handles: Vec<_> = source
        .entities()
        .filter_map(|e| matches!(e, EntityType::Insert(_)).then_some(e.common().handle))
        .collect();
    for handle in handles {
        let EntityType::Insert(insert) = source.get_entity_mut(handle).unwrap() else {
            panic!()
        };
        if insert.block_name == "Inner" {
            insert.block_name = "*U42".into();
            insert.set_x_scale(-2.);
            insert.set_y_scale(-2.);
            insert.set_z_scale(-2.);
        }
    }
    for transport in 0..3 {
        let source = match transport {
            0 => source.clone(),
            1 => {
                DxfReader::from_reader(Cursor::new(DxfWriter::new(&source).write_to_vec().unwrap()))
                    .unwrap()
                    .read()
                    .unwrap()
            }
            _ => DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(&source).unwrap()))
                .read()
                .unwrap(),
        };
        let native =
            cad_document_to_encoded_ifccad(&source, metadata(), Default::default()).unwrap();
        let raw: Value = serde_json::from_slice(native.encoded().bytes()).unwrap();
        let block = raw["data"]
            .as_array()
            .unwrap()
            .iter()
            .find_map(|n| {
                n["attributes"]
                    .get("ifccad::blockDefinition")
                    .filter(|b| b["name"] == "*U42")
            })
            .expect("ordinary anonymous block omitted");
        assert_eq!(block["anonymous"], json!(true));
        assert_eq!(block["explodable"], json!(false));
        assert_eq!(block["description"], "Reusable anonymous definition");
        assert_eq!(block["uniformScaling"], json!(true));
        let target = ifccad_document_to_cad_document(
            native.validated_source().document(),
            Default::default(),
        )
        .unwrap();
        let block = target
            .document()
            .block_records
            .iter()
            .find(|b| b.name == "*U42")
            .unwrap();
        assert!(block.flags.anonymous);
        assert!(!block.explodable);
        assert!(block.scale_uniformly);
        assert_eq!(block.description, "Reusable anonymous definition");
        let EntityType::Block(marker) = target
            .document()
            .get_entity(block.block_entity_handle)
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(marker.description, block.description);
        let mut invalid = native.validated_source().document().clone();
        let definition = invalid.blocks.iter().find(|b| b.name == "*U42").unwrap().id;
        let mut changed = 0;
        for entity in invalid.model.entities.iter_mut().chain(
            invalid
                .blocks
                .iter_mut()
                .flat_map(|b| b.entities.iter_mut()),
        ) {
            if let IfccadEntityKind::BlockInstance {
                definition_id,
                transform,
            } = &mut entity.as_native_mut().unwrap().kind
            {
                if *definition_id == definition {
                    transform.scale = [-2., 2., 2.];
                    changed += 1;
                }
            }
        }
        assert!(changed > 0);
        for loss_policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
            assert!(matches!(
                ifccad_document_to_cad_document(
                    &invalid,
                    IfccadToCadOptions {
                        loss_policy,
                        ..Default::default()
                    }
                ),
                Err(IfccadConversionError::CoreValidation(_))
            ));
        }
    }
}

#[test]
fn unused_anonymous_definition_is_allocated_and_cad_role_names_are_not_local_blocks() {
    let mut document = nested([0.; 3]);
    let mut unused = document.blocks[0].clone();
    unused.id = document.id_counters.allocate_block_id().unwrap();
    unused.name = "*U99".into();
    unused.description = "Unused ordinary anonymous block".into();
    unused.anonymous = true;
    unused.explodable = false;
    unused.uniform_scaling = true;
    unused.entities.clear();
    unused.bounds = None;
    unused.bounds_quality = None;
    let id = unused.id;
    document.blocks.push(unused);
    let output = ifccad_document_to_cad_document(
        &document,
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        },
    )
    .unwrap();
    let handle = output.mappings().blocks.cad_handle(id).unwrap();
    let record = output
        .document()
        .block_records
        .iter()
        .find(|b| b.handle == handle)
        .unwrap();
    assert_eq!(record.name, "*U99");
    assert!(record.flags.anonymous && record.scale_uniformly);
    assert!(!record.explodable);
    for name in ["*Model_Space", "*Paper_Space", "*Paper_Space9"] {
        document.blocks.last_mut().unwrap().name = name.into();
        ocdraw::ifccad::validate_ifccad_document(&document).unwrap();
        let allowed = ifccad_document_to_cad_document(&document, Default::default()).unwrap();
        assert!(allowed.mappings().blocks.cad_handle(id).is_none());
        assert!(allowed
            .diagnostics()
            .iter()
            .any(|d| d.code == "block-skipped"));
        assert!(ifccad_document_to_cad_document(
            &document,
            IfccadToCadOptions {
                loss_policy: IfccadLossPolicy::Reject,
                ..Default::default()
            }
        )
        .is_err());
    }
}
