use ocdraw::geometry_kernel::hatch::*;
use ocdraw::ocdraw::*;
use ocdraw_convert::{ocdraw_document_to_cad_document, OcdrawToCadOptions};
#[test]
fn native_hatch_is_materialized_with_stored_contours() {
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("hatch-converter-guard", "mm")).unwrap();
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
    b.add_hatch(HatchEntityDefinition {
        scope_id: 0,
        layer_id: layer,
        visible: true,
        appearance: Default::default(),
        placement: Default::default(),
        loops: vec![OcdrawHatchLoop {
            boundary: HatchBoundary2::Circle {
                center: [0.0, 0.0],
                radius: 1.0,
            },
            source_entity_id: None,
        }],
        area_rule: HatchAreaRule::Normal,
        join_tolerance: DEFAULT_HATCH_JOIN_TOLERANCE,
        fill: HatchFill::Solid,
    })
    .unwrap();
    let d = b.build_document().unwrap();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ocdraw_document_to_cad_document(&d, OcdrawToCadOptions::default())
    }))
    .expect("recognized native types must not panic in conversion");
    let out = result.unwrap();
    assert_eq!(
        out.document()
            .entities()
            .filter(|e| matches!(e, opencadcodec::EntityType::Hatch(_)))
            .count(),
        1
    );
    assert!(!out.geometry_assessment().is_complete());
}
