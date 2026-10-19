use ifccad::{color::foreground_srgb, IfccadColor};

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
    assert!(IfccadColor::Foreground.with_indexed("ACI", 7).is_err());
    assert!(IfccadColor::Foreground.with_named("Book", "Name").is_err());
}

#[test]
fn authored_legacy_aci_seven_is_rejected_by_both_logical_validators() {
    let mut ifc = ifccad::load_ifccad_bytes(
        include_bytes!("fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    ifc.layers[0].appearance.color = IfccadColor::rgb(0, 0, 0).with_indexed("ACI", 7).unwrap();
    assert!(ifccad::validate_ifccad_document(&ifc).is_err());
    assert!(ifccad::encode_ifccad_document(&ifc).is_err());
}
