mod common;
use common::*;
use ifccad_convert::*;
use ocdraw::ifccad::*;

#[test]
fn fresh_cad_import_emits_valid_counters_under_both_policies() {
    let source = primitives();
    let cad = to_cad(&validated(&source)).unwrap();
    for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        let back = cad_document_to_encoded_ifccad(
            cad.document(),
            metadata(),
            CadToIfccadOptions {
                loss_policy: policy,
                ..Default::default()
            },
        )
        .unwrap();
        let loaded = load_ifccad_bytes(back.encoded().bytes(), Default::default()).unwrap();
        assert_eq!(
            loaded.document().id_counters,
            IfccadIdCounters {
                next_text_style_id: 2,
                next_preservation_record_id: 1,
                next_ucs_id: 1,
                next_model_window_id: 2,

                next_entity_id: 4,
                next_layer_id: 3,
                next_layout_id: 2,
                next_block_id: 1,
                next_line_pattern_id: 2,
            }
        );
        for (original, restored) in source
            .model
            .entities
            .iter()
            .zip(&loaded.document().model.entities)
        {
            assert_eq!(
                cad.mappings().entities.cad_handle(original.id()),
                back.mappings().entities.cad_handle(restored.id())
            );
        }
        assert_eq!(
            loaded
                .document()
                .layers
                .iter()
                .find(|layer| layer.name == "0")
                .unwrap()
                .id,
            1
        );
    }
}

#[test]
fn empty_roundtrip_keeps_unit_and_unused_layer() {
    let source = validated(&empty());
    let cad = to_cad(&source).unwrap();
    let result = from_cad(cad.document(), metadata()).unwrap();
    let doc = result.validated_source().document();
    assert_eq!(doc.header, header());
    assert_eq!(doc.drawing_id, 7);
    assert_eq!(doc.length_unit, "mm");
    assert!(doc.model.entities.is_empty());
    assert_eq!(
        doc.layers
            .iter()
            .map(|l| l.name.as_str())
            .collect::<Vec<_>>(),
        ["0", "Notes"]
    );
    ocdraw::ifccad::load_ifccad_bytes(result.encoded().bytes(), Default::default()).unwrap();
}

