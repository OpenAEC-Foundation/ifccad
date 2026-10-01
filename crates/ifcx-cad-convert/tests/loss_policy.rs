mod common;
use common::*;
use ifcx_cad_convert::*;
use ocdraw::ifcx_cad::*;

#[test]
fn default_allow_keeps_supported_entities_and_reports_skipped_arc() {
    let mut c = cad();
    let first = c
        .add_entity(cadcodec::EntityType::Line(cadcodec::Line::from_coords(
            0., 0., 0., 1., 0., 0.,
        )))
        .unwrap();
    let arc = c
        .add_entity(cadcodec::EntityType::Arc(cadcodec::Arc::new()))
        .unwrap();
    let last = c
        .add_entity(cadcodec::EntityType::Line(cadcodec::Line::from_coords(
            3., 0., 0., 4., 0., 0.,
        )))
        .unwrap();
    let out = cad_document_to_ifcx_cad(&c, metadata()).unwrap();
    let entities = &out.validated_ifcx().document().model.entities;
    assert_eq!(entities.len(), 2);
    assert_eq!(
        out.mappings().entities.cad_handle(entities[0].id),
        Some(first)
    );
    assert_eq!(
        out.mappings().entities.cad_handle(entities[1].id),
        Some(last)
    );
    assert!(out.mappings().entities.ifcx_id(arc).is_none());
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "entity-skipped" && d.location == format!("entity/{arc}")));
    read_native_cad_ifcx(out.ifcx_bytes()).unwrap();
}

#[test]
fn default_allow_converts_unmodified_hello_cad_with_nearest_line_weight() {
    let source = read_native_cad_ifcx(include_bytes!(
        "../../../examples/ifcx-native-cad/hello-cad.ifcx"
    ))
    .unwrap();
    let out = ifcx_cad_to_cad_document(&source).unwrap();
    assert_eq!(
        out.document().layers.get("0").unwrap().line_weight,
        cadcodec::LineWeight::Value(9)
    );
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "appearance" && d.location == "layer/0.line_weight"));
    assert_eq!(source.document().layers[0].appearance.line_weight, 0.1);
    let bytes = cadcodec::DxfWriter::new(out.document())
        .write_to_vec()
        .unwrap();
    let readback = cadcodec::DxfReader::from_reader(std::io::Cursor::new(bytes))
        .unwrap()
        .read()
        .unwrap();
    let restored = cad_document_to_ifcx_cad(&readback, metadata()).unwrap();
    assert_eq!(restored.validated_ifcx().document().model.entities.len(), 5);
    assert_eq!(
        restored.validated_ifcx().document().blocks[0]
            .entities
            .len(),
        1
    );
}

fn reject() -> IfcxCadConversionOptions {
    IfcxCadConversionOptions {
        loss_policy: IfcxCadLossPolicy::Reject,
    }
}

#[test]
fn reject_returns_losses_while_allow_returns_them_on_success() {
    let mut c = cad();
    c.header.project_name = "project".into();
    let allow = cad_document_to_ifcx_cad(&c, metadata()).unwrap();
    assert!(allow
        .diagnostics()
        .iter()
        .any(|d| d.is_loss() && d.location == "header.project_name"));
    let Err(IfcxCadConversionError::Unsupported(rejected)) =
        cad_document_to_ifcx_cad_with_options(&c, metadata(), reject())
    else {
        panic!()
    };
    assert_eq!(allow.diagnostics(), rejected);
    let source = read_native_cad_ifcx(include_bytes!(
        "../../../examples/ifcx-native-cad/hello-cad.ifcx"
    ))
    .unwrap();
    assert!(matches!(
        ifcx_cad_to_cad_document_with_options(&source, reject()),
        Err(IfcxCadConversionError::Unsupported(_))
    ));
}

