mod common;
use ifccad_convert::*;
use ocdraw::ifccad::*;
use opencadcodec::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter};
use std::io::Cursor;

fn file_exchange(cad: &CadDocument, dwg: bool) -> CadDocument {
    if dwg {
        DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(cad).unwrap()))
            .read()
            .unwrap()
    } else {
        DxfReader::from_reader(Cursor::new(DxfWriter::new(cad).write_to_vec().unwrap()))
            .unwrap()
            .read()
            .unwrap()
    }
}
fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-10, "{a} != {b}");
}
fn bounds_equal(a: &IfccadEntityKind, b: &IfccadEntityKind) {
    let bounds = |k: &IfccadEntityKind| {
        ocdraw::geometry_kernel::geometry_bounds(k.as_shared_geometry().unwrap().unwrap()).unwrap()
    };
    let (a, b) = (bounds(a), bounds(b));
    for (a, b) in a.min().components().into_iter().zip(b.min().components()) {
        close(a, b);
    }
    for (a, b) in a.max().components().into_iter().zip(b.max().components()) {
        close(a, b);
    }
}
fn path_equal(a: &IfccadEntityKind, b: &IfccadEntityKind) {
    if let (
        IfccadEntityKind::PlanarPolyline {
            vertices: a,
            bulges: ab,
            closed: ac,
            line_pattern_generation: ag,
            placement: af,
        },
        IfccadEntityKind::PlanarPolyline {
            vertices: b,
            bulges: bb,
            closed: bc,
            line_pattern_generation: bg,
            placement: bf,
        },
    ) = (a, b)
    {
        assert_eq!((ab, ac, ag), (bb, bc, bg));
        assert_eq!(a.len(), b.len());
        for (a, b) in a.iter().zip(b) {
            for i in 0..3 {
                close(
                    af.origin[i] + a[0] * af.x_axis[i] + a[1] * af.y_axis[i],
                    bf.origin[i] + b[0] * bf.x_axis[i] + b[1] * bf.y_axis[i],
                );
            }
        }
    }
}

#[test]
fn expanded_families_exchange_through_actual_dxf_and_dwg_readers() {
    let mut d = common::empty();
    let template = common::primitives().model.entities[0].clone();
    let frame = common::placement([2., 3., 4.]);
    let kinds = vec![
        IfccadEntityKind::Point {
            placement: frame.clone(),
        },
        IfccadEntityKind::Arc {
            placement: frame.clone(),
            radius: 2.,
            start_parameter: 0.25,
            sweep_parameter: -1.,
        },
        IfccadEntityKind::Ellipse {
            placement: frame.clone(),
            semi_major_radius: 3.,
            semi_minor_radius: 2.,
        },
        IfccadEntityKind::EllipseArc {
            placement: frame.clone(),
            semi_major_radius: 3.,
            semi_minor_radius: 2.,
            start_parameter: 0.25,
            sweep_parameter: 1.,
        },
        IfccadEntityKind::PlanarPolyline {
            placement: frame,
            vertices: vec![[-1., 0.], [1., 0.], [2., 2.]],
            bulges: vec![1., -0.25, 7.],
            closed: false,
            line_pattern_generation: IfccadLinePatternGeneration::Continuous,
        },
        IfccadEntityKind::SpatialPolyline {
            vertices: vec![[0.; 3], [1., 2., 3.]],
            closed: false,
            line_pattern_generation: IfccadLinePatternGeneration::PerSegment,
        },
    ];
    for kind in kinds {
        let mut e = template.clone();
        e.as_native_mut().unwrap().id = d.id_counters.allocate_entity_id().unwrap();
        e.as_native_mut().unwrap().kind = kind;
        d.model.entities.push(e);
    }
    let cad = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    for dwg in [false, true] {
        let file = file_exchange(cad.document(), dwg);
        let restored =
            cad_document_to_encoded_ifccad(&file, common::metadata(), Default::default()).unwrap();
        let actual = restored.validated_source().document();
        assert_eq!(actual.model.entities.len(), d.model.entities.len());
        for (expected, actual) in d.model.entities.iter().zip(&actual.model.entities) {
            assert_eq!(
                std::mem::discriminant(&expected.as_native().unwrap().kind),
                std::mem::discriminant(&actual.as_native().unwrap().kind)
            );
            bounds_equal(
                &expected.as_native().unwrap().kind,
                &actual.as_native().unwrap().kind,
            );
            match (
                &expected.as_native().unwrap().kind,
                &actual.as_native().unwrap().kind,
            ) {
                (
                    IfccadEntityKind::PlanarPolyline { .. },
                    IfccadEntityKind::PlanarPolyline { .. },
                ) => path_equal(
                    &expected.as_native().unwrap().kind,
                    &actual.as_native().unwrap().kind,
                ),
                (
                    IfccadEntityKind::SpatialPolyline {
                        vertices: a,
                        closed: ac,
                        ..
                    },
                    IfccadEntityKind::SpatialPolyline {
                        vertices: b,
                        closed: bc,
                        ..
                    },
                ) => assert_eq!((a, ac), (b, bc)),
                _ => {}
            }
        }
        assert!(actual.model.bounds.is_some());
        validate_ifccad_document(actual).unwrap();
        assert_eq!(
            load_ifccad_bytes(restored.encoded().bytes(), Default::default())
                .unwrap()
                .document(),
            actual
        );
    }
}

