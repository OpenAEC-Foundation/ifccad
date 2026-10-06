use ocdraw::ifccad::*;
use serde_json::{json, Value};

fn base() -> IfccadDocument {
    load_ifccad_bytes(
        include_bytes!("../examples/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document()
}
fn frame() -> IfccadPlacement {
    IfccadPlacement {
        origin: [1., 2., 3.],
        x_axis: [1., 0., 0.],
        y_axis: [0., 1., 0.],
    }
}
fn kinds() -> Vec<IfccadEntityKind> {
    vec![
        IfccadEntityKind::Point { placement: frame() },
        IfccadEntityKind::Arc {
            radius: 2.,
            start_parameter: 0.25,
            sweep_parameter: -1.,
            placement: frame(),
        },
        IfccadEntityKind::Ellipse {
            semi_major_radius: 3.,
            semi_minor_radius: 2.,
            placement: frame(),
        },
        IfccadEntityKind::EllipseArc {
            semi_major_radius: 3.,
            semi_minor_radius: 2.,
            start_parameter: 0.25,
            sweep_parameter: 1.,
            placement: frame(),
        },
        IfccadEntityKind::PlanarPolyline {
            vertices: vec![[0., 0.], [2., 0.], [2., 2.]],
            bulges: vec![1., -0.25, 7.],
            closed: false,
            placement: frame(),
            line_pattern_generation: IfccadLinePatternGeneration::Continuous,
        },
        IfccadEntityKind::SpatialPolyline {
            vertices: vec![[0., 0., 0.], [1., 2., 3.], [1., 2., 3.]],
            closed: true,
            line_pattern_generation: IfccadLinePatternGeneration::Continuous,
        },
    ]
}
#[test]
fn new_families_preserve_parameters_in_every_owner() {
    let mut d = base();
    let template = d.model.entities[0].clone();
    d.id_counters.next_entity_id = 9_007_199_254_740_993;
    let mut entities = Vec::new();
    for kind in kinds() {
        let mut e = template.clone();
        e.id = d.id_counters.allocate_entity_id().unwrap();
        e.kind = kind;
        entities.push(e);
    }
    d.model.entities.extend(entities);
    let layout_id = d.id_counters.allocate_layout_id().unwrap();
    let mut paper = IfccadPaperLayout {
        bounds: None,
        id: layout_id,
        name: "Geometry sheet".into(),
        tab_index: (d.paper_layouts.len() + 1) as u32,
        length_unit: "mm".into(),
        paper: None,
        entities: vec![],
    };
    for kind in kinds() {
        let mut e = template.clone();
        e.id = d.id_counters.allocate_entity_id().unwrap();
        e.kind = kind;
        paper.entities.push(e);
    }
    d.paper_layouts.push(paper);
    let id = d.id_counters.allocate_block_id().unwrap();
    let mut block = IfccadBlockDefinition {
        bounds: None,
        id,
        name: "Geometry block".into(),
        base_point: [4., 5., 0.],
        insertion_unit: "mm".into(),
        entities: vec![],
    };
    for kind in kinds() {
        let mut e = template.clone();
        e.id = d.id_counters.allocate_entity_id().unwrap();
        e.kind = kind;
        block.entities.push(e);
    }
    d.blocks.push(block);
    validate_ifccad_document(&d).unwrap();
    let encoded = encode_ifccad_document(&d).unwrap();
    let readback = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    assert_eq!(readback.document(), &d);
    assert!(readback
        .document()
        .model
        .entities
        .iter()
        .any(|e| e.id == 9_007_199_254_740_993));
}
fn polyline() -> (IfccadDocument, u64) {
    let mut d = base();
    let mut e = d.model.entities[0].clone();
    e.id = d.id_counters.allocate_entity_id().unwrap();
    e.kind = kinds().remove(4);
    let id = e.id;
    d.model.entities.push(e);
    (d, id)
}
#[test]
fn polyline_wire_defaults_and_dormant_values_are_explicit() {
    let (mut d, id) = polyline();
    let raw: Value = serde_json::from_slice(encode_ifccad_document(&d).unwrap().bytes()).unwrap();
    let node = raw["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["path"] == format!("/cad/d{}/e{id}", d.drawing_id))
        .unwrap();
    assert_eq!(
        node["attributes"]["ifccad::geom::planarPolyline"]["bulges"],
        json!([1., -0.25, 7.])
    );
    let IfccadEntityKind::PlanarPolyline { bulges, .. } =
        &mut d.model.entities.last_mut().unwrap().kind
    else {
        panic!()
    };
    *bulges = vec![0.; 3];
    let raw: Value = serde_json::from_slice(encode_ifccad_document(&d).unwrap().bytes()).unwrap();
    let node = raw["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["path"] == format!("/cad/d{}/e{id}", d.drawing_id))
        .unwrap();
    assert!(node["attributes"]["ifccad::geom::planarPolyline"]
        .get("bulges")
        .is_none());
    assert_eq!(
        load_ifccad_bytes(&serde_json::to_vec(&raw).unwrap(), Default::default())
            .unwrap()
            .document(),
        &d
    );
}
#[test]
fn malformed_curve_payloads_fail_both_authored_and_wire_validation() {
    let (mut d, id) = polyline();
    let good = encode_ifccad_document(&d).unwrap();
    let IfccadEntityKind::PlanarPolyline { bulges, .. } =
        &mut d.model.entities.last_mut().unwrap().kind
    else {
        panic!()
    };
    bulges.pop();
    assert!(validate_ifccad_document(&d).is_err());
    for bad in [json!([1.]), Value::Null, json!([1., 0., 0., 0.])] {
        let mut raw: Value = serde_json::from_slice(good.bytes()).unwrap();
        let node = raw["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["path"] == format!("/cad/d{}/e{id}", d.drawing_id))
            .unwrap();
        node["attributes"]["ifccad::geom::planarPolyline"]["bulges"] = bad;
        assert!(load_ifccad_bytes(&serde_json::to_vec(&raw).unwrap(), Default::default()).is_err());
    }
}

#[test]
fn arc_ranges_and_unknown_core_fields_are_rejected() {
    let mut d = base();
    for sweep in [0., std::f64::consts::TAU, -std::f64::consts::TAU, f64::NAN] {
        d.model.entities[0].kind = IfccadEntityKind::Arc {
            radius: 1.,
            start_parameter: 0.,
            sweep_parameter: sweep,
            placement: frame(),
        };
        assert!(validate_ifccad_document(&d).is_err());
    }
    d.model.entities[0].kind = IfccadEntityKind::Point { placement: frame() };
    let mut raw: Value =
        serde_json::from_slice(encode_ifccad_document(&d).unwrap().bytes()).unwrap();
    let node = raw["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["path"] == format!("/cad/d{}/e{}", d.drawing_id, d.model.entities[0].id))
        .unwrap();
    node["attributes"]["ifccad::geom::point"]["position"] = json!([1, 2, 3]);
    assert!(load_ifccad_bytes(&serde_json::to_vec(&raw).unwrap(), Default::default()).is_err());
}
