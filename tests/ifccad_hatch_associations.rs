use ifccad::geometry_kernel::hatch::*;
use ifccad::*;
#[path = "support/ifccad_hatch.rs"]
mod fixture;

fn hatch_mut(d: &mut IfccadDocument) -> &mut IfccadHatch {
    let IfccadEntityKind::Hatch(h) = &mut d.model.entities[0].as_native_mut().unwrap().kind else {
        panic!()
    };
    h
}

#[test]
fn ordered_disabled_sources_survive_strict_readback_and_source_edits() {
    let mut d = fixture::drawing();
    let h = hatch_mut(&mut d);
    h.association_enabled = false;
    h.loops[1].source_entity_ids = vec![fixture::SOURCE; 3];
    h.loops[1].text_island = true;
    h.loops[1].provenance = Some(HatchLoopProvenance {
        text_box: true,
        ..Default::default()
    });
    let expected = d.model.entities[0].clone();
    d.model.entities[1].as_native_mut().unwrap().kind = IfccadEntityKind::LineSegment {
        start: [0., 0., 2.],
        end: [3., 0., 2.],
        thickness: 0.,
        extrusion_direction: [0., 0., 1.],
    };
    let out = encode_ifccad_document(&d).unwrap();
    let read = load_ifccad_bytes(out.bytes(), Default::default())
        .unwrap()
        .into_document();
    assert_eq!(read.model.entities[0], expected);
    let text = String::from_utf8_lossy(out.bytes());
    assert!(text.contains(&format!("/cad/d{}/e9007199254740993", d.drawing_id)));
}

#[test]
fn disabled_sources_validate_and_detach_without_geometry_changes() {
    let mut d = fixture::drawing();
    let h = hatch_mut(&mut d);
    h.association_enabled = false;
    let old = h.loops[1].boundary.clone();
    h.loops[1].source_entity_ids = vec![u64::MAX];
    assert!(validate_ifccad_document(&d).is_err());
    hatch_mut(&mut d).loops[1].source_entity_ids.clear();
    validate_ifccad_document(&d).unwrap();
    assert_eq!(hatch_mut(&mut d).loops[1].boundary, old);
}

#[test]
fn sources_require_canonical_full_paths_and_closed_metadata() {
    let d = fixture::drawing();
    let out = encode_ifccad_document(&d).unwrap();
    let value: serde_json::Value = serde_json::from_slice(out.bytes()).unwrap();
    for (key, bad) in [
        ("sources", serde_json::json!(null)),
        ("sources", serde_json::json!(["/cad/d1/e0001"])),
        ("sources", serde_json::json!(["e1"])),
        ("sources", serde_json::json!(["/cad/d999/e1"])),
        ("provenance", serde_json::json!({})),
        (
            "provenance",
            serde_json::json!({"textBox":true,"secret":true}),
        ),
        ("textIsland", serde_json::json!(null)),
        ("source", serde_json::json!("/cad/d1/e1")),
    ] {
        let mut v = value.clone();
        let node = v["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|n| n["attributes"].get("ifccad::hatch").is_some())
            .unwrap();
        node["attributes"]["ifccad::hatch"]["loops"][1][key] = bad;
        assert!(
            load_ifccad_bytes(&serde_json::to_vec(&v).unwrap(), Default::default()).is_err(),
            "{key}"
        );
    }
}

#[test]
fn native_boundary_providers_include_text_mtext_blocks_and_paper_viewports() {
    for bytes in [
        include_bytes!("../conformance/next/ifccad/valid/geometry-families.ifcx").as_slice(),
        include_bytes!("../conformance/next/ifccad/valid/text-rich.ifcx").as_slice(),
        include_bytes!("../conformance/next/ifccad/valid/text-owners.ifcx").as_slice(),
        include_bytes!("../conformance/next/ifccad/valid/viewport-forward-clip.ifcx").as_slice(),
        include_bytes!("fixtures/ifccad/hello-nested-blocks.ifcx").as_slice(),
    ] {
        let mut d = load_ifccad_bytes(bytes, Default::default())
            .unwrap()
            .into_document();
        let mut next = d.id_counters.next_entity_id;
        let mut added = Vec::new();
        for entities in std::iter::once(&mut d.model.entities)
            .chain(d.paper_layouts.iter_mut().map(|p| &mut p.entities))
            .chain(d.blocks.iter_mut().map(|b| &mut b.entities))
        {
            let sources = entities
                .iter()
                .filter_map(IfccadEntity::as_native)
                .filter(|e| {
                    !matches!(
                        e.kind,
                        IfccadEntityKind::Point { .. } | IfccadEntityKind::Hatch(_)
                    )
                })
                .cloned()
                .collect::<Vec<_>>();
            for mut source in sources {
                let source_id = source.id;
                source.id = next;
                next += 1;
                source.kind = IfccadEntityKind::Hatch(IfccadHatch {
                    classic_settings: None,
                    boundary_construction: None,
                    association_enabled: false,
                    gradient: None,
                    placement: fixture::plane(),
                    fill: HatchFill::Solid,
                    area_rule: HatchAreaRule::Normal,
                    join_tolerance: 1e-9,
                    loops: vec![IfccadHatchLoop {
                        boundary: HatchBoundary2::Circle {
                            center: [0., 0.],
                            radius: 0.5,
                        },
                        source_entity_ids: vec![source_id, source_id],
                        provenance: None,
                        text_island: false,
                    }],
                });
                added.push(source.id);
                entities.push(IfccadEntity::Native(source));
            }
        }
        assert!(!added.is_empty());
        d.id_counters.next_entity_id = next;
        recompute_ifccad_document_bounds(&mut d).unwrap();
        let out = encode_ifccad_document(&d).unwrap();
        let read = load_ifccad_bytes(out.bytes(), Default::default()).unwrap();
        validate_ifccad_document(read.document()).unwrap();
    }
}

#[test]
fn text_island_survives_fill_switch_and_unbound_enabled_state() {
    let mut d = fixture::drawing();
    let h = hatch_mut(&mut d);
    h.association_enabled = true;
    for l in &mut h.loops {
        l.source_entity_ids.clear();
        l.text_island = true;
    }
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
        let h = hatch_mut(&mut d);
        h.fill = fill;
        if matches!(h.fill, HatchFill::Gradient) {
            h.gradient = Some(HatchGradient {
                gradient_type: Some(HatchGradientType::Linear),
                colors: Some([IfccadColor::rgb(0, 0, 0), IfccadColor::rgb(255, 255, 255)]),
                rotation: 0.,
                shift: 0.,
                color_mode: HatchGradientColorMode::TwoColor,
                tint: 0.,
            });
        }
        let out = encode_ifccad_document(&d).unwrap();
        let mut read = load_ifccad_bytes(out.bytes(), Default::default())
            .unwrap()
            .into_document();
        let h = hatch_mut(&mut read);
        assert!(h.association_enabled);
        assert!(h
            .loops
            .iter()
            .all(|l| l.text_island && l.source_entity_ids.is_empty()));
    }
}