#[test]
fn ellipse_and_bulged_clip_identity_survives_real_file_exchange() {
    for bulged in [false, true] {
        let mut d = common::viewport_drawing();
        let frame = common::placement([100., 75., 0.]);
        d.paper_layouts[0].entities[2].as_native_mut().unwrap().kind = if bulged {
            IfccadEntityKind::PlanarPolyline {
                placement: frame,
                vertices: vec![[-50., 0.], [50., 0.]],
                bulges: vec![1., 1.],
                closed: true,
                line_pattern_generation: IfccadLinePatternGeneration::PerSegment,
            }
        } else {
            IfccadEntityKind::Ellipse {
                placement: frame,
                semi_major_radius: 60.,
                semi_minor_radius: 40.,
            }
        };
        let cad = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
        for dwg in [false, true] {
            let file = file_exchange(cad.document(), dwg);
            let back =
                cad_document_to_encoded_ifccad(&file, common::metadata(), Default::default())
                    .unwrap();
            let paper = &back.validated_source().document().paper_layouts[0];
            let v = paper
                .entities
                .iter()
                .find_map(|e| {
                    if let IfccadEntityKind::Viewport(v) = &e.as_native().unwrap().kind {
                        Some(v)
                    } else {
                        None
                    }
                })
                .unwrap();
            assert!(v.paper_clip.enabled);
            let boundary = paper
                .entities
                .iter()
                .find(|e| Some(e.id()) == v.paper_clip.boundary_entity_id)
                .unwrap();
            bounds_equal(
                &d.paper_layouts[0].entities[2].as_native().unwrap().kind,
                &boundary.as_native().unwrap().kind,
            );
            path_equal(
                &d.paper_layouts[0].entities[2].as_native().unwrap().kind,
                &boundary.as_native().unwrap().kind,
            );
            assert_eq!(
                std::mem::discriminant(&boundary.as_native().unwrap().kind),
                std::mem::discriminant(&d.paper_layouts[0].entities[2].as_native().unwrap().kind)
            );
        }
    }
}

#[test]
fn classic_and_generic_polyline_sources_use_the_same_native_geometry() {
    use opencadcodec::{
        entities::{Polyline2D, Vertex2D},
        EntityType, Polyline, Vector3,
    };
    let mut cad = common::cad();
    let mut classic = Polyline2D::new();
    classic.elevation = 7.;
    classic.vertices = vec![
        Vertex2D::new(Vector3::new(-1., 0., 0.)),
        Vertex2D::new(Vector3::new(1., 0., 0.)),
    ];
    classic.vertices[0].bulge = 1.;
    classic.vertices[1].bulge = 7.;
    let spatial = Polyline::from_points(vec![Vector3::ZERO, Vector3::new(1., 2., 3.)]);
    cad.add_entity(EntityType::Polyline2D(classic.clone()))
        .unwrap();
    cad.add_entity(EntityType::Polyline(spatial)).unwrap();
    for format in [0, 1, 2] {
        let source = match format {
            0 => cad.clone(),
            1 => file_exchange(&cad, false),
            _ => file_exchange(&cad, true),
        };
        let restored =
            cad_document_to_encoded_ifccad(&source, common::metadata(), Default::default())
                .unwrap();
        let entities = &restored.validated_source().document().model.entities;
        assert_eq!(entities.len(), 2, "{format}: {:?}", restored.diagnostics());
        let IfccadEntityKind::PlanarPolyline {
            bulges,
            vertices,
            placement,
            ..
        } = &entities[0].as_native().unwrap().kind
        else {
            panic!("classic planar family")
        };
        assert_eq!(bulges, &[1., 7.]);
        assert_eq!(vertices, &[[-1., 0.], [1., 0.]]);
        assert_eq!(placement.origin[2], 7.);
        let IfccadEntityKind::SpatialPolyline { vertices, .. } =
            &entities[1].as_native().unwrap().kind
        else {
            panic!("spatial family")
        };
        assert_eq!(vertices, &[[0.; 3], [1., 2., 3.]]);
    }
    classic.vertices[0].start_width = 2.;
    let mut cad = common::cad();
    cad.add_entity(EntityType::Polyline2D(classic)).unwrap();
    let omitted =
        cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
    assert!(omitted
        .validated_source()
        .document()
        .model
        .entities
        .is_empty());
    assert!(omitted.diagnostics().iter().any(|d| d.is_semantic_loss()));
}
