use ocdraw_convert::*;
use opencadcodec::EntityType;
#[path = "../../../tests/support/cad_hatch_associations.rs"]
mod fixture;

#[test]
fn multiple_and_disabled_sources_roundtrip() {
    for enabled in [false, true] {
        let (d, hh, ids) = fixture::source(enabled, false);
        let out =
            cad_document_to_ocdraw_document_with_id(&d, "hatch-associations", Default::default())
                .unwrap();
        let h = &out.document().hatch_entities[0];
        assert_eq!(h.id, out.entity_mapping()[&hh]);
        assert_eq!(h.association_enabled, enabled);
        assert_eq!(
            h.loops[0].source_entity_ids,
            [
                out.entity_mapping()[&ids[2]],
                out.entity_mapping()[&ids[0]],
                out.entity_mapping()[&ids[2]],
                out.entity_mapping()[&ids[1]]
            ]
        );
        assert!(h.loops[0].text_island);
        assert!(h.loops[0].provenance.as_ref().unwrap().text_box);
        let encoded = ocdraw::ocdraw::encode_ocdraw_document(out.document()).unwrap();
        ocdraw::ocdraw::load_ocdraw_bytes(encoded.bytes()).unwrap();
        let cad = ocdraw_document_to_cad_document(out.document(), Default::default()).unwrap();
        let back = cad
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
        assert_eq!(back.is_associative, enabled);
        assert_eq!(back.paths[0].boundary_handles.len(), 4);
        assert_eq!(
            back.paths[0].boundary_handles[0],
            back.paths[0].boundary_handles[2]
        );
        assert_eq!(back.paths[0].flags.bits() & !2, 136);
        for dwg in [false, true] {
            let file = fixture::exchange(cad.document(), dwg);
            let read =
                cad_document_to_ocdraw_document_with_id(&file, "physical", Default::default())
                    .unwrap();
            let h = &read.document().hatch_entities[0];
            assert_eq!(h.association_enabled, enabled);
            assert_eq!(h.loops[0].source_entity_ids.len(), 4);
            assert_eq!(h.loops[1].source_entity_ids.len(), 1);
            assert_eq!(
                h.loops[0].source_entity_ids[0],
                h.loops[0].source_entity_ids[2]
            );
            assert!(h
                .loops
                .iter()
                .all(|l| l.text_island && l.provenance.as_ref().unwrap().text_box));
            let bytes = ocdraw::ocdraw::encode_ocdraw_document(read.document()).unwrap();
            ocdraw::ocdraw::load_ocdraw_bytes(bytes.bytes()).unwrap();
        }
    }
}

#[test]
fn one_unsupported_source_omits_only_its_complete_group() {
    let (d, _, _) = fixture::source(true, true);
    let out = cad_document_to_ocdraw_document_with_id(&d, "hatch-associations", Default::default())
        .unwrap();
    let h = &out.document().hatch_entities[0];
    assert!(h.association_enabled);
    assert!(h.loops[0].source_entity_ids.is_empty());
    assert_eq!(h.loops[1].source_entity_ids.len(), 1);
    assert!(h.loops.iter().all(|l| l.text_island));
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| format!("{d:?}").contains("source group omitted")));
    assert!(matches!(
        cad_document_to_ocdraw_document_with_id(
            &d,
            "hatch-associations",
            CadToOcdrawOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(CadToOcdrawError::LossRejected { .. })
    ));
}

#[test]
fn unbound_enabled_switch_is_preserved() {
    let (mut d, hh, _) = fixture::source(true, false);
    let EntityType::Hatch(h) = d.get_entity_mut(hh).unwrap() else {
        panic!()
    };
    for p in &mut h.paths {
        p.boundary_handles.clear();
    }
    let out = cad_document_to_ocdraw_document_with_id(&d, "unbound", Default::default()).unwrap();
    assert!(out.document().hatch_entities[0].association_enabled);
    assert!(out.document().hatch_entities[0]
        .loops
        .iter()
        .all(|l| l.source_entity_ids.is_empty()));
}

#[test]
fn only_confirmed_hatch_reactors_are_accounted() {
    let (mut d, hh, ids) = fixture::source(true, false);
    d.get_entity_mut(ids[0]).unwrap().common_mut().reactors = vec![hh];
    let out = cad_document_to_ocdraw_document_with_id(&d, "reactors", Default::default()).unwrap();
    assert!(!out
        .diagnostics()
        .iter()
        .any(|d| d.reasons().contains(&CadToOcdrawLossReason::EntityReactors)));
    d.get_entity_mut(ids[0])
        .unwrap()
        .common_mut()
        .reactors
        .push(ids[2]);
    let out = cad_document_to_ocdraw_document_with_id(&d, "reactors", Default::default()).unwrap();
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.reasons().contains(&CadToOcdrawLossReason::EntityReactors)));
}
