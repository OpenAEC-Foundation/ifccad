use cad_geometry_convert::hatch::*;
use ifccad::geometry_kernel::{hatch::*, CoordinateFrame3};
use opencadcodec::{entities::hatch::*, Vector2};
#[test]
fn original_definition_is_captured_directly_and_candidate_owns_its_evidence() {
    let mut h = Hatch::with_pattern(HatchPattern::new("FP_1"));
    h.pattern.lines.push(HatchPatternLine {
        angle: 0.7853981633974487,
        base_point: Vector2::new(89.80256121069161, -0.),
        offset: Vector2::new(89.80256121069161, 89.80256121069145),
        dash_lengths: vec![],
    });
    let editing = prepare_editing_from_cad(&h, Default::default()).unwrap();
    let c = editing.classic_settings.unwrap();
    let line = &c.definition_lines.as_ref().unwrap()[0];
    assert_eq!(line.offset_vector, [89.80256121069161, 89.80256121069145]);
    assert_eq!(line.base_point[1].to_bits(), (-0f64).to_bits());
    assert!(c.origin.is_none());
    let boundaries = [HatchBoundary2::Circle {
        center: [0., 0.],
        radius: 10.,
    }];
    let mut fill = pattern::prepare_pattern_from_cad(&h, Default::default(), &boundaries)
        .unwrap()
        .pattern;
    fill.families[0].base_point[1] += 2f64.powi(-40);
    let active = HatchFill::LinePattern(fill);
    let fallback = prepare_hatch_to_cad(
        Default::default(),
        &boundaries,
        HatchAreaRule::Normal,
        1e-9,
        &active,
        None,
    )
    .unwrap();
    let before = fallback.hatch.clone();
    let alternatives = prepare_hatch_editing_candidates(
        fallback,
        Default::default(),
        &active,
        Some(&c),
        None,
        true,
    )
    .unwrap();
    assert_eq!(alternatives.fallback.hatch, before);
    let preferred = alternatives.preferred.unwrap();
    assert_eq!(preferred.hatch.pattern.lines, h.pattern.lines);
    assert!(preferred.hatch.stored_pattern_origin().is_none());
    assert!(preferred
        .pairs
        .iter()
        .flat_map(|p| &p.points)
        .any(|(_, d)| *d > cad_geometry_convert::geometry::numeric::exact(0.)));
}

