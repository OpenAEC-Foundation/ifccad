use ifccad::{
    select_ifccad_annotation_context, IfccadAnnotation, IfccadAnnotationContext,
    IfccadAnnotationContextKey, IfccadAnnotationMode, IfccadAnnotationScaleId,
    IfccadAnnotationSelection,
};

fn key(id: u64) -> IfccadAnnotationContextKey {
    IfccadAnnotationContextKey {
        scale_id: IfccadAnnotationScaleId(id),
    }
}

#[test]
fn selected_snapshot_is_independent_of_default_and_never_generated() {
    let annotation = IfccadAnnotation {
        orientation: None,
        mode: IfccadAnnotationMode::Enabled,
        reference: Some(key(1)),
        contexts: vec![
            IfccadAnnotationContext {
                key: key(1),
                presentation: Some(12u32),
            },
            IfccadAnnotationContext {
                key: key(2),
                presentation: Some(37),
            },
            IfccadAnnotationContext {
                key: key(3),
                presentation: None,
            },
        ],
    };
    let selection = select_ifccad_annotation_context(Some(&annotation), true, Some(&key(2)));
    assert!(
        matches!(selection, IfccadAnnotationSelection::Available(context) if context.presentation == Some(37))
    );
    assert!(matches!(
        select_ifccad_annotation_context(Some(&annotation), true, Some(&key(3))),
        IfccadAnnotationSelection::MembershipOnly(selected) if selected == &key(3)
    ));
    assert!(matches!(
        select_ifccad_annotation_context(Some(&annotation), true, Some(&key(4))),
        IfccadAnnotationSelection::ScaleNotAssociated
    ));
    assert_eq!(annotation.reference, Some(key(1)));
    assert_eq!(annotation.contexts[0].presentation, Some(12));
}

#[test]
fn disabled_unspecified_and_empty_states_are_distinct_and_ordered() {
    let empty = IfccadAnnotation::<u32> {
        orientation: None,
        mode: IfccadAnnotationMode::Enabled,
        reference: None,
        contexts: vec![],
    };
    assert!(matches!(
        select_ifccad_annotation_context(Some(&empty), false, None),
        IfccadAnnotationSelection::Disabled
    ));
    assert!(matches!(
        select_ifccad_annotation_context(Some(&empty), true, None),
        IfccadAnnotationSelection::SelectionUnspecified
    ));
    assert!(matches!(
        select_ifccad_annotation_context(Some(&empty), true, Some(&key(1))),
        IfccadAnnotationSelection::NoStoredContexts
    ));
    assert!(matches!(
        select_ifccad_annotation_context::<u32>(None, true, Some(&key(1))),
        IfccadAnnotationSelection::NoStoredContexts
    ));
}

#[test]
fn scale_identity_does_not_round_through_javascript_numbers() {
    let exact_id = (1u64 << 53) + 1;
    let annotation = IfccadAnnotation {
        orientation: None,
        mode: IfccadAnnotationMode::ByDefinition,
        reference: None,
        contexts: vec![IfccadAnnotationContext {
            key: key(exact_id),
            presentation: Some(11),
        }],
    };
    assert!(matches!(
        select_ifccad_annotation_context(Some(&annotation), true, Some(&key(exact_id - 1))),
        IfccadAnnotationSelection::ScaleNotAssociated
    ));
    assert!(matches!(
        select_ifccad_annotation_context(Some(&annotation), true, Some(&key(exact_id))),
        IfccadAnnotationSelection::Available(_)
    ));
}
