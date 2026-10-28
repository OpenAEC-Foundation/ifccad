mod common;
#[path = "support/filled_type_oracle.rs"]
mod oracle;
use ifccad::*;
use ifccad_convert::*;
use opencadcodec::{DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType};
use std::io::Cursor;
fn source() -> opencadcodec::CadDocument {
    DxfReader::from_reader(Cursor::new(include_bytes!(
        "fixtures/filled-primitives-reference.dxf"
    )))
    .unwrap()
    .read()
    .unwrap()
}
fn filled(doc: &opencadcodec::CadDocument) -> Vec<&opencadcodec::entities::Solid> {
    doc.entities()
        .filter_map(|e| {
            if let EntityType::Solid(s) = e {
                Some(s)
            } else {
                None
            }
        })
        .collect()
}
#[test]
fn filled_reference_import_matches_literal_geometry() {
    let cad = source();
    let s = filled(&cad);
    assert_eq!(s.len(), 32);
    assert_eq!(s.iter().filter(|s| s.is_trace).count(), 16);
    let out = cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
    assert_eq!(
        out.validated_source()
            .document()
            .model
            .entities
            .iter()
            .filter(|e| matches!(
                e.as_native().map(|e| &e.kind),
                Some(IfccadEntityKind::Solid(_) | IfccadEntityKind::Trace(_))
            ))
            .count(),
        32
    );
}
#[test]
fn filled_bytes_have_independent_type_evidence() {
    let source = source();
    let native =
        cad_document_to_ifccad_document(&source, common::metadata(), Default::default()).unwrap();
    let mut cad = ifccad_document_to_cad_document(native.document(), Default::default())
        .unwrap()
        .into_document();
    cad.version = opencadcodec::types::DxfVersion::AC1032;
    for binary in [false, true] {
        let bytes = if binary {
            DxfWriter::new_binary(&cad).write_to_vec().unwrap()
        } else {
            DxfWriter::new(&cad).write_to_vec().unwrap()
        };
        let kinds = if binary {
            oracle::binary_entity_types(&bytes)
        } else {
            oracle::ascii_entity_types(&bytes)
        };
        assert_eq!(kinds.iter().filter(|s| *s == "SOLID").count(), 16);
        assert_eq!(kinds.iter().filter(|s| *s == "TRACE").count(), 16);
        check_literal_tags(
            &cad,
            if binary {
                oracle::binary_tags(&bytes)
            } else {
                oracle::ascii_tags(&bytes)
            },
        );
        let loaded = DxfReader::from_reader(Cursor::new(bytes))
            .unwrap()
            .read()
            .unwrap();
        compare(&cad, &loaded);
    }
    let bytes = DwgWriter::write_to_vec(&cad).unwrap();
    let types = oracle::dwg_object_types(&bytes);
    assert_eq!(types.iter().filter(|&&v| v == 31).count(), 16);
    assert_eq!(types.iter().filter(|&&v| v == 32).count(), 16);
    let loaded = DwgReader::from_stream(Cursor::new(bytes)).read().unwrap();
    compare(&cad, &loaded);
}
fn compare(a: &opencadcodec::CadDocument, b: &opencadcodec::CadDocument) {
    cad_document_to_encoded_ifccad(b, common::metadata(), Default::default()).unwrap();
    let a = filled(a);
    let b = filled(b);
    assert_eq!(a.len(), b.len());
    for a in a {
        assert!(b.iter().any(|b| a.is_trace == b.is_trace
            && a.first_corner == b.first_corner
            && a.second_corner == b.second_corner
            && a.third_corner == b.third_corner
            && a.fourth_corner == b.fourth_corner
            && a.normal == b.normal
            && a.thickness == b.thickness));
    }
}

fn check_literal_tags(doc: &opencadcodec::CadDocument, tags: Vec<(i16, String)>) {
    let records: Vec<_> = tags.split(|(c, _)| *c == 0).collect();
    for s in filled(doc) {
        let kind = if s.is_trace { "TRACE" } else { "SOLID" };
        let wanted = [
            s.first_corner,
            s.second_corner,
            s.third_corner,
            s.fourth_corner,
        ];
        // Group-zero starts are kept separately so SOLID and TRACE remain independently checked.
        let starts: Vec<_> = tags
            .iter()
            .enumerate()
            .filter(|(_, t)| t.0 == 0 && t.1 == kind)
            .map(|(i, _)| i)
            .collect();
        assert!(starts.into_iter().any(|start| {
            let end = tags[start + 1..]
                .iter()
                .position(|t| t.0 == 0)
                .map_or(tags.len(), |i| start + 1 + i);
            let r = &tags[start + 1..end];
            let number = |code, default| {
                r.iter()
                    .find(|(c, _)| *c == code)
                    .map_or(default, |(_, v)| v.parse::<f64>().unwrap())
            };
            wanted.iter().enumerate().all(|(i, p)| {
                [p.x, p.y, p.z]
                    .into_iter()
                    .enumerate()
                    .all(|(axis, v)| number(10 + i as i16 + axis as i16 * 10, f64::NAN) == v)
            }) && number(39, 0.) == s.thickness
                && [s.normal.x, s.normal.y, s.normal.z]
                    .into_iter()
                    .enumerate()
                    .all(|(i, v)| number(210 + i as i16 * 10, if i == 2 { 1. } else { 0. }) == v)
        }));
    }
    let _ = records;
}
#[test]
fn filled_signed_sweep_and_tilted_ocs_exchange() {
    let mut d = common::empty();
    let template = common::primitives().model.entities[0].clone();
    for (i, (h, trace)) in [
        (-3., false),
        (3., true),
        (0., true),
        (-3., true),
        (3., false),
    ]
    .into_iter()
    .enumerate()
    {
        let mut e = template.clone();
        let n = e.as_native_mut().unwrap();
        n.id = i as u64 + 1;
        n.kind = if trace {
            IfccadEntityKind::Trace(IfccadFilledQuadrilateral {
                placement: IfccadPlacement {
                    origin: [2., 3., 4.],
                    x_axis: [1., 0., 0.],
                    y_axis: [0., 0.8, -0.6],
                },
                corners: [[0., 0.], [8., 0.], [0., 8.], [8., 8.]],
                thickness: h,
            })
        } else {
            IfccadEntityKind::Solid(IfccadFilledQuadrilateral {
                placement: IfccadPlacement {
                    origin: [2., 3., 4.],
                    x_axis: [1., 0., 0.],
                    y_axis: [0., -1., 0.],
                },
                corners: [[0., 0.], [0., 0.], [0., 8.], [8., 0.]],
                thickness: h,
            })
        };
        d.model.entities.push(e);
    }
    d.id_counters.next_entity_id = 6;
    let mut cad = ifccad_document_to_cad_document(&d, Default::default())
        .unwrap()
        .into_document();
    cad.version = opencadcodec::types::DxfVersion::AC1032;
    for binary in [false, true] {
        let b = if binary {
            DxfWriter::new_binary(&cad).write_to_vec().unwrap()
        } else {
            DxfWriter::new(&cad).write_to_vec().unwrap()
        };
        check_literal_tags(
            &cad,
            if binary {
                oracle::binary_tags(&b)
            } else {
                oracle::ascii_tags(&b)
            },
        );
        compare(
            &cad,
            &DxfReader::from_reader(Cursor::new(b))
                .unwrap()
                .read()
                .unwrap(),
        );
    }
    let bytes = DwgWriter::write_to_vec(&cad).unwrap();
    compare(
        &cad,
        &DwgReader::from_stream(Cursor::new(bytes)).read().unwrap(),
    );
}
