mod common;
use ifccad::*;
use ifccad_convert::*;
use opencadcodec::{
    CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType, LwPolyline, Vector2,
    Vector3,
};
use std::io::Cursor;

fn path(closed: bool, bulge: f64) -> LwPolyline {
    let mut p = LwPolyline::from_points(vec![
        Vector2::new(2., 3.),
        Vector2::new(2., 3.),
        Vector2::new(5., 7.),
        Vector2::new(2., 3.),
    ]);
    p.vertices[0].bulge = bulge;
    p.vertices[3].bulge = -bulge; // active coincident closure or authored dormant value
    p.is_closed = closed;
    p.elevation = -544.;
    p.plinegen = true;
    p
}
fn assert_parameters(p: &LwPolyline, expected: &LwPolyline) {
    assert_eq!(p.vertices, expected.vertices);
    assert_eq!(
        (p.is_closed, p.plinegen, p.elevation, p.normal),
        (
            expected.is_closed,
            expected.plinegen,
            expected.elevation,
            expected.normal
        )
    );
}
#[test]
fn coincident_source_parameters_survive_native_and_three_real_cad_formats() {
    for normal in [
        Vector3::UNIT_Z,
        Vector3::new(0., 0., -1.),
        Vector3::new(1., 0., 0.),
    ] {
        for closed in [false, true] {
            for bulge in [0.414213562373095, -0.414213562373095] {
                let mut p = path(closed, bulge);
                p.normal = normal;
                let mut source = CadDocument::new();
                source
                    .add_entity(EntityType::LwPolyline(p.clone()))
                    .unwrap();
                let encoded = cad_document_to_encoded_ifccad(
                    &source,
                    common::metadata(),
                    CadToIfccadOptions {
                        geometry_tolerance: IfccadGeometryTolerance::exact(),
                        ..Default::default()
                    },
                )
                .unwrap();
                let native = encoded.validated_source().document();
                let IfccadEntityKind::PlanarPolyline {
                    vertices,
                    bulges,
                    closed: actual,
                    line_pattern_generation,
                    ..
                } = &native.model.entities[0].as_native().unwrap().kind
                else {
                    panic!()
                };
                assert_eq!(
                    vertices,
                    &p.vertices
                        .iter()
                        .map(|v| [v.location.x, v.location.y])
                        .collect::<Vec<_>>()
                );
                assert_eq!(
                    bulges,
                    &p.vertices.iter().map(|v| v.bulge).collect::<Vec<_>>()
                );
                assert_eq!(*actual, closed);
                assert_eq!(
                    *line_pattern_generation,
                    IfccadLinePatternGeneration::Continuous
                );
                let cbor = encode_ifccad_cbor_document(native).unwrap();
                let loaded = load_ifccad_cbor_bytes(cbor.bytes(), Default::default()).unwrap();
                let cad = ifccad_document_to_cad_document(
                    loaded.document(),
                    IfccadToCadOptions {
                        geometry_tolerance: IfccadGeometryTolerance::exact(),
                        ..Default::default()
                    },
                )
                .unwrap();
                for format in 0..3 {
                    let returned = match format {
                        0 | 1 => DxfReader::from_reader(Cursor::new(
                            if format == 0 {
                                DxfWriter::new(cad.document())
                            } else {
                                DxfWriter::new_binary(cad.document())
                            }
                            .write_to_vec()
                            .unwrap(),
                        ))
                        .unwrap()
                        .read()
                        .unwrap(),
                        _ => DwgReader::from_stream(Cursor::new(
                            DwgWriter::write_to_vec(cad.document()).unwrap(),
                        ))
                        .read()
                        .unwrap(),
                    };
                    let actual = returned
                        .entities()
                        .find_map(|e| {
                            if let EntityType::LwPolyline(p) = e {
                                Some(p)
                            } else {
                                None
                            }
                        })
                        .unwrap();
                    assert_parameters(actual, &p);
                    let back = cad_document_to_encoded_ifccad(
                        &returned,
                        common::metadata(),
                        Default::default(),
                    )
                    .unwrap();
                    validate_ifccad_document(back.validated_source().document()).unwrap();
                }
            }
        }
    }
}

