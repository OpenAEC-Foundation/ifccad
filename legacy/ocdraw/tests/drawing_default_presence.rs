use ocdraw::{ifccad as ifc, ocdraw as oc};
use serde_json::{json, Value};
#[path = "support/drawing_defaults.rs"]
mod support;

fn encode_checked(source: &Value, ifccad: bool) -> Value {
    let bytes = serde_json::to_vec(source).unwrap();
    if ifccad {
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
    } else {
        let doc = oc::load_ocdraw_bytes(&bytes).unwrap().into_document();
        let out = oc::encode_ocdraw_document(&doc).unwrap();
        support::assert_ocdraw_eq(
            &oc::load_ocdraw_bytes(out.bytes()).unwrap().into_document(),
            &doc,
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
    let mut v = load("conformance/next/ocdraw/valid/text-rich-content.ocdraw.json");
    v["textStyles"][0]["font"]["bold"] = json!(false);
    v["textStyles"][0]["font"]["italic"] = json!(false);
    let out = encode_checked(&v, false);
    for stream in ["textStream", "mTextStream"] {
        for key in ["rotation", "backward", "upsideDown"] {
            assert!(out["streams"][stream].get(key).is_none(), "{stream}.{key}");
        }
    }
    for key in ["obliqueAngle", "thickness"] {
        assert!(out["streams"]["textStream"].get(key).is_none());
    }
    for key in [
        "widthFactor",
        "obliqueAngle",
        "vertical",
        "creationBackward",
        "creationUpsideDown",
    ] {
        assert!(out["textStyles"][0].get(key).is_none(), "{key}");
    }
    assert_eq!(out["textStyles"][0]["font"]["bold"], json!(false));
    assert_eq!(out["textStyles"][0]["font"]["italic"], json!(false));

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
    for (ifccad, path) in [
        (
            false,
            "conformance/next/ocdraw/valid/line-pattern-text.ocdraw.json",
        ),
        (true, "conformance/next/ifccad/valid/line-pattern-text.ifcx"),
    ] {
        let mut v = load(path);
        let pattern = if ifccad {
            &mut v["data"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|n| n["attributes"]["ifccad::linePattern"]["name"] == "GAS")
                .unwrap()["attributes"]["ifccad::linePattern"]["pattern"]
        } else {
            &mut v["linePatterns"][1]["pattern"]
        };
        pattern[1]["marker"]["scale"] = json!(1.);
        let out = encode_checked(&v, ifccad);
        let pattern = if ifccad {
            &out["data"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["attributes"]["ifccad::linePattern"]["name"] == "GAS")
                .unwrap()["attributes"]["ifccad::linePattern"]["pattern"]
        } else {
            &out["linePatterns"][1]["pattern"]
        };
        for key in ["scale", "offset", "rotation"] {
            assert!(pattern[1]["marker"].get(key).is_none(), "{key}");
        }
        let pattern = if ifccad {
            &mut v["data"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|n| n["attributes"]["ifccad::linePattern"]["name"] == "GAS")
                .unwrap()["attributes"]["ifccad::linePattern"]["pattern"]
        } else {
            &mut v["linePatterns"][1]["pattern"]
        };
        pattern[1]["marker"]["rotation"] = json!({"mode":"absolute","angle":0.});
        pattern[1]["marker"]["offset"] = json!([-0., 0.]);
        let out = encode_checked(&v, ifccad);
        let pattern = if ifccad {
            &out["data"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| n["attributes"]["ifccad::linePattern"]["name"] == "GAS")
                .unwrap()["attributes"]["ifccad::linePattern"]["pattern"]
        } else {
            &out["linePatterns"][1]["pattern"]
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
    for (ifccad, path) in [
        (
            false,
            "conformance/next/ocdraw/valid/hatch-pattern.ocdraw.json",
        ),
        (true, "conformance/next/ifccad/valid/hatch-pattern.ifcx"),
    ] {
        let mut v = load(path);
        let fill = if ifccad {
            &mut v["data"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|n| n["attributes"].get("ifccad::hatch").is_some())
                .unwrap()["attributes"]["ifccad::hatch"]["fill"]
        } else {
            &mut v["streams"]["hatchStream"]["fill"][0]
        };
        fill["origin"] = json!([0., 0.]);
        fill["rotation"] = json!(0.);
        fill["scale"] = json!(1.);
        let out = encode_checked(&v, ifccad);
        let fill = if ifccad {
            &attr(&out, "ifccad::hatch")["fill"]
        } else {
            &out["streams"]["hatchStream"]["fill"][0]
        };
        for key in ["origin", "rotation", "scale"] {
            assert!(fill.get(key).is_none(), "{key}");
        }
    }
}

#[test]
fn authored_presence_is_not_default_stripped() {
    for path in [
        "conformance/next/ocdraw/valid/text-rich-content.ocdraw.json",
        "conformance/next/ocdraw/valid/hatch-gradient-inactive.ocdraw.json",
        "tests/fixtures/ocdraw/workspace-state.ocdraw.json",
        "tests/fixtures/ocdraw/preserved-spline.ocdraw.json",
        "tests/fixtures/ocdraw/layouts-viewports.ocdraw.json",
    ] {
        encode_checked(&load(path), false);
    }
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
