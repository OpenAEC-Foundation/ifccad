use cad_geometry_convert::hatch::pattern::*;
use ocdraw::geometry_kernel::hatch::*;
use opencadcodec::{entities::hatch::*, Vector2};
fn bounded_pattern() -> Hatch {
    let mut h = Hatch::with_pattern(HatchPattern::new("Literal"));
    h.pattern.lines.push(HatchPatternLine {
        angle: 0.,
        base_point: Vector2::ZERO,
        offset: Vector2::new(1., 2.),
        dash_lengths: vec![2., -1., 0.],
    });
    let mut path = BoundaryPath::new();
    path.edges.push(BoundaryEdge::CircularArc(CircularArcEdge {
        center: Vector2::ZERO,
        radius: 10.,
        start_angle: 0.,
        end_angle: std::f64::consts::TAU,
        counter_clockwise: true,
    }));
    h.paths.push(path);
    h
}

#[test]
fn whole_pattern_preparation_retains_fill_and_contour_certificates() {
    let h = bounded_pattern();
    let p = cad_geometry_convert::hatch::prepare_hatch_from_cad(&h, 1e-9).unwrap();
    assert_eq!(p.boundaries.len(), 1);
    assert!(p.pairs.iter().any(|v| !v.points.is_empty()));
    assert!(p.pairs.iter().any(|v| !v.curves.is_empty()));
    let q = cad_geometry_convert::hatch::prepare_hatch_to_cad(
        p.placement,
        &p.boundaries,
        p.area_rule,
        p.join_tolerance,
        &p.fill,
    )
    .unwrap();
    assert!(!q.hatch.is_solid);
    assert_eq!(q.hatch.pattern.lines, h.pattern.lines);
}

#[test]
fn invalid_pattern_numeric_fields_are_hard_errors() {
    let mut h = bounded_pattern();
    h.pattern_angle = f64::NAN;
    assert!(matches!(
        prepare_pattern_from_cad(
            &h,
            Default::default(),
            &[HatchBoundary2::Circle {
                center: [0., 0.],
                radius: 10.
            }]
        ),
        Err(cad_geometry_convert::hatch::CadHatchPreparationError::Primitive(_))
    ));
    h.pattern_type = HatchPatternType::UserDefined;
    h.is_double = true;
    h.pattern.lines[0].dash_lengths.clear();
    assert!(matches!(
        cad_geometry_convert::hatch::prepare_hatch_from_cad_with_pattern_context(&h, 1e-9, true),
        Err(cad_geometry_convert::hatch::CadHatchPreparationError::Primitive(_))
    ));
}

#[test]
fn only_the_qualified_origin_slot_is_consumed() {
    use opencadcodec::xdata::{ExtendedDataRecord, XDataValue};
    use opencadcodec::Vector3;
    let mut h = bounded_pattern();
    let mut record = ExtendedDataRecord::new("ACAD");
    record.values = vec![
        XDataValue::ControlString("{".into()),
        XDataValue::Point3D(Vector3::new(99., 98., 97.)),
        XDataValue::ControlString("}".into()),
        XDataValue::Point3D(Vector3::new(3., 4., 0.)),
        XDataValue::String("unrelated".into()),
    ];
    h.common.extended_data.add_record(record.clone());
    let residual = cad_geometry_convert::hatch::pattern::residual_common(&h).unwrap();
    let remaining = residual.extended_data.get_record("ACAD").unwrap();
    assert_eq!(remaining.values.len(), 4);
    assert_eq!(remaining.values[1], record.values[1]);
    assert_eq!(remaining.values[3], record.values[4]);
    assert_eq!(h.common.extended_data.get_record("ACAD"), Some(&record));
}

