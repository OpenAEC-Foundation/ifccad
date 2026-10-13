use ocdraw::{color::foreground_srgb, ifccad::IfccadColor, ocdraw::DrawingColor};

#[test]
fn foreground_resolution_uses_effective_srgb_background() {
    for (background, expected) in [
        ([0, 0, 0], [255, 255, 255]),
        ([255, 255, 255], [0, 0, 0]),
        ([0, 0, 255], [255, 255, 255]),
        ([255, 255, 0], [0, 0, 0]),
        ([117, 117, 117], [255, 255, 255]),
        ([118, 118, 118], [0, 0, 0]),
    ] {
        assert_eq!(
            foreground_srgb(background),
            expected,
            "background {background:?}"
        );
    }
}

#[test]
fn metadata_builders_refuse_foreground() {
    assert!(DrawingColor::Foreground.with_indexed("ACI", 7).is_err());
    assert!(DrawingColor::Foreground.with_named("Book", "Name").is_err());
    assert!(IfccadColor::Foreground.with_indexed("ACI", 7).is_err());
    assert!(IfccadColor::Foreground.with_named("Book", "Name").is_err());
}

#[test]
fn authored_legacy_aci_seven_is_rejected_by_both_logical_validators() {
    use ocdraw::ocdraw::*;
    let mut b = OcdrawBuilder::new(OcdrawBuildOptions::new("legacy-color", "mm")).unwrap();
    let pattern = b.ensure_continuous_line_pattern().unwrap();
    b.add_layer(LayerDefinition::new(
        "0",
        DrawingColor::rgb(255, 255, 255),
        pattern,
    ))
    .unwrap();
    let mut d = b.build_document().unwrap();
    d.layers[0].color = DrawingColor::rgb(255, 255, 255)
        .with_indexed("aCi", 7)
        .unwrap();
    assert!(validate_ocdraw_document(&d).is_err());
    assert!(encode_ocdraw_document(&d).is_err());
    let mut ifc = ocdraw::ifccad::load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    ifc.layers[0].appearance.color = IfccadColor::rgb(0, 0, 0).with_indexed("ACI", 7).unwrap();
    assert!(ocdraw::ifccad::validate_ifccad_document(&ifc).is_err());
    assert!(ocdraw::ifccad::encode_ifccad_document(&ifc).is_err());
}