#[test]
fn allow_skips_whole_incompatible_geometry_in_both_directions() {
    for field in 0..5 {
        let mut c = ifcx_cad_to_cad_document(&validated(&primitives()))
            .unwrap()
            .into_document();
        let h = c.block_records.get("*Model_Space").unwrap().entity_handles[1];
        let cadcodec::EntityType::LwPolyline(p) = c.get_entity_mut(h).unwrap() else {
            panic!()
        };
        match field {
            0 => p.vertices[0].bulge = 0.5,
            1 => p.vertices[0].start_width = 2.,
            2 => p.normal = cadcodec::Vector3::UNIT_Y,
            3 => p.thickness = 2.,
            _ => p.vertices[0].vertex_id = 10,
        }
        let out = cad_document_to_ifcx_cad(&c, metadata()).unwrap();
        assert_eq!(out.validated_ifcx().document().model.entities.len(), 2);
        assert!(out.mappings().entities.ifcx_id(h).is_none());
        assert!(out
            .diagnostics()
            .iter()
            .any(|d| d.code == "entity-skipped" && d.action == IfcxCadDiagnosticAction::Omitted));
    }
    let mut d = primitives();
    let IfcxCadEntityKind::Circle { placement, .. } = &mut d.model.entities[2].kind else {
        panic!()
    };
    placement.x_axis = [0., 1., 0.];
    placement.y_axis = [-1., 0., 0.];
    let out = ifcx_cad_to_cad_document(&validated(&d)).unwrap();
    assert!(out.mappings().entities.cad_handle(41).is_none());
    assert_eq!(out.mappings().entities.iter().count(), 2);
}

#[test]
fn allow_reports_partial_nested_definitions_at_each_affected_instance() {
    let mut c = ifcx_cad_to_cad_document(&validated(&nested([0.; 3])))
        .unwrap()
        .into_document();
    let h = c.block_records.get("Inner").unwrap().entity_handles[0];
    let mut arc = cadcodec::Arc::new();
    arc.common = c.get_entity(h).unwrap().common().clone();
    *c.get_entity_mut(h).unwrap() = cadcodec::EntityType::Arc(arc);
    let out = cad_document_to_ifcx_cad(&c, metadata()).unwrap();
    let d = out.validated_ifcx().document();
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
        if matches!(e, cadcodec::EntityType::Insert(_)) {
            assert!(out
                .diagnostics()
                .iter()
                .any(|d| d.code == "block-content-loss"
                    && d.location == format!("entity/{}", e.common().handle)));
        }
    }
    let back = ifcx_cad_to_cad_document(out.validated_ifcx()).unwrap();
    assert_eq!(back.mappings().entities.iter().count(), 4);
}

#[test]
fn omitted_definitions_do_not_leave_dangling_inserts_or_mappings() {
    let mut c = ifcx_cad_to_cad_document(&validated(&nested([0.; 3])))
        .unwrap()
        .into_document();
    let h = c.block_records.get("Inner").unwrap().handle;
    c.block_records.get_mut("Inner").unwrap().flags.is_xref = true;
    let out = cad_document_to_ifcx_cad(&c, metadata()).unwrap();
    assert!(out.mappings().blocks.ifcx_id(h).is_none());
    let d = out.validated_ifcx().document();
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
    ifcx_cad_to_cad_document(out.validated_ifcx()).unwrap();

    let mut d = nested([0.; 3]);
    d.blocks
        .iter_mut()
        .find(|b| b.name == "Inner")
        .unwrap()
        .name = "*U1".into();
    let out = ifcx_cad_to_cad_document(&validated(&d)).unwrap();
    assert!(out.mappings().blocks.cad_handle(3).is_none());
    assert_eq!(out.document().entities_in_block("Outer").count(), 1);
}

#[test]
fn appearance_fallbacks_are_explicit_and_preserve_inherited_entity_modes() {
    let mut c = cad();
    let mut pattern = cadcodec::LineType::dashed();
    pattern.handle = c.allocate_handle();
    c.line_types.add(pattern).unwrap();
    let layer = c.layers.get_mut("Notes").unwrap();
    layer.color = cadcodec::Color::Index(1);
    layer.line_weight = cadcodec::LineWeight::Default;
    layer.line_type = "Dashed".into();
    let out = cad_document_to_ifcx_cad(&c, metadata()).unwrap();
    let layer = out
        .validated_ifcx()
        .document()
        .layers
        .iter()
        .find(|l| l.name == "Notes")
        .unwrap();
    assert_eq!(layer.appearance.color, "#FF0000");
    assert_eq!(layer.appearance.line_weight, 0.25);
    let pattern = out
        .validated_ifcx()
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
        .all(|d| d.action == IfcxCadDiagnosticAction::Modified));
    let mut d = primitives();
    d.layers[0].appearance.opacity = 0.5;
    let out = ifcx_cad_to_cad_document(&validated(&d)).unwrap();
    assert_eq!(
        out.document().layers.get("0").unwrap().transparency,
        cadcodec::Transparency::Explicit(128)
    );
    assert_eq!(
        out.document()
            .get_entity(out.mappings().entities.cad_handle(90).unwrap())
            .unwrap()
            .common()
            .transparency,
        cadcodec::Transparency::ByBlock
    );
}

