mod common;
use ifccad::{dimension::*, geometry_kernel::Point3, *};
use ifccad_convert::*;
use opencadcodec::{
    entities::{Dimension, DimensionAligned, DimensionLinear},
    CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType, Vector3,
};
use std::io::Cursor;

fn sample(normal: Vector3, aligned: bool, explicit_text: bool) -> (CadDocument, [[f64; 3]; 3]) {
    let basis = cad_geometry_convert::geometry::cad_plane(normal).unwrap();
    let world = |p: [f64; 3]| -> Vector3 {
        let q: [f64; 3] =
            std::array::from_fn(|i| basis.u[i] * p[0] + basis.v[i] * p[1] + basis.n[i] * p[2]);
        Vector3::new(q[0], q[1], q[2])
    };
    let a = world([10., 20., 544.]);
    let b = world([13., 24., 1632.]);
    let line = world([10., 30., -17.]);
    let mut dimension = if aligned {
        let mut d = DimensionAligned::new(a, b);
        d.definition_point = line;
        Dimension::Aligned(d)
    } else {
        let mut d = DimensionLinear::new(a, b);
        d.definition_point = line;
        Dimension::Linear(d)
    };
    let base = dimension.base_mut();
    base.normal = normal;
    base.definition_point = line;
    base.text_user_positioned = explicit_text;
    base.text_middle_point = Vector3::new(12., 31., -17.);
    let mut doc = CadDocument::new();
    doc.add_entity(EntityType::Dimension(Box::new(dimension)))
        .unwrap();
    (
        doc,
        [[a.x, a.y, a.z], [b.x, b.y, b.z], [line.x, line.y, line.z]],
    )
}

fn assert_native_points(doc: &IfccadDocument, expected: [[f64; 3]; 3], measurement: f64) {
    let IfccadEntityKind::Dimension(d) = &doc.model.entities[0].as_native().unwrap().kind else {
        panic!("dimension")
    };
    let (a, b) = d.definition.reference_points();
    let line = match d.definition {
        DimensionDefinition::Linear {
            dimension_line_point,
            ..
        }
        | DimensionDefinition::Aligned {
            dimension_line_point,
            ..
        } => dimension_line_point,
    };
    let frame = d.placement.coordinate_frame().unwrap();
    for (p, expected) in [a, b, [line[0], line[1], 0.]].into_iter().zip(expected) {
        let actual = frame
            .try_to_scope_spatial_point(Point3::new(p[0], p[1], p[2]))
            .unwrap()
            .components();
        for (a, e) in actual.into_iter().zip(expected) {
            assert!((a - e).abs() <= 1e-9, "{actual:?} vs {expected:?}");
        }
    }
    let DimensionMeasurement::Distance(value) = evaluate_dimension(&d.definition).unwrap() else {
        panic!("distance")
    };
    assert!((value - measurement).abs() < 1e-9);
}

#[test]
fn independent_xyz_anchors_and_explicit_ocs_text_survive_three_actual_cad_formats() {
    for normal in [
        Vector3::new(0., 0., 1.),
        Vector3::new(0., 0., -1.),
        Vector3::new(1., 0., 0.),
    ] {
        for aligned in [false, true] {
            let (cad, expected) = sample(normal, aligned, true);
            let native = cad_document_to_encoded_ifccad(
                &cad,
                common::metadata(),
                CadToIfccadOptions {
                    geometry_tolerance: IfccadGeometryTolerance::exact(),
                    ..Default::default()
                },
            )
            .unwrap();
            let doc = native.validated_source().document();
            assert_native_points(doc, expected, if aligned { 5. } else { 3. });
            let cbor = encode_ifccad_cbor_document(doc).unwrap();
            let loaded = load_ifccad_cbor_bytes(cbor.bytes(), Default::default()).unwrap();
            assert_native_points(loaded.document(), expected, if aligned { 5. } else { 3. });
            let output =
                ifccad_document_to_cad_document(loaded.document(), Default::default()).unwrap();
            for format in 0..3 {
                let returned = match format {
                    0 | 1 => DxfReader::from_reader(Cursor::new(
                        if format == 0 {
                            DxfWriter::new(output.document())
                        } else {
                            DxfWriter::new_binary(output.document())
                        }
                        .write_to_vec()
                        .unwrap(),
                    ))
                    .unwrap()
                    .read()
                    .unwrap(),
                    _ => DwgReader::from_stream(Cursor::new(
                        DwgWriter::write_to_vec(output.document()).unwrap(),
                    ))
                    .read()
                    .unwrap(),
                };
                let rebuilt = cad_document_to_encoded_ifccad(
                    &returned,
                    common::metadata(),
                    Default::default(),
                )
                .unwrap();
                assert_native_points(
                    rebuilt.validated_source().document(),
                    expected,
                    if aligned { 5. } else { 3. },
                );
                let IfccadEntityKind::Dimension(d) =
                    &rebuilt.validated_source().document().model.entities[0]
                        .as_native()
                        .unwrap()
                        .kind
                else {
                    panic!("dimension")
                };
                assert!(matches!(
                    d.text_placement,
                    DimensionTextPlacement::Explicit {
                        position: [2., 1.],
                        ..
                    }
                ));
            }
        }
    }
}

