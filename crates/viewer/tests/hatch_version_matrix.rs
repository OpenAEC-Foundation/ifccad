//! Physical qualification, independently expected semantics for every offered target.
use ifccad::*;
use ifccad_convert::{opencadcodec::*, *};
use opencadcodec::entities::hatch::*;
use std::io::Cursor;

const VERSIONS: [DxfVersion; 6] = [
    DxfVersion::AC1015,
    DxfVersion::AC1018,
    DxfVersion::AC1021,
    DxfVersion::AC1024,
    DxfVersion::AC1027,
    DxfVersion::AC1032,
];

fn metadata() -> IfccadTargetMetadata {
    let d = load_ifccad_bytes(
        include_bytes!("../../../tests/fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    IfccadTargetMetadata {
        header: d.header,
        drawing_id: d.drawing_id,
    }
}
fn source(version: DxfVersion, bound: bool, enabled: bool, bits: u32) -> (CadDocument, Handle) {
    let mut d = CadDocument::new();
    d.version = version;
    let a = d
        .add_entity(EntityType::Line(Line::from_points(
            Vector3::ZERO,
            Vector3::new(10., 0., 0.),
        )))
        .unwrap();
    let b = d.add_entity(EntityType::Circle(Circle::new())).unwrap();
    let mut h = Hatch::solid();
    h.is_associative = enabled;
    let mut p = BoundaryPath::new();
    p.flags = BoundaryPathFlags::from_bits(bits);
    p.edges.push(BoundaryEdge::CircularArc(CircularArcEdge {
        center: Vector2::ZERO,
        radius: 3.,
        start_angle: 0.,
        end_angle: std::f64::consts::TAU,
        counter_clockwise: true,
    }));
    if bound {
        p.boundary_handles = vec![b, a, b];
    }
    h.paths.push(p);
    let handle = d.add_entity(EntityType::Hatch(Box::new(h))).unwrap();
    (d, handle)
}
fn exchange(d: &CadDocument, dwg: bool) -> CadDocument {
    let bytes = if dwg {
        DwgWriter::write_to_vec(d).unwrap()
    } else {
        DxfWriter::new(d).write_to_vec().unwrap()
    };
    if dwg {
        assert_eq!(&bytes[..6], d.version.as_str().as_bytes());
    } else {
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains(d.version.as_str()));
    }
    let back = if dwg {
        DwgReader::from_stream(Cursor::new(bytes)).read().unwrap()
    } else {
        DxfReader::from_reader(Cursor::new(bytes))
            .unwrap()
            .read()
            .unwrap()
    };
    assert_eq!(back.dwg_source_version.unwrap_or(back.version), d.version);
    back
}

#[test]
fn all_target_cells_preserve_sources_provenance_and_text_islands() {
    for version in VERSIONS {
        for dwg in [false, true] {
            let (mut d, template) = source(version, true, false, 0);
            let EntityType::Hatch(base) = d.get_entity(template).unwrap().clone() else {
                unreachable!()
            };
            let mut cases = Vec::new();
            for bound in [false, true] {
                for enabled in [false, true] {
                    for mask in 0..32 {
                        let bits = (mask & 1)
                            | ((mask & 2) << 1)
                            | ((mask & 4) << 1)
                            | ((mask & 8) << 1)
                            | ((mask & 16) << 3);
                        let mut h = base.clone();
                        h.common.handle = Handle::NULL;
                        h.is_associative = enabled;
                        h.paths[0].flags = BoundaryPathFlags::from_bits(bits);
                        if !bound {
                            h.paths[0].boundary_handles.clear();
                        }
                        let id = d.add_entity(EntityType::Hatch(h)).unwrap();
                        cases.push((id, bits, bound, enabled));
                    }
                }
            }
            let back = exchange(&d, dwg);
            for (id, bits, bound, enabled) in cases {
                let EntityType::Hatch(h) = back.get_entity(id).unwrap() else {
                    panic!("missing Hatch")
                };
                assert_eq!(h.paths[0].flags.bits(), bits, "{version:?} dwg={dwg}");
                // Both writers currently infer activation from the presence of sources.
                assert_eq!(
                    h.is_associative,
                    enabled && bound,
                    "{version:?} dwg={dwg} bound={bound}"
                );
                let EntityType::Hatch(w) = d.get_entity(id).unwrap() else {
                    unreachable!()
                };
                assert_eq!(
                    h.paths[0].boundary_handles, w.paths[0].boundary_handles,
                    "{version:?} dwg={dwg}"
                );
                assert_eq!(h.paths[0].edges, w.paths[0].edges);
            }
            let native =
                cad_document_to_ifccad_document(&back, metadata(), Default::default()).unwrap();
            let encoded = encode_ifccad_document(native.document()).unwrap();
            load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
        }
    }
}

