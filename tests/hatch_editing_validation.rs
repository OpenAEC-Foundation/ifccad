use ifccad::geometry_kernel::hatch::*;

#[test]
fn direct_definition_values_have_their_own_intrinsic_rules() {
    let line = HatchPatternDefinitionLine {
        angle: 40.,
        base_point: [0., -0.],
        offset_vector: [2., 0.],
        dash_lengths: vec![2., -1., 0., -0.],
    };
    validate_hatch_pattern_definition_line(&line).unwrap();
    let mut c = HatchClassicSettings {
        definition_lines: Some(vec![line.clone(), line.clone()]),
        ..Default::default()
    };
    validate_hatch_classic_settings(&c).unwrap();
    c.definition_lines = Some(vec![]);
    validate_hatch_classic_settings(&c).unwrap();
    for bad in [
        HatchPatternDefinitionLine {
            angle: f64::NAN,
            ..line.clone()
        },
        HatchPatternDefinitionLine {
            base_point: [f64::INFINITY, 0.],
            ..line.clone()
        },
        HatchPatternDefinitionLine {
            offset_vector: [0., -0.],
            ..line.clone()
        },
        HatchPatternDefinitionLine {
            offset_vector: [f64::INFINITY, 1.],
            ..line.clone()
        },
        HatchPatternDefinitionLine {
            dash_lengths: vec![0., -0.],
            ..line.clone()
        },
        HatchPatternDefinitionLine {
            dash_lengths: vec![f64::MAX, -f64::MAX],
            ..line.clone()
        },
        HatchPatternDefinitionLine {
            dash_lengths: vec![f64::NAN],
            ..line.clone()
        },
    ] {
        assert!(validate_hatch_pattern_definition_line(&bad).is_err());
    }
    validate_hatch_pattern_definition_line(&HatchPatternDefinitionLine {
        dash_lengths: vec![],
        ..line
    })
    .unwrap();
}

#[test]
fn partial_classic_presence_and_construction_are_independent_of_fill() {
    let default = HatchClassicSettings::default();
    assert!(default.is_empty());
    assert!(validate_hatch_classic_settings(&default).is_err());
    for c in [
        HatchClassicSettings {
            preferred_fill: Some(HatchClassicFill::Solid),
            ..default.clone()
        },
        HatchClassicSettings {
            creation_type: Some(HatchCreationType::UserDefined),
            ..default.clone()
        },
        HatchClassicSettings {
            name: Some(String::new()),
            ..default.clone()
        },
        HatchClassicSettings {
            description: Some(String::new()),
            ..default.clone()
        },
        HatchClassicSettings {
            origin: Some([0., 0.]),
            ..default.clone()
        },
        HatchClassicSettings {
            definition_lines: Some(vec![]),
            ..default.clone()
        },
        HatchClassicSettings {
            rotation: 8. * std::f64::consts::PI,
            ..default.clone()
        },
    ] {
        assert!(!c.is_empty());
        validate_hatch_classic_settings(&c).unwrap();
    }
    for scale in [0., -1., f64::INFINITY, f64::NAN] {
        assert!(validate_hatch_classic_settings(&HatchClassicSettings {
            scale,
            ..default.clone()
        })
        .is_err());
    }
    assert!(validate_hatch_boundary_construction(&HatchBoundaryConstruction::default()).is_err());
    let mut construction = HatchBoundaryConstruction {
        seed_points: vec![[1e10, -2.], [1e10, -2.]],
        pixel_size: 0.,
    };
    validate_hatch_boundary_construction(&construction).unwrap();
    construction.pixel_size = -1.;
    assert!(validate_hatch_boundary_construction(&construction).is_err());
    construction.pixel_size = f64::INFINITY;
    assert!(validate_hatch_boundary_construction(&construction).is_err());
}

#[test]
fn invalid_dormant_families_are_rejected() {
    let family = HatchPatternDefinitionLine {
        angle: 0.,
        base_point: [0., 0.],
        offset_vector: [0., 1.],
        dash_lengths: vec![0.],
    };
    let c = HatchClassicSettings {
        definition_lines: Some(vec![family]),
        ..Default::default()
    };
    assert!(validate_hatch_classic_settings(&c).is_err());
    let c = HatchClassicSettings {
        origin: Some([f64::NAN, 0.]),
        ..Default::default()
    };
    assert!(validate_hatch_classic_settings(&c).is_err());
}
