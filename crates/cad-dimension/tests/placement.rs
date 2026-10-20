use cad_dimension::*;
use ifccad::dimension::*;
use opencadcodec::{
    entities::{Dimension, DimensionAligned, DimensionLinear},
    CadDocument, Vector3,
};
#[test]
fn projected_linear_measurement_does_not_use_stale_cad_cache() {
    let mut d = DimensionLinear::new(Vector3::new(10.0, 20.0, 0.0), Vector3::new(13.0, 24.0, 0.0));
    d.definition_point = Vector3::new(10.0, 30.0, 0.0);
    d.base.definition_point = d.definition_point;
    d.base.actual_measurement = 999.0;
    let p = prepare_dimension_from_cad(&Dimension::Linear(d), &CadDocument::new()).unwrap();
    assert_eq!(
        evaluate_dimension(&p.definition).unwrap(),
        DimensionMeasurement::Distance(3.0)
    );
    assert!(!p.pairs.is_empty());
}
#[test]
fn tilted_plane_keeps_wcs_definition_and_ocs_text_anchor() {
    let mut d = DimensionAligned::new(Vector3::new(3.0, 0.0, 0.0), Vector3::new(3.0, 4.0, 5.0));
    d.base.normal = Vector3::new(1.0, 0.0, 0.0);
    d.definition_point = Vector3::new(3.0, 8.0, 0.0);
    d.base.definition_point = d.definition_point;
    d.base.text_user_positioned = true;
    d.base.text_middle_point = Vector3::new(2.0, 7.0, 3.0);
    let p = prepare_dimension_from_cad(&Dimension::Aligned(d), &CadDocument::new()).unwrap();
    assert!(matches!(
        p.text_placement,
        DimensionTextPlacement::Explicit { .. }
    ));
    assert_eq!(p.placement.origin().components(), [3.0, 8.0, 0.0]);
    let DimensionTextPlacement::Explicit { position, .. } = p.text_placement else {
        unreachable!()
    };
    assert_eq!(position, [-6.0, 7.0]);
}

#[test]
fn automatic_text_keeps_style_vertical_placement() {
    let doc = CadDocument::new();
    let d = Dimension::Aligned(DimensionAligned::new(
        Vector3::ZERO,
        Vector3::new(5.0, 0.0, 0.0),
    ));
    let p = prepare_dimension_from_cad(&d, &doc).unwrap();
    assert_eq!(p.overrides.text.attachment, None);
    let mut effective = prepare_style_from_cad(doc.dim_styles.get("Standard").unwrap(), &doc)
        .unwrap()
        .values;
    p.overrides.apply(&mut effective);
    assert_eq!(
        effective.text.attachment,
        ifccad::text::MTextAttachment::BottomCenter
    );
}

#[test]
fn linear_axis_variants_use_pure_projected_measurement() {
    let doc = CadDocument::new();
    for (angle, expected) in [(0.0, 3.0), (std::f64::consts::FRAC_PI_2, 4.0)] {
        let d = Dimension::Linear(DimensionLinear::rotated(
            Vector3::ZERO,
            Vector3::new(3.0, 4.0, 0.0),
            angle,
        ));
        let p = prepare_dimension_from_cad(&d, &doc).unwrap();
        let DimensionMeasurement::Distance(n) = evaluate_dimension(&p.definition).unwrap() else {
            unreachable!()
        };
        assert!((n - expected).abs() < 1e-14);
    }
    let d = Dimension::Linear(DimensionLinear::rotated(
        Vector3::ZERO,
        Vector3::new(3.0, 4.0, 0.0),
        0.7,
    ));
    let p = prepare_dimension_from_cad(&d, &doc).unwrap();
    assert!(matches!(
        p.definition,
        DimensionDefinition::Linear {
            direction_angle: 0.7,
            ..
        }
    ));
    assert!(p.pairs.iter().all(|p| p
        .points
        .iter()
        .all(|(_, d)| d == &num_rational::BigRational::from_integer(0.into()))));
}

#[test]
fn dxf_oblique_offset_becomes_absolute_native_extension_direction() {
    let mut d = DimensionLinear::rotated(Vector3::ZERO, Vector3::new(3.0, 4.0, 0.0), 0.2);
    d.ext_line_rotation = 0.6;
    let p = prepare_dimension_from_cad(&Dimension::Linear(d), &CadDocument::new()).unwrap();
    let DimensionDefinition::Linear {
        extension_direction_angle: Some(angle),
        ..
    } = p.definition
    else {
        panic!("extension direction")
    };
    assert_eq!(angle, 0.8);
    let target =
        prepare_dimension_to_cad(&p.definition, p.placement, &p.text_placement, &p.label).unwrap();
    let Dimension::Linear(d) = target.entity else {
        panic!("linear")
    };
    assert_eq!(d.rotation + d.ext_line_rotation, angle);
}

#[test]
fn legacy_control_codes_are_not_misclassified_as_plain_literal_labels() {
    let mut d = DimensionAligned::new(Vector3::ZERO, Vector3::new(3.0, 4.0, 0.0));
    d.base.set_text_override(Some("<> %%d".into()));
    assert!(
        matches!(prepare_dimension_from_cad(&Dimension::Aligned(d),&CadDocument::new()),Err(CadDimensionPreparationError::Unsupported{field,..}) if field=="text")
    );
    let label = DimensionLabel::Template(vec![DimensionLabelPart::Literal(vec![
        ifccad::text::TextRun {
            text: "Literal %%d {group}".into(),
            ..Default::default()
        },
    ])]);
    assert!(prepare_dimension_to_cad(
        &DimensionDefinition::Aligned {
            first_point: [0.0; 3],
            second_point: [3.0, 4.0, 0.0],
            dimension_line_point: [0.0, 8.0],
            extension_direction_angle: None
        },
        ifccad::geometry_kernel::CoordinateFrame3::default(),
        &DimensionTextPlacement::StyleDriven,
        &label
    )
    .is_err());
}
