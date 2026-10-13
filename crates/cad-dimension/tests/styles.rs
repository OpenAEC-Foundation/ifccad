use cad_dimension::*;
use ocdraw::dimension::*;
use opencadcodec::{tables::DimStyle, CadDocument};
#[test]
fn paper_only_measure_factor_is_explicit_without_changing_coordinates() {
    let mut s = DimStyle::new("Building");
    s.dimlfac = -2.0;
    s.dimtxt = 2.5;
    let p = prepare_style_from_cad(&s, &CadDocument::new()).unwrap();
    assert_eq!(p.values.numbers.primary.measurement_factor, 2.0);
    assert_eq!(
        p.values.numbers.primary.application,
        DimensionFactorApplication::PaperOnly
    );
    assert_eq!(p.values.text.height, 2.5);
}
#[test]
fn annotative_style_is_a_located_restriction() {
    let mut s = DimStyle::new("Anno");
    s.annotative = true;
    assert!(matches!(
        prepare_style_from_cad(&s, &CadDocument::new()),
        Err(CadDimensionPreparationError::Unsupported { .. })
    ));
}

#[test]
fn automatic_dimension_scale_is_unsupported_context_instead_of_invalid_data() {
    let mut s = DimStyle::new("Automatic");
    s.dimscale = 0.0;
    assert!(
        matches!(prepare_style_from_cad(&s,&CadDocument::new()),Err(CadDimensionPreparationError::Unsupported{field,..}) if field=="DIMSCALE")
    );
}

#[test]
fn active_unrepresented_style_semantics_are_not_silently_discarded() {
    let doc = CadDocument::new();
    let mut outside = DimStyle::new("Outside");
    outside.dimsoxd = true;
    let mut fraction = DimStyle::new("Stacked");
    fraction.dimlunit = 5;
    fraction.dimfrac = 0;
    let mut factor = DimStyle::new("ZeroSuffix");
    factor.dimmzs = "zero".into();
    let mut tolerance = DimStyle::new("Tolerance");
    tolerance.dimtol = true;
    tolerance.dimtfac = 0.5;
    for s in [outside, fraction, factor, tolerance] {
        assert!(
            matches!(
                prepare_style_from_cad(&s, &doc),
                Err(CadDimensionPreparationError::Unsupported { .. })
            ),
            "{}",
            s.name
        );
    }
}

#[test]
fn ordinary_style_values_roundtrip_without_coordinate_rescaling() {
    let doc = CadDocument::new();
    let source = doc.dim_styles.get("Standard").unwrap();
    let prepared = prepare_style_from_cad(source, &doc).unwrap();
    let target = prepare_style_to_cad("Building", &prepared.values).unwrap();
    let returned = prepare_style_from_cad(&target.style, &doc).unwrap();
    assert_eq!(returned.values.text.height, prepared.values.text.height);
    assert_eq!(
        returned.values.numbers.primary.measurement_factor,
        prepared.values.numbers.primary.measurement_factor
    );
    assert_eq!(
        returned.values.dimension_line.suppress_first,
        prepared.values.dimension_line.suppress_first
    );
}

#[test]
fn null_target_references_and_zero_center_mark_modes_do_not_become_defaults() {
    use ocdraw::dimension::*;
    let doc = CadDocument::new();
    let base = prepare_style_from_cad(doc.dim_styles.get("Standard").unwrap(), &doc)
        .unwrap()
        .values;
    let mut v = base.clone();
    v.text.style = opencadcodec::Handle::NULL;
    assert!(matches!(
        prepare_style_to_cad("Null font", &v),
        Err(CadDimensionPreparationError::Invalid { .. })
    ));
    let mut v = base.clone();
    v.arrows.first = DimensionMarker::LocalBlock {
        block: opencadcodec::Handle::NULL,
    };
    assert!(matches!(
        prepare_style_to_cad("Null marker", &v),
        Err(CadDimensionPreparationError::Invalid { .. })
    ));
    let mut v = base.clone();
    v.dimension_line.appearance.line_pattern =
        DimensionAppearanceChoice::Explicit(opencadcodec::Handle::NULL);
    assert!(matches!(
        prepare_style_to_cad("Null pattern", &v),
        Err(CadDimensionPreparationError::Invalid { .. })
    ));
    let mut v = base;
    v.arrows.center_mark = DimensionCenterMark::Mark { size: 0.0 };
    assert!(matches!(
        prepare_style_to_cad("Zero mark", &v),
        Err(CadDimensionPreparationError::Unsupported { .. })
    ));
}
