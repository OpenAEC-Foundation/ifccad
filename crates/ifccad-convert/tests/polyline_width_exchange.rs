mod common;
#[path = "support/polyline_width_oracle.rs"]
mod oracle;
use ifccad::*;
use ifccad_convert::*;
use opencadcodec::{
    DwgReader, DwgWriter, DxfReader, DxfVersion, DxfWriter, EntityType, Polyline2DWriteMode,
};
use std::io::Cursor;
fn source() -> IfccadDocument {
    load_ifccad_bytes(
        include_bytes!("../../../conformance/next/ifccad/valid/polyline-defaults-overrides.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document()
}
fn widths(d: &mut IfccadDocument) -> &mut IfccadPlanarPolylineWidths {
    d.model
        .entities
        .iter_mut()
        .filter_map(IfccadEntity::as_native_mut)
        .find_map(|e| {
            if let IfccadEntityKind::PlanarPolyline { widths, .. } = &mut e.kind {
                Some(widths)
            } else {
                None
            }
        })
        .unwrap()
}
fn native_chain(d: &IfccadDocument) -> IfccadDocument {
    let e = encode_ifccad_document(d).unwrap();
    let j = load_ifccad_bytes(e.bytes(), Default::default()).unwrap();
    let c = encode_ifccad_cbor_document(j.document()).unwrap();
    load_ifccad_cbor_bytes(c.bytes(), Default::default())
        .unwrap()
        .into_document()
}
const LEAVES: [[Option<f64>; 2]; 4] = [
    [None, None],
    [Some(0.), None],
    [None, Some(0.)],
    [Some(0.2), Some(0.4)],
];
#[test]
fn classic_default_dependencies_survive_native_and_real_dxf() {
    let d = native_chain(&source());
    let cad = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    for binary in [false, true] {
        let mut w = DxfWriter::new(cad.document())
            .with_polyline2d_write_mode(Polyline2DWriteMode::PreserveClassic);
        w.set_binary(binary);
        let bytes = w.write_to_vec().unwrap();
        assert_eq!(
            oracle::classic_width_tags(&bytes, binary),
            ([0.2, 0.4], LEAVES.to_vec())
        );
    }
}

fn classic(c: &opencadcodec::CadDocument) -> &opencadcodec::entities::Polyline2D {
    c.entities()
        .find_map(|e| {
            if let EntityType::Polyline2D(p) = e {
                Some(p)
            } else {
                None
            }
        })
        .unwrap()
}
#[test]
fn changing_defaults_after_dxf_roundtrip_only_changes_inherited_leaves() {
    let d = native_chain(&source());
    let cad = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    for binary in [false, true] {
        let mut w = DxfWriter::new(cad.document())
            .with_polyline2d_write_mode(Polyline2DWriteMode::PreserveClassic);
        w.set_binary(binary);
        let bytes = w.write_to_vec().unwrap();
        let mut back = DxfReader::from_reader(Cursor::new(bytes))
            .unwrap()
            .read()
            .unwrap();
        let mut p = classic(&back).clone();
        p.start_width = 0.6;
        p.end_width = 0.8;
        assert_eq!(
            (0..4)
                .map(|i| p.effective_vertex_widths(i).unwrap())
                .collect::<Vec<_>>(),
            vec![[0.6, 0.8], [0., 0.8], [0.6, 0.], [0.2, 0.4]]
        );
        let _ = &mut back;
    }
}
#[test]
fn explicit_zero_is_written_not_omitted() {
    classic_default_dependencies_survive_native_and_real_dxf()
}
#[test]
fn dormant_equal_override_stays_explicit() {
    classic_default_dependencies_survive_native_and_real_dxf()
}
#[test]
fn ordinary_version_matrix() {
    for mode in 0..4 {
        let mut d = source();
        let w = widths(&mut d);
        match mode {
            0 => *w = IfccadPlanarPolylineWidths::Constant { width: 0.15 },
            1 => {
                *w = IfccadPlanarPolylineWidths::PerVertex {
                    widths: vec![[0.2, 0.4], [0., 0.4], [0.2, 0.], [0.2, 0.4]],
                }
            }
            3 => {
                if let IfccadPlanarPolylineWidths::Defaulted { overrides, .. } = w {
                    for v in overrides {
                        v.start = Some(v.start.unwrap_or(0.2));
                        v.end = Some(v.end.unwrap_or(0.4));
                    }
                }
            }
            _ => {}
        }
        let d = native_chain(&d);
        for version in [
            DxfVersion::AC1015,
            DxfVersion::AC1018,
            DxfVersion::AC1021,
            DxfVersion::AC1024,
            DxfVersion::AC1027,
            DxfVersion::AC1032,
        ] {
            for format in [
                IfccadPolylineWidthCadFormat::AsciiDxf,
                IfccadPolylineWidthCadFormat::BinaryDxf,
                IfccadPolylineWidthCadFormat::Dwg,
            ] {
                let target = IfccadPolylineWidthCadTarget { format, version };
                let issues =
                    assess_ifccad_polyline_width_target(&d, target, IfccadLossPolicy::Allow)
                        .unwrap();
                let promotion = format == IfccadPolylineWidthCadFormat::BinaryDxf
                    && version < DxfVersion::AC1021;
                let loss = format == IfccadPolylineWidthCadFormat::Dwg && mode == 2;
                assert_eq!(
                    assess_ifccad_polyline_width_target(&d, target, IfccadLossPolicy::Reject)
                        .is_err(),
                    promotion || loss,
                    "{mode} {format:?} {version:?}"
                );
                assert_eq!(issues.is_empty(), !(promotion || loss));
                let mut cad = ifccad_document_to_cad_document(&d, Default::default())
                    .unwrap()
                    .into_document();
                cad.version = version;
                let (bytes, back) = if format == IfccadPolylineWidthCadFormat::Dwg {
                    let b = DwgWriter::write_to_vec(&cad).unwrap();
                    let r = DwgReader::from_stream(Cursor::new(b.clone()))
                        .read()
                        .unwrap();
                    (b, r)
                } else {
                    let binary = format == IfccadPolylineWidthCadFormat::BinaryDxf;
                    let mut w = DxfWriter::new(&cad)
                        .with_polyline2d_write_mode(Polyline2DWriteMode::PreserveClassic);
                    w.set_binary(binary);
                    let b = w.write_to_vec().unwrap();
                    let r = DxfReader::from_reader(Cursor::new(b.clone()))
                        .unwrap()
                        .read()
                        .unwrap();
                    assert_eq!(
                        oracle::actual_header_version(&b, binary),
                        format!(
                            "{:?}",
                            if promotion {
                                DxfVersion::AC1024
                            } else {
                                version
                            }
                        )
                    );
                    (b, r)
                };
                if format == IfccadPolylineWidthCadFormat::Dwg {
                    assert_eq!(&bytes[..6], format!("{version:?}").as_bytes());
                }
                if mode < 2 {
                    let p = back
                        .model_space_entities()
                        .find(|e| matches!(e, EntityType::LwPolyline(_)))
                        .unwrap();
                    let mut authored = d.clone();
                    let wanted = widths(&mut authored).clone();
                    assert_eq!(
                        cad_geometry_convert::geometry::polyline_width::from_cad_polyline_widths(p)
                            .unwrap(),
                        wanted
                    );
                    if format == IfccadPolylineWidthCadFormat::Dwg {
                        let expected = if mode == 0 {
                            (0.15, vec![])
                        } else {
                            (0., vec![[0.2, 0.4], [0., 0.4], [0.2, 0.], [0.2, 0.4]])
                        };
                        assert_eq!(oracle::dwg_lw_widths(&bytes, version), expected);
                    }
                }
                if mode >= 2 {
                    let p = classic(&back);
                    assert_eq!([p.start_width, p.end_width], [0.2, 0.4]);
                    let expected = vec![[0.2, 0.4], [0., 0.4], [0.2, 0.], [0.2, 0.4]];
                    assert_eq!(
                        (0..4)
                            .map(|i| p.effective_vertex_widths(i).unwrap())
                            .collect::<Vec<_>>(),
                        expected
                    );
                    if format == IfccadPolylineWidthCadFormat::Dwg {
                        assert_eq!(
                            oracle::dwg_classic_widths(&bytes, version),
                            ([0.2, 0.4], expected)
                        );
                        assert!(p
                            .vertices
                            .iter()
                            .all(|v| v.start_width.is_some() && v.end_width.is_some()));
                    } else {
                        let expected = if mode == 2 {
                            LEAVES.to_vec()
                        } else {
                            LEAVES
                                .iter()
                                .map(|p| [Some(p[0].unwrap_or(0.2)), Some(p[1].unwrap_or(0.4))])
                                .collect()
                        };
                        assert_eq!(
                            oracle::classic_width_tags(
                                &bytes,
                                format == IfccadPolylineWidthCadFormat::BinaryDxf
                            ),
                            ([0.2, 0.4], expected)
                        );
                    }
                }
                let native =
                    cad_document_to_encoded_ifccad(&back, common::metadata(), Default::default())
                        .unwrap();
                assert!(!native
                    .validated_source()
                    .document()
                    .model
                    .entities
                    .is_empty());
                eprintln!("width-matrix mode={mode} format={format:?} requested={version:?} actual={:?} promotion={promotion} dependency_loss={loss}",back.version);
            }
        }
    }
}
#[test]
fn constant_output_has_no_competing_width_tags() {
    let mut d = source();
    *widths(&mut d) = IfccadPlanarPolylineWidths::Constant { width: 0.15 };
    let c = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    for binary in [false, true] {
        let mut w = DxfWriter::new(c.document());
        w.set_binary(binary);
        let b = w.write_to_vec().unwrap();
        let rs: Vec<_> = oracle::records(&oracle::tags(&b), "LWPOLYLINE")
            .into_iter()
            .filter(|r| r.iter().any(|p| p.0 == 43))
            .collect();
        assert_eq!(rs.len(), 1);
        assert!(rs[0].contains(&(43, "0.15".into())));
        assert!(!rs[0].iter().any(|p| p.0 == 40 || p.0 == 41));
    }
}
#[test]
fn dwg_inheritance_loss_is_not_a_preservation_pass() {
    let d = source();
    let t = IfccadPolylineWidthCadTarget {
        format: IfccadPolylineWidthCadFormat::Dwg,
        version: DxfVersion::AC1032,
    };
    let issues = assess_ifccad_polyline_width_target(&d, t, IfccadLossPolicy::Allow).unwrap();
    assert_eq!(
        issues
            .iter()
            .filter(|x| x.code == "polyline-width-dependency")
            .count(),
        4
    );
    assert!(assess_ifccad_polyline_width_target(&d, t, IfccadLossPolicy::Reject).is_err());
}

#[test]
fn widths_in_two_paper_mappings_and_model_keep_authored_state() {
    let source = source();
    let mut d = common::viewport_drawing();
    let mut e = source
        .model
        .entities
        .iter()
        .find(|e| {
            matches!(
                e.as_native().map(|e| &e.kind),
                Some(IfccadEntityKind::PlanarPolyline { .. })
            )
        })
        .unwrap()
        .clone();
    e.as_native_mut().unwrap().id = 2000;
    e.as_native_mut().unwrap().layer_id = 0;
    d.model.entities.push(e.clone());
    e.as_native_mut().unwrap().id = 2001;
    d.paper_layouts[0].entities[0] = e.clone();
    let mut paper = d.paper_layouts[0].clone();
    paper.id = 43;
    paper.name = "Second width sheet".into();
    paper.tab_index = 2;
    e.as_native_mut().unwrap().id = 2002;
    paper.entities = vec![e];
    paper.settings = common::paper_settings(ifccad::plot_kernel::PlotUnit::Inch, 2.);
    d.paper_layouts.push(paper);
    d.id_counters.next_entity_id = 2003;
    d.id_counters.next_layout_id = 44;
    d.model.bounds = None;
    d.model.bounds_quality = None;
    validate_ifccad_document(&d).unwrap();
    let converted = ifccad_document_to_cad_document(&native_chain(&d), Default::default()).unwrap();
    assert_eq!(
        converted
            .geometry_assessment()
            .polyline_widths()
            .iter()
            .filter(|a| matches!(a.widths, IfccadPlanarPolylineWidths::Defaulted { .. }))
            .count(),
        3
    );
    for binary in [false, true] {
        let mut writer = DxfWriter::new(converted.document())
            .with_polyline2d_write_mode(Polyline2DWriteMode::PreserveClassic);
        writer.set_binary(binary);
        let back = DxfReader::from_reader(Cursor::new(writer.write_to_vec().unwrap()))
            .unwrap()
            .read()
            .unwrap();
        let n =
            cad_document_to_encoded_ifccad(&back, common::metadata(), Default::default()).unwrap();
        assert_eq!(n.validated_source().document().paper_layouts.len(), 2);
        assert_eq!(
            n.geometry_assessment()
                .polyline_widths()
                .iter()
                .filter(|a| matches!(a.widths, IfccadPlanarPolylineWidths::Defaulted { .. }))
                .count(),
            3
        );
    }
}

#[path = "support/dimension_graphics.rs"]
mod block_fixture;
#[test]
fn bulged_closed_widths_keep_shared_unused_and_reflected_nested_occurrences() {
    use opencadcodec::{entities::Insert, LineWeight, LwPolyline, Vector2, Vector3};
    let mut cad = common::cad();
    let mut p = LwPolyline::from_points(vec![
        Vector2::new(0., 0.),
        Vector2::new(10., 0.),
        Vector2::new(10., 10.),
    ]);
    p.is_closed = true;
    p.normal = Vector3::UNIT_X;
    p.vertices[0].bulge = 0.7;
    for (v, pair) in p
        .vertices
        .iter_mut()
        .zip([[0.2, 0.4], [0., 0.4], [0.2, 0.]])
    {
        v.start_width = pair[0];
        v.end_width = pair[1];
    }
    p.common.line_weight = LineWeight::W0_50;
    block_fixture::block(
        &mut cad,
        "UnusedWidth",
        vec![EntityType::LwPolyline(p.clone())],
    );
    block_fixture::block(&mut cad, "InnerWidth", vec![EntityType::LwPolyline(p)]);
    let mut i = Insert::new("InnerWidth", Vector3::new(2., 3., 4.));
    i.set_x_scale(-2.);
    i.set_y_scale(3.);
    i.set_z_scale(1.);
    i.rotation = 0.4;
    block_fixture::block(&mut cad, "OuterWidth", vec![EntityType::Insert(i)]);
    for x in [0., 100.] {
        cad.add_entity(EntityType::Insert(Insert::new(
            "OuterWidth",
            Vector3::new(x, 0., 0.),
        )))
        .unwrap();
    }
    let native =
        cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
    assert!(native.geometry_assessment().polyline_widths().len() >= 4);
    assert!(native
        .geometry_assessment()
        .polyline_widths()
        .iter()
        .all(|a| a.segments.len() == 3
            && a.segments
                .iter()
                .all(|s| s.band_coverage == IfccadPolylineWidthBandCoverage::Proven)));
    let exported =
        ifccad_document_to_cad_document(native.validated_source().document(), Default::default())
            .unwrap();
    for route in 0..3 {
        let back = if route == 2 {
            DwgReader::from_stream(Cursor::new(
                DwgWriter::write_to_vec(exported.document()).unwrap(),
            ))
            .read()
            .unwrap()
        } else {
            let mut w = DxfWriter::new(exported.document())
                .with_polyline2d_write_mode(Polyline2DWriteMode::PreserveClassic);
            w.set_binary(route == 1);
            DxfReader::from_reader(Cursor::new(w.write_to_vec().unwrap()))
                .unwrap()
                .read()
                .unwrap()
        };
        let paths: Vec<_> = back
            .entities()
            .filter_map(|e| {
                if let EntityType::LwPolyline(p) = e {
                    Some(p)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(paths.len(), 2);
        for p in paths {
            assert!(p.is_closed);
            assert_eq!(p.common.line_weight, LineWeight::W0_50);
            assert_eq!(p.vertices[0].bulge, 0.7);
            assert_eq!(p.vertices[2].end_width, 0.);
        }
        cad_document_to_encoded_ifccad(&back, common::metadata(), Default::default()).unwrap();
    }
}