#[test]
fn fills_area_rules_and_both_contour_backings_are_written_in_every_cell() {
    for version in VERSIONS {
        for dwg in [false, true] {
            let (mut d, id) = source(version, true, false, 157);
            let EntityType::Hatch(base) = d.get_entity(id).unwrap().clone() else {
                unreachable!()
            };
            let mut cases = Vec::new();
            for fill in 0..3 {
                for style in [
                    HatchStyleType::Normal,
                    HatchStyleType::Outer,
                    HatchStyleType::Ignore,
                ] {
                    for polyline in [false, true] {
                        let mut h = base.clone();
                        h.common.handle = Handle::NULL;
                        h.style = style;
                        if polyline {
                            h.paths[0].flags = BoundaryPathFlags::from_bits(159);
                            h.paths[0].edges = vec![BoundaryEdge::Polyline(PolylineEdge {
                                is_closed: true,
                                vertices: vec![
                                    Vector3::new(0., 0., 0.),
                                    Vector3::new(3., 0., 0.),
                                    Vector3::new(3., 3., 0.),
                                    Vector3::new(0., 3., 0.),
                                ],
                            })];
                        }
                        if fill == 1 {
                            h.is_solid = false;
                            h.pattern_type = HatchPatternType::Custom;
                            h.pattern = HatchPattern::new("LITERAL");
                            h.pattern.lines.push(HatchPatternLine {
                                angle: 0.,
                                base_point: Vector2::ZERO,
                                offset: Vector2::new(0., 1.),
                                dash_lengths: vec![],
                            });
                        }
                        if fill == 2 {
                            h.gradient_color.enabled = true;
                            h.gradient_color.name = "LINEAR".into();
                            h.gradient_color.colors = vec![
                                GradientColorEntry {
                                    value: 0.,
                                    color: Color::Rgb {
                                        r: 10,
                                        g: 20,
                                        b: 30,
                                    },
                                    color_name: None,
                                    book_name: None,
                                },
                                GradientColorEntry {
                                    value: 1.,
                                    color: Color::Rgb {
                                        r: 40,
                                        g: 50,
                                        b: 60,
                                    },
                                    color_name: None,
                                    book_name: None,
                                },
                            ];
                        }
                        let handle = d.add_entity(EntityType::Hatch(h)).unwrap();
                        cases.push((handle, fill));
                    }
                }
            }
            let back = exchange(&d, dwg);
            for (id, fill) in cases {
                let EntityType::Hatch(w) = d.get_entity(id).unwrap() else {
                    unreachable!()
                };
                let EntityType::Hatch(r) = back.get_entity(id).unwrap() else {
                    panic!("missing Hatch")
                };
                assert_eq!(w.paths, r.paths, "{version:?} dwg={dwg}");
                assert_eq!(w.style, r.style);
                assert_eq!(w.is_solid, r.is_solid);
                assert_eq!(
                    r.gradient_color.enabled,
                    fill == 2 && !(dwg && version == DxfVersion::AC1015),
                    "{version:?} dwg={dwg}"
                );
                if fill == 1 {
                    assert_eq!(w.pattern, r.pattern);
                }
            }
            let native =
                cad_document_to_ifccad_document(&back, metadata(), Default::default()).unwrap();
            let bytes = encode_ifccad_document(native.document()).unwrap();
            load_ifccad_bytes(bytes.bytes(), Default::default()).unwrap();
        }
    }
}