#[test]
fn translated_definition_preserves_vectors_origin_absence_and_unrelated_xdata() {
    use ifccad::geometry_kernel::{Point3, Vector3};
    use opencadcodec::xdata::{ExtendedDataRecord, XDataValue};
    let mut h = Hatch::with_pattern(HatchPattern::new("Original"));
    h.pattern.lines.push(HatchPatternLine {
        angle: 0.,
        base_point: Vector2::ZERO,
        offset: Vector2::new(0., 2.),
        dash_lengths: vec![],
    });
    let c = prepare_editing_from_cad(&h, Default::default())
        .unwrap()
        .classic_settings
        .unwrap();
    let b = [HatchBoundary2::Circle {
        center: [0., 0.],
        radius: 10.,
    }];
    let fill = HatchFill::LinePattern(
        pattern::prepare_pattern_from_cad(&h, Default::default(), &b)
            .unwrap()
            .pattern,
    );
    let placement = CoordinateFrame3::try_new(
        Point3::new(10., 20., 0.),
        Vector3::new(1., 0., 0.),
        Vector3::new(0., 1., 0.),
    )
    .unwrap();
    let mut fallback =
        prepare_hatch_to_cad(placement, &b, HatchAreaRule::Normal, 1e-9, &fill, None).unwrap();
    let mut unrelated = ExtendedDataRecord::new("Other");
    unrelated.values = vec![XDataValue::String("retained".into())];
    fallback
        .hatch
        .common
        .extended_data
        .add_record(unrelated.clone());
    let before = fallback.hatch.clone();
    let alternatives =
        prepare_hatch_editing_candidates(fallback, placement, &fill, Some(&c), None, true).unwrap();
    assert_eq!(alternatives.fallback.hatch, before);
    let preferred = alternatives.preferred.unwrap();
    assert_eq!(
        preferred.hatch.pattern.lines[0].base_point,
        Vector2::new(10., 20.)
    );
    assert_eq!(
        preferred.hatch.pattern.lines[0].offset,
        Vector2::new(0., 2.)
    );
    assert!(preferred.hatch.stored_pattern_origin().is_none());
    assert_eq!(
        preferred.hatch.common.extended_data.get_record("Other"),
        Some(&unrelated)
    );
    assert!(preferred
        .pairs
        .iter()
        .flat_map(|p| &p.points)
        .all(|(_, d)| *d == cad_geometry_convert::geometry::numeric::exact(0.)));
}
#[test]
fn capture_defaults_and_dormant_construction_without_fabricating_derived() {
    let mut h = Hatch::solid();
    let p = prepare_editing_from_cad(&h, CoordinateFrame3::default()).unwrap();
    assert!(p.classic_settings.is_none());
    assert!(p.boundary_construction.is_none());
    h.pattern.description = "authored".into();
    h.seed_points = vec![Vector2::new(20., -2.), Vector2::new(20., -2.)];
    h.pixel_size = 0.25;
    let p = prepare_editing_from_cad(&h, CoordinateFrame3::default()).unwrap();
    assert_eq!(
        p.classic_settings.as_ref().unwrap().description.as_deref(),
        Some("authored")
    );
    let mut back = Hatch::solid();
    let losses = apply_editing_to_cad(
        &mut back,
        CoordinateFrame3::default(),
        &HatchFill::Solid,
        p.classic_settings.as_ref(),
        p.boundary_construction.as_ref(),
        true,
    )
    .unwrap();
    assert!(losses.is_empty(), "{losses:?}");
    assert_eq!(back.seed_points, h.seed_points);
    assert_eq!(back.pixel_size, 0.25);
    assert!(back.paths.is_empty());
}
#[test]
fn invalid_inactive_scalars_are_hard_and_partial_presence_is_diagnosed() {
    let mut h = Hatch::solid();
    h.pattern_scale = f64::NAN;
    assert!(prepare_editing_from_cad(&h, CoordinateFrame3::default()).is_err());
    let c = HatchClassicSettings {
        name: Some(String::new()),
        ..Default::default()
    };
    let mut h = Hatch::solid();
    assert!(!apply_editing_to_cad(
        &mut h,
        CoordinateFrame3::default(),
        &HatchFill::Solid,
        Some(&c),
        None,
        true
    )
    .unwrap()
    .is_empty());
    let losses = apply_editing_to_cad(
        &mut h,
        CoordinateFrame3::default(),
        &HatchFill::Solid,
        Some(&c),
        None,
        true,
    )
    .unwrap();
    assert!(losses[0].field.starts_with("classicSettings/"));
}
#[test]
fn user_defined_double_recipe_restores_without_replacing_literal_geometry() {
    let mut h = Hatch::solid();
    h.is_solid = false;
    h.pattern_type = HatchPatternType::UserDefined;
    h.is_double = true;
    h.pattern = HatchPattern::new("U");
    h.pattern.lines.push(HatchPatternLine {
        angle: 0.,
        base_point: Vector2::ZERO,
        offset: Vector2::new(0., 1.),
        dash_lengths: vec![],
    });
    let mut path = BoundaryPath::new();
    path.edges.push(BoundaryEdge::CircularArc(CircularArcEdge {
        center: Vector2::ZERO,
        radius: 3.,
        start_angle: 0.,
        end_angle: std::f64::consts::TAU,
        counter_clockwise: true,
    }));
    h.paths.push(path);
    let p = prepare_hatch_from_cad_with_pattern_context(&h, 1e-9, true).unwrap();
    assert_eq!(
        p.classic_settings
            .as_ref()
            .unwrap()
            .definition_lines
            .as_ref()
            .unwrap()
            .len(),
        1
    );
    let HatchFill::LinePattern(active) = &p.fill else {
        panic!()
    };
    assert_eq!(active.families.len(), 2);
    let target = prepare_hatch_to_cad(
        p.placement,
        &p.boundaries,
        p.area_rule,
        p.join_tolerance,
        &p.fill,
        None,
    )
    .unwrap();
    let alternatives = prepare_hatch_editing_candidates(
        target.clone(),
        p.placement,
        &p.fill,
        p.classic_settings.as_ref(),
        None,
        true,
    )
    .unwrap();
    let restored = alternatives.preferred.unwrap();
    assert!(restored.losses.is_empty());
    assert_eq!(restored.hatch.pattern_type, HatchPatternType::UserDefined);
    assert!(restored.hatch.is_double);
    assert_eq!(restored.hatch.pattern.lines.len(), 1);
    assert_eq!(
        prepare_hatch_from_cad_with_pattern_context(&restored.hatch, 1e-9, true)
            .unwrap()
            .fill,
        p.fill
    );
    let mut stale = p.classic_settings.unwrap();
    stale.scale = 2.;
    let alternatives =
        prepare_hatch_editing_candidates(target, p.placement, &p.fill, Some(&stale), None, true)
            .unwrap();
    assert!(alternatives.preferred.is_none());
    assert!(!alternatives.fallback.losses.is_empty());
}
#[test]
fn finite_direct_dormant_recipe_does_not_need_normalization() {
    let mut h = Hatch::solid();
    h.pattern.name = "Dormant".into();
    h.pattern_scale = f64::from_bits(1);
    h.pattern.lines = vec![HatchPatternLine {
        angle: 0.,
        base_point: Vector2::new(f64::MAX, 0.),
        offset: Vector2::new(0., 1.),
        dash_lengths: vec![],
    }];
    let p = prepare_editing_from_cad(&h, CoordinateFrame3::default()).unwrap();
    assert_eq!(
        p.classic_settings.unwrap().definition_lines.unwrap()[0].base_point[0],
        f64::MAX
    );
    assert!(p.losses.is_empty());
}
#[test]
fn finite_unwritable_native_recipe_keeps_active_solid() {
    let mut h = Hatch::solid();
    let c = HatchClassicSettings {
        preferred_fill: Some(HatchClassicFill::Solid),
        creation_type: Some(HatchCreationType::Custom),
        name: Some("BIG".into()),
        description: Some(String::new()),
        origin: Some([0., 0.]),
        scale: f64::MAX,
        definition_lines: Some(vec![HatchPatternDefinitionLine {
            angle: 0.,
            base_point: [f64::MAX, 0.],
            offset_vector: [0., 1.],
            dash_lengths: vec![],
        }]),
        ..Default::default()
    };
    let losses = apply_editing_to_cad(
        &mut h,
        CoordinateFrame3::try_new(
            ifccad::geometry_kernel::Point3::new(1., 0., 0.),
            ifccad::geometry_kernel::Vector3::new(1., 0., 0.),
            ifccad::geometry_kernel::Vector3::new(0., 1., 0.),
        )
        .unwrap(),
        &HatchFill::Solid,
        Some(&c),
        None,
        true,
    )
    .unwrap();
    assert!(h.is_solid);
    assert_eq!(h.pattern.name, "SOLID");
    assert_eq!(losses.len(), 1);
}
#[test]
fn frames_and_translation_keep_construction_coordinates_explicit() {
    use ifccad::geometry_kernel::{Point3, Vector3};
    let mut h = Hatch::solid();
    let c = HatchBoundaryConstruction {
        seed_points: vec![[2., 3.], [2., 3.]],
        pixel_size: 0.25,
    };
    let placement = CoordinateFrame3::try_new(
        Point3::new(10., 20., 0.),
        Vector3::new(1., 0., 0.),
        Vector3::new(0., 1., 0.),
    )
    .unwrap();
    assert!(
        apply_editing_to_cad(&mut h, placement, &HatchFill::Solid, None, Some(&c), true)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        h.seed_points,
        vec![Vector2::new(12., 23.), Vector2::new(12., 23.)]
    );
    let rotated = CoordinateFrame3::try_new(
        Point3::new(0., 0., 0.),
        Vector3::new(0., 1., 0.),
        Vector3::new(-1., 0., 0.),
    )
    .unwrap();
    let before = h.clone();
    assert_eq!(
        apply_editing_to_cad(&mut h, rotated, &HatchFill::Solid, None, Some(&c), true)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(h, before);
    let extreme = CoordinateFrame3::try_new(
        Point3::new(f64::MAX, 0., 0.),
        Vector3::new(1., 0., 0.),
        Vector3::new(0., 1., 0.),
    )
    .unwrap();
    let seeds = HatchBoundaryConstruction {
        seed_points: vec![[f64::MAX, 0.]],
        pixel_size: 0.,
    };
    assert_eq!(
        apply_editing_to_cad(&mut h, extreme, &HatchFill::Solid, None, Some(&seeds), true)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(h, before);
}
#[test]
fn nonfinite_dormant_origin_is_not_silently_treated_as_absent() {
    use opencadcodec::{xdata::XDataValue, Vector3};
    let mut h = Hatch::solid();
    h.record_pattern_origin(Vector2::ZERO);
    let mut record = h.common.extended_data.get_record("ACAD").unwrap().clone();
    record.values[0] = XDataValue::Point3D(Vector3::new(f64::NAN, 0., 0.));
    h.common.extended_data.upsert_record(record);
    assert!(prepare_editing_from_cad(&h, CoordinateFrame3::default()).is_err());
}
#[test]
fn invalid_dormant_spacing_and_period_are_hard_source_errors() {
    for (offset, dashes) in [
        (Vector2::ZERO, vec![]),
        (Vector2::new(0., 1.), vec![f64::MAX, f64::MAX]),
    ] {
        let mut h = Hatch::solid();
        h.pattern.lines = vec![HatchPatternLine {
            angle: 0.,
            base_point: Vector2::ZERO,
            offset,
            dash_lengths: dashes,
        }];
        assert!(matches!(
            prepare_editing_from_cad(&h, CoordinateFrame3::default()),
            Err(CadHatchPreparationError::Primitive(_)) | Err(CadHatchPreparationError::Editing(_))
        ));
    }
}
#[test]
fn matching_active_recipe_uses_the_prepared_target_phase_after_translation() {
    use ifccad::geometry_kernel::{Point3, Vector3};
    let mut h = Hatch::solid();
    h.is_solid = false;
    h.pattern_type = HatchPatternType::Custom;
    h.pattern = HatchPattern::new("U");
    h.pattern.lines.push(HatchPatternLine {
        angle: 0.,
        base_point: Vector2::ZERO,
        offset: Vector2::new(0., 1.),
        dash_lengths: vec![],
    });
    h.record_pattern_origin(Vector2::ZERO);
    let mut path = BoundaryPath::new();
    path.edges.push(BoundaryEdge::CircularArc(CircularArcEdge {
        center: Vector2::ZERO,
        radius: 3.,
        start_angle: 0.,
        end_angle: std::f64::consts::TAU,
        counter_clockwise: true,
    }));
    h.paths.push(path);
    let p = prepare_hatch_from_cad(&h, 1e-9).unwrap();
    let placement = CoordinateFrame3::try_new(
        Point3::new(10., 20., 0.),
        Vector3::new(1., 0., 0.),
        Vector3::new(0., 1., 0.),
    )
    .unwrap();
    let target = prepare_hatch_to_cad(
        placement,
        &p.boundaries,
        p.area_rule,
        p.join_tolerance,
        &p.fill,
        None,
    )
    .unwrap();
    let alternatives = prepare_hatch_editing_candidates(
        target,
        placement,
        &p.fill,
        p.classic_settings.as_ref(),
        None,
        true,
    )
    .unwrap();
    let restored = alternatives.preferred.unwrap();
    assert!(restored.losses.is_empty());
    assert_eq!(
        restored.hatch.stored_pattern_origin(),
        Some(Vector2::new(10., 20.))
    );
}
#[test]
fn identical_oblique_axes_preserve_local_seed_values_exactly() {
    let mut h = Hatch::solid();
    h.normal = opencadcodec::Vector3::new(1., 0., 1.);
    let basis = opencadcodec::types::Matrix3::arbitrary_axis(h.normal).m;
    use ifccad::geometry_kernel::{Point3, Vector3};
    let frame = CoordinateFrame3::try_new(
        Point3::new(0., 0., 0.),
        Vector3::new(basis[0][0], basis[1][0], basis[2][0]),
        Vector3::new(basis[0][1], basis[1][1], basis[2][1]),
    )
    .unwrap();
    let c = HatchBoundaryConstruction {
        seed_points: vec![[2., -3.]],
        pixel_size: 0.25,
    };
    assert!(
        apply_editing_to_cad(&mut h, frame, &HatchFill::Solid, None, Some(&c), true)
            .unwrap()
            .is_empty()
    );
    assert_eq!(h.seed_points, vec![Vector2::new(2., -3.)]);
}
#[test]
fn invalid_source_frames_are_errors_even_for_standalone_editing_helpers() {
    for elevation in [false, true] {
        let mut h = Hatch::solid();
        if elevation {
            h.elevation = f64::NAN
        } else {
            h.normal.z = f64::NAN
        }
        assert!(prepare_editing_from_cad(&h, CoordinateFrame3::default()).is_err());
        assert!(apply_editing_to_cad(
            &mut h,
            CoordinateFrame3::default(),
            &HatchFill::Solid,
            None,
            None,
            true
        )
        .is_err());
    }
}
#[test]
fn invalid_active_fill_is_rejected_before_recipe_phase_projection() {
    let family = HatchLineFamily {
        angle: 0.,
        base_point: [0., 0.],
        offset: [0., 1.],
        dashes: vec![],
    };
    let c = HatchClassicSettings {
        preferred_fill: Some(HatchClassicFill::LinePattern),
        creation_type: Some(HatchCreationType::Custom),
        name: Some("U".into()),
        description: Some(String::new()),
        origin: Some([0., 0.]),
        definition_lines: Some(vec![HatchPatternDefinitionLine {
            angle: family.angle,
            base_point: family.base_point,
            offset_vector: family.offset,
            dash_lengths: vec![],
        }]),
        ..Default::default()
    };
    let bad = HatchFill::LinePattern(HatchLinePattern {
        name: Some("U".into()),
        description: Some(String::new()),
        origin: [f64::NAN, 0.],
        rotation: 0.,
        scale: 1.,
        families: vec![family],
    });
    assert!(apply_editing_to_cad(
        &mut Hatch::solid(),
        CoordinateFrame3::default(),
        &bad,
        Some(&c),
        None,
        true
    )
    .is_err());
}
