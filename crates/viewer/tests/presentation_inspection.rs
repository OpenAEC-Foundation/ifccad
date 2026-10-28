use viewer::inspect_ifccad_bytes;

#[test]
fn ifccad_inspection_exposes_native_presentation_and_point_display() {
    use ifccad::*;
    let mut d = load_ifccad_bytes(
        include_bytes!("../../../tests/fixtures/ifccad/hello-viewports.ifcx"),
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
        .unwrap()
        .with_named("Book", "Red")
        .unwrap();
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
    v.plot_shading_override = Some(ifccad::plot_kernel::ShadedPlotMode::Hidden);
    v.layer_overrides = vec![IfccadViewportLayerOverride {
        layer_id: layer,
        frozen: true,
        color: Some(IfccadColor::rgb(255, 0, 0).with_indexed("ACI", 1).unwrap()),
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
    let result = inspect_ifccad_bytes("presentation.ifcx", bytes.bytes());
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