fn native_path(origin: [f64; 3], vertices: Vec<[f64; 2]>) -> IfccadEntityKind {
    IfccadEntityKind::PlanarPolyline {
        widths: Default::default(),
        bulges: vec![1., 0.],
        vertices,
        closed: false,
        placement: common::placement(origin),
        line_pattern_generation: IfccadLinePatternGeneration::Continuous,
    }
}
#[test]
fn newly_collapsed_nonzero_chord_is_rejected_under_both_policies() {
    let mut d = common::primitives();
    d.model.entities.truncate(1);
    d.model.bounds = None;
    d.model.entities[0].as_native_mut().unwrap().kind =
        native_path([1e20, 0., 0.], vec![[0., 0.], [0.0001, 0.]]);
    for loss_policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        assert!(matches!(
            ifccad_document_to_cad_document(
                &d,
                IfccadToCadOptions {
                    loss_policy,
                    geometry_tolerance: IfccadGeometryTolerance::drawing_units(1e10).unwrap(),
                    ..Default::default()
                }
            ),
            Err(IfccadConversionError::Geometry(_))
        ));
    }
}
#[test]
fn source_degenerate_point_still_obeys_nested_signed_occurrence_tolerance() {
    let mut d = common::nested([0.; 3]);
    d.blocks
        .iter_mut()
        .find(|b| b.name == "Inner")
        .unwrap()
        .entities[0]
        .as_native_mut()
        .unwrap()
        .kind = native_path([1e20, 0., 0.], vec![[1., 0.], [1., 0.]]);
    for loss_policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        let options = IfccadToCadOptions {
            loss_policy,
            geometry_tolerance: IfccadGeometryTolerance::drawing_units(10.).unwrap(),
            ..Default::default()
        };
        assert!(ifccad_document_to_cad_document(&d, options).is_ok());
        let mut amplified = d.clone();
        for e in &mut amplified.model.entities {
            if let IfccadEntityKind::BlockInstance { transform, .. } =
                &mut e.as_native_mut().unwrap().kind
            {
                transform.scale = [-100., 200., -300.];
            }
        }
        let Err(IfccadConversionError::Geometry(failure)) =
            ifccad_document_to_cad_document(&amplified, options)
        else {
            panic!("nested amplification must fail")
        };
        assert!(
            matches!(&failure.failure.source,IfccadGeometryEntitySource::BlockOccurrence {path,..} if path.len()>=2)
        );
    }
}

#[test]
fn model_paper_and_block_paths_retain_degenerate_parameters() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../../../examples/ifccad/dimensions.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let kind = native_path([0., 0., -544.], vec![[2., 3.], [2., 3.]]);
    let mut expected = Vec::new();
    for scope in std::iter::once(&mut d.model.entities)
        .chain(d.paper_layouts.iter_mut().map(|p| &mut p.entities))
        .chain(d.blocks.iter_mut().map(|b| &mut b.entities))
    {
        for e in scope.iter_mut().filter_map(IfccadEntity::as_native_mut) {
            if matches!(e.kind, IfccadEntityKind::Dimension(_)) {
                e.kind = kind.clone();
                expected.push(e.id);
            }
        }
    }
    assert_eq!(expected.len(), 5);
    let cad = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    for id in expected {
        let EntityType::LwPolyline(p) = cad
            .document()
            .get_entity(cad.mappings().entities.cad_handle(id).unwrap())
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(p.vertices.len(), 2);
        assert_eq!(p.vertices[0].location, p.vertices[1].location);
        assert_eq!(p.vertices[0].bulge, 1.);
        assert_eq!(p.elevation, -544.);
    }
    let back =
        cad_document_to_encoded_ifccad(cad.document(), common::metadata(), Default::default())
            .unwrap();
    let native = back.validated_source().document();
    for scope in std::iter::once(&native.model.entities)
        .chain(native.paper_layouts.iter().map(|p| &p.entities))
        .chain(native.blocks.iter().map(|b| &b.entities))
    {
        for e in scope.iter().filter_map(IfccadEntity::as_native) {
            if let IfccadEntityKind::PlanarPolyline {
                vertices, bulges, ..
            } = &e.kind
            {
                assert_eq!(vertices[0], vertices[1]);
                assert_eq!(bulges, &vec![1., 0.]);
            }
        }
    }
}