#[test]
fn editing_parameters_construction_and_inactive_gradients_have_explicit_profiles() {
    for version in VERSIONS {
        for dwg in [false, true] {
            let (mut d, id) = source(version, true, false, 0);
            let EntityType::Hatch(base) = d.get_entity(id).unwrap().clone() else {
                unreachable!()
            };
            let mut cases = Vec::new();
            for derived in [false, true] {
                for solid in [false, true] {
                    let mut h = base.clone();
                    h.common.handle = Handle::NULL;
                    h.is_solid = solid;
                    h.pattern.description = "source description".into();
                    h.pattern_angle = 0.75;
                    h.pattern_scale = 2.;
                    h.is_double = true;
                    h.pattern_type = HatchPatternType::Custom;
                    h.pattern.lines = vec![HatchPatternLine {
                        angle: 0.,
                        base_point: Vector2::ZERO,
                        offset: Vector2::new(0., 1.),
                        dash_lengths: vec![],
                    }];
                    h.record_pattern_origin(Vector2::new(2., 3.));
                    h.paths[0].flags = BoundaryPathFlags::from_bits(if derived { 4 } else { 0 });
                    h.seed_points = vec![Vector2::new(20., -2.), Vector2::new(20., -2.)];
                    h.pixel_size = 0.25;
                    h.gradient_color.angle = 0.5;
                    let handle = d.add_entity(EntityType::Hatch(h)).unwrap();
                    cases.push((handle, derived, solid));
                }
            }
            let back = exchange(&d, dwg);
            for (id, derived, solid) in cases {
                let EntityType::Hatch(w) = d.get_entity(id).unwrap() else {
                    unreachable!()
                };
                let EntityType::Hatch(r) = back.get_entity(id).unwrap() else {
                    panic!("missing Hatch")
                };
                assert_eq!(r.pattern.description, "");
                assert_eq!(r.pattern_type, w.pattern_type);
                assert_eq!(r.seed_points, w.seed_points);
                assert_eq!(r.pixel_size, if derived { 0.25 } else { 0. });
                assert_eq!(r.stored_pattern_origin(), w.stored_pattern_origin());
                assert_eq!(r.pattern_angle, if solid { 0. } else { 0.75 });
                assert_eq!(r.pattern_scale, if solid { 1. } else { 2. });
                assert_eq!(r.is_double, !solid);
                assert_eq!(
                    r.pattern.lines,
                    if solid {
                        vec![]
                    } else {
                        w.pattern.lines.clone()
                    }
                );
                assert_eq!(
                    r.gradient_color.angle,
                    if dwg && version != DxfVersion::AC1015 {
                        0.5
                    } else {
                        0.
                    }
                );
            }
        }
    }
}
#[test]
fn providers_and_model_paper_block_owners_keep_whole_ordered_source_groups() {
    for bytes in [
        include_bytes!("../../../conformance/next/ifccad/valid/geometry-families.ifcx").as_slice(),
        include_bytes!("../../../conformance/next/ifccad/valid/text-rich.ifcx").as_slice(),
        include_bytes!("../../../conformance/next/ifccad/valid/text-owners.ifcx").as_slice(),
        include_bytes!("../../../conformance/next/ifccad/valid/viewport-forward-clip.ifcx")
            .as_slice(),
        include_bytes!("../../../tests/fixtures/ifccad/hello-nested-blocks.ifcx").as_slice(),
    ] {
        let mut native = load_ifccad_bytes(bytes, Default::default())
            .unwrap()
            .into_document();
        // Deliberately qualified synthetic text profile. Rich background opacity
        // is a separate unsupported presentation feature, not a boundary kind.
        for entities in std::iter::once(&mut native.model.entities)
            .chain(native.paper_layouts.iter_mut().map(|p| &mut p.entities))
            .chain(native.blocks.iter_mut().map(|b| &mut b.entities))
        {
            for e in entities {
                if let Some(e) = e.as_native_mut() {
                    if let IfccadEntityKind::MText(t) = &mut e.kind {
                        t.background = None;
                        t.columns = None;
                        t.character_format = Default::default();
                        t.paragraph_format = Default::default();
                        t.content = vec![ifccad::text::MTextParagraph {
                            inlines: vec![ifccad::text::MTextInline::Run {
                                text: "Source boundary".into(),
                                character_format: Default::default(),
                            }],
                            ..Default::default()
                        }];
                    }
                }
            }
        }
        let mut next = native.id_counters.next_entity_id;
        let mut added = Vec::new();
        for entities in std::iter::once(&mut native.model.entities)
            .chain(native.paper_layouts.iter_mut().map(|p| &mut p.entities))
            .chain(native.blocks.iter_mut().map(|b| &mut b.entities))
        {
            let sources = entities
                .iter()
                .filter_map(IfccadEntity::as_native)
                .filter(|e| {
                    !matches!(
                        e.kind,
                        IfccadEntityKind::Point { .. } | IfccadEntityKind::Hatch(_)
                    )
                })
                .cloned()
                .collect::<Vec<_>>();
            for mut template in sources {
                let source = template.id;
                template.id = next;
                added.push(next);
                next += 1;
                template.kind = IfccadEntityKind::Hatch(IfccadHatch {
                    classic_settings: None,
                    boundary_construction: None,
                    association_enabled: true,
                    gradient: None,
                    placement: IfccadPlacement {
                        origin: [0., 0., 0.],
                        x_axis: [1., 0., 0.],
                        y_axis: [0., 1., 0.],
                    },
                    loops: vec![IfccadHatchLoop {
                        boundary: ifccad::geometry_kernel::hatch::HatchBoundary2::Circle {
                            center: [0., 0.],
                            radius: 0.5,
                        },
                        source_entity_ids: vec![source, source],
                        provenance: None,
                        text_island: true,
                    }],
                    area_rule: ifccad::geometry_kernel::hatch::HatchAreaRule::Normal,
                    join_tolerance: 1e-9,
                    fill: ifccad::geometry_kernel::hatch::HatchFill::Solid,
                });
                entities.push(IfccadEntity::Native(template));
            }
        }
        native.id_counters.next_entity_id = next;
        recompute_ifccad_document_bounds(&mut native).unwrap();
        let mapped = ifccad_document_to_cad_document(&native, Default::default()).unwrap();
        for id in added {
            let handle = mapped
                .mappings()
                .entities
                .cad_handle(id)
                .expect("new Hatch mapped");
            let EntityType::Hatch(h) = mapped.document().get_entity(handle).unwrap() else {
                panic!()
            };
            assert_eq!(
                h.paths[0].boundary_handles.len(),
                2,
                "native Hatch {id}: {:?}",
                mapped.diagnostics()
            );
        }
        for version in VERSIONS {
            for dwg in [false, true] {
                let mut d = mapped.document().clone();
                d.version = version;
                let back = exchange(&d, dwg);
                for e in d.entities() {
                    if let EntityType::Hatch(w) = e {
                        let EntityType::Hatch(r) = back.get_entity(w.common.handle).unwrap() else {
                            panic!("Hatch kind changed")
                        };
                        assert_eq!(w.common.owner_handle, r.common.owner_handle);
                        assert_eq!(w.paths[0].boundary_handles, r.paths[0].boundary_handles);
                        assert_eq!(w.paths[0].flags, r.paths[0].flags);
                        assert_eq!(
                            r.is_associative,
                            w.is_associative
                                && w.paths.iter().any(|p| !p.boundary_handles.is_empty())
                        );
                    }
                }
                let output =
                    cad_document_to_ifccad_document(&back, metadata(), Default::default()).unwrap();
                let encoded = encode_ifccad_document(output.document()).unwrap();
                load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
            }
        }
    }
}

