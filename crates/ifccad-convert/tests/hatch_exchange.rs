use ifccad_convert::*;
use ocdraw::geometry_kernel::hatch::*;
use ocdraw::ifccad::*;
use opencadcodec::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter};
use std::io::Cursor;
#[path = "../../../tests/support/hatch.rs"]
mod contours;
#[path = "../../../tests/support/ifccad_hatch.rs"]
mod fixture;
fn exchange(d: &CadDocument, dwg: bool) -> CadDocument {
    if dwg {
        DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(d).unwrap()))
            .read()
            .unwrap()
    } else {
        DxfReader::from_reader(Cursor::new(DxfWriter::new(d).write_to_vec().unwrap()))
            .unwrap()
            .read()
            .unwrap()
    }
}
#[test]
fn explicit_pattern_survives_dxf_and_dwg_with_phase_and_ordered_dashes() {
    let d = load_ifccad_bytes(
        include_bytes!("../../../conformance/next/ifccad/valid/hatch-pattern.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let cad = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    let find = |d: &IfccadDocument| {
        d.model
            .entities
            .iter()
            .filter_map(IfccadEntity::as_native)
            .find_map(|e| {
                if let IfccadEntityKind::Hatch(h) = &e.kind {
                    Some(h.clone())
                } else {
                    None
                }
            })
            .unwrap()
    };
    for dwg in [false, true] {
        let file = exchange(cad.document(), dwg);
        let back = cad_document_to_ifccad_document(
            &file,
            IfccadTargetMetadata {
                header: d.header.clone(),
                drawing_id: d.drawing_id,
            },
            Default::default(),
        )
        .unwrap();
        let h = find(back.document());
        let expected = find(&d);
        let HatchFill::LinePattern(a) = h.fill else {
            panic!("pattern lost")
        };
        let HatchFill::LinePattern(b) = expected.fill else {
            panic!()
        };
        assert_eq!(a.origin, b.origin);
        assert!((a.rotation - b.rotation).abs() < 1e-14);
        assert_eq!(a.scale, b.scale);
        assert_eq!(a.families.len(), b.families.len());
        for (a, b) in a.families.iter().zip(b.families) {
            assert!((a.angle - b.angle).abs() < 1e-14);
            for (x, y) in a.base_point.iter().zip(b.base_point) {
                assert!((x - y).abs() < 1e-12);
            }
            assert_eq!(a.dashes, b.dashes);
            for (x, y) in a.offset.iter().zip(b.offset) {
                assert!((x - y).abs() < 1e-12);
            }
        }
        assert!(load_ifccad_bytes(
            encode_ifccad_document(back.document()).unwrap().bytes(),
            Default::default()
        )
        .is_ok());
        assert!(!back.geometry_assessment().is_complete());
    }
}
#[test]
fn tilted_curves_multiple_regions_and_tolerant_gap_survive_without_bridging() {
    for dwg in [false, true] {
        let mut d = fixture::drawing();
        let IfccadEntityKind::Hatch(h) = &mut d.model.entities[0].as_native_mut().unwrap().kind
        else {
            panic!()
        };
        h.placement = IfccadPlacement {
            origin: [0., 0., 4.],
            x_axis: [1., 0., 0.],
            y_axis: [0., 0., 1.],
        };
        h.loops = contours::contours()
            .into_iter()
            .chain([contours::tolerant_loop()])
            .map(|boundary| IfccadHatchLoop {
                boundary,
                source_entity_id: None,
            })
            .collect();
        let meta = || IfccadTargetMetadata {
            header: d.header.clone(),
            drawing_id: d.drawing_id,
        };
        let cad = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
        let expected =
            cad_document_to_ifccad_document(cad.document(), meta(), Default::default()).unwrap();
        let file = exchange(cad.document(), dwg);
        let back = cad_document_to_ifccad_document(&file, meta(), Default::default()).unwrap();
        let hatch = |d: &IfccadDocument| {
            d.model
                .entities
                .iter()
                .filter_map(IfccadEntity::as_native)
                .find_map(|e| {
                    if let IfccadEntityKind::Hatch(h) = &e.kind {
                        Some(h.clone())
                    } else {
                        None
                    }
                })
                .unwrap()
        };
        assert_eq!(hatch(back.document()), hatch(expected.document()));
        assert_eq!(hatch(back.document()).loops.len(), 9);
        let bytes = encode_ifccad_document(back.document()).unwrap();
        assert!(load_ifccad_bytes(bytes.bytes(), Default::default()).is_ok());
    }
}
#[test]
fn all_area_rules_hole_first_and_association_survive_actual_files() {
    for dwg in [false, true] {
        for rule in [
            HatchAreaRule::Normal,
            HatchAreaRule::Outer,
            HatchAreaRule::Ignore,
        ] {
            for reverse in [false, true] {
                let mut d = fixture::drawing();
                let IfccadEntityKind::Hatch(h) =
                    &mut d.model.entities[0].as_native_mut().unwrap().kind
                else {
                    panic!()
                };
                h.area_rule = rule;
                if reverse {
                    h.loops.reverse();
                }
                let expected = h.clone();
                let cad = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
                let file = exchange(cad.document(), dwg);
                let back = cad_document_to_ifccad_document(
                    &file,
                    IfccadTargetMetadata {
                        header: d.header,
                        drawing_id: d.drawing_id,
                    },
                    Default::default(),
                )
                .unwrap();
                let h = back
                    .document()
                    .model
                    .entities
                    .iter()
                    .filter_map(IfccadEntity::as_native)
                    .find_map(|e| {
                        if let IfccadEntityKind::Hatch(h) = &e.kind {
                            Some(h)
                        } else {
                            None
                        }
                    })
                    .unwrap();
                assert_eq!(h.area_rule, rule);
                assert_eq!(h.loops.len(), expected.loops.len());
                for (a, b) in h.loops.iter().zip(&expected.loops) {
                    assert_eq!(a.boundary, b.boundary);
                    assert_eq!(a.source_entity_id.is_some(), b.source_entity_id.is_some());
                }
                let bytes = encode_ifccad_document(back.document()).unwrap();
                assert!(load_ifccad_bytes(bytes.bytes(), Default::default()).is_ok());
                assert!(!back.geometry_assessment().is_complete());
            }
        }
    }
}
#[test]
fn curved_hatch_occurrences_survive_nested_nonuniform_blocks() {
    for dwg in [false, true] {
        let source = contours::cad_nonuniform_block();
        let d = fixture::drawing();
        let meta = || IfccadTargetMetadata {
            header: d.header.clone(),
            drawing_id: d.drawing_id,
        };
        let n = cad_document_to_ifccad_document(&source, meta(), Default::default()).unwrap();
        let cad = ifccad_document_to_cad_document(n.document(), Default::default()).unwrap();
        let file = exchange(cad.document(), dwg);
        let back = cad_document_to_ifccad_document(&file, meta(), Default::default()).unwrap();
        assert_eq!(back.document().blocks.len(), 2);
        let h = back
            .document()
            .blocks
            .iter()
            .flat_map(|b| &b.entities)
            .filter_map(IfccadEntity::as_native)
            .find_map(|e| {
                if let IfccadEntityKind::Hatch(h) = &e.kind {
                    Some(h)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(h.loops.len(), 8);
        assert!(!back.geometry_assessment().is_complete());
        assert!(
            back.geometry_assessment()
                .domains()
                .iter()
                .map(|d| d.assessed_entities())
                .sum::<usize>()
                >= 3
        );
        assert!(load_ifccad_bytes(
            encode_ifccad_document(back.document()).unwrap().bytes(),
            Default::default()
        )
        .is_ok());
    }
}

#[test]
fn user_defined_layer_zero_block_dependency_is_not_assumed_continuous() {
    use opencadcodec::{entities::hatch::*, EntityType, Vector2};
    for (mode, accepted) in [("Continuous", true), ("ByLayer", false)] {
        let mut source = contours::cad_nonuniform_block();
        let hh = source
            .entities()
            .find_map(|e| {
                if let EntityType::Hatch(h) = e {
                    Some(h.common.handle)
                } else {
                    None
                }
            })
            .unwrap();
        let EntityType::Hatch(h) = source.get_entity_mut(hh).unwrap() else {
            panic!()
        };
        h.is_solid = false;
        h.pattern_type = HatchPatternType::UserDefined;
        h.common.linetype = mode.into();
        h.pattern = HatchPattern::new("U");
        h.pattern.lines.push(HatchPatternLine {
            angle: 0.,
            base_point: Vector2::ZERO,
            offset: Vector2::new(0., 1.),
            dash_lengths: vec![],
        });
        let d = fixture::drawing();
        let out = cad_document_to_ifccad_document(
            &source,
            IfccadTargetMetadata {
                header: d.header,
                drawing_id: d.drawing_id,
            },
            Default::default(),
        )
        .unwrap();
        let count = out
            .document()
            .blocks
            .iter()
            .flat_map(|b| &b.entities)
            .filter_map(IfccadEntity::as_native)
            .filter(|e| matches!(e.kind, IfccadEntityKind::Hatch(_)))
            .count();
        assert_eq!(count, usize::from(accepted));
    }
}
