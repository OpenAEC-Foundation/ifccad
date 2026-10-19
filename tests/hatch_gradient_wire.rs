use ifccad::{geometry_kernel::hatch::*, *};
#[path = "support/ifccad_hatch.rs"]
mod fixture;

fn state<C>(a: C, b: C) -> HatchGradient<C> {
    HatchGradient {
        gradient_type: Some(HatchGradientType::InverseSpherical),
        colors: Some([a, b]),
        rotation: 0.75,
        shift: 0.5,
        color_mode: HatchGradientColorMode::OneColor,
        tint: 0.8,
    }
}

fn ifccad_drawing() -> IfccadDocument {
    let mut d = fixture::drawing();
    let IfccadEntityKind::Hatch(h) = &mut d.model.entities[0].as_native_mut().unwrap().kind else {
        panic!()
    };
    h.fill = HatchFill::Gradient;
    h.gradient = Some(state(
        IfccadColor::rgb(220, 80, 30),
        IfccadColor::rgb(30, 60, 150),
    ));
    d
}

#[test]
fn both_native_routes_retain_active_parameters_and_partial_inactive_settings() {
    let mut i = ifccad_drawing();
    for active in [true, false] {
        if !active {
            let IfccadEntityKind::Hatch(h) = &mut i.model.entities[0].as_native_mut().unwrap().kind
            else {
                panic!()
            };
            h.fill = HatchFill::Solid;
            let g = h.gradient.as_mut().unwrap();
            g.gradient_type = None;
            g.colors = None;
        }

        let bytes = encode_ifccad_document(&i).unwrap();
        assert_eq!(
            load_ifccad_bytes(bytes.bytes(), Default::default())
                .unwrap()
                .document(),
            &i
        );
    }
}

#[test]
fn native_gradient_color_identity_keeps_full_u64_indices_and_named_metadata() {
    let mut i = ifccad_drawing();
    let IfccadEntityKind::Hatch(h) = &mut i.model.entities[0].as_native_mut().unwrap().kind else {
        panic!()
    };
    h.gradient.as_mut().unwrap().colors.as_mut().unwrap()[0] = IfccadColor::rgb(220, 80, 30)
        .with_indexed("Custom", u64::MAX)
        .unwrap()
        .with_named("Book", "Ink")
        .unwrap();
    let expected = h.gradient.clone();
    let b = encode_ifccad_document(&i).unwrap();
    let loaded = load_ifccad_bytes(b.bytes(), Default::default()).unwrap();
    let IfccadEntityKind::Hatch(h) = &loaded.document().model.entities[0]
        .as_native()
        .unwrap()
        .kind
    else {
        panic!()
    };
    assert_eq!(h.gradient, expected);
}

#[test]
fn closed_wire_states_reject_null_unknown_incomplete_and_invalid_values() {
    let i = ifccad_drawing();
    {
        let ifccad = true;
        let bytes = { encode_ifccad_document(&i).unwrap().bytes().to_vec() };
        for mutation in 0..10 {
            let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            let g = {
                &mut v["data"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|n| n["attributes"].get("ifccad::hatch").is_some())
                    .unwrap()["attributes"]["ifccad::hatch"]["gradient"]
            };
            match mutation {
                0 => *g = serde_json::Value::Null,
                1 => g["unknown"] = serde_json::json!(1),
                2 => g["type"] = serde_json::json!("unknown"),
                3 => {
                    g.as_object_mut().unwrap().remove("type");
                }
                4 => g["colors"] = serde_json::json!([{"rgb":[1,2,3]}]),
                5 => g["shift"] = serde_json::json!(1.1),
                6 => g["tint"] = serde_json::json!(-0.1),
                7 => g["rotation"] = serde_json::Value::Null,
                8 => g["colorMode"] = serde_json::json!("auto"),
                _ => g["colors"][0]["namedColor"] = serde_json::json!({"catalog":"","name":"bad"}),
            }
            let b = serde_json::to_vec(&v).unwrap();
            assert!(
                { load_ifccad_bytes(&b, Default::default()).is_err() },
                "route {ifccad}, mutation {mutation}"
            );
        }
    }
}

#[test]
fn gradient_endpoints_reject_symbolic_foreground_in_authored_and_wire_data() {
    for active in [true, false] {
        let mut i = ifccad_drawing();
        if !active {
            let IfccadEntityKind::Hatch(h) = &mut i.model.entities[0].as_native_mut().unwrap().kind
            else {
                panic!()
            };
            h.fill = HatchFill::Solid;
        }
        {
            let ifccad = true;
            let bytes = { encode_ifccad_document(&i).unwrap().bytes().to_vec() };
            let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            let g = {
                &mut v["data"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|n| n["attributes"].get("ifccad::hatch").is_some())
                    .unwrap()["attributes"]["ifccad::hatch"]["gradient"]
            };
            g["colors"][0] = serde_json::json!({"kind":"Foreground"});
            let b = serde_json::to_vec(&v).unwrap();
            assert!(
                { load_ifccad_bytes(&b, Default::default()).is_err() },
                "route {ifccad}, active {active}"
            );
        }
        let IfccadEntityKind::Hatch(h) = &mut i.model.entities[0].as_native_mut().unwrap().kind
        else {
            panic!()
        };
        h.gradient.as_mut().unwrap().colors.as_mut().unwrap()[0] = IfccadColor::Foreground;
        assert!(encode_ifccad_document(&i).is_err());
    }
}
