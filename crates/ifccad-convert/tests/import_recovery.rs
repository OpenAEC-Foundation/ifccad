mod common;
use common::*;
use ifccad_convert::*;
use opencadcodec::{objects::ObjectType, CadDocument, Handle};

fn sample() -> CadDocument {
    to_cad(&validated(&primitives())).unwrap().into_document()
}
fn convert(
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
fn assert_recovery(cad: &CadDocument, code: &str) {
    let before = serde_json::to_value(cad).unwrap();
    let result = convert(cad, IfccadLossPolicy::Reject).unwrap();
    assert_eq!(result.validated_source().document().model.entities.len(), 3);
    assert!(result
        .diagnostics()
        .iter()
        .any(|d| d.code == code && !d.is_loss()));
    assert_eq!(serde_json::to_value(cad).unwrap(), before);
    ifccad::load_ifccad_bytes(result.encoded().bytes(), Default::default()).unwrap();
}

#[test]
fn unique_named_membership_recovers_null_dictionary_and_layout_owners() {
    let mut cad = sample();
    let dictionary = cad.header.acad_layout_dict_handle;
    let ObjectType::Dictionary(d) = cad.objects.get_mut(&dictionary).unwrap() else {
        panic!()
    };
    d.owner = Handle::NULL;
    for object in cad.objects.values_mut() {
        if let ObjectType::Layout(l) = object {
            l.owner = Handle::NULL;
        }
    }
    assert_recovery(&cad, "layout-owner-recovered");
}

#[test]
fn a_cache_pointing_to_unrelated_metadata_does_not_override_named_layout_role() {
    let mut cad = sample();
    let handle = cad.allocate_handle();
    let mut d = opencadcodec::objects::Dictionary::new();
    d.handle = handle;
    cad.objects.insert(handle, ObjectType::Dictionary(d));
    cad.header.acad_layout_dict_handle = handle;
    // The extra metadata object is a loss, independently of resolving the cache.
    let result = convert(&cad, IfccadLossPolicy::Allow).unwrap();
    assert_eq!(result.validated_source().document().model.entities.len(), 3);
    assert!(result
        .diagnostics()
        .iter()
        .any(|d| d.code == "layout-dictionary-cache-recovered" && !d.is_loss()));
}

#[test]
fn missing_editor_dictionary_target_is_located_loss() {
    let mut cad = sample();
    let root = cad.header.named_objects_dict_handle;
    let ObjectType::Dictionary(d) = cad.objects.get_mut(&root).unwrap() else {
        panic!()
    };
    d.entries.push(("QCAD_OBJECTS".into(), Handle::new(99999)));
    let before = serde_json::to_value(&cad).unwrap();
    let result = convert(&cad, IfccadLossPolicy::Allow).unwrap();
    assert_eq!(result.validated_source().document().model.entities.len(), 3);
    assert!(result
        .diagnostics()
        .iter()
        .any(|d| d.code == "dictionary-entry-missing"
            && d.location.contains("QCAD_OBJECTS")
            && d.message.contains("99999")
            && d.is_semantic_loss()));
    assert!(matches!(
        convert(&cad, IfccadLossPolicy::Reject),
        Err(IfccadConversionError::Unsupported(_))
    ));
    assert_eq!(serde_json::to_value(&cad).unwrap(), before);
}

#[test]
fn null_owner_is_not_recovered_when_multiple_dictionaries_claim_the_layout() {
    let mut cad = sample();
    let layout = cad
        .objects
        .values_mut()
        .find_map(|o| match o {
            ObjectType::Layout(l) if l.name == "Model" => Some(l),
            _ => None,
        })
        .unwrap();
    layout.owner = Handle::NULL;
    let target = layout.handle;
    let handle = cad.allocate_handle();
    let mut d = opencadcodec::objects::Dictionary::new();
    d.handle = handle;
    d.entries.push(("Model".into(), target));
    cad.objects.insert(handle, ObjectType::Dictionary(d));
    assert!(matches!(
        convert(&cad, IfccadLossPolicy::Allow),
        Err(IfccadConversionError::InvalidStructure(_))
    ));
}

#[test]
fn fontless_style_is_omitted_without_a_substitute_font() {
    let mut cad = sample();
    let mut style = opencadcodec::tables::TextStyle::new("QCADDimTextStyle");
    style.font_file.clear();
    style.true_type_font.clear();
    style.handle = cad.allocate_handle();
    let style_handle = style.handle;
    cad.text_styles.add(style).unwrap();
    let mut dimension_style = cad.dim_styles.get("Standard").unwrap().clone();
    dimension_style.name = "QCADDimStyle".into();
    dimension_style.handle = cad.allocate_handle();
    dimension_style.dimtxsty = "QCADDimTextStyle".into();
    dimension_style.dimtxsty_handle = style_handle;
    cad.dim_styles.add(dimension_style).unwrap();
    let mut text = opencadcodec::Text::with_value("No font", opencadcodec::Vector3::ZERO);
    text.style = "QCADDimTextStyle".into();
    let text_handle = cad
        .add_entity(opencadcodec::EntityType::Text(text))
        .unwrap();
    let mut dimension = opencadcodec::entities::DimensionLinear::new(
        opencadcodec::Vector3::ZERO,
        opencadcodec::Vector3::new(3., 0., 0.),
    );
    dimension.base.style_name = "QCADDimStyle".into();
    let dimension_handle = cad
        .add_entity(opencadcodec::EntityType::Dimension(Box::new(
            opencadcodec::entities::Dimension::Linear(dimension),
        )))
        .unwrap();
    let result = convert(&cad, IfccadLossPolicy::Allow).unwrap();
    assert_eq!(result.validated_source().document().model.entities.len(), 3);
    assert!(!result
        .validated_source()
        .document()
        .text_styles
        .iter()
        .any(|s| s.name == "QCADDimTextStyle"));
    assert!(!result
        .validated_source()
        .document()
        .dimension_styles
        .iter()
        .any(|s| s.name == "QCADDimStyle"));
    assert!(result
        .diagnostics()
        .iter()
        .any(|d| d.location == format!("entity/{text_handle}") && d.is_semantic_loss()));
    assert!(result
        .diagnostics()
        .iter()
        .any(|d| d.location == format!("entity/{dimension_handle}") && d.is_semantic_loss()));
    assert!(result.diagnostics().iter().any(|d| d.code == "text-skipped"
        && d.location.starts_with("textStyle/")
        && d.is_semantic_loss()));
    assert!(matches!(
        convert(&cad, IfccadLossPolicy::Reject),
        Err(IfccadConversionError::Unsupported(_))
    ));
}

#[test]
fn a_single_zero_tab_paper_has_unique_relative_order_after_model() {
    let mut cad = to_cad(&validated(&viewport_drawing()))
        .unwrap()
        .into_document();
    for object in cad.objects.values_mut() {
        if let ObjectType::Layout(l) = object {
            l.tab_order = 0;
        }
    }
    let result = convert(&cad, IfccadLossPolicy::Allow).unwrap();
    assert_eq!(result.validated_source().document().paper_layouts.len(), 1);
    assert_eq!(
        result.validated_source().document().paper_layouts[0].tab_index,
        1
    );
    assert!(result
        .diagnostics()
        .iter()
        .any(|d| d.code == "layout-tabs-normalized" && !d.is_loss()));
    cad.add_layout("AmbiguousSheet").unwrap();
    for object in cad.objects.values_mut() {
        if let ObjectType::Layout(l) = object {
            l.tab_order = 0;
        }
    }
    assert!(matches!(
        convert(&cad, IfccadLossPolicy::Allow),
        Err(IfccadConversionError::InvalidStructure(_))
    ));
}

fn handleless_dictionary_dxf() -> CadDocument {
    let dxf = "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1015\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLINE\n8\n0\n10\n0\n20\n0\n30\n0\n11\n12\n21\n9\n31\n0\n0\nENDSEC\n0\nSECTION\n2\nOBJECTS\n0\nDICTIONARY\n100\nAcDbDictionary\n281\n1\n0\nENDSEC\n0\nEOF\n";
    opencadcodec::DxfReader::from_reader(std::io::Cursor::new(dxf))
        .unwrap()
        .read()
        .unwrap()
}

#[test]
fn handleless_empty_dictionary_model_scaffold_has_a_unique_root_recovery() {
    let cad = handleless_dictionary_dxf();
    assert!(!cad
        .objects
        .contains_key(&cad.header.named_objects_dict_handle));
    let before = serde_json::to_value(&cad).unwrap();
    let result = convert(&cad, IfccadLossPolicy::Allow).unwrap();
    assert_eq!(result.validated_source().document().model.entities.len(), 1);
    assert!(result
        .diagnostics()
        .iter()
        .any(|d| d.code == "model-scaffold-recovered" && !d.is_loss()));
    assert_eq!(serde_json::to_value(&cad).unwrap(), before);
    let exported =
        ifccad_source_to_cad_document(result.validated_source(), Default::default()).unwrap();
    let dxf = opencadcodec::DxfWriter::new(exported.document())
        .write_to_vec()
        .unwrap();
    let reopened = opencadcodec::DxfReader::from_reader(std::io::Cursor::new(dxf))
        .unwrap()
        .read()
        .unwrap();
    assert_eq!(reopened.model_space_entities().count(), 1);
}

#[test]
fn incomplete_model_scaffold_does_not_recover_authored_or_ambiguous_roots() {
    for case in 0..6 {
        let mut cad = handleless_dictionary_dxf();
        let handle = cad.allocate_handle();
        let mut d = opencadcodec::objects::Dictionary::new();
        d.handle = handle;
        match case {
            0 => {
                cad.objects.insert(handle, ObjectType::Dictionary(d));
            }
            1 => {
                let root = cad
                    .objects
                    .values_mut()
                    .find_map(|o| match o {
                        ObjectType::Dictionary(d) if d.owner.is_null() => Some(d),
                        _ => None,
                    })
                    .unwrap();
                root.entries.push(("Authored".into(), Handle::new(99999)));
            }
            2 => {
                cad.header.named_objects_dict_handle = handle;
                cad.objects.insert(handle, ObjectType::Dictionary(d));
            }
            3 => {
                let layout = cad
                    .objects
                    .values_mut()
                    .find_map(|o| match o {
                        ObjectType::Layout(l) => Some(l),
                        _ => None,
                    })
                    .unwrap();
                layout.plot_page_name = "Authored page".into();
            }
            4 => {
                let ObjectType::Dictionary(d) = cad
                    .objects
                    .get_mut(&cad.header.acad_layout_dict_handle)
                    .unwrap()
                else {
                    panic!()
                };
                d.owner = Handle::new(99999);
            }
            5 => {
                let layout = cad
                    .objects
                    .values_mut()
                    .find_map(|o| match o {
                        ObjectType::Layout(l) => Some(l),
                        _ => None,
                    })
                    .unwrap();
                layout.handle = Handle::new(99999);
            }
            _ => unreachable!(),
        }
        assert!(matches!(
            convert(&cad, IfccadLossPolicy::Allow),
            Err(IfccadConversionError::InvalidStructure(_))
        ));
    }
}

#[test]
fn fontless_resource_does_not_hide_invalid_metrics_or_invent_missing_layers() {
    let mut cad = sample();
    let mut style = opencadcodec::tables::TextStyle::new("NoFont");
    style.handle = cad.allocate_handle();
    style.font_file.clear();
    style.true_type_font.clear();
    style.width_factor = -1.;
    cad.text_styles.add(style).unwrap();
    assert!(matches!(
        convert(&cad, IfccadLossPolicy::Allow),
        Err(IfccadConversionError::InvalidStructure(_))
    ));
    let mut cad = sample();
    let entity = cad.entities_mut().next().unwrap();
    let handle = entity.common().handle;
    entity.common_mut().layer = "MissingLayer".into();
    for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        let Err(IfccadConversionError::InvalidStructure(message)) = convert(&cad, policy) else {
            panic!()
        };
        assert!(message.contains("MissingLayer") && message.contains(&handle.to_string()));
    }
}
