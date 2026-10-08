use viewer::{export_drawing_bytes, inspect_drawing_bytes};

#[test]
fn ifccad_inspection_exposes_native_presentation_and_point_display() {
    use ocdraw::ifccad::*;
    let mut d = load_ifccad_bytes(
        include_bytes!("../../../examples/ifccad/hello-viewports.ifcx"),
        Default::default(),
    )
    .unwrap()
    .document()
    .clone();
    let layer = d.layers[0].id;
    let pattern = d.line_patterns[0].id;
    d.layers[0].description = Some("Described".into());
    d.layers[0].visible = false;
    d.layers[0].frozen = true;
    d.layers[0].locked = true;
    d.layers[0].plottable = false;
    d.layers[0].frozen_in_new_viewports = true;
    d.layers[0].appearance.color = IfccadColor::rgb(255, 0, 0)
        .with_indexed("ACI", 1)
        .with_named("Book", "Red");
    d.point_display = Some(IfccadPointDisplay {
        glyph: IfccadPointGlyph::Plus,
        circle: true,
        square: false,
        size: IfccadPointSize::ViewportPercent(2.),
    });
    let entity = d.paper_layouts[0]
        .entities
        .iter_mut()
        .find_map(|e| {
            e.as_native_mut()
                .filter(|e| matches!(e.kind, IfccadEntityKind::Viewport(_)))
        })
        .unwrap();
    entity.visible = false;
    let IfccadEntityKind::Viewport(v) = &mut entity.kind else {
        panic!()
    };
    v.plot_shading_override = Some(ocdraw::plot_kernel::ShadedPlotMode::Hidden);
    v.layer_overrides = vec![IfccadViewportLayerOverride {
        layer_id: layer,
        frozen: true,
        color: Some(IfccadColor::rgb(255, 0, 0).with_indexed("ACI", 1)),
        opacity: Some(1.),
        line_pattern_id: Some(pattern),
        line_weight: Some(0.25),
    }];
    let id = d.id_counters.allocate_block_id().unwrap();
    d.blocks.push(IfccadBlockDefinition {
        id,
        base_point: [0.; 3],
        insertion_unit: "mm".into(),
        name: "Unused".into(),
        description: "Block description".into(),
        anonymous: true,
        explodable: false,
        uniform_scaling: true,
        entities: vec![],
        bounds: None,
        bounds_quality: None,
    });
    let bytes = encode_ifccad_document(&d).unwrap();
    let result = inspect_drawing_bytes("presentation.ifcx", bytes.bytes());
    assert_eq!(result["validation"]["strictAvailable"], true);
    assert_eq!(
        result["presentation"]["pointDisplay"]["form"]["glyph"],
        "plus"
    );
    let layer = &result["presentation"]["layers"][0]["attributes"]["ifccad::layer"];
    assert_eq!(layer["description"], "Described");
    assert_eq!(layer["visible"], false);
    assert_eq!(layer["frozen"], true);
    assert_eq!(layer["locked"], true);
    assert_eq!(layer["plottable"], false);
    assert_eq!(layer["frozenInNewViewports"], true);
    assert_eq!(layer["appearance"]["color"]["namedColor"]["name"], "Red");
    let block =
        &result["presentation"]["blockDefinitions"][0]["attributes"]["ifccad::blockDefinition"];
    assert_eq!(block["description"], "Block description");
    assert_eq!(block["anonymous"], true);
    assert_eq!(block["explodable"], false);
    assert_eq!(block["uniformScaling"], true);
    let v = result["presentation"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["attributes"].get("ifccad::viewport").is_some())
        .unwrap();
    assert_eq!(v["attributes"]["ifccad::entity"]["visible"], false);
    assert_eq!(
        v["attributes"]["ifccad::viewport"]["plotShadingOverride"],
        "Hidden"
    );
    assert_eq!(
        v["attributes"]["ifccad::viewport"]["layerOverrides"][0]["color"]["indexedColor"]["index"],
        1
    );
}

#[test]
fn ocdraw_inspection_exposes_typed_viewport_override_refs_and_mode() {
    use ocdraw::ocdraw::*;
    let mut d = load_ocdraw_bytes(include_bytes!(
        "../../../conformance/next/ocdraw/valid/paper-viewport.ocdraw.json"
    ))
    .unwrap()
    .document()
    .clone();
    d.viewports[0].plot_shading_override = Some(ShadedPlotMode::Hidden);
    d.viewports[0].layer_overrides = vec![DrawingViewportLayerOverride {
        layer_id: d.layers[0].id,
        frozen: true,
        color: Some(DrawingColor::rgb(255, 0, 0).with_indexed("ACI", 1)),
        opacity: Some(1.),
        line_pattern_id: Some(d.line_patterns[0].id),
        line_weight: Some(0.25),
    }];
    let bytes = encode_ocdraw_document(&d).unwrap();
    let result = inspect_drawing_bytes("presentation.ocdraw.json", bytes.bytes());
    let v = result["presentation"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["type"] == "viewport")
        .unwrap();
    assert_eq!(v["plotShadingOverride"], "Hidden");
    assert_eq!(v["layerOverrides"][0]["layerId"], d.layers[0].id);
    assert_eq!(
        v["layerOverrides"][0]["linePatternId"],
        d.line_patterns[0].id.0
    );
    assert_eq!(v["layerOverrides"][0]["color"]["indexed"][1], 1);
}

#[test]
fn physical_shadeplot_loss_is_explicit_for_dxf_but_not_dwg_in_both_routes() {
    use ocdraw::{ifccad::*, ocdraw::*};
    let mut oc = load_ocdraw_bytes(include_bytes!(
        "../../../conformance/next/ocdraw/valid/paper-viewport.ocdraw.json"
    ))
    .unwrap()
    .document()
    .clone();
    oc.viewports[0].plot_shading_override = Some(ShadedPlotMode::Hidden);
    let oc = encode_ocdraw_document(&oc).unwrap();
    let mut ifc = load_ifccad_bytes(
        include_bytes!("../../../examples/ifccad/hello-viewports.ifcx"),
        Default::default(),
    )
    .unwrap()
    .document()
    .clone();
    for entity in &mut ifc.paper_layouts[0].entities {
        if let Some(e) = entity.as_native_mut() {
            if let IfccadEntityKind::Viewport(v) = &mut e.kind {
                v.plot_shading_override = Some(ShadedPlotMode::Hidden);
            }
        }
    }
    let ifc = encode_ifccad_document(&ifc).unwrap();
    for (name, bytes) in [
        ("presentation.ocdraw.json", oc.bytes()),
        ("presentation.ifcx", ifc.bytes()),
    ] {
        for format in ["dxf", "dwg"] {
            let result = export_drawing_bytes(name, bytes, format, "AC1032");
            assert!(result["failure"].is_null(), "{:?}", result["failure"]);
            let diagnostics = result["export"]["diagnostics"].as_array().unwrap();
            assert_eq!(
                diagnostics
                    .iter()
                    .filter(|d| d["code"] == "TARGET_CODEC_VIEWPORT_SHADEPLOT_LOSS")
                    .count(),
                usize::from(format == "dxf"),
                "{name}/{format}: {diagnostics:?}"
            );
        }
    }
}
