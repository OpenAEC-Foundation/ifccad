mod common;

use ifccad::*;
use ifccad_convert::*;
use opencadcodec::{entities::Spline, CadDocument, DxfReader, DxfWriter, EntityType, Vector3};
use std::io::Cursor;

fn source_with_splines(count: u32) -> CadDocument {
    let mut cad = CadDocument::new();
    cad.header.insertion_units = 4;
    for index in 0..count {
        let x = f64::from(index) * 10.;
        let spline = Spline::from_control_points(
            3,
            vec![
                Vector3::new(x, 0., 0.),
                Vector3::new(x + 2., 4., 0.),
                Vector3::new(x + 4., 4., 0.),
                Vector3::new(x + 6., 0., 0.),
            ],
        );
        cad.add_entity(EntityType::Spline(spline)).unwrap();
        if index == count / 2 {
            cad.add_entity(EntityType::Line(opencadcodec::entities::Line::from_points(
                Vector3::new(-1., 2., 0.),
                Vector3::new(-3., 4., 0.),
            )))
            .unwrap();
        }
    }
    cad
}

fn assert_ordered_geometry(expected: &CadDocument, actual: &CadDocument) {
    let expected: Vec<_> = expected.entities().collect();
    let actual: Vec<_> = actual.entities().collect();
    assert_eq!(actual.len(), expected.len());
    for (expected, actual) in expected.into_iter().zip(actual) {
        match (expected, actual) {
            (EntityType::Spline(expected), EntityType::Spline(actual)) => {
                assert_eq!(actual.degree, expected.degree);
                assert_eq!(actual.flags, expected.flags);
                assert_eq!(actual.knots, expected.knots);
                assert_eq!(actual.control_points, expected.control_points);
                assert_eq!(actual.weights, expected.weights);
                assert_eq!(actual.fit_points, expected.fit_points);
                assert_eq!(actual.normal, expected.normal);
                assert_eq!(actual.begin_tangent, expected.begin_tangent);
                assert_eq!(actual.end_tangent, expected.end_tangent);
                assert_eq!(actual.knot_tolerance, expected.knot_tolerance);
                assert_eq!(actual.control_tolerance, expected.control_tolerance);
                assert_eq!(actual.fit_tolerance, expected.fit_tolerance);
                assert_eq!(actual.knot_parameterization, expected.knot_parameterization);
            }
            (EntityType::Line(expected), EntityType::Line(actual)) => {
                assert_eq!(actual.start, expected.start);
                assert_eq!(actual.end, expected.end);
            }
            _ => panic!("mixed native/opaque source order changed"),
        }
    }
}

#[test]
fn mixed_record_id_lengths_survive_public_source_and_document_exports() {
    for count in [10, 121] {
        let cad = source_with_splines(count);
        let before: Vec<_> = cad.entities().cloned().collect();
        let converted = cad_document_to_encoded_ifccad(
            &cad,
            common::metadata(),
            CadToIfccadOptions {
                preservation: IfccadPreservationCapture::SupportedTyped,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(cad.entities().cloned().collect::<Vec<_>>(), before);
        let loaded = load_ifccad_bytes(converted.encoded().bytes(), Default::default()).unwrap();
        validate_ifccad_document(loaded.document()).unwrap();
        assert_eq!(
            loaded
                .document()
                .preservation
                .as_ref()
                .unwrap()
                .records
                .len(),
            count as usize
        );
        let document_before = loaded.document().clone();
        let graph_before = loaded.graph().composed_ifcx().clone();
        for restored in [
            ifccad_source_to_cad_document(&loaded, Default::default()).unwrap(),
            ifccad_document_to_cad_document(loaded.document(), Default::default()).unwrap(),
        ] {
            assert_ordered_geometry(&cad, restored.document());
            let bytes = DxfWriter::new(restored.document()).write_to_vec().unwrap();
            let physical = DxfReader::from_reader(Cursor::new(bytes))
                .unwrap()
                .read()
                .unwrap();
            assert_ordered_geometry(&cad, &physical);
        }
        assert_eq!(loaded.document(), &document_before);
        assert_eq!(loaded.graph().composed_ifcx(), &graph_before);
        let encoded = encode_ifccad_document(loaded.document()).unwrap();
        let reopened = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
        assert_eq!(reopened.document(), loaded.document());
    }
}
