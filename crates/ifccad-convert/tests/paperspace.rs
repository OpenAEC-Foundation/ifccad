mod common;
use common::*;
use ifccad_convert::*;
use ocdraw::ifccad::*;
use opencadcodec::objects::{Layout, ObjectType};
use opencadcodec::{CadDocument, EntityType};

fn layout_mut<'a>(cad: &'a mut CadDocument, name: &str) -> &'a mut Layout {
    cad.objects
        .values_mut()
        .find_map(|o| match o {
            ObjectType::Layout(l) if l.name == name => Some(l),
            _ => None,
        })
        .unwrap()
}
fn source() -> CadDocument {
    let mut cad = to_cad(&validated(&nested([0., 0., 0.])))
        .unwrap()
        .into_document();
    cad.add_layout("Empty").unwrap();
    cad.add_layout("Inches").unwrap();
    let layout = layout_mut(&mut cad, "Inches");
    layout.paper_width = 297.;
    layout.paper_height = 210.;
    layout.plot_paper_units = 0;
    layout.plot_scale_type = 1;
    layout.plot_flags.use_standard_scale = false;
    cad.add_entity_to_layout(
        EntityType::Line(opencadcodec::Line::from_coords(1., 2., 0., 3., 4., 0.)),
        "Inches",
    )
    .unwrap();
    cad.add_entity_to_layout(
        EntityType::Circle(opencadcodec::Circle::from_center_radius(
            opencadcodec::Vector3::new(4., 5., 0.),
            2.,
        )),
        "Inches",
    )
    .unwrap();
    let target = cad
        .block_records
        .iter()
        .find(|b| !b.is_model_space() && !b.is_paper_space())
        .unwrap()
        .name
        .clone();
    cad.add_entity_to_layout(
        EntityType::Insert(opencadcodec::entities::Insert::new(
            &target,
            opencadcodec::Vector3::new(8., 9., 0.),
        )),
        "Inches",
    )
    .unwrap();
    cad
}
fn import(
    cad: &CadDocument,
    policy: IfccadLossPolicy,
) -> Result<CadToEncodedIfccadOutcome, IfccadConversionError> {
    cad_document_to_encoded_ifccad(
        cad,
        metadata(),
        CadToIfccadOptions {
            loss_policy: policy,
            ..Default::default()
        },
    )
}
#[test]
fn cad_import_retains_multiple_empty_and_populated_paper_layouts() {
    let cad = source();
    for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        let result = import(&cad, policy).unwrap();
        let loaded = load_ifccad_bytes(result.encoded().bytes(), Default::default()).unwrap();
        let doc = loaded.document();
        assert_eq!(doc.paper_layouts.len(), 2, "{:?}", result.diagnostics());
        let empty = &doc.paper_layouts[0];
        let inches = &doc.paper_layouts[1];
        assert_eq!(
            (&*empty.name, empty.tab_index, &*empty.length_unit),
            ("Empty", 1, "unitless")
        );
        assert!(empty.paper.is_none() && empty.entities.is_empty());
        assert_eq!(
            (&*inches.name, inches.tab_index, &*inches.length_unit),
            ("Inches", 2, "in")
        );
        assert_eq!(
            inches.paper,
            Some(IfccadPaperSize {
                width: 297.,
                height: 210.,
                length_unit: "mm".into()
            })
        );
        assert_eq!(inches.entities.len(), 3);
        for paper in &doc.paper_layouts {
            let handle = result.mappings().layouts.cad_handle(paper.id).unwrap();
            let ObjectType::Layout(original) = &cad.objects[&handle] else {
                panic!()
            };
            let authored: Vec<_> = cad
                .block_records
                .iter()
                .find(|b| b.handle == original.block_record)
                .unwrap()
                .entity_handles
                .iter()
                .filter(|h| !matches!(cad.get_entity(**h), Some(EntityType::Viewport(_))))
                .copied()
                .collect();
            assert_eq!(
                paper
                    .entities
                    .iter()
                    .map(|e| result.mappings().entities.cad_handle(e.id).unwrap())
                    .collect::<Vec<_>>(),
                authored
            );
        }
        assert!(
            matches!(&inches.entities[0].kind,IfccadEntityKind::LineSegment{start,end} if *start==[1.,2.,0.] && *end==[3.,4.,0.])
        );
        assert!(matches!(
            inches.entities[2].kind,
            IfccadEntityKind::BlockInstance { .. }
        ));
        assert!(doc.id_counters.next_layout_id > inches.id);
        validate_ifccad_document(doc).unwrap();
    }
}
#[test]
fn tab_gaps_are_recovery_but_ambiguous_order_and_links_are_fatal() {
    let mut cad = source();
    layout_mut(&mut cad, "Empty").tab_order = 2;
    layout_mut(&mut cad, "Inches").tab_order = 7;
    let result = import(&cad, IfccadLossPolicy::Reject).unwrap();
    assert!(result.diagnostics().iter().any(
        |d| d.code == "layout-tabs-normalized" && d.action == IfccadDiagnosticAction::Recovery
    ));
    for change in 0..4 {
        let mut bad = cad.clone();
        match change {
            0 => layout_mut(&mut bad, "Inches").tab_order = 2,
            1 => layout_mut(&mut bad, "Inches").tab_order = -1,
            2 => layout_mut(&mut bad, "Model").tab_order = 1,
            _ => {
                let block = layout_mut(&mut bad, "Empty").block_record;
                layout_mut(&mut bad, "Inches").block_record = block;
            }
        }
        for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
            assert!(matches!(
                import(&bad, policy),
                Err(IfccadConversionError::InvalidStructure(_))
            ));
        }
    }
}
#[test]
fn invalid_media_and_unsupported_viewports_keep_supported_siblings_with_losses() {
    let mut cad = source();
    layout_mut(&mut cad, "Inches").paper_height = 0.;
    let result = import(&cad, IfccadLossPolicy::Allow).unwrap();
    let doc = result.validated_source().document();
    assert_eq!(doc.paper_layouts.len(), 2);
    assert_eq!(doc.paper_layouts[1].entities.len(), 3);
    assert!(doc.paper_layouts[1].paper.is_none());
    assert!(result
        .diagnostics()
        .iter()
        .any(|d| d.code == "paper-medium"));
    assert!(matches!(
        import(&cad, IfccadLossPolicy::Reject),
        Err(IfccadConversionError::Unsupported(_))
    ));
    let mut viewport = opencadcodec::entities::Viewport::new();
    viewport.center.z = 1.;
    let h = cad
        .add_entity_to_layout(EntityType::Viewport(viewport), "Inches")
        .unwrap();
    let result = import(&cad, IfccadLossPolicy::Allow).unwrap();
    assert_eq!(
        result.validated_source().document().paper_layouts[1]
            .entities
            .len(),
        3
    );
    assert!(result
        .diagnostics()
        .iter()
        .any(|d| d.location == format!("entity/{h}") && d.is_loss()));
}