#[test]
fn user_defined_double_needs_context_and_preserves_both_directions() {
    use cad_geometry_convert::hatch::*;
    let mut h = bounded_pattern();
    h.pattern_type = HatchPatternType::UserDefined;
    h.is_double = true;
    h.pattern_scale = 2.;
    h.pattern.lines[0].offset = Vector2::new(0., 2.);
    h.pattern.lines[0].dash_lengths.clear();
    assert!(prepare_hatch_from_cad(&h, 1e-9).is_err());
    assert!(prepare_hatch_from_cad_with_pattern_context(&h, 1e-9, false).is_err());
    let p = prepare_hatch_from_cad_with_pattern_context(&h, 1e-9, true).unwrap();
    let HatchFill::LinePattern(fill) = p.fill else {
        panic!()
    };
    assert_eq!(fill.families.len(), 2);
    assert_eq!(fill.families[1].angle, std::f64::consts::FRAC_PI_2);
    assert!(p.losses.iter().any(|l| l.field == "pattern_type"));
    assert!(p
        .pairs
        .iter()
        .flat_map(|p| &p.points)
        .all(|(_, d)| d < &cad_geometry_convert::geometry::numeric::exact(1e-18)));
}
#[test]
fn already_effective_families_are_inverted_once_and_phase_is_preserved() {
    let mut h = Hatch::with_pattern(HatchPattern::new("Explicit families"));
    h.pattern_angle = std::f64::consts::FRAC_PI_2;
    h.pattern_scale = 2.;
    h.record_pattern_origin(Vector2::new(10., 20.));
    h.pattern.lines.push(HatchPatternLine {
        angle: std::f64::consts::FRAC_PI_2,
        base_point: Vector2::new(10., 22.),
        offset: Vector2::new(-6., 4.),
        dash_lengths: vec![6., -2., 0.],
    });
    let contours = vec![HatchBoundary2::Circle {
        center: [0., 0.],
        radius: 10.,
    }];
    let p = prepare_pattern_from_cad(&h, Default::default(), &contours).unwrap();
    assert_eq!(p.pattern.origin, [10., 20.]);
    assert_eq!(p.pattern.rotation, h.pattern_angle);
    assert_eq!(p.pattern.scale, 2.);
    let f = &p.pattern.families[0];
    assert!((f.base_point[0] - 1.).abs() < 1e-12);
    assert!(f.base_point[1].abs() < 1e-12);
    assert!((f.offset[0] - 2.).abs() < 1e-12);
    assert!((f.offset[1] - 3.).abs() < 1e-12);
    assert_eq!(
        f.dashes,
        vec![
            HatchDash::Dash { length: 3. },
            HatchDash::Gap { length: 1. },
            HatchDash::Dot
        ]
    );
    assert!(!p.pair.points.is_empty());
    for (_, d) in &p.pair.points {
        assert!(d < &cad_geometry_convert::geometry::numeric::exact(1e-18));
    }
    let q = prepare_pattern_to_cad(
        &p.pattern,
        Default::default(),
        opencadcodec::Vector3::UNIT_Z,
        0.,
        &contours,
    )
    .unwrap();
    assert_eq!(q.pattern.lines.len(), 1);
    assert!((q.pattern.lines[0].base_point.y - 22.).abs() < 1e-12);
    assert!((q.pattern.lines[0].offset.x + 6.).abs() < 1e-12);
}

#[test]
fn mirrored_occurrence_cannot_hide_certificate_error() {
    use cad_geometry_convert::geometry::{blocks::EvaluatedBlock, numeric::exact};
    use ocdraw::geometry_kernel::{BlockTransform, CoordinateFrame3, Scale3};
    let mut h = Hatch::with_pattern(HatchPattern::new("Dashed"));
    h.pattern.lines.push(HatchPatternLine {
        angle: 0.25,
        base_point: Vector2::new(1., 2.),
        offset: Vector2::new(-0.5, 2.),
        dash_lengths: vec![1., -0.5, 0.],
    });
    let contours = vec![HatchBoundary2::Circle {
        center: [0., 0.],
        radius: 10.,
    }];
    let p = prepare_pattern_from_cad(&h, Default::default(), &contours).unwrap();
    let e = EvaluatedBlock::native(
        BlockTransform::try_new(CoordinateFrame3::default(), 0., Scale3::new(-2., 3., 1.)).unwrap(),
        [0., 0., 0.],
    );
    for (mut point, _) in p.pair.points {
        point.apply(&e, &e);
        assert!(point.squared_deviation().1 < exact(1e-18));
    }
}

