use cad_geometry_convert::hatch::*;
use opencadcodec::entities::hatch::*;
use opencadcodec::{Color, Vector2};

fn hatch(fill: usize, style: HatchStyleType, bits: u32) -> Hatch {
    let mut h = Hatch::solid();
    h.style = style;
    let mut p = BoundaryPath::new();
    p.flags = BoundaryPathFlags::from_bits(bits);
    p.edges.push(BoundaryEdge::CircularArc(CircularArcEdge {
        center: Vector2::ZERO,
        radius: 3.,
        start_angle: 0.,
        end_angle: std::f64::consts::TAU,
        counter_clockwise: true,
    }));
    h.paths.push(p);
    if fill == 1 {
        h.is_solid = false;
        h.pattern_type = HatchPatternType::Custom;
        h.pattern.name = "CUSTOM".into();
        h.pattern.lines = vec![HatchPatternLine {
            angle: 0.,
            base_point: Vector2::ZERO,
            offset: Vector2::new(0., 1.),
            dash_lengths: vec![],
        }];
    }
    if fill == 2 {
        h.gradient_color.enabled = true;
        h.gradient_color.name = "LINEAR".into();
        h.gradient_color.colors = vec![
            GradientColorEntry {
                color_name: None,
                book_name: None,
                value: 0.,
                color: Color::Rgb { r: 0, g: 0, b: 0 },
            },
            GradientColorEntry {
                color_name: None,
                book_name: None,
                value: 1.,
                color: Color::Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                },
            },
        ];
    }
    h
}

#[test]
fn known_loop_flags_roundtrip_for_every_fill_and_rule() {
    for fill in 0..3 {
        for style in [
            HatchStyleType::Normal,
            HatchStyleType::Outer,
            HatchStyleType::Ignore,
        ] {
            for bits in [0, 1, 4, 8, 16, 128, 136, 157] {
                let h = hatch(fill, style, bits);
                let p = prepare_hatch_from_cad(&h, 1e-9).unwrap();
                assert_eq!(p.loop_flags.len(), 1);
                assert_eq!(p.loop_flags[0].text_island, bits & 128 != 0);
                let flags = prepare_loop_flags_from_cad(&h).unwrap();
                let mut out = prepare_hatch_to_cad(
                    p.placement,
                    &p.boundaries,
                    p.area_rule,
                    p.join_tolerance,
                    &p.fill,
                    p.gradient.as_ref(),
                )
                .unwrap()
                .hatch;
                apply_loop_flags_to_cad(&mut out, &flags).unwrap();
                assert_eq!(out.paths[0].flags.bits() & !2, bits);
                assert_eq!(out.style, style);
                assert_eq!(out.gradient_color.enabled, fill == 2);
                assert!(!p.losses.iter().any(|l| l.field == "paths.flags"));
            }
        }
    }
}

#[test]
fn unknown_active_flags_remain_unsupported() {
    for bits in [32, 64, 256, 512] {
        assert!(matches!(
            prepare_hatch_from_cad(&hatch(0, HatchStyleType::Normal, bits), 1e-9),
            Err(CadHatchPreparationError::Unsupported { .. })
        ));
    }
}

#[test]
fn loop_count_mismatch_is_hard_error() {
    let mut h = hatch(0, HatchStyleType::Normal, 128);
    assert!(matches!(
        apply_loop_flags_to_cad(&mut h, &[]),
        Err(CadHatchPreparationError::LoopCountMismatch {
            expected: 1,
            actual: 0
        })
    ));
}

#[test]
fn text_island_flags_survive_ac1032_dxf_and_dwg_for_every_fill_and_rule() {
    use opencadcodec::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType};
    use std::io::Cursor;
    for fill in 0..3 {
        for style in [
            HatchStyleType::Normal,
            HatchStyleType::Outer,
            HatchStyleType::Ignore,
        ] {
            for bits in [128, 136, 157] {
                let mut d = CadDocument::new();
                d.add_entity(EntityType::Hatch(Box::new(hatch(fill, style, bits))))
                    .unwrap();
                for dwg in [false, true] {
                    let read = if dwg {
                        DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(&d).unwrap()))
                            .read()
                            .unwrap()
                    } else {
                        DxfReader::from_reader(Cursor::new(
                            DxfWriter::new(&d).write_to_vec().unwrap(),
                        ))
                        .unwrap()
                        .read()
                        .unwrap()
                    };
                    let h = read
                        .entities()
                        .find_map(|e| {
                            if let EntityType::Hatch(h) = e {
                                Some(h)
                            } else {
                                None
                            }
                        })
                        .unwrap();
                    assert_eq!(h.paths[0].flags.bits() & !2, bits);
                    assert_eq!(h.style, style);
                    assert_eq!(h.gradient_color.enabled, fill == 2);
                    assert_eq!(h.is_solid, fill != 1);
                    let native = prepare_hatch_from_cad(h, 1e-9).unwrap();
                    assert!(native.loop_flags[0].text_island);
                }
            }
        }
    }
}

#[test]
fn hand_authored_text_island_flags_have_independent_expectations() {
    use opencadcodec::{DxfReader, EntityType};
    use std::io::Cursor;
    let read = DxfReader::from_reader(Cursor::new(include_bytes!(
        "fixtures/hatch-text-island-reference.dxf"
    )))
    .unwrap()
    .read()
    .unwrap();
    let hatches = read
        .entities()
        .filter_map(|e| {
            if let EntityType::Hatch(h) = e {
                Some(h)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(hatches.len(), 2);
    for (h, bits) in hatches.iter().zip([128, 136]) {
        assert_eq!(h.paths.len(), 3);
        assert_eq!(h.paths[2].flags.bits(), bits);
        let native = prepare_hatch_from_cad(h, 1e-9).unwrap();
        assert!(native.loop_flags[2].text_island);
        assert_eq!(
            native.loop_flags[2]
                .provenance
                .as_ref()
                .is_some_and(|p| p.text_box),
            bits == 136
        );
    }
}
