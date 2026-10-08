mod common;
use common::*;
use ifccad_convert::*;
use ocdraw::ifccad::*;

#[test]
fn default_allow_keeps_supported_entities_and_reports_skipped_spline() {
    let mut c = cad();
    let first = c
        .add_entity(opencadcodec::EntityType::Line(
            opencadcodec::Line::from_coords(0., 0., 0., 1., 0., 0.),
        ))
        .unwrap();
    let arc = c
        .add_entity(opencadcodec::EntityType::Spline(opencadcodec::Spline::new()))
        .unwrap();
    let last = c
        .add_entity(opencadcodec::EntityType::Line(
            opencadcodec::Line::from_coords(3., 0., 0., 4., 0., 0.),
        ))
        .unwrap();
    let out = cad_document_to_encoded_ifccad(&c, metadata(), Default::default()).unwrap();
    let entities = &out.validated_source().document().model.entities;
    assert_eq!(entities.len(), 2);
    assert_eq!(
        out.mappings().entities.cad_handle(entities[0].id()),
        Some(first)
    );
    assert_eq!(
        out.mappings().entities.cad_handle(entities[1].id()),
        Some(last)
    );
    assert!(out.mappings().entities.ifccad_id(arc).is_none());
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "entity-skipped" && d.location == format!("entity/{arc}")));
    load_ifccad_bytes(out.encoded().bytes(), Default::default()).unwrap();
}

#[test]
fn default_allow_converts_unmodified_hello_cad_with_nearest_line_weight() {
    let source = load_ifccad_bytes(
        include_bytes!("../../../examples/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap();
    let out = ifccad_source_to_cad_document(&source, Default::default()).unwrap();
    assert_eq!(
        out.document().layers.get("0").unwrap().line_weight,
        opencadcodec::LineWeight::Value(9)
    );
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "appearance" && d.location == "layer/0.line_weight"));
    assert_eq!(source.document().layers[0].appearance.line_weight, 0.1);
    let bytes = opencadcodec::DxfWriter::new(out.document())
        .write_to_vec()
        .unwrap();
    let readback = opencadcodec::DxfReader::from_reader(std::io::Cursor::new(bytes))
        .unwrap()
        .read()
        .unwrap();
    let restored =
        cad_document_to_encoded_ifccad(&readback, metadata(), Default::default()).unwrap();
    assert_eq!(
        restored.validated_source().document().model.entities.len(),
        5
    );
    assert_eq!(
        restored.validated_source().document().blocks[0]
            .entities
            .len(),
        1
    );
}

fn reject() -> CadToIfccadOptions {
    CadToIfccadOptions {
        loss_policy: IfccadLossPolicy::Reject,
        ..Default::default()
    }
}

