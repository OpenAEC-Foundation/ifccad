#[test]
fn line_pattern_inspection_keeps_declared_and_derived_evidence_separate() {
    {
        let (name, bytes, claim) = (
            "patterns.ifcx",
            include_bytes!("../../../tests/fixtures/ifccad/complex-line-patterns.ifcx").as_slice(),
            "storedQuality",
        );
        let result = viewer::inspect_ifccad_bytes(name, bytes);
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
    let result = viewer::inspect_ifccad_bytes(
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
