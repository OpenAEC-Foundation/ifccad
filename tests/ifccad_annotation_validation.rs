use ifccad::*;

fn scale(id: u64, name: &str) -> IfccadAnnotationScale {
    IfccadAnnotationScale {
        id: IfccadAnnotationScaleId(id),
        name: name.into(),
        paper_units: 1.0,
        drawing_units: 50.0,
        unit_scale: false,
    }
}
fn key(id: u64) -> IfccadAnnotationContextKey {
    IfccadAnnotationContextKey {
        scale_id: IfccadAnnotationScaleId(id),
    }
}

#[test]
fn equal_ratios_remain_distinct_but_duplicate_id_or_folded_name_is_invalid() {
    let source = vec![scale(1, "Straße"), scale(2, "Other")];
    assert!(validate_ifccad_annotation_scale_registry(&source, 3).is_ok());
    let mut duplicate = source.clone();
    duplicate[1].id = duplicate[0].id;
    assert!(validate_ifccad_annotation_scale_registry(&duplicate, 3).is_err());
    let mut duplicate = source.clone();
    duplicate[1].name = "STRASSE".into();
    assert!(validate_ifccad_annotation_scale_registry(&duplicate, 3).is_err());
    assert!(validate_ifccad_annotation_scale_registry(&source, 2).is_err());
    assert!(validate_ifccad_annotation_scale_registry(&[], 0).is_err());
}

#[test]
fn registry_validity_does_not_require_an_individual_float_quotient() {
    let mut source = scale(1, "Extreme");
    source.paper_units = f64::from_bits(1);
    source.drawing_units = f64::MAX;
    assert!(validate_ifccad_annotation_scale_registry(&[source.clone()], 2).is_ok());
    source.paper_units = 0.0;
    assert!(validate_ifccad_annotation_scale_registry(&[source], 2).is_err());
}

#[test]
fn dormant_memberships_keep_their_values_but_internal_links_must_resolve() {
    let scales = vec![scale(1, "First"), scale(2, "Second")];
    let mut annotation = IfccadAnnotation {
        orientation: None,
        mode: IfccadAnnotationMode::Disabled,
        reference: Some(key(2)),
        contexts: vec![
            IfccadAnnotationContext {
                key: key(1),
                presentation: Some(7),
            },
            IfccadAnnotationContext {
                key: key(2),
                presentation: None,
            },
        ],
    };
    assert!(annotation.validate_memberships(&scales, false).is_ok());
    annotation.reference = Some(key(3));
    assert!(annotation.validate_memberships(&scales, false).is_err());
    annotation.reference = None;
    annotation.contexts.push(annotation.contexts[0].clone());
    assert!(annotation.validate_memberships(&scales, false).is_err());
    annotation.contexts.clear();
    annotation.mode = IfccadAnnotationMode::ByDefinition;
    assert!(annotation.validate_memberships(&scales, false).is_err());
    assert!(annotation.validate_memberships(&scales, true).is_ok());
    annotation.mode = IfccadAnnotationMode::Enabled;
    assert!(annotation.validate_memberships(&scales, false).is_ok());
}
