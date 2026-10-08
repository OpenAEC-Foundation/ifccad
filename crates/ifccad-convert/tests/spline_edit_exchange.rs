mod spline_support;
use ifccad_convert::*;
use ocdraw::ifccad::*;
use opencadcodec::{
    CadDocument, Color, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType, Vector3,
};
use spline_support::captured;
use std::io::Cursor;
fn reopened(d: &IfccadDocument) -> IfccadDocument {
    let b = encode_ifccad_document(d).unwrap();
    load_ifccad_bytes(b.bytes(), Default::default())
        .unwrap()
        .into_document()
}
fn physical(c: &CadDocument, dwg: bool) -> CadDocument {
    if dwg {
        DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(c).unwrap()))
            .read()
            .unwrap()
    } else {
        DxfReader::from_reader(Cursor::new(DxfWriter::new(c).write_to_vec().unwrap()))
            .unwrap()
            .read()
            .unwrap()
    }
}
fn spline(c: &CadDocument) -> &opencadcodec::entities::Spline {
    c.entities()
        .find_map(|e| {
            if let EntityType::Spline(s) = e {
                Some(s)
            } else {
                None
            }
        })
        .unwrap()
}
#[test]
fn edited_native_content_and_independent_spline_oracle_survive_dxf_and_dwg() {
    let original = captured();
    let preserved = original.preservation.clone();
    for mutation in 0..8 {
        let mut d = original.clone();
        match mutation {
            0 => d.layers[0].name = "Renamed".into(),
            1 => d.layers[0].appearance.color = "#ff0011".into(),
            2 => {
                d.model.entities[0]
                    .as_opaque_mut()
                    .unwrap()
                    .appearance
                    .as_mut()
                    .unwrap()
                    .appearance
                    .color = IfccadMode::Explicit("#123456".into())
            }
            3 => d.model.entities[0].as_opaque_mut().unwrap().visible = false,
            4 => {
                d.model.entities[0]
                    .as_opaque_mut()
                    .unwrap()
                    .appearance
                    .as_mut()
                    .unwrap()
                    .line_pattern_scale = 2.
            }
            5 => d.line_pattern_scale = 3.,
            6 => {
                let id = d.id_counters.allocate_layer_id().unwrap();
                let mut l = d.layers[0].clone();
                l.id = id;
                l.name = "New".into();
                d.layers.push(l);
                d.model.entities[0].as_opaque_mut().unwrap().layer_id = Some(id);
            }
            _ => {
                let id = d.id_counters.allocate_entity_id().unwrap();
                let a = d.model.entities[0]
                    .as_opaque()
                    .unwrap()
                    .appearance
                    .as_ref()
                    .unwrap()
                    .appearance
                    .clone();
                d.model.entities.insert(
                    0,
                    IfccadEntity::Native(IfccadNativeEntity {
                        id,
                        layer_id: d.layers[0].id,
                        appearance: a,
                        line_pattern_scale: 1.,
                        kind: IfccadEntityKind::LineSegment {
                            start: [8., 9., 0.],
                            end: [10., 11., 0.],
                        },
                    }),
                );
            }
        }
        let d = reopened(&d);
        assert_eq!(d.preservation, preserved);
        let cad = ifccad_document_to_cad_document(&d, Default::default())
            .unwrap()
            .into_document();
        for dwg in [false, true] {
            let back = physical(&cad, dwg);
            let s = spline(&back);
            assert_eq!(s.degree, 3);
            assert_eq!(
                s.control_points,
                vec![
                    Vector3::new(0., 0., 0.),
                    Vector3::new(2., 4., 0.),
                    Vector3::new(4., 4., 0.),
                    Vector3::new(6., 0., 0.)
                ]
            );
            assert_eq!(s.knots, vec![0., 0., 0., 0., 1., 1., 1., 1.]);
            if mutation == 2 {
                assert_eq!(s.common.color, Color::from_rgb(18, 52, 86));
            }
            if mutation == 3 {
                assert!(s.common.invisible);
            }
            if mutation == 4 {
                assert_eq!(s.common.linetype_scale, 2.);
            }
            if mutation == 5 {
                assert_eq!(back.header.linetype_scale, 3.);
            }
            if mutation == 6 {
                assert_eq!(s.common.layer, "New");
            }
            if mutation == 7 {
                assert!(back.entities().any(|e|matches!(e,EntityType::Line(l) if l.start==Vector3::new(8.,9.,0.)&&l.end==Vector3::new(10.,11.,0.))));
            }
            let again = cad_document_to_ifccad_document(
                &back,
                IfccadTargetMetadata {
                    drawing_id: 1,
                    header: d.header.clone(),
                },
                CadToIfccadOptions {
                    preservation: IfccadPreservationCapture::SupportedTyped,
                    ..Default::default()
                },
            )
            .unwrap();
            encode_ifccad_document(again.document()).unwrap();
        }
    }
}
#[test]
fn save_close_reopen_and_fresh_dwg_uses_production_storage() {
    let d = captured();
    let expected = d.preservation.clone();
    let b = encode_ifccad_document(&d).unwrap();
    drop(d);
    let name = format!(
        "ifccad-spline-{}-{}.ifcx",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let path = std::env::temp_dir().join(name);
    b.write_file(&path).unwrap();
    assert!(b.write_file(&path).is_err());
    drop(b);
    let loaded = load_ifccad_file(&path, Default::default()).unwrap();
    std::fs::remove_file(&path).unwrap();
    assert_eq!(loaded.document().preservation, expected);
    let cad = ifccad_source_to_cad_document(&loaded, Default::default()).unwrap();
    assert_eq!(
        spline(&physical(cad.document(), true)).control_points[1],
        Vector3::new(2., 4., 0.)
    );
}
#[test]
fn fit_parameterization_survives_merged_dxf_fix() {
    for parameter in [1, 2] {
        let d = captured();
        let mut cad = ifccad_document_to_cad_document(&d, Default::default())
            .unwrap()
            .into_document();
        let h = spline(&cad).common.handle;
        let EntityType::Spline(s) = cad.get_entity_mut(h).unwrap() else {
            panic!()
        };
        s.control_points.clear();
        s.knots.clear();
        s.fit_points = vec![Vector3::new(0., 0., 0.), Vector3::new(3., 1., 0.)];
        s.begin_tangent = Vector3::new(1., 0., 0.);
        s.end_tangent = Vector3::new(1., 0., 0.);
        s.knot_parameterization = parameter;
        s.dwg_flags1 = 9;
        let capture = cad_document_to_ifccad_document(
            &cad,
            IfccadTargetMetadata {
                drawing_id: 1,
                header: d.header.clone(),
            },
            CadToIfccadOptions {
                preservation: IfccadPreservationCapture::SupportedTyped,
                ..Default::default()
            },
        )
        .unwrap();
        let d = reopened(capture.document());
        drop(capture);
        drop(cad);
        let restored = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
        assert_eq!(spline(restored.document()).knot_parameterization, parameter);
        let dxf = physical(restored.document(), false);
        assert_eq!(
            spline(&dxf).knot_parameterization,
            parameter,
            "merged #99 must retain fit parameterization through actual DXF readback"
        );
        let dwg = physical(restored.document(), true);
        assert_eq!(spline(&dwg).knot_parameterization, parameter);
        assert_eq!(
            spline(&dwg).fit_points,
            vec![Vector3::new(0., 0., 0.), Vector3::new(3., 1., 0.)]
        );
    }
}
