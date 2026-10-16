use cad_dimension::*;
use ocdraw::dimension::*;
use opencadcodec::{
    entities::{Dimension, DimensionAligned},
    xdata::{ExtendedDataRecord, XDataValue as X},
    CadDocument, Handle, Vector3,
};

fn dimension(pairs: &[(i16, X)]) -> Dimension {
    let mut d = DimensionAligned::new(Vector3::ZERO, Vector3::new(48.25, 0.0, 0.0));
    d.definition_point = Vector3::new(0.0, 10.0, 0.0);
    d.base.definition_point = d.definition_point;
    let mut values = vec![X::String("DSTYLE".into()), X::ControlString("{".into())];
    for (code, value) in pairs {
        values.push(X::Integer16(*code));
        values.push(value.clone());
    }
    values.push(X::ControlString("}".into()));
    d.base.common.extended_data.add_record(ExtendedDataRecord {
        application_name: "ACAD".into(),
        values,
    });
    Dimension::Aligned(d)
}

#[test]
fn dormant_text_offset_and_subunit_defaults_do_not_reject_style() {
    let doc = CadDocument::new();
    let mut s = doc.dim_styles.get("Standard").unwrap().clone();
    s.dimtad = 1;
    s.dimtvp = 1.0;
    s.dimzin = 8;
    s.dimmzf = 100.0;
    assert!(prepare_style_from_cad(&s, &doc).is_ok());
    s.dimtad = 0;
    assert!(
        matches!(prepare_style_from_cad(&s,&doc),Err(CadDimensionPreparationError::Unsupported {field,..}) if field=="style/textPlacement")
    );
    s.dimtvp = 0.0;
    s.dimzin = 12;
    s.dimmzs = "cm".into();
    assert!(
        matches!(prepare_style_from_cad(&s,&doc),Err(CadDimensionPreparationError::Unsupported {field,..}) if field=="DIMMZF/DIMMZS")
    );
}

#[test]
fn fixed_text_height_materializes_without_mutating_shared_style() {
    let mut doc = CadDocument::new();
    doc.text_styles.get_mut("Standard").unwrap().height = 50.0;
    let mut s = doc.dim_styles.get("Standard").unwrap().clone();
    s.dimtxt = 2.5;
    s.dimscale = 2.0;
    let prepared = prepare_style_from_cad(&s, &doc).unwrap();
    assert_eq!(prepared.values.text.height, 50.0);
    assert_eq!(prepared.values.presentation_scale, 2.0);
    assert_eq!(doc.text_styles.get("Standard").unwrap().height, 50.0);
    assert!(prepared
        .losses
        .iter()
        .any(|l| l.field == "DIMTXSTY/fixedHeight"));
}

#[test]
fn complete_red_dimension_profile_and_fixed_extensions_are_order_independent() {
    let doc = CadDocument::new();
    let mut pairs = vec![
        (276, X::Integer16(1)),
        (78, X::Integer16(0)),
        (277, X::Integer16(4)),
        (44, X::Real(1.0)),
        (46, X::Real(0.0)),
        (176, X::Integer16(1)),
        (343, X::Handle(Handle::NULL)),
        (173, X::Integer16(1)),
        (344, X::Handle(Handle::NULL)),
        (49, X::Real(20.0)),
        (290, X::Integer16(1)),
        (146, X::Real(0.75)),
    ];
    let first = prepare_dimension_from_cad(&dimension(&pairs), &doc).unwrap();
    pairs.reverse();
    let reversed = prepare_dimension_from_cad(&dimension(&pairs), &doc).unwrap();
    assert_eq!(first.overrides, reversed.overrides);
    assert_eq!(
        first.overrides.numbers.primary.fraction_layout,
        Some(DimensionFractionLayout::Diagonal)
    );
    assert_eq!(first.overrides.numbers.primary.fraction_scale, Some(0.75));
    assert!(matches!(
        first.overrides.extension_lines.first.length,
        Some(DimensionExtensionLength::Fixed { length: 20.0 })
    ));
    assert!(matches!(
        first.overrides.dimension_line.appearance.color,
        Some(DimensionAppearanceChoice::Explicit(_))
    ));
    let base = prepare_style_from_cad(doc.dim_styles.get("Standard").unwrap(), &doc)
        .unwrap()
        .values;
    let mut returned = dimension(&[]);
    returned.base_mut().common.extended_data = Default::default();
    apply_entity_overrides_to_cad(&mut returned, &first.overrides, &base).unwrap();
    let read = prepare_dimension_from_cad(&returned, &doc).unwrap();
    assert_eq!(read.overrides, first.overrides);
}

