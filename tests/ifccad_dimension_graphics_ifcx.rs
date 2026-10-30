#[path = "support/ifccad_dimension_graphics.rs"]
mod support;
use ifccad::*;
use serde_json::{json, Value};
fn wire() -> Value {
    serde_json::from_slice(encode_ifccad_document(&support::drawing()).unwrap().bytes()).unwrap()
}
fn read(v: &Value) -> Result<ValidatedIfccad, IfccadReadError> {
    load_ifccad_bytes(&serde_json::to_vec(v).unwrap(), Default::default())
}
fn group(v: &mut Value) -> &mut Value {
    v["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| n["attributes"].get("ifccad::dimensionGraphics").is_some())
        .unwrap()
}
fn node(v: &mut Value, i: usize) -> &mut Value {
    v["data"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|n| {
            n["path"]
                .as_str()
                .unwrap()
                .ends_with(&format!("/graphics/p{i}"))
        })
        .unwrap()
}
#[test]
fn graphics_ifcx_roundtrip_roles_and_order() {
    let mut d = support::drawing();
    let before = d.id_counters;
    let b = encode_ifccad_document(&d).unwrap();
    let r = load_ifccad_bytes(b.bytes(), Default::default())
        .unwrap()
        .into_document();
    assert_eq!(r, d);
    assert_eq!(r.id_counters, before);
    let mut v = wire();
    assert_eq!(
        group(&mut v)["children"]["10"],
        "/cad/d1/e9007199254740993/graphics/p10"
    );
    assert!(
        node(&mut v, 0)["attributes"]["ifccad::dimensionGraphicsPart"]
            .as_object()
            .unwrap()
            .is_empty()
    );
    for parts in [
        vec![],
        vec![support::part(IfccadDimensionGraphicsKind::Point {
            placement: support::dimension(&mut d).placement.clone(),
            thickness: 0.,
        })],
    ] {
        support::dimension(&mut d).graphics.as_mut().unwrap().parts = parts;
        let b = encode_ifccad_document(&d).unwrap();
        assert_eq!(
            load_ifccad_bytes(b.bytes(), Default::default())
                .unwrap()
                .document(),
            &d
        );
    }
}
#[test]
fn graphics_composition_does_not_delete_earlier_children() {
    let mut v = wire();
    let mut fragment = group(&mut v).clone();
    fragment["children"]
        .as_object_mut()
        .unwrap()
        .retain(|k, _| k == "0" || k == "1");
    fragment.as_object_mut().unwrap().remove("attributes");
    v["data"].as_array_mut().unwrap().push(fragment);
    let mut d = read(&v).unwrap().into_document();
    assert_eq!(
        support::dimension(&mut d)
            .graphics
            .as_ref()
            .unwrap()
            .parts
            .len(),
        12
    );
}
#[test]
fn graphics_nodes_require_exact_ownership() {
    let base = wire();
    for change in 0..11 {
        let mut v = base.clone();
        match change {
            0 => group(&mut v)["children"]["0"] = json!("/cad/d1/e1/graphics/p0"),
            1 => group(&mut v)["children"]["x"] = json!("/foreign"),
            2 => group(&mut v)["attributes"]["ifccad::geom::placement"] = json!({}),
            3 => node(&mut v, 0)["attributes"]["ifccad::entity"] = json!({}),
            4 => node(&mut v, 0)["attributes"]["ifccad::dimensionGraphicsPart"] = Value::Null,
            5 => {
                node(&mut v, 0)["attributes"]["ifccad::dimensionGraphicsPart"]["unknown"] = json!(1)
            }
            6 => {
                node(&mut v, 0)["attributes"]["ifccad::geom::placement"] =
                    json!({"origin":[0,0,0],"xAxis":[1,0,0],"yAxis":[0,1,0]})
            }
            7 => node(&mut v, 0)["attributes"]["ifccad::geom::circle"] = json!({"radius":1}),
            8 => {
                group(&mut v)["children"]
                    .as_object_mut()
                    .unwrap()
                    .remove("0");
            }
            9 => {
                let n = node(&mut v, 0).clone();
                let mut n = n;
                n["path"] = json!("/cad/d1/e1/graphics/p0");
                v["data"].as_array_mut().unwrap().push(n);
            }
            _ => node(&mut v, 0)["children"] = json!({"0":"/foreign"}),
        }
        assert!(read(&v).is_err(), "mutation {change} accepted");
    }
}
#[test]
fn graphics_source_keeps_full_width_references() {
    let mut d = support::drawing();
    let layer = d.layers[0].clone();
    let mut other = layer;
    other.id = u64::MAX - 1;
    other.name = "High".into();
    d.layers.push(other);
    d.id_counters.next_layer_id = u64::MAX;
    d.layers.sort_by_key(|l| l.id.to_string());
    let g = support::dimension(&mut d).graphics.as_mut().unwrap();
    g.parts[0].layer_id = Some(u64::MAX - 1);
    g.parts[1].layer_id = Some(0);
    let b = encode_ifccad_document(&d).unwrap();
    let loaded = load_ifccad_bytes(b.bytes(), Default::default()).unwrap();
    assert_eq!(loaded.document(), &d);
    let v: Value = serde_json::from_slice(b.bytes()).unwrap();
    let path = v["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["path"].as_str().unwrap().ends_with("/graphics/p0"))
        .unwrap();
    assert_eq!(
        path["attributes"]["ifccad::dimensionGraphicsPart"]["layer"],
        "/cad/d1/layer/18446744073709551614"
    );
}

#[test]
fn graphics_ifcx_reuses_every_allowed_payload_body() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../examples/ifccad/entities.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let mut parts = Vec::new();
    let mut kinds = std::collections::HashSet::new();
    for e in &d.model.entities {
        let Some(e) = e.as_native() else { continue };
        if let IfccadEntityKind::Hatch(h) = &e.kind {
            if !matches!(h.fill, geometry_kernel::hatch::HatchFill::Solid) {
                continue;
            }
        }
        if let Ok(kind) = IfccadDimensionGraphicsKind::try_from(e.kind.clone()) {
            kinds.insert(std::mem::discriminant(&kind));
            parts.push(support::part(kind));
        }
    }
    let idx = d
        .model
        .entities
        .iter()
        .position(|e| {
            matches!(
                e.as_native().map(|e| &e.kind),
                Some(IfccadEntityKind::Dimension(_))
            )
        })
        .unwrap();
    let IfccadEntityKind::Dimension(v) = &mut d.model.entities[idx].as_native_mut().unwrap().kind
    else {
        panic!()
    };
    assert_eq!(kinds.len(), 14);
    v.graphics = Some(IfccadDimensionGraphics {
        origin: IfccadDimensionGraphicsOrigin::Generated,
        parts,
    });
    // Stored bounds were computed without the new picture; recomputation belongs to the later bounds task.
    d.model.bounds = None;
    d.model.bounds_quality = None;
    let encoded = encode_ifccad_document(&d).unwrap();
    let loaded = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    let IfccadEntityKind::Dimension(v) = &loaded.document().model.entities[idx]
        .as_native()
        .unwrap()
        .kind
    else {
        panic!()
    };
    assert!(v.graphics.as_ref().unwrap().parts.len() >= 14);
}