#[test]
fn independently_authored_dxf_text_loops_keep_their_literal_classification() {
    let source = DxfReader::from_reader(Cursor::new(
        include_bytes!("../../cad-geometry-convert/tests/fixtures/hatch-text-island-reference.dxf")
            .to_vec(),
    ))
    .unwrap()
    .read()
    .unwrap();
    for version in VERSIONS {
        for dwg in [false, true] {
            let mut d = source.clone();
            d.version = version;
            let back = exchange(&d, dwg);
            for e in d.entities() {
                if let EntityType::Hatch(w) = e {
                    let EntityType::Hatch(r) = back.get_entity(w.common.handle).unwrap() else {
                        panic!()
                    };
                    assert_eq!(w.paths, r.paths);
                    assert_eq!(w.style, r.style);
                    assert_eq!(w.is_solid, r.is_solid);
                }
            }
        }
    }
}
#[test]
fn all_gradient_types_modes_and_concrete_endpoints_have_version_specific_readback() {
    for version in VERSIONS {
        for dwg in [false, true] {
            let (mut d, id) = source(version, true, false, 157);
            let EntityType::Hatch(base) = d.get_entity(id).unwrap().clone() else {
                unreachable!()
            };
            let mut cases = Vec::new();
            for name in [
                "LINEAR",
                "CYLINDER",
                "INVCYLINDER",
                "SPHERICAL",
                "INVSPHERICAL",
                "HEMISPHERICAL",
                "INVHEMISPHERICAL",
                "CURVED",
                "INVCURVED",
            ] {
                for single in [false, true] {
                    for indexed in [false, true] {
                        let mut h = base.clone();
                        h.common.handle = Handle::NULL;
                        h.gradient_color.enabled = true;
                        h.gradient_color.name = name.into();
                        h.gradient_color.angle = 0.5;
                        h.gradient_color.shift = 0.5;
                        h.gradient_color.color_tint = 0.5;
                        h.gradient_color.is_single_color = single;
                        h.gradient_color.colors = vec![
                            GradientColorEntry {
                                value: 0.,
                                color: if indexed {
                                    Color::Index(1)
                                } else {
                                    Color::Rgb {
                                        r: 10,
                                        g: 20,
                                        b: 30,
                                    }
                                },
                                color_name: None,
                                book_name: None,
                            },
                            GradientColorEntry {
                                value: 1.,
                                color: if indexed {
                                    Color::Index(2)
                                } else {
                                    Color::Rgb {
                                        r: 40,
                                        g: 50,
                                        b: 60,
                                    }
                                },
                                color_name: None,
                                book_name: None,
                            },
                        ];
                        cases.push(d.add_entity(EntityType::Hatch(h)).unwrap());
                    }
                }
            }
            let back = exchange(&d, dwg);
            for id in cases {
                let EntityType::Hatch(w) = d.get_entity(id).unwrap() else {
                    unreachable!()
                };
                let EntityType::Hatch(r) = back.get_entity(id).unwrap() else {
                    panic!()
                };
                assert_eq!(
                    r.gradient_color,
                    if dwg && version == DxfVersion::AC1015 {
                        HatchGradientPattern::default()
                    } else {
                        w.gradient_color.clone()
                    },
                    "{version:?} dwg={dwg}"
                );
            }
            let native =
                cad_document_to_ifccad_document(&back, metadata(), Default::default()).unwrap();
            let encoded = encode_ifccad_document(native.document()).unwrap();
            load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
        }
    }
}
