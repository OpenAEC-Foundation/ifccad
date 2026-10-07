//! Qualification of independently authored source fixtures, not native conversion.
use opencadcodec::{DxfReader, EntityType, Vector3};
use std::io::Cursor;

#[test]
fn literal_text_fixture_retains_active_and_dormant_source_fields() {
    let document = DxfReader::from_reader(Cursor::new(include_bytes!(
        "../../../tests/fixtures/text-preparation/text-layouts.dxf"
    )))
    .unwrap()
    .read()
    .unwrap();
    let texts = document
        .entities()
        .filter_map(|entity| match entity {
            EntityType::Text(text) => Some(text),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(texts.len(), 8);
    let find = |value: &str| {
        texts
            .iter()
            .copied()
            .find(|text| text.value == value)
            .unwrap()
    };
    let baseline = find("D-01");
    assert_eq!(baseline.insertion_point, Vector3::new(10.0, 20.0, 0.0));
    assert_eq!(
        baseline.alignment_point,
        Some(Vector3::new(999.0, 777.0, 0.0))
    );
    assert_eq!(
        find("RIGHT").alignment_point,
        Some(Vector3::new(30.0, 40.0, 0.0))
    );
    assert_eq!(
        find("ALIGNED").alignment_point,
        Some(Vector3::new(3.0, 4.0, 0.0))
    );
    let fit = find("FIT");
    assert_eq!(fit.insertion_point, Vector3::new(5.0, 6.0, 7.0));
    assert_eq!(fit.alignment_point, Some(Vector3::new(13.0, 6.0, 7.0)));
    assert_eq!(find("TILTED").normal, Vector3::UNIT_Y);
    assert_eq!(find("AB").generation_flags, 6);
    let middles = texts
        .iter()
        .filter(|text| text.value == "gyp")
        .collect::<Vec<_>>();
    assert_eq!(middles.len(), 2);
    assert_ne!(
        middles[0].horizontal_alignment,
        middles[1].horizontal_alignment
    );
    assert!(texts
        .iter()
        .all(|text| text.style == "Labels" && text.height == 2.5));
    assert!(document.text_styles.get("Unused").is_some());
}

#[test]
fn literal_mtext_fixture_retains_wcs_and_authored_source_structure() {
    let document = DxfReader::from_reader(Cursor::new(include_bytes!(
        "../../../tests/fixtures/text-preparation/mtext-markup.dxf"
    )))
    .unwrap()
    .read()
    .unwrap();
    let texts = document
        .entities()
        .filter_map(|entity| match entity {
            EntityType::MText(text) => Some(text),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(texts.len(), 3);
    let structured = texts
        .iter()
        .find(|text| text.value.contains("Header"))
        .unwrap();
    assert_eq!(structured.insertion_point, Vector3::new(100.0, 50.0, 7.0));
    assert_eq!(structured.normal, Vector3::UNIT_Z);
    assert_eq!(structured.rectangle_width, 80.0);
    assert!((structured.rotation - std::f64::consts::FRAC_PI_2).abs() < 1e-15);
    // The ASCII stream reader decodes caret controls before CadDocument exists.
    assert_eq!(structured.value, "\\PHeader\\P\\PValue:\t\\S1/2;\\P");
    let scoped = texts
        .iter()
        .find(|text| text.value.contains("quarter"))
        .unwrap();
    assert_eq!(scoped.value, r"{\L\H0.2x;small\H1.25x;quarter}plain");
    assert!(texts
        .iter()
        .all(|text| text.style == "Labels" && text.height == 2.5));
    assert!(document.text_styles.get("Unused").is_some());
}
