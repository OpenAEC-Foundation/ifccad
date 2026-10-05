mod common;
use common::*;
use ifccad_convert::*;
use ocdraw::ifccad::load_ifccad_bytes;
use opencadcodec::{objects::ObjectType, CadDocument, EntityType, Handle};
use std::io::Cursor;

fn sample() -> CadDocument {
    to_cad(&validated(&primitives())).unwrap().into_document()
}
fn layout_mut<'a>(cad: &'a mut CadDocument, name: &str) -> &'a mut opencadcodec::objects::Layout {
    cad.objects
        .values_mut()
        .find_map(|o| match o {
            ObjectType::Layout(l) if l.name == name => Some(l),
            _ => None,
        })
        .unwrap()
}
fn relocate_layout_dictionary(cad: &mut CadDocument) -> (Handle, Handle) {
    let old = cad.header.acad_layout_dict_handle;
    let new = cad.allocate_handle();
    let ObjectType::Dictionary(mut dictionary) = cad.objects.remove(&old).unwrap() else {
        panic!()
    };
    dictionary.handle = new;
    let ObjectType::Dictionary(root) = cad
        .objects
        .get_mut(&cad.header.named_objects_dict_handle)
        .unwrap()
    else {
        panic!()
    };
    root.entries
        .iter_mut()
        .find(|(name, _)| name == "ACAD_LAYOUT")
        .unwrap()
        .1 = new;
    for object in cad.objects.values_mut() {
        if let ObjectType::Layout(l) = object {
            l.owner = new;
        }
    }
    cad.objects.insert(new, ObjectType::Dictionary(dictionary));
    (old, new)
}
fn encoded(
    cad: &CadDocument,
    policy: IfccadLossPolicy,
) -> Result<CadToEncodedIfccadOutcome, IfccadConversionError> {
    cad_document_to_encoded_ifccad(
        cad,
        metadata(),
        CadToIfccadOptions {
            loss_policy: policy,
        },
    )
}
fn assert_invalid(cad: &CadDocument) {
    for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        let options = CadToIfccadOptions {
            loss_policy: policy,
        };
        assert!(matches!(
            cad_document_to_ifccad_document(cad, metadata(), options),
            Err(IfccadConversionError::InvalidStructure(_))
        ));
        assert!(matches!(
            encoded(cad, policy),
            Err(IfccadConversionError::InvalidStructure(_))
        ));
    }
}
#[test]
fn stale_layout_dictionary_cache_recovers_without_source_mutation_or_false_loss() {
    for cache in 0..3 {
        let mut cad = sample();
        let (old, new) = relocate_layout_dictionary(&mut cad);
        match cache {
            0 => {}
            1 => cad.header.acad_layout_dict_handle = Handle::NULL,
            2 => cad.header.acad_layout_dict_handle = layout_mut(&mut cad, "Model").handle,
            _ => unreachable!(),
        }
        let before = serde_json::to_value(&cad).unwrap();
        for policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
            let result = encoded(&cad, policy).unwrap();
            let logical = cad_document_to_ifccad_document(
                &cad,
                metadata(),
                CadToIfccadOptions {
                    loss_policy: policy,
                },
            )
            .unwrap();
            assert_eq!(logical.document(), result.validated_source().document());
            assert_eq!(logical.diagnostics(), result.diagnostics());
            assert_eq!(result.validated_source().document().model.entities.len(), 3);
            assert!(result
                .diagnostics()
                .iter()
                .any(|d| d.code == "layout-dictionary-cache-recovered"
                    && d.location == "header.acad_layout_dict_handle"
                    && !d.is_loss()));
            assert!(!result
                .diagnostics()
                .iter()
                .any(|d| d.code == "object" && d.location == format!("object/{new}")));
            load_ifccad_bytes(result.encoded().bytes(), Default::default()).unwrap();
        }
        assert_eq!(serde_json::to_value(&cad).unwrap(), before);
        assert!(!cad.objects.contains_key(&old));
    }
}
#[test]
fn dxf_import_resolves_relocated_named_layout_dictionary() {
    let mut cad = sample();
    let (_, new) = relocate_layout_dictionary(&mut cad);
    cad.header.acad_layout_dict_handle = new;
    let bytes = opencadcodec::DxfWriter::new(&cad).write_to_vec().unwrap();
    let read = opencadcodec::DxfReader::from_reader(Cursor::new(bytes))
        .unwrap()
        .read()
        .unwrap();
    assert_ne!(
        read.header.acad_layout_dict_handle, new,
        "characterize pinned DXF cache defect"
    );
    let result = encoded(&read, IfccadLossPolicy::Reject).unwrap();
    assert_eq!(result.validated_source().document().model.entities.len(), 3);
    assert!(result
        .diagnostics()
        .iter()
        .any(|d| d.code == "layout-dictionary-cache-recovered" && !d.is_loss()));
}
#[test]
fn layout_dictionary_real_conflicts_remain_fatal() {
    for case in 0..12 {
        let mut cad = sample();
        let (_, target) = relocate_layout_dictionary(&mut cad);
        match case {
            0 => {
                cad.objects.remove(&target);
            }
            1 => {
                cad.header.acad_layout_dict_handle = cad.header.named_objects_dict_handle;
            }
            2 => {
                let ObjectType::Dictionary(root) = cad
                    .objects
                    .get_mut(&cad.header.named_objects_dict_handle)
                    .unwrap()
                else {
                    panic!()
                };
                root.entries.push(("ACAD_LAYOUT".into(), target));
            }
            3 => {
                let ObjectType::Dictionary(d) = cad.objects.get_mut(&target).unwrap() else {
                    panic!()
                };
                d.owner = Handle::new(99999);
            }
            4 => {
                let ObjectType::Dictionary(d) = cad.objects.get_mut(&target).unwrap() else {
                    panic!()
                };
                d.handle = Handle::new(99999);
            }
            5 => {
                layout_mut(&mut cad, "Model").owner = Handle::new(99999);
            }
            6 => {
                let ObjectType::Dictionary(d) = cad.objects.get_mut(&target).unwrap() else {
                    panic!()
                };
                d.entries.iter_mut().find(|(n, _)| n == "Model").unwrap().0 = "WrongModel".into();
            }
            7 => {
                let ObjectType::Dictionary(root) = cad
                    .objects
                    .get_mut(&cad.header.named_objects_dict_handle)
                    .unwrap()
                else {
                    panic!()
                };
                root.entries.retain(|(n, _)| n != "ACAD_LAYOUT");
            }
            8 => {
                let model = layout_mut(&mut cad, "Model").handle;
                let ObjectType::Dictionary(root) = cad
                    .objects
                    .get_mut(&cad.header.named_objects_dict_handle)
                    .unwrap()
                else {
                    panic!()
                };
                root.entries
                    .iter_mut()
                    .find(|(n, _)| n == "ACAD_LAYOUT")
                    .unwrap()
                    .1 = model;
            }
            9 => {
                cad.objects.remove(&cad.header.named_objects_dict_handle);
            }
            10 => {
                let ObjectType::Dictionary(root) = cad
                    .objects
                    .get_mut(&cad.header.named_objects_dict_handle)
                    .unwrap()
                else {
                    panic!()
                };
                root.handle = Handle::new(99999);
            }
            11 => {
                let ObjectType::Dictionary(root) = cad
                    .objects
                    .get_mut(&cad.header.named_objects_dict_handle)
                    .unwrap()
                else {
                    panic!()
                };
                root.owner = target;
            }
            _ => unreachable!(),
        }
        assert_invalid(&cad);
    }
}
#[test]
fn valid_model_vport_reference_is_located_loss_instead_of_structure_failure() {
    for list in [false, true] {
        let mut cad = sample();
        let handle = cad.vports.get("*Active").unwrap().handle;
        if list {
            layout_mut(&mut cad, "Model").viewports.push(handle);
        } else {
            layout_mut(&mut cad, "Model").viewport = handle;
        }
        let before = serde_json::to_value(&cad).unwrap();
        let result = encoded(&cad, IfccadLossPolicy::Allow).unwrap();
        let logical =
            cad_document_to_ifccad_document(&cad, metadata(), Default::default()).unwrap();
        assert_eq!(logical.document(), result.validated_source().document());
        assert_eq!(logical.diagnostics(), result.diagnostics());
        assert_eq!(result.validated_source().document().model.entities.len(), 3);
        let location = if list {
            "layout/Model.viewports"
        } else {
            "layout/Model.viewport"
        };
        assert!(result
            .diagnostics()
            .iter()
            .any(|d| d.code == "model-viewport-selection"
                && d.location == location
                && d.is_loss()));
        assert!(
            matches!(encoded(&cad,IfccadLossPolicy::Reject),Err(IfccadConversionError::Unsupported(d)) if d.iter().any(|d|d.code=="model-viewport-selection"))
        );
        assert_eq!(serde_json::to_value(&cad).unwrap(), before);
    }
}
#[test]
fn dwg_model_vport_reference_survives_reader_and_strict_native_readback() {
    let mut cad = sample();
    let vport = cad.vports.get("*Active").unwrap().handle;
    layout_mut(&mut cad, "Model").viewport = vport;
    let bytes = opencadcodec::DwgWriter::write_to_vec(&cad).unwrap();
    let read = opencadcodec::DwgReader::from_stream(Cursor::new(bytes))
        .read()
        .unwrap();
    let result = encoded(&read, IfccadLossPolicy::Allow).unwrap();
    assert_eq!(result.validated_source().document().model.entities.len(), 3);
    assert!(result
        .diagnostics()
        .iter()
        .any(|d| d.code == "model-viewport-selection"));
    load_ifccad_bytes(result.encoded().bytes(), Default::default()).unwrap();
}
#[test]
fn missing_and_wrong_kind_viewport_references_remain_fatal() {
    for paper in [false, true] {
        for case in 0..3 {
            let mut cad = sample();
            if paper {
                cad.add_layout("Sheet").unwrap();
            }
            let handle = match case {
                0 => Handle::new(99999),
                1 => cad.layers.get("0").unwrap().handle,
                2 => {
                    if paper {
                        cad.vports.get("*Active").unwrap().handle
                    } else {
                        let mut v = opencadcodec::entities::Viewport::new();
                        v.id = 2;
                        cad.add_entity_to_layout(EntityType::Viewport(v), "Layout1")
                            .unwrap()
                    }
                }
                _ => unreachable!(),
            };
            layout_mut(&mut cad, if paper { "Sheet" } else { "Model" }).viewport = handle;
            assert_invalid(&cad);
        }
    }
}
#[test]
fn paper_viewport_reference_requires_the_same_paper_owner() {
    let mut cad = sample();
    cad.add_layout("Sheet").unwrap();
    let mut v = opencadcodec::entities::Viewport::new();
    v.id = 2;
    let handle = cad
        .add_entity_to_layout(EntityType::Viewport(v), "Layout1")
        .unwrap();
    layout_mut(&mut cad, "Sheet").viewport = handle;
    assert_invalid(&cad);
}
