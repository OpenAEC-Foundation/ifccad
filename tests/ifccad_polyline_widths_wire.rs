use ifccad::*;
use serde_json::json;
fn document() -> IfccadDocument {
    load_ifccad_bytes(
        include_bytes!("../conformance/next/ifccad/valid/polyline-defaults-overrides.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document()
}
fn widths(d: &IfccadDocument) -> &IfccadPlanarPolylineWidths {
    d.model
        .entities
        .iter()
        .filter_map(IfccadEntity::as_native)
        .find_map(|e| {
            if let IfccadEntityKind::PlanarPolyline { widths, .. } = &e.kind {
                Some(widths)
            } else {
                None
            }
        })
        .unwrap()
}
fn widths_mut(d: &mut IfccadDocument) -> &mut IfccadPlanarPolylineWidths {
    d.model
        .entities
        .iter_mut()
        .filter_map(IfccadEntity::as_native_mut)
        .find_map(|e| {
            if let IfccadEntityKind::PlanarPolyline { widths, .. } = &mut e.kind {
                Some(widths)
            } else {
                None
            }
        })
        .unwrap()
}
fn reopen(d: &IfccadDocument, cbor: bool) -> IfccadDocument {
    if cbor {
        let e = encode_ifccad_cbor_document(d).unwrap();
        load_ifccad_cbor_bytes(e.bytes(), Default::default())
            .unwrap()
            .into_document()
    } else {
        let e = encode_ifccad_document(d).unwrap();
        load_ifccad_bytes(e.bytes(), Default::default())
            .unwrap()
            .into_document()
    }
}
#[test]
fn default_edit_after_native_reload_preserves_authored_state() {
    for cbor in [false, true] {
        let d = document();
        let mut back = reopen(&d, cbor);
        assert_eq!(d, back);
        if let IfccadPlanarPolylineWidths::Defaulted {
            default_start_width,
            default_end_width,
            ..
        } = widths_mut(&mut back)
        {
            *default_start_width = 0.6;
            *default_end_width = 0.8
        } else {
            panic!("mode lost")
        }
        let w = widths(&back);
        for (i, pair) in [[0.6, 0.8], [0., 0.8], [0.6, 0.], [0.2, 0.4]]
            .into_iter()
            .enumerate()
        {
            assert_eq!(w.effective_at(i, 4).unwrap(), pair)
        }
        assert!(w.effective_at(4, 4).is_err());
    }
}
#[test]
fn all_native_modes_preserve_signed_zero_subnormal_and_dormant_bits() {
    for cbor in [false, true] {
        for v in [-0.0, 0.0, f64::from_bits(1), 0.15] {
            for w in [
                IfccadPlanarPolylineWidths::Constant { width: v },
                IfccadPlanarPolylineWidths::PerVertex {
                    widths: vec![[v, v]; 4],
                },
                IfccadPlanarPolylineWidths::Defaulted {
                    default_start_width: v,
                    default_end_width: v,
                    overrides: vec![
                        IfccadPolylineVertexWidthOverride {
                            start: Some(v),
                            end: None
                        };
                        4
                    ],
                },
            ] {
                let mut d = document();
                *widths_mut(&mut d) = w.clone();
                let back = reopen(&d, cbor);
                assert_eq!(widths(&back), &w);
                for i in 0..4 {
                    assert_eq!(
                        widths(&back).effective_at(i, 4).unwrap().map(f64::to_bits),
                        [v.to_bits(); 2]
                    );
                }
                if let IfccadPlanarPolylineWidths::Defaulted { overrides, .. } = widths(&back) {
                    assert_eq!(overrides[3].start.unwrap().to_bits(), v.to_bits());
                    assert!(overrides[3].end.is_none());
                }
            }
        }
    }
}
#[test]
fn width_records_are_closed_required_and_not_nullable() {
    let d = document();
    let encoded = encode_ifccad_document(&d).unwrap();
    let base: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    for invalid in [
        json!(null),
        json!({"kind":"unknown"}),
        json!({"kind":"constant"}),
        json!({"kind":"constant","width":0,"other":1}),
        json!({"kind":"perVertex","widths":[[0]]}),
        json!({"kind":"defaulted","defaultStartWidth":0,"overrides":[{},{},{},{}]}),
        json!({"kind":"defaulted","defaultStartWidth":0,"defaultEndWidth":0,"overrides":[{"start":null},{},{},{}]}),
        json!({"kind":"defaulted","defaultStartWidth":0,"defaultEndWidth":0,"overrides":[{"other":0},{},{},{}]}),
    ] {
        let mut graph = base.clone();
        let node = graph["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| {
                n["attributes"]
                    .get("ifccad::geom::planarPolyline")
                    .is_some()
            })
            .unwrap();
        node["attributes"]["ifccad::geom::planarPolyline"]["widths"] = invalid;
        assert!(
            load_ifccad_bytes(&serde_json::to_vec(&graph).unwrap(), Default::default()).is_err()
        );
    }
}
#[test]
fn ordinary_and_graphics_share_width_validation() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../conformance/next/ifccad/valid/dimension-graphics.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let kind = document()
        .model
        .entities
        .iter()
        .filter_map(IfccadEntity::as_native)
        .find(|e| matches!(e.kind, IfccadEntityKind::PlanarPolyline { .. }))
        .unwrap()
        .kind
        .clone();
    let dim = d
        .model
        .entities
        .iter_mut()
        .filter_map(IfccadEntity::as_native_mut)
        .find_map(|e| {
            if let IfccadEntityKind::Dimension(d) = &mut e.kind {
                Some(d)
            } else {
                None
            }
        })
        .unwrap();
    let part = dim
        .graphics
        .as_mut()
        .unwrap()
        .parts
        .iter_mut()
        .find(|p| p.purpose == IfccadDimensionGraphicsPurpose::Display)
        .unwrap();
    part.kind = kind.try_into().unwrap();
    for cbor in [false, true] {
        assert_eq!(reopen(&d, cbor), d);
    }
    let dim = d
        .model
        .entities
        .iter_mut()
        .filter_map(IfccadEntity::as_native_mut)
        .find_map(|e| {
            if let IfccadEntityKind::Dimension(d) = &mut e.kind {
                Some(d)
            } else {
                None
            }
        })
        .unwrap();
    if let IfccadDimensionGraphicsKind::PlanarPolyline { widths, .. } = &mut dim
        .graphics
        .as_mut()
        .unwrap()
        .parts
        .iter_mut()
        .find(|p| matches!(p.kind, IfccadDimensionGraphicsKind::PlanarPolyline { .. }))
        .unwrap()
        .kind
    {
        *widths = IfccadPlanarPolylineWidths::PerVertex { widths: vec![] };
    }
    assert!(encode_ifccad_document(&d).is_err());
    assert!(encode_ifccad_cbor_document(&d).is_err());
}
#[test]
fn width_default_expansion_preserves_hatch_editing_and_upright() {
    let mut d = load_ifccad_bytes(
        include_bytes!("../conformance/next/ifccad/valid/hatch-editing.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    let u = load_ifccad_bytes(
        include_bytes!("../conformance/next/ifccad/valid/line-pattern-upright.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    d.model.bounds = None;
    d.model.bounds_quality = None;
    d.line_patterns = u.line_patterns;
    d.symbol_sources = u.symbol_sources;
    d.text_styles = u.text_styles;
    d.id_counters.next_line_pattern_id = u.id_counters.next_line_pattern_id;
    d.id_counters.next_symbol_source_id = u.id_counters.next_symbol_source_id;
    d.id_counters.next_text_style_id = u.id_counters.next_text_style_id;
    let mut entity = document()
        .model
        .entities
        .iter()
        .filter_map(IfccadEntity::as_native)
        .find(|e| matches!(e.kind, IfccadEntityKind::PlanarPolyline { .. }))
        .unwrap()
        .clone();
    entity.id = d.id_counters.next_entity_id;
    d.id_counters.next_entity_id += 1;
    entity.layer_id = d.layers[0].id;
    d.model.entities.push(IfccadEntity::Native(entity));
    for cbor in [false, true] {
        assert_eq!(reopen(&d, cbor), d);
    }
}