#[test]
fn reject_returns_losses_while_allow_returns_them_on_success() {
    let mut c = cad();
    c.header.project_name = "project".into();
    let allow = cad_document_to_encoded_ifccad(&c, metadata(), Default::default()).unwrap();
    assert!(allow
        .diagnostics()
        .iter()
        .any(|d| d.is_loss() && d.location == "header.project_name"));
    let Err(IfccadConversionError::Unsupported(rejected)) =
        cad_document_to_encoded_ifccad(&c, metadata(), reject())
    else {
        panic!()
    };
    assert_eq!(allow.diagnostics(), rejected);
    let source = load_ifccad_bytes(
        include_bytes!("../../../examples/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap();
    assert!(matches!(
        ifccad_source_to_cad_document(
            &source,
            IfccadToCadOptions {
                loss_policy: (reject()).loss_policy,
                ..Default::default()
            }
        ),
        Err(IfccadConversionError::Unsupported(_))
    ));
}

#[test]
fn allow_skips_unsupported_source_properties_and_retains_supported_native_placement() {
    for field in 0..5 {
        let mut c = ifccad_source_to_cad_document(&validated(&primitives()), Default::default())
            .unwrap()
            .into_document();
        let h = c.block_records.get("*Model_Space").unwrap().entity_handles[1];
        let opencadcodec::EntityType::LwPolyline(p) = c.get_entity_mut(h).unwrap() else {
            panic!()
        };
        match field {
            0 => p.vertices[0].bulge = 0.5,
            1 => p.vertices[0].start_width = 2.,
            2 => p.normal = opencadcodec::Vector3::UNIT_Y,
            3 => p.thickness = 2.,
            _ => p.vertices[0].vertex_id = 10,
        }
        let out = cad_document_to_encoded_ifccad(&c, metadata(), Default::default()).unwrap();
        let mapped = matches!(field, 0 | 2);
        assert_eq!(
            out.validated_source().document().model.entities.len(),
            if mapped { 3 } else { 2 }
        );
        assert_eq!(out.mappings().entities.ifccad_id(h).is_some(), mapped);
        if mapped {
            continue;
        }
        assert!(out
            .diagnostics()
            .iter()
            .any(|d| d.code == "entity-skipped" && d.action == IfccadDiagnosticAction::Omitted));
    }
    let mut d = primitives();
    let IfccadEntityKind::Circle { placement, .. } =
        &mut d.model.entities[2].as_native_mut().unwrap().kind
    else {
        panic!()
    };
    placement.x_axis = [0., 1., 0.];
    placement.y_axis = [-1., 0., 0.];
    let out = ifccad_source_to_cad_document(&validated(&d), Default::default()).unwrap();
    assert!(out.mappings().entities.cad_handle(41).is_some());
    assert_eq!(out.mappings().entities.iter().count(), 3);
}

#[test]
fn allow_reports_partial_nested_definitions_at_each_affected_instance() {
    let mut c = ifccad_source_to_cad_document(&validated(&nested([0.; 3])), Default::default())
        .unwrap()
        .into_document();
    let h = c.block_records.get("Inner").unwrap().entity_handles[0];
    let mut arc = opencadcodec::Spline::new();
    arc.common = c.get_entity(h).unwrap().common().clone();
    *c.get_entity_mut(h).unwrap() = opencadcodec::EntityType::Spline(arc);
    let out = cad_document_to_encoded_ifccad(&c, metadata(), Default::default()).unwrap();
    let d = out.validated_source().document();
    assert!(d
        .blocks
        .iter()
        .find(|b| b.name == "Inner")
        .unwrap()
        .entities
        .is_empty());
    assert_eq!(
        d.blocks
            .iter()
            .find(|b| b.name == "Outer")
            .unwrap()
            .entities
            .len(),
        2
    );
    assert_eq!(d.model.entities.len(), 2);
    for e in c.model_space_entities().chain(c.entities_in_block("Outer")) {
        if matches!(e, opencadcodec::EntityType::Insert(_)) {
            assert!(out
                .diagnostics()
                .iter()
                .any(|d| d.code == "block-content-loss"
                    && d.location == format!("entity/{}", e.common().handle)));
        }
    }
    let back = ifccad_source_to_cad_document(out.validated_source(), Default::default()).unwrap();
    assert_eq!(back.mappings().entities.iter().count(), 4);
}

#[test]
fn omitted_definitions_do_not_leave_dangling_inserts_or_mappings() {
    let mut c = ifccad_source_to_cad_document(&validated(&nested([0.; 3])), Default::default())
        .unwrap()
        .into_document();
    let h = c.block_records.get("Inner").unwrap().handle;
    c.block_records.get_mut("Inner").unwrap().flags.is_xref = true;
    let out = cad_document_to_encoded_ifccad(&c, metadata(), Default::default()).unwrap();
    assert!(out.mappings().blocks.ifccad_id(h).is_none());
    let d = out.validated_source().document();
    assert!(d.blocks.iter().all(|b| b.name != "Inner"));
    assert_eq!(
        d.blocks
            .iter()
            .find(|b| b.name == "Outer")
            .unwrap()
            .entities
            .len(),
        1
    );
    assert!(out.diagnostics().iter().any(|d| d.code == "block-skipped"));
    ifccad_source_to_cad_document(out.validated_source(), Default::default()).unwrap();

    let mut d = nested([0.; 3]);
    d.blocks
        .iter_mut()
        .find(|b| b.name == "Inner")
        .unwrap()
        .name = "*Paper_Space9".into();
    let out = ifccad_source_to_cad_document(&validated(&d), Default::default()).unwrap();
    assert!(out.mappings().blocks.cad_handle(3).is_none());
    assert_eq!(out.document().entities_in_block("Outer").count(), 1);
}

#[test]
fn indexed_color_default_weight_and_quantization_preserve_inherited_entity_modes() {
    let mut c = cad();
    let mut pattern = opencadcodec::LineType::dashed();
    pattern.handle = c.allocate_handle();
    c.line_types.add(pattern).unwrap();
    let layer = c.layers.get_mut("Notes").unwrap();
    layer.color = opencadcodec::Color::Index(1);
    layer.line_weight = opencadcodec::LineWeight::Default;
    layer.line_type = "Dashed".into();
    let out = cad_document_to_encoded_ifccad(&c, metadata(), Default::default()).unwrap();
    let layer = out
        .validated_source()
        .document()
        .layers
        .iter()
        .find(|l| l.name == "Notes")
        .unwrap();
    assert_eq!(
        layer.appearance.color,
        IfccadColor::rgb(255, 0, 0).with_indexed("ACI", 1)
    );
    assert_eq!(layer.appearance.line_weight, 0.25);
    let pattern = out
        .validated_source()
        .document()
        .line_patterns
        .iter()
        .find(|p| p.id == layer.appearance.line_pattern)
        .unwrap();
    assert_eq!(pattern.name, "Dashed");
    assert!(!pattern.pattern.is_empty());
    assert!(out
        .diagnostics()
        .iter()
        .filter(|d| d.code == "appearance")
        .all(|d| d.action == IfccadDiagnosticAction::Modified));
    let mut d = primitives();
    d.layers[0].appearance.opacity = 0.5;
    let out = ifccad_source_to_cad_document(&validated(&d), Default::default()).unwrap();
    assert_eq!(
        out.document().layers.get("0").unwrap().transparency,
        opencadcodec::Transparency::Explicit(128)
    );
    assert_eq!(
        out.document()
            .get_entity(out.mappings().entities.cad_handle(90).unwrap())
            .unwrap()
            .common()
            .transparency,
        opencadcodec::Transparency::ByBlock
    );
}

#[test]
fn source_structure_and_numeric_failures_remain_fatal_under_both_policies() {
    for options in [CadToIfccadOptions::default(), reject()] {
        let mut c = cad();
        c.block_records
            .get_mut("*Model_Space")
            .unwrap()
            .entity_handles
            .push(opencadcodec::Handle::new(99999));
        assert!(matches!(
            cad_document_to_encoded_ifccad(&c, metadata(), options),
            Err(IfccadConversionError::InvalidStructure(_))
        ));
        let mut d = primitives();
        let IfccadEntityKind::PlanarPolyline {
            placement,
            vertices,
            ..
        } = &mut d.model.entities[1].as_native_mut().unwrap().kind
        else {
            panic!()
        };
        placement.origin[0] = 1e20;
        vertices[0][0] = 1.;
        assert!(ifccad_source_to_cad_document(
            &validated(&d),
            IfccadToCadOptions {
                loss_policy: (options).loss_policy,
                ..Default::default()
            }
        )
        .is_err());
        let mut d = nested([0.; 3]);
        let IfccadEntityKind::BlockInstance { transform, .. } =
            &mut d.model.entities[0].as_native_mut().unwrap().kind
        else {
            panic!()
        };
        transform.scale[0] = 1e-14;
        assert!(ifccad_source_to_cad_document(
            &validated(&d),
            IfccadToCadOptions {
                loss_policy: (options).loss_policy,
                ..Default::default()
            }
        )
        .is_err());
        let mut c = cad();
        let mut line = opencadcodec::Line::new();
        line.start.x = f64::NAN;
        line.thickness = 1.;
        c.add_entity(opencadcodec::EntityType::Line(line)).unwrap();
        assert!(cad_document_to_encoded_ifccad(&c, metadata(), options).is_err());
    }
}

#[test]
fn recovery_does_not_count_as_semantic_loss() {
    let mut c = cad();
    c.header.model_space_block_handle = opencadcodec::Handle::new(99999);
    let out = cad_document_to_encoded_ifccad(&c, metadata(), reject()).unwrap();
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.action == IfccadDiagnosticAction::Recovery));
    assert!(out.diagnostics().iter().all(|d| !d.is_loss()));
}

