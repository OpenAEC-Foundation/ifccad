use cad_dimension::*;
use opencadcodec::xdata::{ExtendedDataRecord, XDataValue as X};

fn record(values: Vec<X>) -> ExtendedDataRecord {
    ExtendedDataRecord {
        application_name: "ACAD".into(),
        values,
    }
}
#[test]
fn dstyle_keeps_equal_zero_false_and_empty_overrides() {
    let records = vec![record(vec![
        X::String("DSTYLE".into()),
        X::ControlString("{".into()),
        X::Integer16(140),
        X::Real(2.5),
        X::Integer16(281),
        X::Integer16(0),
        X::Integer16(147),
        X::Real(0.0),
        X::Integer16(3),
        X::String(String::new()),
        X::ControlString("}".into()),
    ])];
    let parsed = parse_dimension_overrides(&records).unwrap();
    assert_eq!(parsed.values.text.height, Some(2.5));
    assert_eq!(parsed.values.dimension_line.suppress_first, Some(false));
    assert_eq!(parsed.values.text.gap, Some(0.0));
    assert_eq!(parsed.values.numbers.primary.suffix.as_deref(), Some(""));
}
#[test]
fn malformed_or_duplicate_dstyle_is_not_a_default_style() {
    for values in [
        vec![
            X::String("DSTYLE".into()),
            X::ControlString("{".into()),
            X::Integer16(140),
            X::Integer16(2),
        ],
        vec![
            X::String("DSTYLE".into()),
            X::ControlString("{".into()),
            X::Integer16(140),
            X::Real(2.5),
            X::Integer16(140),
            X::Real(3.5),
            X::ControlString("}".into()),
        ],
    ] {
        assert!(parse_dimension_overrides(&[record(values)]).is_err());
    }
}
#[test]
fn unrelated_acad_values_are_retained_as_residual_evidence() {
    let r = record(vec![X::String("OTHER".into()), X::Integer16(3)]);
    let parsed = parse_dimension_overrides(std::slice::from_ref(&r)).unwrap();
    assert_eq!(parsed.residual, vec![r]);
}

#[test]
fn writer_rejects_unmapped_override_instead_of_omitting_it() {
    let base: CadDimensionStyleValues =
        ocdraw::dimension::DimensionStyleSettings::new(opencadcodec::Handle::new(5));
    let mut over: CadDimensionStyleOverrides = Default::default();
    over.arrows.flip_first = Some(true);
    assert!(encode_dimension_overrides(&over, &base).is_err());
}

#[test]
fn duplicate_subsections_and_unknown_dimvars_are_distinct_failures() {
    let subsection = record(vec![
        X::String("DSTYLE".into()),
        X::ControlString("{".into()),
        X::ControlString("}".into()),
    ]);
    assert!(matches!(
        parse_dimension_overrides(&[subsection.clone(), subsection]),
        Err(CadDimensionPreparationError::Invalid { .. })
    ));
    let unknown = record(vec![
        X::String("DSTYLE".into()),
        X::ControlString("{".into()),
        X::Integer16(999),
        X::Real(1.0),
        X::ControlString("}".into()),
    ]);
    assert!(matches!(
        parse_dimension_overrides(&[unknown]),
        Err(CadDimensionPreparationError::Unsupported { .. })
    ));
}

#[test]
fn per_entity_fields_and_dstyle_reset_are_materialized_separately() {
    use ocdraw::dimension::*;
    let base: CadDimensionStyleValues = DimensionStyleSettings::new(opencadcodec::Handle::new(5));
    let mut over: CadDimensionStyleOverrides = Default::default();
    over.arrows.flip_first = Some(false);
    over.arrows.flip_second = Some(false);
    over.text.line_spacing = Some(DimensionLineSpacing::Exact { factor: 1.2 });
    over.dimension_line.suppress_first = Some(false);
    let mut entity = opencadcodec::entities::Dimension::Aligned(Default::default());
    apply_entity_overrides_to_cad(&mut entity, &over, &base).unwrap();
    assert_eq!(entity.base().line_spacing_style, 2);
    assert_eq!(entity.base().line_spacing_factor, 1.2);
    let parsed = parse_dimension_overrides(entity.base().common.extended_data.records()).unwrap();
    assert_eq!(parsed.values.dimension_line.suppress_first, Some(false));
    assert_eq!(parsed.values.arrows.flip_first, None);
}
