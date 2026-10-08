use cad_text::*;
use ocdraw::text::*;
use opencadcodec::{entities::LineSpacingStyle, MText};

#[test]
fn explicit_static_and_dynamic_column_states_are_not_recomputed() {
    let mut source = MText::new();
    source.value = "A\\NB".into();
    source.rectangle_width = 85.;
    source.rectangle_height = Some(60.);
    source.column_data.column_type = 1;
    source.column_data.column_count = 2;
    source.column_data.width = 40.;
    source.column_data.gutter = 5.;
    let parsed = prepare_mtext_from_cad(&source).unwrap();
    assert_eq!(parsed.wrap_width, None);
    assert_eq!(
        parsed.columns,
        Some(MTextColumns::Static {
            count: 2,
            column_width: 40.,
            gutter: 5.,
            column_height: 60.,
            flow_reversed: false
        })
    );
    source.column_data.column_type = 2;
    source.column_data.auto_height = true;
    assert!(matches!(
        prepare_mtext_from_cad(&source).unwrap().columns,
        Some(MTextColumns::DynamicAutoHeight {
            current_column_count: 2,
            column_height: 60.,
            ..
        })
    ));
    source.column_data.auto_height = false;
    source.column_data.heights = vec![60., 45.];
    assert!(
        matches!(prepare_mtext_from_cad(&source).unwrap().columns,Some(MTextColumns::DynamicManualHeight{column_heights,..})if column_heights == vec![MTextColumnHeight::Fixed { distance: 60. }, MTextColumnHeight::Auto])
    );
    source.column_data.heights = vec![60., 0.];
    assert!(matches!(
        prepare_mtext_from_cad(&source).unwrap().columns,
        Some(MTextColumns::DynamicManualHeight { column_heights, .. }) if column_heights.last() == Some(&MTextColumnHeight::Auto)
    ));
    source.column_data.heights = vec![60.];
    assert!(matches!(
        prepare_mtext_from_cad(&source),
        Err(CadTextError::InvalidSource(_))
    ));
    source.column_data.column_count = 0;
    source.column_data.auto_height = true;
    source.column_data.heights.clear();
    source.extents_width = 999.;
    source.extents_height = 999.;
    assert!(
        matches!(
            prepare_mtext_from_cad(&source),
            Err(CadTextError::Unsupported(_))
        ),
        "cached extents cannot recover actual column count"
    );
}

#[test]
fn spacing_reference_changes_and_inactive_state_have_explicit_evidence() {
    let mut source = MText::new();
    source.value = "A".into();
    source.height = 2.5;
    assert!(prepare_mtext_from_cad(&source).unwrap().issues.is_empty());
    source.line_spacing_style = LineSpacingStyle::Exactly;
    source.line_spacing_factor = 1.2;
    let parsed = prepare_mtext_from_cad(&source).unwrap();
    assert!(
        matches!(parsed.content.paragraph_format.line_spacing,Some(TextLineSpacing::Exact{distance})if(distance-5.).abs()<1e-14)
    );
    assert!(parsed
        .issues
        .contains(&CadTextIssue::LineSpacingReferenceChanged));
    source.column_data.width = 40.;
    assert!(prepare_mtext_from_cad(&source)
        .unwrap()
        .issues
        .contains(&CadTextIssue::InactiveColumnPropertiesOmitted));
}

#[test]
fn frame_only_canvas_and_transparency_restrictions_remain_distinct() {
    let mut source = MText::new();
    source.value = "A".into();
    source.background_fill_flags = 0x10;
    let parsed = prepare_mtext_from_cad(&source).unwrap();
    let bg = parsed.background.unwrap();
    assert_eq!(bg.fill, MTextFill::None);
    assert!(bg.frame);
    assert_eq!(bg.padding, TextPadding::Relative { factor: 0.5 });
    source.background_fill_flags = 3;
    assert_eq!(
        prepare_mtext_from_cad(&source)
            .unwrap()
            .background
            .unwrap()
            .fill,
        MTextFill::Canvas
    );
    source.background_transparency = 0x02000080;
    assert!(matches!(
        prepare_mtext_from_cad(&source),
        Err(CadTextError::Unsupported(_))
    ));
    source.background_transparency = 0;
    source.background_fill_flags = 0x20;
    assert!(matches!(
        prepare_mtext_from_cad(&source),
        Err(CadTextError::Unsupported(_))
    ));
}
