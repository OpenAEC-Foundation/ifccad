use ocdraw::{ifccad::*, ocdraw::*};
use viewer::{export_drawing_bytes_with_options, export_ifccad_bytes_with_options};

#[test]
fn unbound_switch_loss_is_visible_and_reject_never_offers_download() {
    for ifccad in [false, true] {
        let bytes = if ifccad {
            let mut d = load_ifccad_bytes(
                include_bytes!("../../../tests/fixtures/ifccad/hello-hatch-solid.ifcx"),
                Default::default(),
            )
            .unwrap()
            .into_document();
            d.layers[0].appearance.line_weight = 0.09;
            for e in &mut d.model.entities {
                if let Some(e) = e.as_native_mut() {
                    if let IfccadEntityKind::Hatch(h) = &mut e.kind {
                        h.association_enabled = true;
                        for l in &mut h.loops {
                            l.source_entity_ids.clear();
                            l.text_island = true;
                        }
                    }
                }
            }
            encode_ifccad_document(&d).unwrap().bytes().to_vec()
        } else {
            let mut d = load_ocdraw_bytes(include_bytes!(
                "../../../tests/fixtures/ocdraw/hello-hatch-solid.ocdraw.json"
            ))
            .unwrap()
            .into_document();
            d.hatch_entities[0].association_enabled = true;
            for l in &mut d.hatch_entities[0].loops {
                l.source_entity_ids.clear();
                l.text_island = true;
            }
            encode_ocdraw_document(&d).unwrap().bytes().to_vec()
        };
        for format in ["dxf", "dwg"] {
            let export = |options| {
                if ifccad {
                    export_ifccad_bytes_with_options(
                        "hatch.ifcx",
                        &bytes,
                        format,
                        "AC1032",
                        options,
                    )
                } else {
                    export_drawing_bytes_with_options(
                        "hatch.ocdraw.json",
                        &bytes,
                        format,
                        "AC1032",
                        options,
                    )
                }
            };
            let allowed = export("{}");
            assert!(allowed["failure"].is_null(), "{}", allowed["failure"]);
            assert!(
                allowed["export"]["diagnostics"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|d| d["code"] == "TARGET_CODEC_HATCH_ASSOCIATION_STATE_LOSS"),
                "{}",
                allowed["export"]["diagnostics"]
            );
            assert!(!allowed["export"]["download"].is_null());
            let rejected = export(r#"{"lossPolicy":"reject"}"#);
            assert_eq!(
                rejected["failure"]["code"], "CAD_EXCHANGE_LOSS_REJECTED",
                "{}",
                rejected["failure"]
            );
            assert!(rejected["export"]["download"].is_null());
        }
    }
}

#[test]
fn hatch_association_file_profile_is_explicitly_ac1032() {
    for ifccad in [false, true] {
        let bytes: &[u8] = if ifccad {
            include_bytes!("../../../conformance/next/ifccad/valid/hatch-associations-unbound.ifcx")
        } else {
            include_bytes!(
                "../../../conformance/next/ocdraw/valid/hatch-associations-unbound.ocdraw.json"
            )
        };
        let out = if ifccad {
            export_ifccad_bytes_with_options("hatch.ifcx", bytes, "dwg", "AC1027", "{}")
        } else {
            export_drawing_bytes_with_options("hatch.ocdraw.json", bytes, "dwg", "AC1027", "{}")
        };
        assert_eq!(
            out["failure"]["code"], "HATCH_ASSOCIATION_TARGET_VERSION_UNSUPPORTED",
            "{}",
            out["failure"]
        );
        assert!(out["export"]["download"].is_null());
    }
}
