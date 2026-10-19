use ocdraw::{
    geometry_kernel::{hatch::*, CoordinateFrame3, Point3, Vector3},
    ocdraw::*,
};
use ocdraw_convert::*;
use opencadcodec::EntityType;
fn drawing() -> OcdrawDocument {
    load_ocdraw_bytes(include_bytes!(
        "../../../conformance/next/ocdraw/valid/hatch-gradient.ocdraw.json"
    ))
    .unwrap()
    .into_document()
}
fn rotated() -> CoordinateFrame3 {
    let a = 0.5_f64;
    CoordinateFrame3::try_new(
        Point3::new(0., 0., 0.),
        Vector3::new(a.cos(), a.sin(), 0.),
        Vector3::new(-a.sin(), a.cos(), 0.),
    )
    .unwrap()
}
#[test]
fn gradient_and_dormant_authorship_survive_native_and_cad_model_routes() {
    for active in [true, false] {
        let mut d = drawing();
        if !active {
            d.hatch_entities[0].fill = HatchFill::Solid;
            let g = d.hatch_entities[0].gradient.as_mut().unwrap();
            g.gradient_type = None;
            g.colors = None;
        }
        let expected = d.hatch_entities[0].gradient.clone();
        let encoded = encode_ocdraw_document(&d).unwrap();
        let native = load_ocdraw_bytes(encoded.bytes()).unwrap();
        let out = ocdraw_source_to_cad_document(&native, Default::default()).unwrap();
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
        assert_eq!(h.gradient_color.angle, 0.75);
        let back = cad_document_to_ocdraw_document(out.document(), Default::default()).unwrap();
        assert_eq!(back.document().hatch_entities[0].gradient, expected);
        encode_ocdraw_document(back.document()).unwrap();
    }
}
#[test]
fn unsupported_gradient_frames_omit_whole_active_entity_but_only_dormant_state() {
    for active in [true, false] {
        let mut d = drawing();
        d.hatch_entities[0].placement = rotated();
        if !active {
            d.hatch_entities[0].fill = HatchFill::Solid;
        }
        recompute_ocdraw_document_bounds(&mut d).unwrap();
        let out = ocdraw_document_to_cad_document(&d, Default::default()).unwrap();
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
        assert!(ocdraw_document_to_cad_document(
            &d,
            OcdrawToCadOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        )
        .is_err());
    }
}
#[test]
fn named_identity_loss_is_located_while_concrete_rgb_survives() {
    let mut d = drawing();
    d.hatch_entities[0]
        .gradient
        .as_mut()
        .unwrap()
        .colors
        .as_mut()
        .unwrap()[1] = DrawingColor::rgb(30, 60, 150)
        .with_named("Book", "Ink")
        .unwrap();
    let out = ocdraw_document_to_cad_document(&d, Default::default()).unwrap();
    assert!(out
        .diagnostics()
        .iter()
        .any(|e| e.location.ends_with("gradient/colors/1") && e.code == "HATCH_GRADIENT_COLOR"));
    assert!(ocdraw_document_to_cad_document(
        &d,
        OcdrawToCadOptions {
            loss_policy: OcdrawLossPolicy::Reject,
            ..Default::default()
        }
    )
    .is_err());
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
    assert_eq!(h.gradient_color.colors[1].color.rgb(), Some((30, 60, 150)));
}

#[test]
fn unrepresentable_hatch_numeric_state_keeps_the_typed_preparation_cause() {
    let mut d = drawing();
    d.hatch_entities[0].loops[0].boundary = HatchBoundary2::Ellipse {
        center: [0., 0.],
        x_axis: [1., 0.],
        semi_major_radius: 1e300,
        semi_minor_radius: 1e-300,
    };
    recompute_ocdraw_document_bounds(&mut d).unwrap();
    assert!(matches!(
        ocdraw_document_to_cad_document(&d, Default::default()),
        Err(OcdrawToCadError::HatchPreparation { .. })
    ));
}
