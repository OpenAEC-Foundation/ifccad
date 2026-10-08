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
        if policy == IfccadLossPolicy::Reject {
            assert!(
                matches!(import(&cad,policy),Err(IfccadConversionError::Unsupported(issues)) if issues.iter().any(|d|d.code == "workspace" && d.location.ends_with(".activeContext/currentUcs")))
            );
            continue;
        }
        let result = import_workspace(&cad);
        let loaded = load_ifccad_bytes(result.encoded().bytes(), Default::default()).unwrap();
        let doc = loaded.document();
        assert_eq!(doc.paper_layouts.len(), 2, "{:?}", result.diagnostics());
        let empty = &doc.paper_layouts[0];
        let inches = &doc.paper_layouts[1];
        assert_eq!((&*empty.name, empty.tab_index), ("Empty", 1));
        assert!(empty.settings.media.is_none() && empty.entities.is_empty());
        assert_eq!((&*inches.name, inches.tab_index), ("Inches", 2));
        assert_eq!(
            inches.settings.media,
            Some(ocdraw::ifccad::IfccadLayoutMedia {
                width: 297.,
                height: 210.,
                unit: ocdraw::ifccad::IfccadMediaUnit::from_token("mm").unwrap()
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
                    .map(|e| result.mappings().entities.cad_handle(e.id()).unwrap())
                    .collect::<Vec<_>>(),
                authored
            );
        }
        assert!(
            matches!(&inches.entities[0].as_native().unwrap().kind,IfccadEntityKind::LineSegment{start,end} if *start==[1.,2.,0.] && *end==[3.,4.,0.])
        );
        assert!(matches!(
            inches.entities[2].as_native().unwrap().kind,
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
    let result = import_workspace(&cad);
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
    assert!(doc.paper_layouts[1].settings.media.is_none());
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
    line.as_native_mut().unwrap().id = 501;
    let instance = instance(502, doc.blocks[0].id, [8., 9., 0.]);
    doc.paper_layouts = vec![
        IfccadPaperLayout {
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
            id: 3,
            name: "Inches".into(),
            tab_index: 2,
            entities: vec![line, instance],
        },
        IfccadPaperLayout {
            canvas: None,

            settings: ocdraw::ifccad::IfccadLayoutSettings {
                media: None,
                ..Default::default()
            },
            bounds: None,
            id: 90,
            name: "Empty".into(),
            tab_index: 1,
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
            let handle = output.mappings().entities.cad_handle(entity.id()).unwrap();
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
    for cad in [
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
    ] {
        let restored = import_workspace(&cad);
        let doc = restored.validated_source().document();
        assert_eq!(doc.paper_layouts.len(), 2);
        assert_eq!(
            (&*doc.paper_layouts[0].name, &*doc.paper_layouts[1].name),
            ("Empty", "Inches")
        );
        assert!(
            doc.paper_layouts[0].settings.media.is_none()
                && doc.paper_layouts[0].entities.is_empty()
        );
        let sheet = &doc.paper_layouts[1];
        assert!(sheet.settings.plot_settings.is_none());
        assert_eq!(
            sheet.settings.media,
            original.paper_layouts[0].settings.media
        );
        assert_eq!(
            sheet.entities[0].as_native().unwrap().kind,
            original.paper_layouts[0].entities[0]
                .as_native()
                .unwrap()
                .kind
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
            &original.paper_layouts[0].entities[1]
                .as_native()
                .unwrap()
                .kind,
            &sheet.entities[1].as_native().unwrap().kind,
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
        let mut expected = original.paper_layouts[0].entities[0]
            .as_native()
            .unwrap()
            .appearance
            .clone();
        if let (IfccadMode::Explicit(before), IfccadMode::Explicit(after)) = (
            &expected.line_pattern,
            &sheet.entities[0]
                .as_native()
                .unwrap()
                .appearance
                .line_pattern,
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
        assert_eq!(sheet.entities[0].as_native().unwrap().appearance, expected);
        validate_ifccad_document(doc).unwrap();
    }
}
#[test]
fn media_conversion_is_exact_and_unsupported_coordinates_are_located() {
    let mut doc = native_papers();
    doc.paper_layouts[0].settings.media = Some(ocdraw::ifccad::IfccadLayoutMedia {
        width: 5.,
        height: 10.,
        unit: ocdraw::ifccad::IfccadMediaUnit::from_token("in").unwrap(),
    });
    let output = to_cad(&validated(&doc)).unwrap();
    let handle = output.mappings().layouts.cad_handle(3).unwrap();
    let ObjectType::Layout(layout) = &output.document().objects[&handle] else {
        panic!()
    };
    assert_eq!((layout.paper_width, layout.paper_height), (127., 254.));
    doc.paper_layouts[0].settings.media.as_mut().unwrap().width = 1.;
    for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        assert!(matches!(
            ifccad_document_to_cad_document(
                &doc,
                IfccadToCadOptions {
                    loss_policy: policy,
                    ..Default::default()
                }
            ),
            Err(IfccadConversionError::PlotNumeric(
                cad_geometry_convert::plot_units::PlotNumericError::Inexact
            ))
        ));
    }
    doc.paper_layouts[0].settings.media = None;
    let output = ifccad_document_to_cad_document(&doc, Default::default()).unwrap();
    assert!(output.mappings().layouts.cad_handle(3).is_some());
    assert!(output.mappings().entities.cad_handle(501).is_some());
}

#[test]
fn authored_plot_mappings_and_settings_have_located_losses() {
    let mut cad = source();
    let sheet = layout_mut(&mut cad, "Inches");
    sheet.plot_paper_units = 2;
    let output = import(&cad, IfccadLossPolicy::Allow).unwrap();
    let paper = &output.validated_source().document().paper_layouts[1];
    assert!(paper.settings.media.is_some());
    assert!(paper.settings.plot_settings.is_none());
    assert_eq!(paper.entities.len(), 3);
    assert!(output
        .diagnostics()
        .iter()
        .any(|d| d.code == "plot-settings"));
    assert!(import(&cad, IfccadLossPolicy::Reject).is_err());
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
            canvas: None,

            settings: ocdraw::ifccad::IfccadLayoutSettings {
                media: None,
                ..Default::default()
            },
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
    doc.paper_layouts[0].settings.media.as_mut().unwrap().unit =
        ocdraw::ifccad::IfccadMediaUnit::from_token("pc").unwrap();
    let output = ifccad_document_to_cad_document(&doc, Default::default()).unwrap();
    assert!(output.mappings().entities.cad_handle(501).is_some());
    assert!(output
        .diagnostics()
        .iter()
        .any(|d| d.code == "paper-medium" && d.location == "layout/Inches"));
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
                    output
                        .mappings()
                        .entities
                        .cad_handle(instance.id())
                        .unwrap()
                )));
    let mut native = native_papers();
    native.blocks[0].entities[0]
        .as_native_mut()
        .unwrap()
        .appearance
        .opacity = IfccadMode::Explicit(0.5);
    let output = ifccad_document_to_cad_document(&native, Default::default()).unwrap();
    assert!(output
        .diagnostics()
        .iter()
        .any(|d| d.code == "block-content-loss" && d.location == "entity/502"));
}

fn import_workspace(cad: &CadDocument) -> CadToEncodedIfccadOutcome {
    let result = import(cad, IfccadLossPolicy::Allow).unwrap();
    assert!(
        result
            .diagnostics()
            .iter()
            .filter(|d| d.is_semantic_loss())
            .all(|d| d.code == "workspace" && d.location.ends_with(".activeContext/currentUcs")),
        "unexpected source loss: {:?}",
        result.diagnostics()
    );
    result
}
