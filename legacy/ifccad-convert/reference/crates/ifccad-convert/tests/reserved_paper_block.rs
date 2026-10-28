mod common;

use common::metadata;
use ifccad::load_ifccad_bytes;
use ifccad_convert::*;
use opencadcodec::{objects::ObjectType, CadDocument, EntityType, Handle, Line};
use std::io::Cursor;

fn model_only() -> CadDocument {
    let mut cad = CadDocument::new();
    cad.add_entity(EntityType::Line(Line::from_coords(0., 0., 0., 5., 2., 0.)))
        .unwrap();
    let paper = cad.header.paper_space_block_handle;
    let layout = cad.block_records.get("*Paper_Space").unwrap().layout;
    assert!(cad
        .block_records
        .get("*Paper_Space")
        .unwrap()
        .entity_handles
        .is_empty());
    cad.objects.remove(&layout).unwrap();
    let Some(ObjectType::Dictionary(dictionary)) =
        cad.objects.get_mut(&cad.header.acad_layout_dict_handle)
    else {
        panic!("layout dictionary");
    };
    dictionary.entries.retain(|(_, h)| *h != layout);
    cad.block_records.get_mut("*Paper_Space").unwrap().layout = Handle::NULL;
    assert!(!cad
        .objects
        .values()
        .any(|o| matches!(o, ObjectType::Layout(l) if l.block_record == paper)));
    cad
}

#[test]
fn model_only_reserved_paper_block_survives_strict_ifccad_and_dxf_roundtrip() {
    for physical_source in [false, true] {
        let source = model_only();
        let cad = if physical_source {
            let bytes = opencadcodec::DxfWriter::new(&source)
                .write_to_vec()
                .unwrap();
            opencadcodec::DxfReader::from_reader(Cursor::new(bytes))
                .unwrap()
                .read()
                .unwrap()
        } else {
            source
        };
        let before = serde_json::to_value(&cad).unwrap();
        for loss_policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
            let native = cad_document_to_encoded_ifccad(
                &cad,
                metadata(),
                CadToIfccadOptions {
                    loss_policy,
                    ..Default::default()
                },
            )
            .unwrap();
            let strict = load_ifccad_bytes(native.encoded().bytes(), Default::default()).unwrap();
            assert_eq!(strict.document().model.entities.len(), 1);
            assert!(strict.document().paper_layouts.is_empty());
            let restored = ifccad_source_to_cad_document(
                &strict,
                IfccadToCadOptions {
                    loss_policy,
                    ..Default::default()
                },
            )
            .unwrap();
            let bytes = opencadcodec::DxfWriter::new(restored.document())
                .write_to_vec()
                .unwrap();
            let returned = opencadcodec::DxfReader::from_reader(Cursor::new(bytes))
                .unwrap()
                .read()
                .unwrap();
            let line = returned
                .entities()
                .find_map(|e| {
                    if let EntityType::Line(l) = e {
                        Some(l)
                    } else {
                        None
                    }
                })
                .unwrap();
            assert_eq!(line.start, opencadcodec::Vector3::ZERO);
            assert_eq!(line.end, opencadcodec::Vector3::new(5., 2., 0.));
        }
        assert_eq!(serde_json::to_value(&cad).unwrap(), before);
    }
}

#[test]
fn unowned_paper_content_or_authored_record_is_still_invalid() {
    for case in 0..7 {
        let mut cad = model_only();
        match case {
            0 => {
                cad.block_records
                    .get_mut("*Paper_Space")
                    .unwrap()
                    .description = "authored".into()
            }
            1 => cad.block_records.get_mut("*Paper_Space").unwrap().layout = Handle::new(99999),
            2 => {
                cad.block_records
                    .get_mut("*Paper_Space")
                    .unwrap()
                    .base_point
                    .x = 3.
            }
            3 => {
                cad.block_records
                    .get_mut("*Paper_Space")
                    .unwrap()
                    .entity_handles
                    .push(Handle::new(99999));
            }
            4 => {
                cad.header.paper_space_block_handle = Handle::new(99999);
            }
            5 => {
                let h = cad.allocate_handle();
                let mut paper = opencadcodec::BlockRecord::new("*Paper_Space0");
                paper.handle = h;
                cad.block_records.add(paper).unwrap();
            }
            6 => {
                let paper = cad.header.paper_space_block_handle;
                let mut line = Line::from_coords(0., 0., 0., 1., 0., 0.);
                line.common.owner_handle = paper;
                cad.add_entity(EntityType::Line(line)).unwrap();
            }
            _ => unreachable!(),
        }
        for loss_policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
            assert!(
                matches!(
                    cad_document_to_encoded_ifccad(
                        &cad,
                        metadata(),
                        CadToIfccadOptions {
                            loss_policy,
                            ..Default::default()
                        }
                    ),
                    Err(IfccadConversionError::InvalidStructure(_))
                ),
                "case {case}"
            );
        }
    }
}
