use ocdraw::{line_pattern::*, ocdraw::*};

#[test]
fn line_pattern_inspection_keeps_source_links() {
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("inspection-markers", "mm")).unwrap();
    let source = b
        .add_symbol_source(SymbolSource::Shx {
            file_name: "missing.shx".into(),
        })
        .unwrap();
    b.add_line_pattern(LinePatternDefinition {
        name: "Shapes".into(),
        description: None,
        pattern: vec![LinePatternElement {
            length: 1.,
            marker: Some(DrawingLinePatternMarker::Symbol {
                symbol_source_id: source,
                shape_number: 42,
                transform: Default::default(),
            }),
        }],
    })
    .unwrap();
    let encoded = b.finish().unwrap();
    let result = viewer::inspect_drawing_bytes("markers.ocdraw.json", encoded.bytes());
    assert_eq!(
        result["presentation"]["linePatterns"][0]["pattern"][0]["marker"]["symbolSourceId"],
        0
    );
    assert_eq!(
        result["presentation"]["symbolSources"][0]["fileName"],
        "missing.shx"
    );
}

#[test]
fn line_pattern_inspection_keeps_declared_and_derived_evidence_separate() {
    for (name, bytes, claim) in [
        (
            "patterns.ocdraw.json",
            include_bytes!("../../../tests/fixtures/ocdraw/complex-line-patterns.ocdraw.json")
                .as_slice(),
            "declaredQuality",
        ),
        (
            "patterns.ifcx",
            include_bytes!("../../../tests/fixtures/ifccad/complex-line-patterns.ifcx").as_slice(),
            "storedQuality",
        ),
    ] {
        let result = viewer::inspect_drawing_bytes(name, bytes);
        let assessments = result["presentation"]["boundsCompleteness"]
            .as_array()
            .unwrap();
        let model = assessments
            .iter()
            .find(|a| a[claim] == "enclosing")
            .unwrap();
        assert_eq!(model["quality"], "partial");
        assert_eq!(model["enclosureVerified"], false);
        assert_eq!(model["safeForNegativeQuery"], false);
        assert!(model["linePatternReasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r == "SymbolFootprintUnavailable"));
    }
}

#[test]
fn ifccad_high_source_paths_remain_exact_in_inspection() {
    let result = viewer::inspect_drawing_bytes(
        "high.ifcx",
        include_bytes!("../../../conformance/next/ifccad/valid/line-pattern-high-source-id.ifcx"),
    );
    let nodes = result["presentation"]["graph"]["data"].as_array().unwrap();
    assert!(nodes
        .iter()
        .any(|n| n["path"] == "/cad/d1/symbolSource/9007199254740993"));
    assert!(nodes
        .iter()
        .filter_map(|n| n["attributes"]["ifccad::linePattern"]["pattern"].as_array())
        .flatten()
        .any(|e| e["marker"]["symbolSource"] == "/cad/d1/symbolSource/9007199254740993"));
}
