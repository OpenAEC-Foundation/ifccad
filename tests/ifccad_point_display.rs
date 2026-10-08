use ocdraw::ifccad::*;
use serde_json::{json, Value};

fn fixture() -> Value {
    serde_json::from_slice(include_bytes!("../examples/ifccad/hello-cad.ifcx")).unwrap()
}
fn drawing(value: &mut Value) -> &mut Value {
    &mut value["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"].get("ifccad::drawing").is_some())
        .unwrap()["attributes"]["ifccad::drawing"]
}
fn display(glyph: &str, circle: bool, square: bool, size: Value) -> Value {
    json!({"form":{"glyph":glyph,"circle":circle,"square":square},"size":size})
}
fn read(value: &Value) -> Result<ValidatedIfccad, IfccadReadError> {
    load_ifccad_bytes(&serde_json::to_vec(value).unwrap(), Default::default())
}

#[test]
fn point_form_and_each_size_meaning_survive_production_native_readback() {
    for glyph in ["dot", "hidden", "plus", "cross", "shortLine"] {
        for circle in [false, true] {
            for square in [false, true] {
                for size in [
                    json!({"kind":"defaultFivePercent"}),
                    json!({"kind":"absolute","value":2.5}),
                    json!({"kind":"viewportPercent","value":5.0}),
                ] {
                    let mut value = fixture();
                    let expected = display(glyph, circle, square, size);
                    drawing(&mut value)["pointDisplay"] = expected.clone();
                    let loaded = read(&value).unwrap();
                    let encoded = encode_ifccad_document(loaded.document()).unwrap();
                    let restored = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
                    assert_eq!(restored.document(), loaded.document());
                    let mut written: Value = serde_json::from_slice(encoded.bytes()).unwrap();
                    assert_eq!(drawing(&mut written)["pointDisplay"], expected);
                }
            }
        }
    }
}

#[test]
fn point_display_is_closed_and_size_variants_are_unambiguous() {
    let good = display("plus", true, false, json!({"kind":"absolute","value":2.5}));
    let mut base = fixture();
    drawing(&mut base)["pointDisplay"] = good.clone();
    assert!(read(&base).is_ok());
    for bad in [
        Value::Null,
        json!({}),
        display(
            "unknown",
            false,
            false,
            json!({"kind":"defaultFivePercent"}),
        ),
        display("dot", false, false, json!({"kind":"absolute","value":0})),
        display("dot", false, false, json!({"kind":"absolute","value":-1})),
        display(
            "dot",
            false,
            false,
            json!({"kind":"viewportPercent","value":0}),
        ),
        display(
            "dot",
            false,
            false,
            json!({"kind":"viewportPercent","value":null}),
        ),
        display(
            "dot",
            false,
            false,
            json!({"kind":"defaultFivePercent","value":5}),
        ),
        json!({"form":{"glyph":"dot","circle":false,"square":false,"unknown":true},"size":{"kind":"defaultFivePercent"}}),
        json!({"form":{"glyph":"dot","circle":null,"square":false},"size":{"kind":"defaultFivePercent"}}),
        json!({"form":{"glyph":"dot","circle":false,"square":false},"size":{"kind":"defaultFivePercent"},"unknown":true}),
    ] {
        let mut value = base.clone();
        drawing(&mut value)["pointDisplay"] = bad.clone();
        assert!(read(&value).is_err(), "accepted {bad}");
    }
}

#[test]
fn presentation_size_does_not_change_geometry_bounds_or_order() {
    let mut value = fixture();
    let before = read(&value).unwrap().into_document();
    drawing(&mut value)["pointDisplay"] = display(
        "hidden",
        true,
        true,
        json!({"kind":"viewportPercent","value":200.0}),
    );
    let after = read(&value).unwrap().into_document();
    assert_eq!(before.model.bounds, after.model.bounds);
    assert_eq!(before.model.entities, after.model.entities);
}
