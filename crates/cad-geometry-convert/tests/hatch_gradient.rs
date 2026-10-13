use cad_geometry_convert::hatch::*;
use ocdraw::geometry_kernel::{hatch::*, CoordinateFrame3, Point3, Vector3 as NativeVector};
use opencadcodec::{entities::hatch::*, Color, Vector2};

fn hatch() -> Hatch {
    let mut h = Hatch::solid();
    let mut path = BoundaryPath::new();
    path.edges.push(BoundaryEdge::CircularArc(CircularArcEdge {
        center: Vector2::new(2., 3.),
        radius: 4.,
        start_angle: 0.,
        end_angle: std::f64::consts::TAU,
        counter_clockwise: true,
    }));
    h.paths.push(path);
    h.gradient_color.enabled = true;
    h.gradient_color.name = "INVSPHERICAL".into();
    h.gradient_color.angle = 0.75;
    h.gradient_color.shift = 0.5;
    h.gradient_color.is_single_color = true;
    h.gradient_color.color_tint = 0.8;
    h.gradient_color.colors = vec![
        GradientColorEntry {
            value: 0.,
            color: Color::Rgb {
                r: 220,
                g: 80,
                b: 30,
            },
        },
        GradientColorEntry {
            value: 1.,
            color: Color::Rgb {
                r: 30,
                g: 60,
                b: 150,
            },
        },
    ];
    h
}

#[test]
fn symbolic_aci_seven_is_unqualified_without_a_concrete_endpoint() {
    for active in [true, false] {
        let mut h = hatch();
        h.gradient_color.enabled = active;
        h.gradient_color.colors[0].color = Color::Index(7);
        let result = prepare_hatch_from_cad(&h, 1e-9);
        if active {
            assert!(matches!(
                result,
                Err(CadHatchPreparationError::Unsupported { .. })
            ));
        } else {
            let result = result.unwrap();
            assert!(result.gradient.is_none());
            assert!(result
                .losses
                .iter()
                .any(|loss| loss.field == "gradient.colors"));
        }
    }
}

#[test]
fn all_standard_types_keep_stored_endpoints_and_authorship_settings() {
    for (name, kind) in [
        ("LINEAR", HatchGradientType::Linear),
        ("CYLINDER", HatchGradientType::Cylinder),
        ("INVCYLINDER", HatchGradientType::InverseCylinder),
        ("SPHERICAL", HatchGradientType::Spherical),
        ("INVSPHERICAL", HatchGradientType::InverseSpherical),
        ("HEMISPHERICAL", HatchGradientType::Hemispherical),
        ("INVHEMISPHERICAL", HatchGradientType::InverseHemispherical),
        ("CURVED", HatchGradientType::Curved),
        ("INVCURVED", HatchGradientType::InverseCurved),
    ] {
        let mut h = hatch();
        h.gradient_color.name = name.into();
        let copy = h.clone();
        let p = prepare_hatch_from_cad(&h, 1e-9).unwrap();
        assert_eq!(p.fill, HatchFill::Gradient);
        let g = p.gradient.as_ref().unwrap();
        assert_eq!(g.gradient_type, Some(kind));
        assert_eq!((g.rotation, g.shift, g.tint), (0.75, 0.5, 0.8));
        assert_eq!(g.color_mode, HatchGradientColorMode::OneColor);
        assert_eq!(
            g.colors.unwrap()[1],
            Color::Rgb {
                r: 30,
                g: 60,
                b: 150
            }
        );
        let out = prepare_hatch_to_cad(
            p.placement,
            &p.boundaries,
            p.area_rule,
            p.join_tolerance,
            &p.fill,
            p.gradient.as_ref(),
        )
        .unwrap();
        assert_eq!(out.hatch.gradient_color, h.gradient_color);
        assert_eq!(h, copy);
    }
}