#[test]
fn active_and_inactive_fixed_length_and_local_text_placement_are_distinct() {
    let mut doc = CadDocument::new();
    let p = prepare_dimension_from_cad(
        &dimension(&[(49, X::Real(20.0)), (290, X::Integer16(0))]),
        &doc,
    )
    .unwrap();
    assert_eq!(
        p.overrides.extension_lines.first.length,
        Some(DimensionExtensionLength::Automatic)
    );
    let style = doc.dim_styles.get_mut("Standard").unwrap();
    style.dimtad = 1;
    style.dimtvp = 1.0;
    assert!(prepare_dimension_from_cad(&dimension(&[(77, X::Integer16(0))]), &doc).is_err());
}

#[test]
fn literal_repair_dxf_retains_complete_source_profile() {
    let doc = opencadcodec::DxfReader::from_reader(std::io::Cursor::new(include_bytes!(
        "../../../tests/fixtures/dimension/repair-profiles.dxf"
    )))
    .unwrap()
    .read()
    .unwrap();
    let d = doc
        .entities()
        .find_map(|e| {
            if let opencadcodec::EntityType::Dimension(d) = e {
                Some(d)
            } else {
                None
            }
        })
        .unwrap();
    let prepared = prepare_dimension_from_cad(d, &doc).unwrap();
    assert_eq!(
        evaluate_dimension(&prepared.definition).unwrap(),
        DimensionMeasurement::Distance(48.25)
    );
    assert_eq!(
        prepared.overrides.numbers.primary.fraction_layout,
        Some(DimensionFractionLayout::Diagonal)
    );
    assert_eq!(
        prepared.overrides.numbers.primary.fraction_scale,
        Some(0.75)
    );
    assert_eq!(
        prepared.overrides.extension_lines.first.length,
        Some(DimensionExtensionLength::Fixed { length: 20.0 })
    );
}

#[test]
fn dimdsep_zero_means_decimal_point_in_styles_and_explicit_overrides() {
    let mut doc = CadDocument::new();
    for (source, expected) in [(0, '.'), (46, '.'), (44, ',')] {
        let style = doc.dim_styles.get_mut("Standard").unwrap();
        style.dimdsep = source;
        style.dimalt = true;
        let prepared =
            prepare_style_from_cad(doc.dim_styles.get("Standard").unwrap(), &doc).unwrap();
        assert_eq!(prepared.values.numbers.primary.decimal_separator, expected);
        assert_eq!(
            prepared
                .values
                .numbers
                .alternate
                .as_ref()
                .unwrap()
                .decimal_separator,
            expected
        );
    }
    for inherited in [44, 46] {
        doc.dim_styles.get_mut("Standard").unwrap().dimdsep = inherited;
        let prepared =
            prepare_dimension_from_cad(&dimension(&[(278, X::Integer16(0))]), &doc).unwrap();
        assert_eq!(
            prepared.overrides.numbers.primary.decimal_separator,
            Some('.')
        );
    }
    let mut values = prepare_style_from_cad(doc.dim_styles.get("Standard").unwrap(), &doc)
        .unwrap()
        .values;
    values.numbers.primary.decimal_separator = char::from(0);
    assert!(validate_dimension_numbers(&values.numbers).is_err());
}