fn native_papers() -> IfccadDocument {
    let mut doc = nested([0., 0., 0.]);
    doc.id_counters.next_layout_id = 91;
    let mut line = primitives().model.entities[0].clone();
    line.id = 501;
    let instance = instance(502, doc.blocks[0].id, [8., 9., 0.]);
    doc.paper_layouts = vec![
        IfccadPaperLayout {
            bounds: None,
            id: 3,
            name: "Inches".into(),
            tab_index: 2,
            length_unit: "in".into(),
            paper: Some(IfccadPaperSize {
                width: 297.,
                height: 210.,
                length_unit: "mm".into(),
            }),
            entities: vec![line, instance],
        },
        IfccadPaperLayout {
            bounds: None,
            id: 90,
            name: "Empty".into(),
            tab_index: 1,
            length_unit: "unitless".into(),
            paper: None,
            entities: vec![],
        },
    ];
    doc
}
#[test]
fn native_papers_allocate_ordered_owners_and_survive_both_cad_codecs() {
    let original = native_papers();
    let output = to_cad(&validated(&original)).unwrap();
    for source in &original.paper_layouts {
        let handle = output.mappings().layouts.cad_handle(source.id).unwrap();
        let ObjectType::Layout(layout) = &output.document().objects[&handle] else {
            panic!()
        };
        assert_eq!(layout.tab_order, i16::try_from(source.tab_index).unwrap());
        for entity in &source.entities {
            let handle = output.mappings().entities.cad_handle(entity.id).unwrap();
            assert_eq!(
                output
                    .document()
                    .get_entity(handle)
                    .unwrap()
                    .common()
                    .owner_handle,
                layout.block_record
            );
        }
    }
    for (codec, cad) in [
        output.document().clone(),
        opencadcodec::DxfReader::from_reader(std::io::Cursor::new(
            opencadcodec::DxfWriter::new(output.document())
                .write_to_vec()
                .unwrap(),
        ))
        .unwrap()
        .read()
        .unwrap(),
        opencadcodec::DwgReader::from_stream(std::io::Cursor::new(
            opencadcodec::DwgWriter::write_to_vec(output.document()).unwrap(),
        ))
        .read()
        .unwrap(),
    ]
    .into_iter()
    .enumerate()
    {
        let restored = import(&cad, IfccadLossPolicy::Reject)
            .unwrap_or_else(|e| panic!("codec {codec}: {e:?}"));
        let doc = restored.validated_source().document();
        assert_eq!(doc.paper_layouts.len(), 2);
        assert_eq!(
            (&*doc.paper_layouts[0].name, &*doc.paper_layouts[1].name),
            ("Empty", "Inches")
        );
        assert!(doc.paper_layouts[0].paper.is_none() && doc.paper_layouts[0].entities.is_empty());
        let sheet = &doc.paper_layouts[1];
        assert_eq!(sheet.length_unit, "in");
        assert_eq!(sheet.paper, original.paper_layouts[0].paper);
        assert_eq!(
            sheet.entities[0].kind,
            original.paper_layouts[0].entities[0].kind
        );
        let (
            IfccadEntityKind::BlockInstance {
                definition_id: before,
                transform: a,
            },
            IfccadEntityKind::BlockInstance {
                definition_id: after,
                transform: b,
            },
        ) = (
            &original.paper_layouts[0].entities[1].kind,
            &sheet.entities[1].kind,
        )
        else {
            panic!()
        };
        assert_eq!(a, b);
        assert_eq!(
            original
                .blocks
                .iter()
                .find(|d| d.id == *before)
                .unwrap()
                .name,
            doc.blocks.iter().find(|d| d.id == *after).unwrap().name
        );
        let mut expected = original.paper_layouts[0].entities[0].appearance.clone();
        if let (IfccadMode::Explicit(before), IfccadMode::Explicit(after)) = (
            &expected.line_pattern,
            &sheet.entities[0].appearance.line_pattern,
        ) {
            assert_eq!(
                original
                    .line_patterns
                    .iter()
                    .find(|p| p.id == *before)
                    .unwrap()
                    .name,
                doc.line_patterns
                    .iter()
                    .find(|p| p.id == *after)
                    .unwrap()
                    .name
            );
            expected.line_pattern = IfccadMode::Explicit(*after);
        }
        assert_eq!(sheet.entities[0].appearance, expected);
        validate_ifccad_document(doc).unwrap();
    }
}
#[test]
fn media_conversion_is_exact_and_unsupported_coordinates_are_located() {
    let mut doc = native_papers();
    doc.paper_layouts[0].paper = Some(IfccadPaperSize {
        width: 5.,
        height: 10.,
        length_unit: "in".into(),
    });
    let output = to_cad(&validated(&doc)).unwrap();
    let handle = output.mappings().layouts.cad_handle(3).unwrap();
    let ObjectType::Layout(layout) = &output.document().objects[&handle] else {
        panic!()
    };
    assert_eq!((layout.paper_width, layout.paper_height), (127., 254.));
    doc.paper_layouts[0].paper.as_mut().unwrap().width = 1.;
    for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        assert!(
            matches!(ifccad_document_to_cad_document(&doc,IfccadToCadOptions{loss_policy:policy, ..Default::default()}),Err(IfccadConversionError::Unsupported(d)) if d.iter().any(|d|d.code=="rounding"))
        );
    }
    doc.paper_layouts[0].paper = None;
    doc.paper_layouts[0].length_unit = "cm".into();
    let output = ifccad_document_to_cad_document(&doc, Default::default()).unwrap();
    assert!(output.mappings().layouts.cad_handle(3).is_none());
    assert!(output.mappings().entities.cad_handle(501).is_none());
    assert!(output
        .diagnostics()
        .iter()
        .any(|d| d.code == "paper-coordinate-unit" && d.location == "layout/3"));
    assert!(ifccad_document_to_cad_document(
        &doc,
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        }
    )
    .is_err());
}