#[test]
fn source_structure_and_numeric_failures_remain_fatal_under_both_policies() {
    for options in [IfcxCadConversionOptions::default(), reject()] {
        let mut c = cad();
        c.block_records
            .get_mut("*Model_Space")
            .unwrap()
            .entity_handles
            .push(cadcodec::Handle::new(99999));
        assert!(matches!(
            cad_document_to_ifcx_cad_with_options(&c, metadata(), options),
            Err(IfcxCadConversionError::InvalidStructure(_))
        ));
        let mut d = primitives();
        let IfcxCadEntityKind::PlanarPolyline {
            placement,
            vertices,
            ..
        } = &mut d.model.entities[1].kind
        else {
            panic!()
        };
        placement.origin[0] = 1e20;
        vertices[0][0] = 1.;
        assert!(ifcx_cad_to_cad_document_with_options(&validated(&d), options).is_err());
        let mut d = nested([0.; 3]);
        let IfcxCadEntityKind::BlockInstance { transform, .. } = &mut d.model.entities[0].kind
        else {
            panic!()
        };
        transform.scale[0] = 1e-14;
        assert!(ifcx_cad_to_cad_document_with_options(&validated(&d), options).is_err());
        let mut c = cad();
        let mut line = cadcodec::Line::new();
        line.start.x = f64::NAN;
        line.thickness = 1.;
        c.add_entity(cadcodec::EntityType::Line(line)).unwrap();
        assert!(cad_document_to_ifcx_cad_with_options(&c, metadata(), options).is_err());
    }
}

#[test]
fn recovery_does_not_count_as_semantic_loss() {
    let mut c = cad();
    c.header.model_space_block_handle = cadcodec::Handle::new(99999);
    let out = cad_document_to_ifcx_cad_with_options(&c, metadata(), reject()).unwrap();
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.action == IfcxCadDiagnosticAction::Recovery));
    assert!(out.diagnostics().iter().all(|d| !d.is_loss()));
}

