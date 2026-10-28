use ifccad::*;
use ifccad_convert::*;
use opencadcodec::EntityType;
#[path = "../../../tests/support/cad_hatch_associations.rs"]
mod fixture;
fn metadata() -> IfccadTargetMetadata {
    let d = load_ifccad_bytes(
        include_bytes!("../../../tests/fixtures/ifccad/hello-cad.ifcx"),
        Default::default(),
    )
    .unwrap()
    .into_document();
    IfccadTargetMetadata {
        header: d.header,
        drawing_id: d.drawing_id,
    }
}
fn hatch(d: &IfccadDocument) -> &IfccadHatch {
    d.model
        .entities
        .iter()
        .filter_map(IfccadEntity::as_native)
        .find_map(|e| {
            if let IfccadEntityKind::Hatch(h) = &e.kind {
                Some(h)
            } else {
                None
            }
        })
        .unwrap()
}

#[test]
fn multiple_and_disabled_sources_roundtrip() {
    for enabled in [false, true] {
        let (d, _, ids) = fixture::source(enabled, false);
        let out = cad_document_to_ifccad_document(&d, metadata(), Default::default()).unwrap();
        let h = hatch(out.document());
        assert_eq!(h.association_enabled, enabled);
        assert_eq!(
            h.loops[0].source_entity_ids,
            [ids[2], ids[0], ids[2], ids[1]].map(|id| out
                .mappings()
                .entities
                .ifccad_id(id)
                .unwrap())
        );
        assert!(h.loops[0].text_island);
        assert!(h.loops[0].provenance.as_ref().unwrap().text_box);
        let encoded = encode_ifccad_document(out.document()).unwrap();
        load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
        let cad = ifccad_document_to_cad_document(out.document(), Default::default()).unwrap();
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
                cad_document_to_ifccad_document(&file, metadata(), Default::default()).unwrap();
            let h = hatch(read.document());
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
            let bytes = encode_ifccad_document(read.document()).unwrap();
            load_ifccad_bytes(bytes.bytes(), Default::default()).unwrap();
        }
    }
}

#[test]
fn one_unsupported_source_omits_only_its_complete_group() {
    let (d, _, _) = fixture::source(true, true);
    let out = cad_document_to_ifccad_document(&d, metadata(), Default::default()).unwrap();
    let h = hatch(out.document());
    assert!(h.association_enabled);
    assert!(h.loops[0].source_entity_ids.is_empty());
    assert_eq!(h.loops[1].source_entity_ids.len(), 1);
    assert!(h.loops.iter().all(|l| l.text_island));
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "hatch-source-group-loss"));
    assert!(matches!(
        cad_document_to_ifccad_document(
            &d,
            metadata(),
            CadToIfccadOptions {
                loss_policy: IfccadLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(IfccadConversionError::Unsupported(_))
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
    let out = cad_document_to_ifccad_document(&d, metadata(), Default::default()).unwrap();
    assert!(hatch(out.document()).association_enabled);
    assert!(hatch(out.document())
        .loops
        .iter()
        .all(|l| l.source_entity_ids.is_empty()));
}

#[test]
fn only_confirmed_hatch_reactors_are_accounted() {
    let (mut d, hh, ids) = fixture::source(true, false);
    d.get_entity_mut(ids[0]).unwrap().common_mut().reactors = vec![hh];
    let out = cad_document_to_ifccad_document(&d, metadata(), Default::default()).unwrap();
    assert!(
        !out.diagnostics()
            .iter()
            .any(|d| d.location.ends_with(".reactor")),
        "{:?}",
        out.diagnostics()
    );
    d.get_entity_mut(ids[0])
        .unwrap()
        .common_mut()
        .reactors
        .push(ids[2]);
    let out = cad_document_to_ifccad_document(&d, metadata(), Default::default()).unwrap();
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.location.ends_with(".reactor")));
}