#[test]
fn dense_continuous_pattern_uses_bounded_certificates_without_a_density_cap() {
    let mut h = Hatch::with_pattern(HatchPattern::new("Dense"));
    h.pattern.lines.push(HatchPatternLine {
        angle: 0.,
        base_point: Vector2::ZERO,
        offset: Vector2::new(0., 1e-200),
        dash_lengths: vec![],
    });
    let contours = [HatchBoundary2::Circle {
        center: [0., 0.],
        radius: 10.,
    }];
    let p = prepare_pattern_from_cad(&h, Default::default(), &contours).unwrap();
    assert_eq!(p.pattern.families[0].offset[1], 1e-200);
    assert_eq!(p.pair.points.len(), 4);
    assert!(p
        .pair
        .points
        .iter()
        .all(|(_, d)| *d == cad_geometry_convert::geometry::numeric::exact(0.)));
}

#[test]
fn invalid_source_plane_is_a_hard_error_without_panicking() {
    let mut h = Hatch::with_pattern(HatchPattern::new("Invalid plane"));
    h.pattern.lines.push(HatchPatternLine {
        angle: 0.,
        base_point: Vector2::ZERO,
        offset: Vector2::new(0., 1.),
        dash_lengths: vec![],
    });
    h.normal.x = f64::NAN;
    let contours = [HatchBoundary2::Circle {
        center: [0., 0.],
        radius: 10.,
    }];
    let result =
        std::panic::catch_unwind(|| prepare_pattern_from_cad(&h, Default::default(), &contours));
    assert!(result.is_ok(), "invalid source data must not panic");
    assert!(result.unwrap().is_err());
}

#[test]
fn spacing_underflow_is_numeric_failure_not_semantic_omission() {
    let mut h = bounded_pattern();
    h.pattern_scale = f64::MAX;
    h.pattern.lines[0].offset = Vector2::new(0., f64::from_bits(1));
    h.pattern.lines[0].dash_lengths.clear();
    assert!(matches!(
        prepare_pattern_from_cad(
            &h,
            Default::default(),
            &[HatchBoundary2::Circle {
                center: [0., 0.],
                radius: 1.
            }]
        ),
        Err(cad_geometry_convert::hatch::CadHatchPreparationError::Primitive(_))
    ));
}

#[test]
fn oblique_plane_pattern_certificate_propagates_through_nested_nonuniform_blocks() {
    use cad_geometry_convert::geometry::{blocks::EvaluatedBlock, numeric::exact};
    use ocdraw::geometry_kernel::{BlockTransform, CoordinateFrame3, Point3, Scale3, Vector3};
    let a = 2f64.sqrt().recip();
    let b = 6f64.sqrt().recip();
    let plane = CoordinateFrame3::try_new(
        Point3::new(7., 8., 9.),
        Vector3::new(a, a, 0.),
        Vector3::new(-b, b, 2. * b),
    )
    .unwrap();
    let p = HatchLinePattern {
        name: Some("Tilted".into()),
        description: Some("".into()),
        origin: [3., 4.],
        rotation: 0.4,
        scale: 2.,
        families: vec![HatchLineFamily {
            angle: 0.7,
            base_point: [1., 2.],
            offset: [0.75, -2.],
            dashes: vec![
                HatchDash::Dash { length: 2. },
                HatchDash::Gap { length: 1. },
                HatchDash::Dot,
            ],
        }],
    };
    let h = cad_geometry_convert::hatch::prepare_hatch_to_cad(
        plane,
        &[HatchBoundary2::Circle {
            center: [0., 0.],
            radius: 10.,
        }],
        HatchAreaRule::Normal,
        1e-9,
        &HatchFill::LinePattern(p),
    )
    .unwrap();
    let transform = EvaluatedBlock::native(
        BlockTransform::try_new(Default::default(), 0.3, Scale3::new(-2., 3., 1.)).unwrap(),
        [1., 2., 3.],
    );
    for (mut point, _) in h.pairs.into_iter().flat_map(|p| p.points) {
        point.apply(&transform, &transform);
        point.apply(&transform, &transform);
        assert!(point.squared_deviation().1 < exact(1e-18));
    }
}
