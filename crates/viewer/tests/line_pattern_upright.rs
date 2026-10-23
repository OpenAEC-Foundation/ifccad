use base64::{engine::general_purpose::STANDARD, Engine};
use ifccad::*;
use ifccad_convert::opencadcodec::{CadDocument, DwgReader, DxfReader};
use serde_json::Value;
use std::io::Cursor;

fn drawing() -> Vec<u8> {
    let mut value: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/fixtures/ifccad/complex-line-patterns.ifcx"
    ))
    .unwrap();
    for node in value["data"].as_array_mut().unwrap() {
        if let Some(elements) = node
            .get_mut("attributes")
            .and_then(|a| a.get_mut("ifccad::linePattern"))
            .and_then(|p| p.get_mut("pattern"))
            .and_then(Value::as_array_mut)
        {
            for element in elements {
                if element.get("marker").is_some() {
                    element["marker"]["rotation"] =
                        serde_json::json!({"mode":"upright","angle":0.25});
                }
            }
        }
    }
    let checked =
        load_ifccad_bytes(&serde_json::to_vec(&value).unwrap(), Default::default()).unwrap();
    encode_ifccad_document(checked.document())
        .unwrap()
        .bytes()
        .to_vec()
}
fn read(format: &str, bytes: Vec<u8>) -> CadDocument {
    if format == "dwg" {
        DwgReader::from_stream(Cursor::new(bytes)).read().unwrap()
    } else {
        DxfReader::from_reader(Cursor::new(bytes))
            .unwrap()
            .read()
            .unwrap()
    }
}
#[test]
fn upright_export_checks_the_dxf_boundary_and_dwg_with_explicit_losses() {
    let bytes = drawing();
    let inspected = viewer::inspect_ifccad_bytes("upright.ifcx", &bytes);
    assert!(inspected["failure"].is_null(), "{inspected}");
    let preflight = inspected["presentation"]["linePatternExportPreflight"]
        .as_array()
        .unwrap();
    for (format, version, retained) in [
        ("dxf", "AC1024", false),
        ("dxf", "AC1027", true),
        ("dwg", "AC1024", true),
    ] {
        let restrictions = &preflight
            .iter()
            .find(|p| p["format"] == format && p["version"] == version)
            .unwrap()["restrictions"];
        assert_eq!(!restrictions.as_array().unwrap().is_empty(), !retained);
        let out =
            viewer::export_ifccad_bytes_with_options("upright.ifcx", &bytes, format, version, "{}");
        assert!(out["failure"].is_null(), "{out}");
        let losses = out["export"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|d| d["code"] == "TARGET_CODEC_LINE_PATTERN_UPRIGHT_LOSS")
            .collect::<Vec<_>>();
        assert_eq!(!losses.is_empty(), !retained, "{out}");
        for loss in losses {
            assert!(loss["location"].as_str().unwrap().ends_with("/rotation"));
            assert_eq!(loss["expected"], "upright");
            assert_eq!(loss["actual"], "relative");
        }
        let actual = read(
            format,
            STANDARD
                .decode(out["export"]["download"]["base64"].as_str().unwrap())
                .unwrap(),
        );
        let markers = actual
            .line_types
            .iter()
            .flat_map(|p| &p.elements)
            .filter_map(|e| e.complex.as_ref())
            .collect::<Vec<_>>();
        assert!(!markers.is_empty());
        for marker in markers {
            assert_eq!(marker.upright, retained);
            assert!(!marker.absolute_rotation);
        }
        assert_eq!(out["export"]["fileCheck"]["ifccadStrictReadback"], true);
        let reject = viewer::export_ifccad_bytes_with_options(
            "upright.ifcx",
            &bytes,
            format,
            version,
            r#"{"lossPolicy":"reject"}"#,
        );
        if retained {
            assert!(reject["failure"].is_null(), "{reject}");
        } else {
            assert_eq!(reject["failure"]["code"], "CAD_EXCHANGE_LOSS_REJECTED");
            assert!(reject["export"].get("download").is_none());
        }
    }
}
