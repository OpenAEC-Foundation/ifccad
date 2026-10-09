use ocdraw::geometry_kernel::hatch::*;
use ocdraw::ocdraw::*;
#[test]
fn builder_rejects_invalid_hatch_without_consuming_an_identity() {
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("hatch-builder", "mm")).unwrap();
    let p = b
        .add_line_pattern(LinePatternDefinition {
            name: "Continuous".into(),
            description: None,
            pattern: vec![],
        })
        .unwrap();
    let layer = b
        .add_layer(LayerDefinition::new("0", RgbColor::new(255, 255, 255), p))
        .unwrap();
    let definition = |radius| HatchEntityDefinition {
        scope_id: 0,
        layer_id: layer,
        visible: true,
        appearance: Default::default(),
        placement: Default::default(),
        area_rule: HatchAreaRule::Normal,
        join_tolerance: DEFAULT_HATCH_JOIN_TOLERANCE,
        fill: HatchFill::Solid,
        loops: vec![OcdrawHatchLoop {
            boundary: HatchBoundary2::Circle {
                center: [0.0, 0.0],
                radius,
            },
            source_entity_id: None,
        }],
    };
    assert!(b.add_hatch(definition(0.0)).is_err());
    let mut invalid_pattern = definition(2.0);
    invalid_pattern.fill = HatchFill::LinePattern(HatchLinePattern {
        name: None,
        description: None,
        origin: [0., 0.],
        rotation: 0.,
        scale: 1.,
        families: vec![HatchLineFamily {
            angle: 0.,
            base_point: [0., 0.],
            offset: [0., 0.],
            dashes: vec![],
        }],
    });
    assert!(b.add_hatch(invalid_pattern).is_err());
    assert_eq!(b.add_hatch(definition(2.0)).unwrap(), 1);
    let doc = b.build_document().unwrap();
    assert_eq!(doc.scopes[0].entities, [1]);
    assert_eq!(doc.next_entity_id, 2);
    let output = encode_ocdraw_document(&doc).unwrap();
    assert_eq!(
        load_ocdraw_bytes(output.bytes()).unwrap().hatch_entities()[0].id,
        1
    );
}