#[test]
fn authored_unsized_paper_is_retained_and_default_scaffold_is_not() {
    let mut c = cad();
    from_cad(&c, metadata()).unwrap();
    c.add_paper_space_entity(opencadcodec::EntityType::Line(
        opencadcodec::Line::from_coords(0., 0., 0., 1., 0., 0.),
    ))
    .unwrap();
    let restored = from_cad(&c, metadata()).unwrap();
    assert_eq!(
        restored.validated_source().document().paper_layouts.len(),
        1
    );
    assert_eq!(
        restored.validated_source().document().paper_layouts[0]
            .entities
            .len(),
        1
    );
    assert!(restored.validated_source().document().paper_layouts[0]
        .settings
        .media
        .is_none());
}
#[test]
fn ambiguous_model_owner_is_fatal() {
    let mut c = cad();
    let mut l = c
        .objects
        .values()
        .find_map(|o| match o {
            opencadcodec::objects::ObjectType::Layout(l) if l.name == "Model" => Some(l.clone()),
            _ => None,
        })
        .unwrap();
    l.handle = c.allocate_handle();
    c.objects
        .insert(l.handle, opencadcodec::objects::ObjectType::Layout(l));
    assert!(matches!(
        from_cad(&c, metadata()),
        Err(IfccadConversionError::InvalidStructure(_))
    ));
}
#[test]
fn invalid_source_references_are_fatal() {
    let mut c = cad();
    c.block_records
        .get_mut("*Model_Space")
        .unwrap()
        .entity_handles
        .push(opencadcodec::Handle::new(99999));
    assert!(matches!(
        from_cad(&c, metadata()),
        Err(IfccadConversionError::InvalidStructure(_))
    ));
}
#[test]
fn foreign_ifcx_information_is_identified() {
    let mut value: serde_json::Value =
        serde_json::from_slice(encode_ifccad_document(&empty()).unwrap().bytes()).unwrap();
    value["data"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"path":"foreign","attributes":{"test::unknown":42}}));
    let source =
        load_ifccad_bytes(&serde_json::to_vec(&value).unwrap(), Default::default()).unwrap();
    assert!(
        matches!(to_cad(&source),Err(IfccadConversionError::Unsupported(d)) if d.iter().any(|d|d.code=="foreign-ifcx"))
    );
}
#[test]
fn primitive_roundtrip_keeps_order_and_modes() {
    let source = primitives();
    let cad = to_cad(&validated(&source)).unwrap();
    let back = from_cad(cad.document(), metadata()).unwrap();
    let out = back.validated_source().document();
    assert_eq!(out.length_unit, "cm");
    for (s, t) in source.model.entities.iter().zip(&out.model.entities) {
        assert_appearance_mapping(
            &s.as_native().unwrap().appearance,
            &t.as_native().unwrap().appearance,
            cad.mappings(),
            back.mappings(),
        );
        assert_eq!(
            cad.mappings().entities.cad_handle(s.id()),
            back.mappings().entities.cad_handle(t.id())
        );
    }
    assert_eq!(
        out.model.entities[0].as_native().unwrap().kind,
        source.model.entities[0].as_native().unwrap().kind
    );
    assert_eq!(
        out.model.entities[2].as_native().unwrap().kind,
        source.model.entities[2].as_native().unwrap().kind
    );
    let IfccadEntityKind::PlanarPolyline {
        vertices,
        closed,
        placement,
        ..
    } = &out.model.entities[1].as_native().unwrap().kind
    else {
        panic!()
    };
    assert_eq!(vertices, &vec![[8., 16.], [10., 16.], [10., 20.]]);
    assert!(*closed);
    assert_eq!(placement.origin, [0., 0., 3.]);
}

