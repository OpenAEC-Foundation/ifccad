use ifccad::{geometry_kernel::hatch::*, *};
use ifccad_convert::*;
use opencadcodec::EntityType;
fn drawing() -> IfccadDocument {
    let mut d = load_ifccad_bytes(
        include_bytes!("../../../conformance/next/ifccad/valid/hatch-gradient.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    d.layers[0].appearance.line_weight = 0.09;
    d
}
fn hatch_mut(d: &mut IfccadDocument) -> &mut IfccadHatch {
    d.model
        .entities
        .iter_mut()
        .find_map(|e| {
            let IfccadEntityKind::Hatch(h) = &mut e.as_native_mut()?.kind else {
                return None;
            };
            Some(h)
        })
        .unwrap()
}
fn hatch(d: &IfccadDocument) -> &IfccadHatch {
    d.model
        .entities
        .iter()
        .find_map(|e| {
            let IfccadEntityKind::Hatch(h) = &e.as_native()?.kind else {
                return None;
            };
            Some(h)
        })
        .unwrap()
}
#[test]
fn gradient_and_partial_dormant_settings_survive_direct_and_encoded_routes() {
    for active in [true, false] {
        let mut d = drawing();
        if !active {
            let h = hatch_mut(&mut d);
            h.fill = HatchFill::Solid;
            let g = h.gradient.as_mut().unwrap();
            g.gradient_type = None;
            g.colors = None;
        }
        let expected = hatch(&d).gradient.clone();
        let b = encode_ifccad_document(&d).unwrap();
        let loaded = load_ifccad_bytes(b.bytes(), Default::default()).unwrap();
        let out = ifccad_source_to_cad_document(&loaded, Default::default()).unwrap();
        let h = out
            .document()
            .entities()
            .find_map(|e| {
                if let EntityType::Hatch(h) = e {
                    Some(h)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(h.gradient_color.enabled, active);
        assert_eq!(h.gradient_color.color_tint, 0.8);
        let back = cad_document_to_ifccad_document(
            out.document(),
            IfccadTargetMetadata {
                header: d.header,
                drawing_id: d.drawing_id,
            },
            Default::default(),
        )
        .unwrap();
        assert_eq!(hatch(back.document()).gradient, expected);
        encode_ifccad_document(back.document()).unwrap();
    }
}
#[test]
fn unqualified_frame_has_explicit_loss_and_reject_refuses() {
    for active in [true, false] {
        let mut d = drawing();
        let h = hatch_mut(&mut d);
        let a = 0.5_f64;
        h.placement.x_axis = [a.cos(), a.sin(), 0.];
        h.placement.y_axis = [-a.sin(), a.cos(), 0.];
        if !active {
            h.fill = HatchFill::Solid;
        }
        recompute_ifccad_document_bounds(&mut d).unwrap();
        let out = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
        assert_eq!(
            out.document()
                .entities()
                .filter(|e| matches!(e, EntityType::Hatch(_)))
                .count(),
            usize::from(!active)
        );
        assert!(out
            .diagnostics()
            .iter()
            .any(|e| e.code == "HATCH_GRADIENT_FRAME_UNSUPPORTED"));
        assert!(ifccad_document_to_cad_document(
            &d,
            IfccadToCadOptions {
                loss_policy: IfccadLossPolicy::Reject,
                ..Default::default()
            }
        )
        .is_err());
    }
}
#[test]
fn native_named_endpoint_keeps_rgb_with_located_target_model_loss() {
    let mut d = drawing();
    hatch_mut(&mut d)
        .gradient
        .as_mut()
        .unwrap()
        .colors
        .as_mut()
        .unwrap()[1] = IfccadColor::rgb(30, 60, 150)
        .with_named("Book", "Ink")
        .unwrap();
    let out = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    assert!(out
        .diagnostics()
        .iter()
        .any(|e| e.location.ends_with("gradient/colors/1") && e.code == "HATCH_GRADIENT_COLOR"));
    assert!(ifccad_document_to_cad_document(
        &d,
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        }
    )
    .is_err());
}
