use base64::{engine::general_purpose::STANDARD, Engine};
use ifccad::{
    dimension::{evaluate_dimension, DimensionMeasurement},
    load_ifccad_bytes, IfccadDocument, IfccadEntityKind,
};
use serde_json::Value;

fn downloaded(result: &Value) -> Vec<u8> {
    assert!(result["failure"].is_null(), "{result}");
    assert_eq!(result["validation"]["strictAvailable"], true, "{result}");
    STANDARD
        .decode(result["export"]["download"]["base64"].as_str().unwrap())
        .unwrap()
}

fn dimensions(drawing: &IfccadDocument) -> Vec<(String, f64, bool)> {
    let owners = std::iter::once(("Model", &drawing.model.entities))
        .chain(
            drawing
                .paper_layouts
                .iter()
                .map(|p| (p.name.as_str(), &p.entities)),
        )
        .chain(
            drawing
                .blocks
                .iter()
                .map(|b| (b.name.as_str(), &b.entities)),
        );
    let mut values = Vec::new();
    for (owner, entities) in owners {
        for entity in entities {
            let Some(native) = entity.as_native() else {
                continue;
            };
            let IfccadEntityKind::Dimension(dimension) = &native.kind else {
                continue;
            };
            let definition = serde_json::to_value(&dimension.definition).unwrap();
            let value = match evaluate_dimension(&dimension.definition).unwrap() {
                DimensionMeasurement::Distance(value) | DimensionMeasurement::Angle(value) => value,
            };
            values.push((
                format!("{owner}/{}", definition["kind"].as_str().unwrap()),
                value,
                dimension.graphics.is_some(),
            ));
        }
    }
    values.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)).then(a.2.cmp(&b.2)));
    values
}

fn roundtrip(name: &str, bytes: &[u8]) {
    let source = load_ifccad_bytes(bytes, Default::default()).unwrap();
    let expected = dimensions(source.document());
    assert!(!expected.is_empty(), "{name} must exercise dimensions");
    for format in ["dxf", "dwg"] {
        let exported = viewer::document_export_ifccad(name, bytes, format, "AC1032", "{}");
        let cad = downloaded(&exported);
        let returned =
            viewer::document_cad_to_ifccad(name, format, &cad, "2026-10-29T14:00:00Z", "{}");
        let native = downloaded(&returned);
        let checked = load_ifccad_bytes(&native, Default::default()).unwrap();
        assert_eq!(
            checked.document().length_unit,
            source.document().length_unit,
            "{name}/{format}"
        );
        let actual = dimensions(checked.document());
        assert_eq!(actual.len(), expected.len(), "{name}/{format}");
        for (before, after) in expected.iter().zip(&actual) {
            assert_eq!(before.0, after.0, "{name}/{format}");
            // A regression budget for these fixtures, not a general CAD accuracy certificate.
            assert!(
                (before.1 - after.1).abs() <= 1e-9 * before.1.abs().max(1.0),
                "{name}/{format}: {before:?} -> {after:?}"
            );
            assert_eq!(
                before.2, after.2,
                "{name}/{format}: saved graphics for {}",
                before.0
            );
        }
    }
}

#[test]
fn shipped_examples_keep_dimension_measurements_and_pictures_through_physical_cad() {
    let catalogue = include_bytes!("../../../examples/ifccad/entities.ifcx");
    let drawing = load_ifccad_bytes(catalogue, Default::default()).unwrap();
    let kinds = dimensions(drawing.document())
        .into_iter()
        .map(|d| d.0.rsplit_once('/').unwrap().1.to_owned())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(kinds.len(), 9, "catalogue must cover all nine kinds");
    roundtrip("entities.ifcx", catalogue);
    roundtrip(
        "house.ifcx",
        include_bytes!("../../../examples/ifccad/house.ifcx"),
    );
}

#[test]
#[ignore = "external practice input: set IFCCAD_REVIEW_SAMPLE_DWG and run explicitly"]
fn sample_ac1032_keeps_all_eleven_dimensions_through_physical_cad() {
    let path = std::env::var("IFCCAD_REVIEW_SAMPLE_DWG").expect("external practice-file path");
    let input = std::fs::read(path).unwrap();
    let opening = viewer::document_cad_to_ifccad(
        "sample_AC1032.dwg",
        "dwg",
        &input,
        "2026-10-29T14:00:00Z",
        "{}",
    );
    let native = downloaded(&opening);
    let drawing = load_ifccad_bytes(&native, Default::default()).unwrap();
    assert_eq!(dimensions(drawing.document()).len(), 11);
    roundtrip("sample_AC1032.ifcx", &native);
}