#[test]
fn supported_entity_with_thickness_or_xdata_is_rejected() {
    let mut c = to_cad(&validated(&primitives())).unwrap().into_document();
    let h = c.block_records.get("*Model_Space").unwrap().entity_handles[0];
    let opencadcodec::EntityType::Line(l) = c.get_entity_mut(h).unwrap() else {
        panic!()
    };
    l.thickness = 2.;
    l.common.color_name = Some("book$name".into());
    let Err(IfccadConversionError::Unsupported(d)) = from_cad(&c, metadata()) else {
        panic!()
    };
    assert!(d.iter().any(|d| d.location.ends_with("thickness")));
    assert!(d.iter().any(|d| d.code == "entity-common"));
    let opencadcodec::EntityType::Line(l) = c.get_entity_mut(h).unwrap() else {
        panic!()
    };
    l.thickness = 0.;
    l.common.color_name = None;
    let mut x = opencadcodec::xdata::ExtendedDataRecord::new("ACAD");
    x.values
        .push(opencadcodec::xdata::XDataValue::String("data".into()));
    c.get_entity_mut(h)
        .unwrap()
        .common_mut()
        .extended_data
        .add_record(x);
    assert!(
        matches!(from_cad(&c,metadata()),Err(IfccadConversionError::Unsupported(d)) if d.iter().any(|d|d.code=="entity-common"))
    );
}
#[test]
fn bulges_are_retained_and_width_remains_a_rejectable_loss() {
    for width in [false, true] {
        let mut c = to_cad(&validated(&primitives())).unwrap().into_document();
        let h = c.block_records.get("*Model_Space").unwrap().entity_handles[1];
        let opencadcodec::EntityType::LwPolyline(l) = c.get_entity_mut(h).unwrap() else {
            panic!()
        };
        if width {
            l.vertices[0].start_width = 2.;
        } else {
            l.vertices[0].bulge = 0.5;
        }
        if width {
            assert!(matches!(
                from_cad(&c, metadata()),
                Err(IfccadConversionError::Unsupported(_))
            ));
        } else {
            let out = from_cad(&c, metadata()).unwrap();
            assert!(out.validated_source().document().model.entities.iter().any(
                |e| matches!(&e.as_native().unwrap().kind,IfccadEntityKind::PlanarPolyline{bulges,..} if bulges[0]==0.5)
            ));
        }
    }
}
#[test]
fn noncanonical_placement_obeys_the_geometric_limit_and_keeps_location() {
    let mut d = primitives();
    let IfccadEntityKind::Circle { placement, .. } =
        &mut d.model.entities[2].as_native_mut().unwrap().kind
    else {
        panic!()
    };
    placement.x_axis = [0., 1., 0.];
    placement.y_axis = [-1., 0., 0.];
    assert!(
        matches!(to_cad(&validated(&d)),Err(IfccadConversionError::Geometry(ref failure)) if matches!(&failure.failure.source,ifccad_convert::IfccadGeometryEntitySource::NativeEntity{entity_id:41,..}))
    );
    assert!(ifccad_convert::ifccad_document_to_cad_document(
        &d,
        ifccad_convert::IfccadToCadOptions {
            loss_policy: ifccad_convert::IfccadLossPolicy::Reject,
            ..Default::default()
        }
    )
    .is_ok());
}
#[test]
fn nonrepresentable_opacity_is_rejected_but_layer_description_is_retained() {
    for value in [0.5, 0.123456] {
        let mut d = empty();
        d.layers[0].appearance.opacity = value;
        assert!(
            matches!(to_cad(&validated(&d)),Err(IfccadConversionError::Unsupported(i)) if i.iter().any(|d|d.code=="appearance"))
        );
    }
    let mut c = cad();
    c.layers.get_mut("Notes").unwrap().description = "authored metadata".into();
    let imported = from_cad(&c, metadata()).unwrap();
    assert_eq!(
        imported
            .validated_source()
            .document()
            .layers
            .iter()
            .find(|l| l.name == "Notes")
            .unwrap()
            .description
            .as_deref(),
        Some("authored metadata")
    );
}
#[test]
fn rounding_is_not_silently_accepted() {
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
    assert!(
        matches!(to_cad(&validated(&d)),Err(IfccadConversionError::Geometry(ref failure)) if failure.reason==ifccad_convert::IfccadGeometryFailureReason::ProvenExceedance)
    );
}
#[test]
fn overall_viewport_scaffold_is_checked_by_role_and_values() {
    let mut c = cad();
    let mut viewport = opencadcodec::entities::Viewport::new();
    viewport.id = 1;
    c.add_entity_to_layout(opencadcodec::EntityType::Viewport(viewport), "Layout1")
        .unwrap();
    from_cad(&c, metadata()).unwrap();
    let h = c.block_records.get("*Paper_Space").unwrap().entity_handles[0];
    let opencadcodec::EntityType::Viewport(v) = c.get_entity_mut(h).unwrap() else {
        panic!()
    };
    v.width = 321.;
    let result = cad_document_to_encoded_ifccad(&c, metadata(), Default::default()).unwrap();
    let paper = &result.validated_source().document().paper_layouts[0];
    assert_eq!(paper.canvas.as_ref().unwrap().frame.unwrap().width, 321.);
    assert!(paper.entities.is_empty());
    assert!(
        matches!(from_cad(&c,metadata()),Err(IfccadConversionError::Unsupported(i)) if i.iter().any(|d|d.code == "workspace" && d.location.contains("activeContext")))
    );
}
#[test]
fn layout_record_metadata_and_dangling_layout_links_are_not_dropped() {
    let mut c = cad();
    c.block_records.get_mut("*Model_Space").unwrap().base_point = opencadcodec::Vector3::UNIT_X;
    assert!(matches!(
        from_cad(&c, metadata()),
        Err(IfccadConversionError::Unsupported(_))
    ));
    let mut c = cad();
    let layout = c
        .objects
        .values_mut()
        .find_map(|o| match o {
            opencadcodec::objects::ObjectType::Layout(l) if l.name == "Layout1" => Some(l),
            _ => None,
        })
        .unwrap();
    layout.viewport = opencadcodec::Handle::new(99999);
    assert!(matches!(
        from_cad(&c, metadata()),
        Err(IfccadConversionError::InvalidStructure(_))
    ));
}

