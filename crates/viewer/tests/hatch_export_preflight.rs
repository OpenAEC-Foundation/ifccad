use ifccad::*;
#[path = "../../../tests/support/ifccad_hatch.rs"]
mod fixture;

#[test]
fn default_recipe_trial_cannot_veto_export_with_an_explicit_qualified_tolerance() {
    use ifccad::geometry_kernel::hatch::*;
    let mut d = fixture::drawing();
    for layer in &mut d.layers {
        layer.appearance.line_weight = 0.09;
    }
    let IfccadEntityKind::Hatch(h) = &mut d.model.entities[0].as_native_mut().unwrap().kind else {
        panic!()
    };
    h.fill = HatchFill::LinePattern(HatchLinePattern {
        name: Some("Original".into()),
        description: Some(String::new()),
        origin: [0., 0.],
        rotation: 0.,
        scale: 1.,
        families: vec![HatchLineFamily {
            angle: 0.,
            base_point: [0., 0.],
            offset: [0., 2.],
            dashes: vec![],
        }],
    });
    h.classic_settings = Some(HatchClassicSettings {
        preferred_fill: Some(HatchClassicFill::LinePattern),
        creation_type: Some(HatchCreationType::Custom),
        name: Some("Original".into()),
        description: Some(String::new()),
        definition_lines: Some(vec![HatchPatternDefinitionLine {
            angle: 0.,
            base_point: [0., 2e-9],
            offset_vector: [0., 2.],
            dash_lengths: vec![],
        }]),
        ..Default::default()
    });
    let bytes = encode_ifccad_document(&d).unwrap();
    let inspected = viewer::inspect_ifccad_bytes("recipe.ifcx", bytes.bytes());
    for entry in inspected["presentation"]["hatchExportPreflight"]
        .as_array()
        .unwrap()
    {
        let restrictions = entry["restrictions"].as_array().unwrap();
        assert!(
            restrictions.iter().any(
                |r| r["code"] == "HATCH_RECIPE_PREFLIGHT_CHECK" && r["policyEffect"] == "check"
            ),
            "{entry}"
        );
        assert!(
            !restrictions
                .iter()
                .any(|r| r["policyEffect"] == "loss" || r["policyEffect"] == "block"),
            "{entry}"
        );
    }
    let out = viewer::export_ifccad_bytes_with_options(
        "recipe.ifcx",
        bytes.bytes(),
        "dwg",
        "AC1024",
        r#"{"lossPolicy":"reject","tolerance":{"mode":"custom","unit":"drawing","value":1e-8}}"#,
    );
    assert!(out["failure"].is_null(), "{out}");
    assert!(!out["export"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d["code"] == "hatch-classic-settings-loss"));
    let exact = viewer::export_ifccad_bytes_with_options(
        "recipe.ifcx",
        bytes.bytes(),
        "dwg",
        "AC1024",
        r#"{"lossPolicy":"reject","tolerance":{"mode":"exact"}}"#,
    );
    assert!(!exact["failure"].is_null());
    assert!(exact["export"]["download"].is_null());
}
#[test]
fn drawing_preflight_uses_all_target_profiles_and_the_same_active_gradient_gate() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../../../conformance/next/ifccad/valid/hatch-gradient.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    d.layers[0].appearance.line_weight = 0.09;
    let bytes = encode_ifccad_document(&d).unwrap();
    let inspected = viewer::inspect_ifccad_bytes("gradient.ifcx", bytes.bytes());
    let entries = inspected["presentation"]["hatchExportPreflight"]
        .as_array()
        .unwrap();
    assert_eq!(entries.len(), 12);
    for e in entries {
        let format = e["format"].as_str().unwrap();
        let version = e["version"].as_str().unwrap();
        let hard = e["restrictions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["policyEffect"] == "block");
        assert_eq!(hard, version == "AC1015", "{e}");
        let actual = viewer::export_ifccad_bytes_with_options(
            "gradient.ifcx",
            bytes.bytes(),
            format,
            version,
            "{}",
        );
        assert_eq!(
            actual["failure"]["code"] == "HATCH_TARGET_FEATURE_UNSUPPORTED",
            hard
        );
    }
}
#[test]
fn a_drawing_without_hatches_has_no_hatch_restrictions() {
    let out = viewer::inspect_ifccad_bytes(
        "basic.ifcx",
        include_bytes!("../../../tests/fixtures/ifccad/hello-cad.ifcx"),
    );
    let entries = out["presentation"]["hatchExportPreflight"]
        .as_array()
        .unwrap();
    assert_eq!(entries.len(), 12);
    assert!(entries
        .iter()
        .all(|e| e["restrictions"].as_array().unwrap().is_empty()));
}
#[test]
fn unavailable_default_numeric_preparation_does_not_block_an_explicit_output_tolerance() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../../../tests/fixtures/ifccad/hello-hatch-solid.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    d.layers[0].appearance.line_weight = 0.09;
    for e in &mut d.model.entities {
        if let Some(e) = e.as_native_mut() {
            if let IfccadEntityKind::Hatch(h) = &mut e.kind {
                h.placement.origin[0] = 1e16;
                h.association_enabled = false;
                h.loops = vec![IfccadHatchLoop {
                    boundary: ifccad::geometry_kernel::hatch::HatchBoundary2::Circle {
                        center: [0.5, 0.],
                        radius: 3.,
                    },
                    source_entity_ids: vec![],
                    provenance: None,
                    text_island: false,
                }];
            }
        }
    }
    recompute_ifccad_document_bounds(&mut d).unwrap();
    let bytes = encode_ifccad_document(&d).unwrap();
    let inspected = viewer::inspect_ifccad_bytes("h.ifcx", bytes.bytes());
    let entries = inspected["presentation"]["hatchExportPreflight"]
        .as_array()
        .unwrap();
    assert!(
        entries.iter().all(|e| e["restrictions"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["policyEffect"] != "block")),
        "{entries:?}"
    );
    let actual = viewer::export_ifccad_bytes_with_options(
        "h.ifcx",
        bytes.bytes(),
        "dxf",
        "AC1024",
        r#"{"tolerance":{"mode":"custom","unit":"drawing","value":1.0}}"#,
    );
    assert!(actual["failure"].is_null(), "{}", actual["failure"]);
    for e in &mut d.model.entities {
        if let Some(e) = e.as_native_mut() {
            if let IfccadEntityKind::Hatch(h) = &mut e.kind {
                h.fill = ifccad::geometry_kernel::hatch::HatchFill::Gradient;
                h.gradient = Some(ifccad::geometry_kernel::hatch::HatchGradient {
                    gradient_type: Some(ifccad::geometry_kernel::hatch::HatchGradientType::Linear),
                    colors: Some([IfccadColor::rgb(10, 20, 30), IfccadColor::rgb(40, 50, 60)]),
                    rotation: 0.,
                    shift: 0.,
                    color_mode: ifccad::geometry_kernel::hatch::HatchGradientColorMode::TwoColor,
                    tint: 0.,
                });
            }
        }
    }
    let bytes = encode_ifccad_document(&d).unwrap();
    let inspected = viewer::inspect_ifccad_bytes("h.ifcx", bytes.bytes());
    for entry in inspected["presentation"]["hatchExportPreflight"]
        .as_array()
        .unwrap()
    {
        assert_eq!(
            entry["restrictions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["policyEffect"] == "block"),
            entry["version"] == "AC1015"
        );
    }
}
