#[path = "support/ifccad_dimension.rs"]
mod support;
use ifccad::{dimension::*, *};
use serde_json::json;

#[test]
fn new_profile_writer_uses_candidate_identifiers_and_strict_readback() {
    let mut d = support::drawing();
    let IfccadEntityKind::Dimension(v) = &mut d.model.entities[0].as_native_mut().unwrap().kind
    else {
        panic!()
    };
    v.definition = serde_json::from_value(
        json!({"kind":"radius","center":[0.,0.,7.],"curvePoint":[3.,4.,9.],"leaderLength":0.}),
    )
    .unwrap();
    let encoded = encode_ifccad_document(&d).unwrap();
    let raw: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    assert!(raw["imports"]
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x["uri"] == "urn:example:ifccad:0.3.0"));
    let loaded = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    validate_ifccad_document(loaded.document()).unwrap();
    let IfccadEntityKind::Dimension(returned) = &loaded.document().model.entities[0]
        .as_native()
        .unwrap()
        .kind
    else {
        panic!()
    };
    assert_eq!(
        evaluate_dimension(&returned.definition).unwrap(),
        DimensionMeasurement::Distance(5.)
    );
}

#[test]
fn every_new_kind_survives_json_and_cbor_with_independent_reference_heights() {
    let cases = [
        json!({"kind":"radius","center":[0.,0.,7.],"curvePoint":[3.,4.,9.],"leaderLength":0.}),
        json!({"kind":"diameter","firstDiameterPoint":[-3.,-4.,7.],"secondDiameterPoint":[3.,4.,9.],"leaderLength":0.}),
        json!({"kind":"angularTwoLines","line1Start":[0.,0.,7.],"line1End":[1.,0.,8.],"line2Start":[0.,0.,9.],"line2End":[0.,1.,10.],"ray1Sign":"positive","ray2Sign":"positive","traversal":"counterclockwise","dimensionArcPoint":[5.,5.]}),
        json!({"kind":"angularThreePoints","vertex":[0.,0.,7.],"firstPoint":[1.,0.,8.],"secondPoint":[0.,1.,9.],"traversal":"clockwise","dimensionArcPoint":[5.,5.]}),
        json!({"kind":"ordinate","origin":[10.,20.,3.],"featurePoint":[7.,24.,8.],"axisAngle":0.,"leaderEnd":[20.,40.],"valueMode":"signed"}),
        json!({"kind":"arcLength","center":[0.,0.,7.],"radius":100.,"startParameter":0.,"sweep":{"kind":"full","traversal":"clockwise"},"dimensionArcPoint":[120.,0.],"leader":{"start":[100.,0.],"end":[120.,0.]}}),
        json!({"kind":"joggedRadius","center":[0.,0.,7.],"curvePoint":[3.,4.,9.],"displayCenter":[100.,200.],"jogPoint":[50.,50.],"jogAngle":0.5}),
    ];
    for case in cases {
        let mut d = support::drawing();
        let expected: DimensionDefinition = serde_json::from_value(case).unwrap();
        let IfccadEntityKind::Dimension(v) = &mut d.model.entities[0].as_native_mut().unwrap().kind
        else {
            panic!()
        };
        v.definition = expected.clone();
        let json = encode_ifccad_document(&d).unwrap();
        let binary = encode_ifccad_cbor_document(&d).unwrap();
        let mut envelope = minicbor::Decoder::new(binary.bytes());
        assert_eq!(envelope.map().unwrap(), Some(3));
        assert_eq!(envelope.u64().unwrap(), 0);
        assert_eq!(envelope.str().unwrap(), "IFCCAD-CBOR");
        assert_eq!(envelope.u64().unwrap(), 1);
        assert_eq!(envelope.u64().unwrap(), 3);
        for returned in [
            load_ifccad_bytes(json.bytes(), Default::default())
                .unwrap()
                .into_document(),
            load_ifccad_cbor_bytes(binary.bytes(), Default::default())
                .unwrap()
                .into_document(),
        ] {
            validate_ifccad_document(&returned).unwrap();
            let IfccadEntityKind::Dimension(v) =
                &returned.model.entities[0].as_native().unwrap().kind
            else {
                panic!()
            };
            assert_eq!(v.definition, expected);
        }
    }
}