#[test]
fn all_unit_codes_and_many_unused_definitions_strict_read_back() {
    let units = [
        "unitless",
        "in",
        "ft",
        "mi",
        "mm",
        "cm",
        "m",
        "km",
        "microin",
        "mil",
        "yd",
        "angstrom",
        "nm",
        "um",
        "dm",
        "dam",
        "hm",
        "Gm",
        "au",
        "ly",
        "pc",
        "usSurveyFoot",
        "usSurveyInch",
        "usSurveyYard",
        "usSurveyMile",
    ];
    for (code, unit) in units.iter().enumerate() {
        let mut source = empty();
        source.length_unit = unit.to_string();
        let cad = to_cad(&validated(&source)).unwrap();
        assert_eq!(cad.document().header.insertion_units, code as i16);
        assert_eq!(
            from_cad(cad.document(), metadata())
                .unwrap()
                .validated_source()
                .document()
                .length_unit,
            *unit
        );
    }
    let mut c = cad();
    for n in 1..=12 {
        let mut layer = opencadcodec::Layer::new(format!("layer{n}"));
        layer.handle = c.allocate_handle();
        layer.color = opencadcodec::Color::from_rgb(255, 255, 255);
        layer.line_weight = opencadcodec::LineWeight::Value(25);
        c.layers.add(layer).unwrap();
        let mut block = opencadcodec::BlockRecord::new(format!("block{n}"));
        block.handle = c.allocate_handle();
        c.block_records.add(block).unwrap();
    }
    let result = from_cad(&c, metadata()).unwrap();
    assert_eq!(result.validated_source().document().blocks.len(), 12);
    assert_eq!(result.validated_source().document().layers.len(), 14);
}
#[test]
fn missing_layer_zero_and_authored_header_are_diagnosed() {
    let mut d = empty();
    d.layers.remove(0);
    assert!(
        matches!(to_cad(&validated(&d)),Err(IfccadConversionError::Unsupported(i)) if i.iter().any(|d|d.code=="layer-0"))
    );
    let mut c = cad();
    c.header.project_name = "authored project".into();
    assert!(
        matches!(from_cad(&c,metadata()),Err(IfccadConversionError::Unsupported(i)) if i.iter().any(|d|d.location=="header.project_name"))
    );
}

#[test]
fn stale_model_cache_requires_unique_agreement() {
    let mut c = cad();
    c.header.model_space_block_handle = opencadcodec::Handle::new(99999);
    let out = from_cad(&c, metadata()).unwrap();
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "model-cache-recovered"));
    c.header.model_space_block_handle = c.header.paper_space_block_handle;
    assert!(matches!(
        from_cad(&c, metadata()),
        Err(IfccadConversionError::InvalidStructure(_))
    ));
}
#[test]
fn equivalent_integer_geometry_is_not_foreign_information() {
    let mut value: serde_json::Value =
        serde_json::from_slice(encode_ifccad_document(&primitives()).unwrap().bytes()).unwrap();
    let line = value["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["path"].as_str().unwrap().ends_with("/e90"))
        .unwrap();
    line["attributes"]["ifccad::geom::lineSegment"]["start"] = serde_json::json!([1, 2, 3]);
    let source =
        load_ifccad_bytes(&serde_json::to_vec(&value).unwrap(), Default::default()).unwrap();
    to_cad(&source).unwrap();
    let line = value["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["path"].as_str().unwrap().ends_with("/e90"))
        .unwrap();
    line["attributes"]["ifccad::geom::lineSegment"]["start"][0] =
        serde_json::json!(9007199254740993_u64);
    let source =
        load_ifccad_bytes(&serde_json::to_vec(&value).unwrap(), Default::default()).unwrap();
    assert!(
        matches!(to_cad(&source),Err(IfccadConversionError::Unsupported(i)) if i.iter().any(|d|d.code=="foreign-ifcx"))
    );
}
