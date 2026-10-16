use ocdraw::geometry_kernel::hatch::*;
use ocdraw::ocdraw::*;
use serde_json::json;

fn drawing() -> OcdrawDocument {
    load_ocdraw_bytes(include_bytes!(
        "fixtures/ocdraw/hello-hatch-solid.ocdraw.json"
    ))
    .unwrap()
    .into_document()
}

#[test]
fn ordered_disabled_sources_survive_strict_readback_and_source_edits() {
    let mut d = drawing();
    let id = 9007199254740993;
    let old = d.geometric_entities[0].id;
    d.geometric_entities[0].id = id;
    for scope in &mut d.scopes {
        for entry in &mut scope.entities {
            if *entry == old {
                *entry = id;
            }
        }
    }
    d.next_entity_id = id + 1;
    let h = &mut d.hatch_entities[0];
    h.association_enabled = false;
    for l in &mut h.loops {
        l.source_entity_ids.clear();
    }
    h.loops[0].source_entity_ids = vec![id, id, id];
    h.loops[0].text_island = true;
    h.loops[0].provenance = Some(HatchLoopProvenance {
        text_box: true,
        ..Default::default()
    });
    let expected = h.clone();
    d.geometric_entities[0].geometry = DrawingGeometry::Line {
        start: [0., 0., 2.],
        end: [3., 0., 2.],
    };
    recompute_ocdraw_document_bounds(&mut d).unwrap();
    let out = encode_ocdraw_document(&d).unwrap();
    let read = load_ocdraw_bytes(out.bytes()).unwrap();
    assert_eq!(read.hatch_entities()[0], expected);
    assert_eq!(
        read.hatch_entities()[0].loops[0].source_entity_ids,
        [9007199254740993; 3]
    );
}

#[test]
fn disabled_sources_require_valid_owners_and_detach_removes_whole_group() {
    let mut d = drawing();
    d.hatch_entities[0].association_enabled = false;
    d.hatch_entities[0].loops[0].source_entity_ids = vec![u64::MAX];
    assert!(validate_ocdraw_document(&d).is_err());
    d.hatch_entities[0].loops[0].source_entity_ids.clear();
    validate_ocdraw_document(&d).unwrap();
}

#[test]
fn text_island_survives_fill_switch_and_unbound_enabled_state() {
    let mut d = drawing();
    for l in &mut d.hatch_entities[0].loops {
        l.source_entity_ids.clear();
        l.text_island = true;
    }
    d.hatch_entities[0].association_enabled = true;
    for fill in [
        HatchFill::Solid,
        HatchFill::LinePattern(HatchLinePattern {
            name: None,
            description: None,
            origin: [0., 0.],
            rotation: 0.,
            scale: 1.,
            families: vec![HatchLineFamily {
                angle: 0.,
                base_point: [0., 0.],
                offset: [0., 1.],
                dashes: vec![],
            }],
        }),
        HatchFill::Gradient,
    ] {
        d.hatch_entities[0].fill = fill;
        if matches!(d.hatch_entities[0].fill, HatchFill::Gradient) {
            d.hatch_entities[0].gradient = Some(HatchGradient {
                gradient_type: Some(HatchGradientType::Linear),
                colors: Some([DrawingColor::rgb(0, 0, 0), DrawingColor::rgb(255, 255, 255)]),
                rotation: 0.,
                shift: 0.,
                color_mode: HatchGradientColorMode::TwoColor,
                tint: 0.,
            });
        }
        let out = encode_ocdraw_document(&d).unwrap();
        let read = load_ocdraw_bytes(out.bytes()).unwrap();
        assert!(read.hatch_entities()[0].association_enabled);
        assert!(read.hatch_entities()[0]
            .loops
            .iter()
            .all(|l| l.text_island && l.source_entity_ids.is_empty()));
    }
}

#[test]
fn closed_wire_contract_rejects_null_unknown_and_empty_provenance() {
    let d = drawing();
    let encoded = encode_ocdraw_document(&d).unwrap();
    let value: serde_json::Value = serde_json::from_slice(encoded.bytes()).unwrap();
    for (key, bad) in [
        ("sourceEntityIds", json!(null)),
        ("provenance", json!({})),
        ("provenance", json!({"textBox":false})),
        ("provenance", json!({"textBox":true,"secret":true})),
        ("textIsland", json!(null)),
        ("sourceEntityId", json!(1)),
    ] {
        let mut v = value.clone();
        v["streams"]["hatchStream"]["loops"][0][0][key] = bad;
        assert!(
            load_ocdraw_bytes(&serde_json::to_vec(&v).unwrap()).is_err(),
            "{key}"
        );
    }
}

#[test]
fn native_boundary_providers_include_text_mtext_blocks_and_paper_viewports() {
    for bytes in [
        include_bytes!("../conformance/next/ocdraw/valid/placed-geometry.ocdraw.json").as_slice(),
        include_bytes!("../conformance/next/ocdraw/valid/text-rich-content.ocdraw.json").as_slice(),
        include_bytes!("../conformance/next/ocdraw/valid/viewport-forward-clip.ocdraw.json")
            .as_slice(),
        include_bytes!("fixtures/ocdraw/layouts-viewports.ocdraw.json").as_slice(),
    ] {
        let original = load_ocdraw_bytes(bytes).unwrap().into_document();
        let sources = original
            .geometric_entities
            .iter()
            .filter(|e| !matches!(e.geometry, DrawingGeometry::Point { .. }))
            .map(|e| e.id)
            .chain(original.text_entities.iter().map(|e| e.id))
            .chain(original.mtext_entities.iter().map(|e| e.id))
            .chain(original.viewports.iter().map(|e| e.id))
            .collect::<Vec<_>>();
        assert!(!sources.is_empty());
        for source in sources {
            let mut d = original.clone();
            let mut h = drawing().hatch_entities.remove(0);
            h.id = d.next_entity_id;
            d.next_entity_id += 1;
            h.layer_id = d.layers[0].id;
            h.association_enabled = false;
            h.loops.truncate(1);
            h.loops[0].source_entity_ids = vec![source, source];
            d.scopes
                .iter_mut()
                .find(|s| s.entities.contains(&source))
                .unwrap()
                .entities
                .push(h.id);
            d.hatch_entities.push(h);
            recompute_ocdraw_document_bounds(&mut d).unwrap();
            let out = encode_ocdraw_document(&d).unwrap();
            let read = load_ocdraw_bytes(out.bytes()).unwrap();
            assert_eq!(
                read.hatch_entities().last().unwrap().loops[0].source_entity_ids,
                [source, source]
            );
        }
    }
}
