#[test]
fn ocdraw_hatch_inspection_exposes_loops_sources_and_limit() {
    let out = viewer::inspect_drawing_bytes(
        "hatch.ocdraw.json",
        include_bytes!("../../../conformance/next/ocdraw/valid/hatch-solid.ocdraw.json"),
    );
    assert_eq!(out["presentation"]["hatchEntityCount"], 1);
    let h = out["presentation"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["geometry"]["type"] == "hatch")
        .unwrap();
    assert_eq!(h["geometry"]["areaRule"], "normal");
    assert_eq!(h["geometry"]["joinTolerance"], 1e-9);
    assert_eq!(h["geometry"]["loops"].as_array().unwrap().len(), 2);
    assert!(h["geometry"]["loops"]
        .as_array()
        .unwrap()
        .iter()
        .any(|l| l["sourceEntityIds"]
            .as_array()
            .is_some_and(|ids| ids.iter().all(|id| id.is_string()))));
    assert_eq!(h["geometry"]["fillEvaluation"], "unassessed");
}

#[test]
fn hatch_association_states_keep_text_island_and_unassessed_fill() {
    use ocdraw::ocdraw::*;
    for (enabled, bound, status) in [
        (false, true, "disabled"),
        (true, true, "active"),
        (true, false, "unbound"),
    ] {
        let mut d = load_ocdraw_bytes(include_bytes!(
            "../../../tests/fixtures/ocdraw/hello-hatch-solid.ocdraw.json"
        ))
        .unwrap()
        .into_document();
        d.hatch_entities[0].association_enabled = enabled;
        for l in &mut d.hatch_entities[0].loops {
            l.text_island = true;
            if !bound {
                l.source_entity_ids.clear();
            }
        }
        let bytes = encode_ocdraw_document(&d).unwrap();
        let out = viewer::inspect_drawing_bytes("hatch.ocdraw.json", bytes.bytes());
        let g = &out["presentation"]["entities"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["geometry"]["type"] == "hatch")
            .unwrap()["geometry"];
        assert_eq!(g["associationStatus"], status);
        assert_eq!(g["associationEnabled"], enabled);
        assert!(g["loops"]
            .as_array()
            .unwrap()
            .iter()
            .all(|l| l["textIsland"] == true));
        assert_eq!(g["fillEvaluation"], "unassessed");
    }
}

#[test]
fn gradient_inspection_keeps_authored_state_without_claiming_fill_evaluation() {
    let out = viewer::inspect_drawing_bytes(
        "gradient.ocdraw.json",
        include_bytes!("../../../conformance/next/ocdraw/valid/hatch-gradient.ocdraw.json"),
    );
    let g = &out["presentation"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["geometry"]["type"] == "hatch")
        .unwrap()["geometry"];
    assert_eq!(g["gradient"]["type"], "inverseSpherical");
    assert_eq!(
        g["gradient"]["colors"][1]["rgb"],
        serde_json::json!([30, 60, 150])
    );
    assert_eq!(g["gradientStatus"], "active");
    assert_eq!(g["fillEvaluation"], "unassessed");
    let out = viewer::inspect_drawing_bytes(
        "partial.ocdraw.json",
        include_bytes!(
            "../../../conformance/next/ocdraw/valid/hatch-gradient-inactive.ocdraw.json"
        ),
    );
    let g = &out["presentation"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["geometry"]["type"] == "hatch")
        .unwrap()["geometry"];
    assert_eq!(g["gradientStatus"], "partial");
    assert!(g["gradient"].get("colors").is_none());
}
#[test]
fn ifccad_hatch_inspection_and_both_export_routes_are_available() {
    let bytes = include_bytes!("../../../conformance/next/ifccad/valid/hatch-solid.ifcx");
    let out = viewer::inspect_drawing_bytes("hatch.ifcx", bytes);
    assert_eq!(out["presentation"]["hatchEntityCount"], 1);
    assert_eq!(out["presentation"]["hatches"].as_array().unwrap().len(), 1);
    for format in ["dxf", "dwg"] {
        let out = viewer::export_drawing_bytes("hatch.ifcx", bytes, format, "AC1032");
        assert!(out["failure"].is_null(), "{out}");
    }
}

#[test]
fn pattern_inspection_exposes_authored_families_and_phase() {
    let out = viewer::inspect_drawing_bytes(
        "pattern.ocdraw.json",
        include_bytes!("../../../conformance/next/ocdraw/valid/hatch-pattern.ocdraw.json"),
    );
    let h = out["presentation"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["geometry"]["type"] == "hatch")
        .unwrap();
    let fill = &h["geometry"]["fill"];
    assert_eq!(fill["kind"], "linePattern");
    assert_eq!(fill["origin"], serde_json::json!([0.5, -0.5]));
    assert_eq!(fill["families"].as_array().unwrap().len(), 2);
    assert_eq!(h["geometry"]["fillEvaluation"], "unassessed");
}

#[test]
fn physical_pattern_description_loss_is_reported_for_both_routes_and_codecs() {
    for (name, bytes) in [
        (
            "pattern.ocdraw.json",
            include_bytes!("../../../tests/fixtures/ocdraw/hello-hatch-pattern.ocdraw.json")
                .as_slice(),
        ),
        (
            "pattern.ifcx",
            include_bytes!("../../../tests/fixtures/ifccad/hello-hatch-pattern.ifcx").as_slice(),
        ),
    ] {
        for format in ["dxf", "dwg"] {
            let out = viewer::export_drawing_bytes(name, bytes, format, "AC1032");
            assert!(out["failure"].is_null(), "{out}");
            let losses = out["export"]["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|d| d["code"] == "TARGET_CODEC_HATCH_DESCRIPTION_LOSS")
                .collect::<Vec<_>>();
            assert_eq!(
                losses.len(),
                1,
                "{name}/{format}: missing description diagnostic"
            );
            assert_eq!(losses[0]["phase"], "cadExchange");
            assert_eq!(losses[0]["action"], "Modified");
        }
    }
}