#[test]
fn authored_plot_mappings_and_settings_have_located_losses() {
    for change in 0..5 {
        let mut cad = source();
        let sheet = layout_mut(&mut cad, "Inches");
        match change {
            0 => sheet.plot_paper_units = 2,
            1 => sheet.plot_scale_denominator = 2.,
            2 => sheet.plot_printer_name = "Authored printer".into(),
            3 => sheet.plot_scale_factor = 2.,
            _ => sheet.plot_flags.use_standard_scale = true,
        }
        let output = import(&cad, IfccadLossPolicy::Allow).unwrap();
        assert_eq!(
            output.validated_source().document().paper_layouts[1]
                .entities
                .len(),
            3
        );
        if change != 2 {
            assert_eq!(
                output.validated_source().document().paper_layouts[1].length_unit,
                "unitless"
            );
            assert!(output
                .diagnostics()
                .iter()
                .any(|d| d.code == "paper-coordinate-unit"
                    && d.location == "layout/Inches.plotMapping"));
        }
        assert!(import(&cad, IfccadLossPolicy::Reject).is_err());
    }
}

#[test]
fn target_layout_name_collisions_and_tab_overflow_are_fatal() {
    let mut doc = native_papers();
    doc.paper_layouts[0].name = "I".into();
    doc.paper_layouts[1].name = "ı".into();
    validate_ifccad_document(&doc).unwrap();
    for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        assert!(matches!(
            ifccad_document_to_cad_document(
                &doc,
                IfccadToCadOptions {
                    loss_policy: policy,
                    ..Default::default()
                }
            ),
            Err(IfccadConversionError::CadConstruction(_))
        ));
    }
    // Keep the contiguous native contract valid while exceeding CAD's signed range.
    let sheet = doc.paper_layouts[1].clone();
    doc.model.id = 0;
    doc.paper_layouts = (1..=32768)
        .map(|tab| IfccadPaperLayout {
            id: u64::from(tab),
            name: format!("Sheet {tab}"),
            tab_index: tab,
            ..sheet.clone()
        })
        .collect();
    doc.id_counters.next_layout_id = 32769;
    validate_ifccad_document(&doc).unwrap();
    assert!(matches!(
        ifccad_document_to_cad_document(&doc, Default::default()),
        Err(IfccadConversionError::CadConstruction(_))
    ));
}