#[test]
fn oblique_source_keeps_spatial_anchors_through_native_storage() {
    for aligned in [false, true] {
        let (cad, expected) = sample(Vector3::new(1., 2., 3.), aligned, false);
        let out =
            cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
        assert_native_points(
            out.validated_source().document(),
            expected,
            if aligned { 5. } else { 3. },
        );
    }
}

#[test]
fn xyz_references_remain_owned_by_model_paper_and_local_blocks() {
    let source = load_ifccad_bytes(
        include_bytes!("../../../examples/ifccad/dimensions.ifcx"),
        Default::default(),
    )
    .unwrap();
    let mut doc = source.into_document();
    let scopes = std::iter::once(&mut doc.model.entities)
        .chain(doc.paper_layouts.iter_mut().map(|p| &mut p.entities))
        .chain(doc.blocks.iter_mut().map(|b| &mut b.entities));
    let mut expected = Vec::new();
    for scope in scopes {
        for e in scope.iter_mut().filter_map(IfccadEntity::as_native_mut) {
            let IfccadEntityKind::Dimension(d) = &mut e.kind else {
                continue;
            };
            let (a, b) = match &mut d.definition {
                DimensionDefinition::Linear {
                    first_point,
                    second_point,
                    ..
                }
                | DimensionDefinition::Aligned {
                    first_point,
                    second_point,
                    ..
                } => (first_point, second_point),
            };
            a[2] = 544.;
            b[2] = 1632.;
            let frame = d.placement.coordinate_frame().unwrap();
            let points = [*a, *b].map(|p| {
                frame
                    .try_to_scope_spatial_point(Point3::new(p[0], p[1], p[2]))
                    .unwrap()
                    .components()
            });
            expected.push((e.id, points));
        }
    }
    assert_eq!(expected.len(), 5);
    let count = |entities: &[IfccadEntity]| {
        entities
            .iter()
            .filter(|e| {
                matches!(
                    e.as_native().map(|n| &n.kind),
                    Some(IfccadEntityKind::Dimension(_))
                )
            })
            .count()
    };
    let owner_counts = [
        count(&doc.model.entities),
        doc.paper_layouts
            .iter()
            .map(|p| count(&p.entities))
            .sum::<usize>(),
        doc.blocks.iter().map(|b| count(&b.entities)).sum::<usize>(),
    ];
    assert!(owner_counts.into_iter().all(|n| n > 0));
    let encoded = encode_ifccad_document(&doc).unwrap();
    let read = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    let target = ifccad_source_to_cad_document(&read, Default::default()).unwrap();
    for (id, expected) in expected {
        let handle = target.mappings().entities.cad_handle(id).unwrap();
        let EntityType::Dimension(d) = target.document().get_entity(handle).unwrap() else {
            panic!("dimension")
        };
        let (a, b) = match d.as_ref() {
            Dimension::Linear(d) => (d.first_point, d.second_point),
            Dimension::Aligned(d) => (d.first_point, d.second_point),
            _ => panic!("kind"),
        };
        for (actual, expected) in [a, b].into_iter().zip(expected) {
            for (a, e) in [actual.x, actual.y, actual.z].into_iter().zip(expected) {
                assert!((a - e).abs() <= 1e-9);
            }
        }
    }
    let rebuilt =
        cad_document_to_encoded_ifccad(target.document(), common::metadata(), Default::default())
            .unwrap();
    let doc = rebuilt.validated_source().document();
    assert!(!doc.paper_layouts.is_empty());
    assert!(!doc.blocks.is_empty());
    let count = |entities: &[IfccadEntity]| {
        entities
            .iter()
            .filter(|e| {
                matches!(
                    e.as_native().map(|n| &n.kind),
                    Some(IfccadEntityKind::Dimension(_))
                )
            })
            .count()
    };
    assert_eq!(count(&doc.model.entities), owner_counts[0]);
    assert_eq!(
        doc.paper_layouts
            .iter()
            .map(|p| count(&p.entities))
            .sum::<usize>(),
        owner_counts[1]
    );
    assert_eq!(
        doc.blocks.iter().map(|b| count(&b.entities)).sum::<usize>(),
        owner_counts[2]
    );
}
