use ocdraw::ocdraw::*;
use ocdraw_convert::*;
#[test]
fn logical_conversion_and_encoded_wrappers_share_content() {
    let mut source = opencadcodec::CadDocument::new();
    let mut line = opencadcodec::Line::from_coords(1., 2., 0., 3., 4., 0.);
    line.common.color = opencadcodec::Color::ByBlock;
    source
        .add_entity(opencadcodec::EntityType::Line(line))
        .unwrap();
    let logical =
        cad_document_to_ocdraw_document_with_id(&source, "logical", CadToOcdrawOptions::default())
            .unwrap();
    validate_ocdraw_document(logical.document()).unwrap();
    let encoded =
        cad_document_to_encoded_ocdraw_with_id(&source, "logical", CadToOcdrawOptions::default())
            .unwrap();
    assert_eq!(
        encode_ocdraw_document(logical.document()).unwrap().bytes(),
        encoded.encoded().bytes()
    );
    assert_eq!(logical.entity_mapping(), encoded.entity_mapping());
    assert_eq!(logical.diagnostics(), encoded.diagnostics());
    assert_eq!(logical.geometry_assessment(), encoded.geometry_assessment());
    let loaded = load_ocdraw_bytes(encoded.encoded().bytes()).ok().unwrap();
    let direct =
        ocdraw_document_to_cad_document(logical.document(), OcdrawToCadOptions::default()).unwrap();
    let wrapper = ocdraw_source_to_cad_document(&loaded, OcdrawToCadOptions::default()).unwrap();
    assert_eq!(direct.entity_mapping(), wrapper.entity_mapping());
    assert_eq!(direct.diagnostics(), wrapper.diagnostics());
    assert_eq!(direct.geometry_assessment(), wrapper.geometry_assessment());
    for outcome in [&direct, &wrapper] {
        let line = outcome
            .document()
            .entities()
            .find_map(|e| match e {
                opencadcodec::EntityType::Line(l) => Some(l),
                _ => None,
            })
            .unwrap();
        assert_eq!(line.start, opencadcodec::Vector3::new(1., 2., 0.));
        assert_eq!(line.end, opencadcodec::Vector3::new(3., 4., 0.));
        assert_eq!(line.common.color, opencadcodec::Color::ByBlock);
    }
}
#[test]
fn invalid_logical_input_is_rejected_under_both_policies() {
    let mut d = OcdrawBuilder::new(OcdrawBuildOptions::new("invalid", "mm"))
        .unwrap()
        .build_document()
        .unwrap();
    d.next_entity_id = 0;
    for policy in [OcdrawLossPolicy::Allow, OcdrawLossPolicy::Reject] {
        assert!(matches!(
            ocdraw_document_to_cad_document(
                &d,
                OcdrawToCadOptions {
                    loss_policy: policy,
                    ..Default::default()
                }
            ),
            Err(OcdrawToCadError::InvalidDocument(_))
        ));
    }
}

#[test]
fn logical_import_uses_sparse_scope_identity_and_preserves_source_order() {
    let d = load_ocdraw_bytes(include_bytes!(
        "../../../conformance/next/ocdraw/valid/sparse-identities.ocdraw.json"
    ))
    .ok()
    .unwrap()
    .into_document();
    let imported = ocdraw_document_to_cad_document(&d, OcdrawToCadOptions::default()).unwrap();
    let cad = imported.document();
    for scope in &d.scopes {
        let handles = scope
            .entities
            .iter()
            .map(|id| imported.entity_mapping()[id])
            .collect::<Vec<_>>();
        if let Some(first) = handles.first() {
            let owner = cad.get_entity(*first).unwrap().common().owner_handle;
            let block = cad
                .block_records
                .iter()
                .find(|b| b.handle == owner)
                .unwrap();
            assert_eq!(block.entity_handles, handles);
        }
    }
    let paper = cad
        .objects
        .values()
        .find_map(|obj| match obj {
            opencadcodec::objects::ObjectType::Layout(l) if l.name == "Sheet" => Some(l),
            _ => None,
        })
        .unwrap();
    assert_eq!(paper.tab_order, 1);
}
#[test]
fn invalid_membership_and_references_never_produce_partial_cad() {
    for mutation in 0..3 {
        let mut d = load_ocdraw_bytes(include_bytes!(
            "../../../conformance/next/ocdraw/valid/sparse-identities.ocdraw.json"
        ))
        .ok()
        .unwrap()
        .into_document();
        let scope = d
            .scopes
            .iter_mut()
            .find(|s| !s.entities.is_empty())
            .unwrap();
        match mutation {
            0 => scope.entities.push(scope.entities[0]),
            1 => {
                scope.entities.pop();
            }
            _ => d.geometric_entities[0].layer_id = 999,
        }
        for policy in [OcdrawLossPolicy::Allow, OcdrawLossPolicy::Reject] {
            assert!(matches!(
                ocdraw_document_to_cad_document(
                    &d,
                    OcdrawToCadOptions {
                        loss_policy: policy,
                        ..Default::default()
                    }
                ),
                Err(OcdrawToCadError::InvalidDocument(_))
            ));
        }
    }
}
#[test]
fn logical_export_keeps_semantic_loss_policy() {
    let mut source = opencadcodec::CadDocument::new();
    source.header.project_name = "unrepresented".into();
    let logical = cad_document_to_ocdraw_document(&source, CadToOcdrawOptions::default()).unwrap();
    let encoded = cad_document_to_encoded_ocdraw(&source, CadToOcdrawOptions::default()).unwrap();
    assert!(!logical.diagnostics().is_empty());
    assert_eq!(logical.diagnostics(), encoded.diagnostics());
    assert!(matches!(
        cad_document_to_ocdraw_document(
            &source,
            CadToOcdrawOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(CadToOcdrawError::LossRejected { .. })
    ));
}