#[test]
fn allow_can_omit_foreign_graph_but_cannot_round_large_integer_geometry() {
    let mut value: serde_json::Value =
        serde_json::from_slice(encode_ifccad_document(&primitives()).unwrap().bytes()).unwrap();
    value["data"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"path":"foreign","attributes":{"test::unknown":42}}));
    let source =
        load_ifccad_bytes(&serde_json::to_vec(&value).unwrap(), Default::default()).unwrap();
    let out = ifccad_source_to_cad_document(&source, Default::default()).unwrap();
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "foreign-ifcx" && d.is_loss()));
    assert_eq!(
        source.graph().composed_ifcx()["data"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()["path"],
        "foreign"
    );
    value["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["path"] == "/cad/d1/e90")
        .unwrap()["attributes"]["ifccad::geom::lineSegment"]["start"][0] =
        serde_json::json!(9007199254740993_u64);
    let source =
        load_ifccad_bytes(&serde_json::to_vec(&value).unwrap(), Default::default()).unwrap();
    assert!(
        matches!(ifccad_source_to_cad_document(&source, Default::default()),Err(IfccadConversionError::Unsupported(d)) if d.iter().any(|d|d.code=="precision"))
    );
}

#[test]
fn arrays_attributes_and_dynamic_definitions_are_omitted_without_flattening() {
    for field in 0..3 {
        let mut c = ifccad_source_to_cad_document(&validated(&nested([0.; 3])), Default::default())
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
        } else {
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
        }
        let out = cad_document_to_encoded_ifccad(&c, metadata(), Default::default()).unwrap();
        let d = out.validated_source().document();
        if field < 2 {
            assert_eq!(d.model.entities.len(), 1);
            assert!(out.mappings().entities.ifccad_id(h).is_none());
        } else {
            assert_eq!(d.model.entities.len(), 2);
            assert!(d.blocks.iter().all(|b| b.name != "Inner"));
            assert_eq!(
                d.blocks
                    .iter()
                    .find(|b| b.name == "Outer")
                    .unwrap()
                    .entities
                    .len(),
                1
            );
        }
        load_ifccad_bytes(out.encoded().bytes(), Default::default()).unwrap();
    }
}

