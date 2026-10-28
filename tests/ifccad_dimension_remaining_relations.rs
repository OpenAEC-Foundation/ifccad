#[path = "support/ifccad_dimension.rs"]
mod support;
use ifccad::{dimension::*, *};
use serde_json::json;

fn circle_relation(selector: serde_json::Value) -> IfccadDocument {
    let mut d = support::drawing();
    let mut source = d.model.entities[0].as_native().unwrap().clone();
    source.id += 1;
    let IfccadEntityKind::Dimension(dimension) =
        &mut d.model.entities[0].as_native_mut().unwrap().kind
    else {
        panic!()
    };
    dimension.definition = serde_json::from_value(
        json!({"kind":"radius","center":[0.,0.,0.],"curvePoint":[3.,4.,0.],"leaderLength":0.}),
    )
    .unwrap();
    dimension.associations = vec![IfccadDimensionAssociation {
        role: DimensionRole::CurvePoint,
        source_entity_id: source.id,
        selector: serde_json::from_value(selector).unwrap(),
    }];
    source.kind = IfccadEntityKind::Circle {
        radius: 20.,
        placement: IfccadPlacement {
            origin: [0.; 3],
            x_axis: [1., 0., 0.],
            y_axis: [0., 1., 0.],
        },
    };
    d.model.entities.push(IfccadEntity::Native(source));
    d.id_counters.next_entity_id += 1;
    d
}

#[test]
fn stale_circular_relation_keeps_independent_definition() {
    let d = circle_relation(json!({"kind":"circlePoint","angle":0.25}));
    validate_ifccad_document(&d).unwrap();
    let IfccadEntityKind::Dimension(v) = &d.model.entities[0].as_native().unwrap().kind else {
        panic!()
    };
    assert_eq!(
        evaluate_dimension(&v.definition).unwrap(),
        DimensionMeasurement::Distance(5.)
    );
}

#[test]
fn arc_fraction_is_not_a_circle_angle() {
    let d = circle_relation(json!({"kind":"arcPoint","fraction":0.5}));
    assert!(validate_ifccad_document(&d).is_err());
}

#[test]
fn parameter_selectors_retain_explicit_wire_codes_and_strict_logical_readback() {
    let plane = IfccadPlacement {
        origin: [0.; 3],
        x_axis: [1., 0., 0.],
        y_axis: [0., 1., 0.],
    };
    for (selector, kind, literal) in [
        (
            json!({"kind":"circlePoint","angle":0.25}),
            IfccadEntityKind::Circle {
                placement: plane.clone(),
                radius: 20.,
            },
            vec![0xa2, 0, 5, 1, 0xa1, 0, 0xfb, 0x3f, 0xd0, 0, 0, 0, 0, 0, 0],
        ),
        (
            json!({"kind":"arcPoint","fraction":0.5}),
            IfccadEntityKind::Arc {
                placement: plane.clone(),
                radius: 20.,
                start_parameter: 0.,
                sweep_parameter: std::f64::consts::FRAC_PI_2,
            },
            vec![0xa2, 0, 6, 1, 0xa1, 0, 0xfb, 0x3f, 0xe0, 0, 0, 0, 0, 0, 0],
        ),
        (
            json!({"kind":"segmentPoint","index":0,"fraction":0.5}),
            IfccadEntityKind::PlanarPolyline {
                placement: plane.clone(),
                widths: IfccadPlanarPolylineWidths::Constant { width: 0. },
                vertices: vec![[0., 0.], [10., 0.], [10., 10.]],
                bulges: vec![1., 0., 1.],
                closed: false,
                line_pattern_generation: IfccadLinePatternGeneration::PerSegment,
            },
            vec![
                0xa2, 0, 7, 1, 0xa2, 0, 0, 1, 0xfb, 0x3f, 0xe0, 0, 0, 0, 0, 0, 0,
            ],
        ),
    ] {
        let mut drawing = circle_relation(selector.clone());
        drawing.model.entities[1].as_native_mut().unwrap().kind = kind;
        validate_ifccad_document(&drawing).unwrap();
        let encoded = encode_ifccad_cbor_document(&drawing).unwrap();
        assert_eq!(
            encoded
                .bytes()
                .windows(literal.len())
                .filter(|bytes| *bytes == literal.as_slice())
                .count(),
            1,
            "independent literal selector code/body"
        );
        let json = encode_ifccad_document(&drawing).unwrap();
        for returned in [
            load_ifccad_cbor_bytes(encoded.bytes(), Default::default())
                .unwrap()
                .into_document(),
            load_ifccad_bytes(json.bytes(), Default::default())
                .unwrap()
                .into_document(),
        ] {
            let IfccadEntityKind::Dimension(d) =
                &returned.model.entities[0].as_native().unwrap().kind
            else {
                panic!()
            };
            assert_eq!(
                serde_json::to_value(d.associations[0].selector).unwrap(),
                selector
            );
        }
    }
}

#[test]
fn closed_segment_is_eligible_but_open_dormant_segment_and_invalid_fractions_fail() {
    let mut drawing = circle_relation(json!({"kind":"segmentPoint","index":2,"fraction":1.}));
    drawing.model.entities[1].as_native_mut().unwrap().kind = IfccadEntityKind::PlanarPolyline {
        placement: IfccadPlacement {
            origin: [0.; 3],
            x_axis: [1., 0., 0.],
            y_axis: [0., 1., 0.],
        },
        widths: IfccadPlanarPolylineWidths::Constant { width: 0. },
        vertices: vec![[0., 0.], [10., 0.], [10., 10.]],
        bulges: vec![0., 0., 1.],
        closed: false,
        line_pattern_generation: IfccadLinePatternGeneration::PerSegment,
    };
    assert!(validate_ifccad_document(&drawing).is_err());
    let IfccadEntityKind::PlanarPolyline { closed, .. } =
        &mut drawing.model.entities[1].as_native_mut().unwrap().kind
    else {
        panic!()
    };
    *closed = true;
    validate_ifccad_document(&drawing).unwrap();
    let IfccadEntityKind::Dimension(d) =
        &mut drawing.model.entities[0].as_native_mut().unwrap().kind
    else {
        panic!()
    };
    d.associations[0].selector = DimensionPointSelector::SegmentPoint {
        index: 2,
        fraction: 1.0000000000000002,
    };
    assert!(validate_ifccad_document(&drawing).is_err());
}
