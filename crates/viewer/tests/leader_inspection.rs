#[test]
fn leader_inspection_keeps_owned_parts_separate_from_ordinary_entities() {
    let output = viewer::inspect_ifccad_bytes(
        "leaders.ifcx",
        include_bytes!("../../../conformance/next/ifccad/valid/leaders-family.ifcx"),
    );
    assert!(output["failure"].is_null(), "{output}");
    let values = output["presentation"]["leaders"]
        .as_array()
        .expect("leader-family inspection");
    assert_eq!(values.len(), 3);
    let multi = values.iter().find(|v| v["kind"] == "MultiLeader").unwrap();
    let parts = multi["ownedNodes"].as_array().unwrap();
    assert!(parts.iter().any(|n| n["attributes"]
        .get("ifccad::multiLeaderBlockLabel")
        .is_some()));
    assert!(parts
        .iter()
        .any(|n| n["attributes"].get("ifccad::multiLeaderLine").is_some()));
    assert!(parts
        .iter()
        .all(|n| n["attributes"].get("ifccad::entity").is_none()));
    assert_eq!(multi["graphics"]["commandCount"], 1);
    assert_eq!(multi["displayEvaluation"], "unassessed");
    assert_eq!(
        output["presentation"]["multiLeaderStyles"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