#[test]
fn common_metadata_loss_keeps_geometry_and_marks_nested_occurrences() {
    let mut c = ifccad_source_to_cad_document(&validated(&nested([0.; 3])), Default::default())
        .unwrap()
        .into_document();
    let h = c.block_records.get("Inner").unwrap().entity_handles[0];
    c.get_entity_mut(h).unwrap().common_mut().color_name =
        Some("incomplete named-color identity".into());
    let out = cad_document_to_encoded_ifccad(&c, metadata(), Default::default()).unwrap();
    assert!(out.mappings().entities.ifccad_id(h).is_some());
    assert_eq!(
        out.validated_source()
            .document()
            .blocks
            .iter()
            .find(|b| b.name == "Inner")
            .unwrap()
            .entities
            .len(),
        1
    );
    assert!(out.diagnostics().iter().any(|d| d.code == "entity-common"));
    assert_eq!(
        out.diagnostics()
            .iter()
            .filter(|d| d.code == "block-content-loss")
            .count(),
        3
    );
    let mut d = nested([0.; 3]);
    let IfccadEntityKind::LineSegment { start, .. } = d
        .blocks
        .iter()
        .find(|b| b.name == "Inner")
        .unwrap()
        .entities[0]
        .as_native()
        .unwrap()
        .kind
    else {
        panic!()
    };
    d.blocks
        .iter_mut()
        .find(|b| b.name == "Inner")
        .unwrap()
        .entities[0]
        .as_native_mut()
        .unwrap()
        .kind = IfccadEntityKind::Circle {
        radius: 1.,
        placement: IfccadPlacement {
            origin: start,
            x_axis: [0., 1., 0.],
            y_axis: [-1., 0., 0.],
        },
    };
    let out = ifccad_source_to_cad_document(&validated(&d), Default::default()).unwrap();
    // A represented rotation is geometric preparation, not missing block content.
    assert!(out.document().entities_in_block("Inner").next().is_some());
    assert_eq!(
        out.diagnostics()
            .iter()
            .filter(|d| d.code == "block-content-loss")
            .count(),
        0
    );
}

