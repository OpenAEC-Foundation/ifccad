use ocdraw::{geometry_kernel::hatch::*, ifccad::*, ocdraw::*};
use viewer::{export_drawing_bytes_with_options, export_ifccad_bytes_with_options};

fn bytes(ifccad: bool, inactive: bool) -> Vec<u8> {
    if ifccad {
        let mut d = load_ifccad_bytes(
            include_bytes!("../../../conformance/next/ifccad/valid/hatch-gradient.ifcx"),
            Default::default(),
        )
        .unwrap()
        .into_document();
        // Isolate file loss from the fixture's unrelated nonstandard lineweight.
        d.layers[0].appearance.line_weight = 0.09;
        if inactive {
            for e in &mut d.model.entities {
                if let Some(e) = e.as_native_mut() {
                    if let IfccadEntityKind::Hatch(h) = &mut e.kind {
                        h.fill = HatchFill::Solid;
                    }
                }
            }
        }
        encode_ifccad_document(&d).unwrap().bytes().to_vec()
    } else {
        let mut d = load_ocdraw_bytes(include_bytes!(
            "../../../conformance/next/ocdraw/valid/hatch-gradient.ocdraw.json"
        ))
        .unwrap()
        .into_document();
        if inactive {
            d.hatch_entities[0].fill = HatchFill::Solid;
        }
        encode_ocdraw_document(&d).unwrap().bytes().to_vec()
    }
}
fn export(ifccad: bool, b: &[u8], format: &str, version: &str, options: &str) -> serde_json::Value {
    if ifccad {
        export_ifccad_bytes_with_options("gradient.ifcx", b, format, version, options)
    } else {
        export_drawing_bytes_with_options("gradient.ocdraw.json", b, format, version, options)
    }
}
#[test]
fn actual_disabled_dxf_loss_is_visible_and_reject_never_offers_download() {
    for ifccad in [false, true] {
        let b = bytes(ifccad, true);
        let allowed = export(ifccad, &b, "dxf", "AC1032", "{}");
        assert!(allowed["failure"].is_null(), "{allowed}");
        assert!(
            allowed["export"]["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["code"] == "TARGET_CODEC_HATCH_GRADIENT_STATE_LOSS"),
            "{allowed}"
        );
        assert!(!allowed["export"]["download"].is_null());
        let rejected = export(ifccad, &b, "dxf", "AC1032", r#"{"lossPolicy":"reject"}"#);
        assert_eq!(
            rejected["failure"]["code"], "CAD_EXCHANGE_LOSS_REJECTED",
            "{rejected}"
        );
        assert!(rejected["export"]["download"].is_null());
    }
}
#[test]
fn only_qualified_ac1032_is_offered_for_active_and_inactive_gradient_state() {
    for ifccad in [false, true] {
        for inactive in [false, true] {
            let b = bytes(ifccad, inactive);
            for version in ["AC1015", "AC1018", "AC1021", "AC1024", "AC1027"] {
                let r = export(ifccad, &b, "dwg", version, "{}");
                assert_eq!(
                    r["failure"]["code"], "HATCH_GRADIENT_TARGET_VERSION_UNSUPPORTED",
                    "{r}"
                );
                assert!(r["export"]["download"].is_null());
            }
        }
    }
}
#[test]
fn explicit_allow_is_accepted_but_invalid_loss_policy_is_not() {
    let b = bytes(false, false);
    let r = export(false, &b, "dxf", "AC1032", r#"{"lossPolicy":"allow"}"#);
    assert!(r["failure"].is_null(), "{r}");
    assert!(!r["export"]["download"].is_null());
    for policy in [r#"{"lossPolicy":"unknown"}"#, r#"{"lossPolicy":null}"#] {
        let r = export(false, &b, "dxf", "AC1032", policy);
        assert_eq!(r["failure"]["code"], "INVALID_CONVERSION_OPTIONS");
    }
}