#[test]
fn allow_can_omit_foreign_graph_but_cannot_round_large_integer_geometry() {
    let mut value: serde_json::Value =
        serde_json::from_slice(&write_native_cad_ifcx(&primitives()).unwrap()).unwrap();
    value["data"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"path":"foreign","attributes":{"test::unknown":42}}));
    let source = read_native_cad_ifcx(&serde_json::to_vec(&value).unwrap()).unwrap();
    let out = ifcx_cad_to_cad_document(&source).unwrap();
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "foreign-ifcx" && d.is_loss()));
    assert_eq!(
        source.raw_ifcx()["data"]
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
    let source = read_native_cad_ifcx(&serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(
        matches!(ifcx_cad_to_cad_document(&source),Err(IfcxCadConversionError::Unsupported(d)) if d.iter().any(|d|d.code=="precision"))
    );
}

#[test]
fn arrays_attributes_and_dynamic_definitions_are_omitted_without_flattening() {
    for field in 0..3 {
        let mut c = ifcx_cad_to_cad_document(&validated(&nested([0.; 3])))
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
        } else {
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
        }
        let out = cad_document_to_ifcx_cad(&c, metadata()).unwrap();
        let d = out.validated_ifcx().document();
        if field < 2 {
            assert_eq!(d.model.entities.len(), 1);
            assert!(out.mappings().entities.ifcx_id(h).is_none());
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
        read_native_cad_ifcx(out.ifcx_bytes()).unwrap();
    }
}

#[test]
fn common_metadata_loss_keeps_geometry_and_marks_nested_occurrences() {
    let mut c = ifcx_cad_to_cad_document(&validated(&nested([0.; 3])))
        .unwrap()
        .into_document();
    let h = c.block_records.get("Inner").unwrap().entity_handles[0];
    c.get_entity_mut(h).unwrap().common_mut().invisible = true;
    let out = cad_document_to_ifcx_cad(&c, metadata()).unwrap();
    assert!(out.mappings().entities.ifcx_id(h).is_some());
    assert_eq!(
        out.validated_ifcx()
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
    let IfcxCadEntityKind::LineSegment { start, .. } = d
        .blocks
        .iter()
        .find(|b| b.name == "Inner")
        .unwrap()
        .entities[0]
        .kind
    else {
        panic!()
    };
    d.blocks
        .iter_mut()
        .find(|b| b.name == "Inner")
        .unwrap()
        .entities[0]
        .kind = IfcxCadEntityKind::Circle {
        radius: 1.,
        placement: IfcxCadPlacement {
            origin: start,
            x_axis: [0., 1., 0.],
            y_axis: [-1., 0., 0.],
        },
    };
    let out = ifcx_cad_to_cad_document(&validated(&d)).unwrap();
    assert!(out.document().entities_in_block("Inner").next().is_none());
    assert_eq!(
        out.diagnostics()
            .iter()
            .filter(|d| d.code == "block-content-loss")
            .count(),
        3
    );
}

#[test]
fn missing_layer_zero_and_paper_have_explicit_loss_evidence() {
    let mut d = empty();
    d.layers.retain(|l| l.name != "0");
    let mut e = primitives().model.entities[0].clone();
    e.id = 45;
    d.paper_layouts.push(IfcxCadPaperLayout {
        id: 8,
        name: "Sheet".into(),
        paper: IfcxCadPaperSize {
            width: 297.,
            height: 210.,
            length_unit: "mm".into(),
        },
        entities: vec![e],
    });
    let source = validated(&d);
    let out = ifcx_cad_to_cad_document(&source).unwrap();
    assert!(out.mappings().entities.cad_handle(45).is_none());
    assert!(out.mappings().layouts.cad_handle(8).is_none());
    let layer = out.document().layers.get("0").unwrap();
    assert_eq!(
        layer.color,
        cadcodec::Color::Rgb {
            r: 255,
            g: 255,
            b: 255
        }
    );
    assert_eq!(layer.line_weight, cadcodec::LineWeight::Value(25));
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "layer-0" && d.action == IfcxCadDiagnosticAction::Modified));
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "paper" && d.action == IfcxCadDiagnosticAction::Omitted));
    assert!(ifcx_cad_to_cad_document_with_options(&source, reject()).is_err());
    let mut c = cad();
    let h = c
        .add_paper_space_entity(cadcodec::EntityType::Line(cadcodec::Line::from_coords(
            0., 0., 0., 1., 0., 0.,
        )))
        .unwrap();
    let out = cad_document_to_ifcx_cad(&c, metadata()).unwrap();
    assert!(out.mappings().entities.ifcx_id(h).is_none());
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "paper" && d.location == format!("entity/{h}")));
}

#[test]
fn appearance_quantization_is_stable_through_dxf_readback() {
    let mut d = primitives();
    d.layers[0].appearance.line_weight = 0.1;
    d.layers[0].appearance.opacity = 0.5;
    d.model.entities[0].appearance.line_weight = IfcxCadMode::Explicit(0.12);
    d.model.entities[0].appearance.opacity = IfcxCadMode::Explicit(0.5);
    let first = ifcx_cad_to_cad_document(&validated(&d)).unwrap();
    let expected = from_cad(first.document(), metadata()).unwrap();
    let bytes = cadcodec::DxfWriter::new(first.document())
        .write_to_vec()
        .unwrap();
    let readback = cadcodec::DxfReader::from_reader(std::io::Cursor::new(bytes))
        .unwrap()
        .read()
        .unwrap();
    let restored = from_cad(&readback, metadata()).unwrap();
    assert_eq!(
        expected.validated_ifcx().document(),
        restored.validated_ifcx().document()
    );
}