#[test]
fn unsupported_medium_omits_only_media_and_definition_loss_reaches_paper() {
    let mut doc = native_papers();
    doc.paper_layouts[0].paper.as_mut().unwrap().length_unit = "pc".into();
    let output = ifccad_document_to_cad_document(&doc, Default::default()).unwrap();
    assert!(output.mappings().entities.cad_handle(501).is_some());
    assert!(output
        .diagnostics()
        .iter()
        .any(|d| d.code == "paper-medium" && d.location == "layout/3.paper"));
    assert!(ifccad_document_to_cad_document(
        &doc,
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        }
    )
    .is_err());
    let mut cad = source();
    let inner = cad.block_records.get("Inner").unwrap().entity_handles[0];
    cad.get_entity_mut(inner).unwrap().common_mut().invisible = true;
    let output = import(&cad, IfccadLossPolicy::Allow).unwrap();
    let sheet = &output.validated_source().document().paper_layouts[1];
    let instance = sheet.entities.last().unwrap();
    assert!(output
        .diagnostics()
        .iter()
        .any(|d| d.code == "block-content-loss"
            && d.location
                == format!(
                    "entity/{}",
                    output.mappings().entities.cad_handle(instance.id).unwrap()
                )));
    let mut native = native_papers();
    native.blocks[0].entities[0].appearance.opacity = IfccadMode::Explicit(0.5);
    let output = ifccad_document_to_cad_document(&native, Default::default()).unwrap();
    assert!(output
        .diagnostics()
        .iter()
        .any(|d| d.code == "block-content-loss" && d.location == "entity/502"));
}
