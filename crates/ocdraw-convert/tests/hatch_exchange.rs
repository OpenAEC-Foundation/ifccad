use ocdraw::geometry_kernel::hatch::*;
use ocdraw::ocdraw::*;
use ocdraw_convert::*;
use opencadcodec::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter};
use std::io::Cursor;
#[path = "../../../tests/support/hatch.rs"]
mod contours;
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
fn tilted_curves_multiple_regions_and_tolerant_gap_survive_without_bridging() {
    for dwg in [false, true] {
        let mut d = load_ocdraw_bytes(include_bytes!(
            "../../../conformance/next/ocdraw/valid/hatch-solid.ocdraw.json"
        ))
        .unwrap()
        .into_document();
        let h = &mut d.hatch_entities[0];
        h.placement = CoordinateFrame3::try_new(
            Point3::new(0., 0., 4.),
            Vector3::new(1., 0., 0.),
            Vector3::new(0., 0., 1.),
        )
        .unwrap();
        h.loops = contours::contours()
            .into_iter()
            .chain([contours::tolerant_loop()])
            .map(|boundary| OcdrawHatchLoop {
                boundary,
                source_entity_id: None,
            })
            .collect();
        recompute_ocdraw_document_bounds(&mut d).unwrap();
        let cad = ocdraw_document_to_cad_document(&d, Default::default()).unwrap();
        let expected =
            cad_document_to_ocdraw_document_with_id(cad.document(), "expected", Default::default())
                .unwrap();
        let file = exchange(cad.document(), dwg);
        let back =
            cad_document_to_ocdraw_document_with_id(&file, "actual", Default::default()).unwrap();
        assert_eq!(
            back.document().hatch_entities[0].loops,
            expected.document().hatch_entities[0].loops
        );
        assert_eq!(
            back.document().hatch_entities[0].placement,
            expected.document().hatch_entities[0].placement
        );
        assert_eq!(back.document().hatch_entities[0].loops.len(), 9);
        let bytes = encode_ocdraw_document(back.document()).unwrap();
        assert!(load_ocdraw_bytes(bytes.bytes()).is_ok());
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
                let mut d = load_ocdraw_bytes(include_bytes!(
                    "../../../conformance/next/ocdraw/valid/hatch-solid.ocdraw.json"
                ))
                .unwrap()
                .into_document();
                d.hatch_entities[0].area_rule = rule;
                if reverse {
                    d.hatch_entities[0].loops.reverse();
                }
                let cad = ocdraw_document_to_cad_document(&d, Default::default()).unwrap();
                let file = exchange(cad.document(), dwg);
                let back = cad_document_to_ocdraw_document_with_id(
                    &file,
                    "hatch-exchange",
                    Default::default(),
                )
                .unwrap();
                let h = &back.document().hatch_entities[0];
                assert_eq!(h.area_rule, rule);
                assert_eq!(h.loops.len(), d.hatch_entities[0].loops.len());
                for (a, b) in h.loops.iter().zip(&d.hatch_entities[0].loops) {
                    assert_eq!(a.boundary, b.boundary);
                    assert_eq!(a.source_entity_id.is_some(), b.source_entity_id.is_some());
                }
                let bytes = encode_ocdraw_document(back.document()).unwrap();
                assert!(load_ocdraw_bytes(bytes.bytes()).is_ok());
                assert!(!back.geometry_assessment().is_complete());
            }
        }
    }
}
#[test]
fn curved_hatch_occurrences_survive_nested_nonuniform_blocks() {
    for dwg in [false, true] {
        let source = contours::cad_nonuniform_block();
        let n =
            cad_document_to_ocdraw_document_with_id(&source, "nested", Default::default()).unwrap();
        assert_eq!(n.document().hatch_entities[0].loops.len(), 8);
        let cad = ocdraw_document_to_cad_document(n.document(), Default::default()).unwrap();
        let file = exchange(cad.document(), dwg);
        let back =
            cad_document_to_ocdraw_document_with_id(&file, "back", Default::default()).unwrap();
        assert_eq!(back.document().hatch_entities[0].loops.len(), 8);
        assert_eq!(back.document().block_definitions.len(), 2);
        assert!(!back.geometry_assessment().is_complete());
        assert!(back.geometry_assessment().assessed_entities() >= 3);
        assert!(
            load_ocdraw_bytes(encode_ocdraw_document(back.document()).unwrap().bytes()).is_ok()
        );
    }
}