#[test]
fn enabled_gradient_bypasses_inactive_user_defined_double_gates() {
    let mut h = hatch();
    h.is_solid = false;
    h.pattern_type = HatchPatternType::UserDefined;
    h.is_double = true;
    let p = prepare_hatch_from_cad_with_pattern_context(&h, 1e-9, false).unwrap();
    assert_eq!(p.fill, HatchFill::Gradient);
    assert!(p.losses.iter().any(|l| l.field == "pattern_context"));
}

#[test]
fn inactive_pattern_origin_is_left_for_common_loss_auditing_on_gradients() {
    let mut h = hatch();
    h.is_solid = false;
    h.record_pattern_origin(Vector2::new(2., 3.));
    let residual = pattern::residual_common(&h).unwrap();
    assert_eq!(residual.extended_data, h.common.extended_data);
}

#[test]
fn disabled_partial_settings_are_not_completed_and_defaults_are_absent() {
    let mut h = Hatch::solid();
    assert!(prepare_gradient_from_cad(&h).unwrap().gradient.is_none());
    h.gradient_color.angle = 0.5;
    h.gradient_color.color_tint = 0.65;
    h.gradient_color.is_single_color = true;
    let p = prepare_gradient_from_cad(&h).unwrap();
    let g = p.gradient.unwrap();
    assert_eq!(g.gradient_type, None);
    assert_eq!(g.colors, None);
    apply_gradient_to_cad(&mut h, Default::default(), &HatchFill::Solid, Some(&g)).unwrap();
    assert!(!h.gradient_color.enabled);
    assert!(h.gradient_color.colors.is_empty());
    assert_eq!(h.gradient_color.name, "");
    assert_eq!(h.gradient_color.color_tint, 0.65);
}

#[test]
fn malformed_values_are_hard_but_unqualified_profiles_are_located() {
    for enabled in [true, false] {
        let mut h = hatch();
        h.gradient_color.enabled = enabled;
        h.gradient_color.shift = f64::NAN;
        assert!(matches!(
            prepare_gradient_from_cad(&h),
            Err(CadHatchPreparationError::Gradient(_))
        ));
        let mut h = hatch();
        h.gradient_color.enabled = enabled;
        h.gradient_color.colors[0].value = f64::NAN;
        assert!(matches!(
            prepare_gradient_from_cad(&h),
            Err(CadHatchPreparationError::InvalidGradientSource { .. })
        ));
        let mut h = hatch();
        h.gradient_color.enabled = enabled;
        h.gradient_color.name = "FUTURE".into();
        if enabled {
            assert!(matches!(
                prepare_gradient_from_cad(&h),
                Err(CadHatchPreparationError::Unsupported { .. })
            ));
        } else {
            let p = prepare_gradient_from_cad(&h).unwrap();
            assert!(p.gradient.is_none());
            assert_eq!(p.losses.len(), 1);
        }
    }
    let mut h = hatch();
    h.gradient_color.colors.pop();
    assert!(matches!(
        prepare_gradient_from_cad(&h),
        Err(CadHatchPreparationError::InvalidGradientSource { .. })
    ));
}

#[test]
fn rotated_basis_never_gets_an_automatic_gradient_angle_repair() {
    let mut h = hatch();
    let g = prepare_gradient_from_cad(&h).unwrap().gradient.unwrap();
    let angle = 0.5_f64;
    let frame = CoordinateFrame3::try_new(
        Point3::new(0., 0., 0.),
        NativeVector::new(angle.cos(), angle.sin(), 0.),
        NativeVector::new(-angle.sin(), angle.cos(), 0.),
    )
    .unwrap();
    assert!(matches!(
        apply_gradient_to_cad(&mut h, frame, &HatchFill::Gradient, Some(&g)),
        Err(CadHatchPreparationError::Unsupported {
            field: "gradient.frame",
            ..
        })
    ));
    let losses = apply_gradient_to_cad(&mut h, frame, &HatchFill::Solid, Some(&g)).unwrap();
    assert_eq!(losses[0].field, "gradient.frame");
    assert!(!h.gradient_color.enabled);
    assert!(h.gradient_color.colors.is_empty());
}
