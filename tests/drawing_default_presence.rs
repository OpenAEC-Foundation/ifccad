use ifccad as ifc;
use serde_json::{json, Value};

fn encode_checked(source: &Value, _ifccad: bool) -> Value {
    let bytes = serde_json::to_vec(source).unwrap();
    {
        let doc = ifc::load_ifccad_bytes(&bytes, Default::default())
            .unwrap()
            .into_document();
        let out = ifc::encode_ifccad_document(&doc).unwrap();
        assert_eq!(
            ifc::load_ifccad_bytes(out.bytes(), Default::default())
                .unwrap()
                .document(),
            &doc
        );
        serde_json::from_slice(out.bytes()).unwrap()
    }
}
fn load(path: &str) -> Value {
    serde_json::from_slice(
        &std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap(),
    )
    .unwrap()
}
fn attr<'a>(v: &'a Value, key: &str) -> &'a Value {
    &v["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["attributes"].get(key).is_some())
        .unwrap()["attributes"][key]
}

#[test]
fn registered_text_and_style_defaults_are_omitted() {
    let mut v = load("conformance/next/ifccad/valid/text-rich.ifcx");
    for node in v["data"].as_array_mut().unwrap() {
        for key in ["ifccad::text", "ifccad::mText"] {
            if let Some(text) = node["attributes"].get_mut(key) {
                text["rotation"] = json!(0.);
            }
        }
    }
    let out = encode_checked(&v, true);
    for kind in ["ifccad::text", "ifccad::mText"] {
        for key in ["rotation", "backward", "upsideDown"] {
            assert!(attr(&out, kind).get(key).is_none(), "{kind}.{key}");
        }
    }
    for key in [
        "widthFactor",
        "obliqueAngle",
        "vertical",
        "creationBackward",
        "creationUpsideDown",
    ] {
        assert!(attr(&out, "ifccad::textStyle").get(key).is_none(), "{key}");
    }
}

#[test]
fn marker_defaults_preserve_absolute_rotation() {
    {
        let (ifccad, path) = (true, "conformance/next/ifccad/valid/line-pattern-text.ifcx");
        let mut v = load(path);
        let pattern = {
            &mut v["data"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|n| n["attributes"]["ifccad::linePattern"]["name"] == "GAS")
                .unwrap()["attributes"]["ifccad::linePattern"]["pattern"]
        };
        pattern[1]["marker"]["scale"] = json!(1.);
        let out = encode_checked(&v, ifccad);
        let pattern = {
            &out["data"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["attributes"]["ifccad::linePattern"]["name"] == "GAS")
                .unwrap()["attributes"]["ifccad::linePattern"]["pattern"]
        };
        for key in ["scale", "offset", "rotation"] {
            assert!(pattern[1]["marker"].get(key).is_none(), "{key}");
        }
        let pattern = {
            &mut v["data"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|n| n["attributes"]["ifccad::linePattern"]["name"] == "GAS")
                .unwrap()["attributes"]["ifccad::linePattern"]["pattern"]
        };
        pattern[1]["marker"]["rotation"] = json!({"mode":"absolute","angle":0.});
        pattern[1]["marker"]["offset"] = json!([-0., 0.]);
        let out = encode_checked(&v, ifccad);
        let pattern = {
            &out["data"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["attributes"]["ifccad::linePattern"]["name"] == "GAS")
                .unwrap()["attributes"]["ifccad::linePattern"]["pattern"]
        };
        assert_eq!(pattern[1]["marker"]["rotation"]["mode"], "absolute");
        assert_eq!(
            pattern[1]["marker"]["offset"][0]
                .as_f64()
                .unwrap()
                .to_bits(),
            (-0.0_f64).to_bits()
        );
    }
}

#[test]
fn hatch_fill_defaults_are_omitted() {
    {
        let (ifccad, path) = (true, "conformance/next/ifccad/valid/hatch-pattern.ifcx");
        let mut v = load(path);
        let fill = {
            &mut v["data"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|n| n["attributes"].get("ifccad::hatch").is_some())
                .unwrap()["attributes"]["ifccad::hatch"]["fill"]
        };
        fill["origin"] = json!([0., 0.]);
        fill["rotation"] = json!(0.);
        fill["scale"] = json!(1.);
        let out = encode_checked(&v, ifccad);
        let fill = { &attr(&out, "ifccad::hatch")["fill"] };
        for key in ["origin", "rotation", "scale"] {
            assert!(fill.get(key).is_none(), "{key}");
        }
    }
}

#[test]
fn authored_presence_is_not_default_stripped() {
    for path in [
        "conformance/next/ifccad/valid/text-rich.ifcx",
        "conformance/next/ifccad/valid/hatch-gradient-inactive.ifcx",
        "conformance/next/ifccad/valid/opaque-preservation.ifcx",
    ] {
        encode_checked(&load(path), true);
    }
}

#[test]
fn opaque_appearance_does_not_inherit_native_default_omission() {
    let mut v = load("conformance/next/ifccad/valid/opaque-preservation.ifcx");
    let layer = v["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["attributes"].get("ifccad::layer").is_some())
        .unwrap()["path"]
        .clone();
    let opaque = &mut v["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"].get("ifccad::opaqueEntity").is_some())
        .unwrap()["attributes"]["ifccad::opaqueEntity"];
    opaque["nativeLayer"] = layer;
    opaque["nativeAppearance"] = json!({"appearance":{"color":{"mode":"ByLayer"},"opacity":{"mode":"ByLayer"},"linePattern":{"mode":"ByLayer"},"lineWeight":{"mode":"ByLayer"}},"linePatternScale":1.});
    encode_checked(&v, true);
    let opaque = &mut v["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"].get("ifccad::opaqueEntity").is_some())
        .unwrap()["attributes"]["ifccad::opaqueEntity"];
    opaque["nativeAppearance"]["appearance"]
        .as_object_mut()
        .unwrap()
        .remove("opacity");
    assert!(ifc::load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).is_err());
}