#[test]
fn missing_layer_zero_has_loss_evidence_while_paper_geometry_is_retained() {
    let mut d = empty();
    d.id_counters.next_entity_id = 46;
    d.id_counters.next_layout_id = 9;
    d.layers.retain(|l| l.name != "0");
    let mut e = primitives().model.entities[0].clone();
    e.as_native_mut().unwrap().id = 45;
    d.paper_layouts.push(IfccadPaperLayout {
        bounds_quality: None,
        canvas: None,

        settings: ocdraw::ifccad::IfccadLayoutSettings {
            media: Some(ocdraw::ifccad::IfccadLayoutMedia {
                width: 297.,
                height: 210.,
                unit: ocdraw::ifccad::IfccadMediaUnit::from_token("mm").unwrap(),
            }),
            ..Default::default()
        },
        bounds: None,
        id: 8,
        name: "Sheet".into(),
        tab_index: 1,
        entities: vec![e],
    });
    let source = validated(&d);
    let out = ifccad_source_to_cad_document(&source, Default::default()).unwrap();
    assert!(out.mappings().entities.cad_handle(45).is_some());
    assert!(out.mappings().layouts.cad_handle(8).is_some());
    let layer = out.document().layers.get("0").unwrap();
    assert_eq!(
        layer.color,
        opencadcodec::Color::Rgb {
            r: 255,
            g: 255,
            b: 255
        }
    );
    assert_eq!(layer.line_weight, opencadcodec::LineWeight::Value(25));
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "layer-0" && d.action == IfccadDiagnosticAction::Modified));
    assert!(out.diagnostics().iter().all(|d| d.code != "paper"));
    assert!(ifccad_source_to_cad_document(
        &source,
        IfccadToCadOptions {
            loss_policy: (reject()).loss_policy,
            ..Default::default()
        }
    )
    .is_err());
    let mut c = cad();
    let h = c
        .add_paper_space_entity(opencadcodec::EntityType::Line(
            opencadcodec::Line::from_coords(0., 0., 0., 1., 0., 0.),
        ))
        .unwrap();
    let out = cad_document_to_encoded_ifccad(&c, metadata(), Default::default()).unwrap();
    assert!(out.mappings().entities.ifccad_id(h).is_some());
    assert!(out
        .diagnostics()
        .iter()
        .all(|d| d.code != "paper" || d.location != format!("entity/{h}")));
}

#[test]
fn appearance_quantization_is_stable_through_dxf_readback() {
    let mut d = primitives();
    d.layers[0].appearance.line_weight = 0.1;
    d.layers[0].appearance.opacity = 0.5;
    d.model.entities[0]
        .as_native_mut()
        .unwrap()
        .appearance
        .line_weight = IfccadMode::Explicit(0.12);
    d.model.entities[0]
        .as_native_mut()
        .unwrap()
        .appearance
        .opacity = IfccadMode::Explicit(0.5);
    let first = ifccad_source_to_cad_document(&validated(&d), Default::default()).unwrap();
    let expected = from_cad(first.document(), metadata()).unwrap();
    let bytes = opencadcodec::DxfWriter::new(first.document())
        .write_to_vec()
        .unwrap();
    let readback = opencadcodec::DxfReader::from_reader(std::io::Cursor::new(bytes))
        .unwrap()
        .read()
        .unwrap();
    let restored = from_cad(&readback, metadata()).unwrap();
    assert_eq!(
        expected.validated_source().document(),
        restored.validated_source().document()
    );
}
