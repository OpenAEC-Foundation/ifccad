use ocdraw::ocdraw::*;
use ocdraw_convert::*;
#[test]
fn logical_conversion_and_encoded_wrappers_share_content() {
    let mut source = cadcodec::CadDocument::new();
    let mut line = cadcodec::Line::from_coords(1., 2., 0., 3., 4., 0.);
    line.common.color = cadcodec::Color::ByBlock;
    source.add_entity(cadcodec::EntityType::Line(line)).unwrap();
    let logical =
        cad_document_to_ocdraw_document_with_id(&source, "logical", ExportOptions::default())
            .unwrap();
    validate_document(logical.document()).unwrap();
    let encoded =
        cad_document_to_drawing_with_id(&source, "logical", ExportOptions::default()).unwrap();
    assert_eq!(
        encode_document(logical.document()).unwrap().bytes(),
        encoded.drawing().bytes()
    );
    assert_eq!(logical.entity_mapping(), encoded.entity_mapping());
    assert_eq!(logical.diagnostics(), encoded.diagnostics());
    assert_eq!(logical.geometry_assessment(), encoded.geometry_assessment());
    let loaded = load_drawing_bytes(encoded.drawing().bytes())
        .into_validated_drawing()
        .unwrap();
    let direct =
        ocdraw_document_to_cad_document(logical.document(), ImportOptions::default()).unwrap();
    let wrapper = ocdraw_to_cad_document(&loaded, ImportOptions::default()).unwrap();
    assert_eq!(direct.entity_mapping(), wrapper.entity_mapping());
    assert_eq!(direct.diagnostics(), wrapper.diagnostics());
    assert_eq!(direct.geometry_assessment(), wrapper.geometry_assessment());
    for outcome in [&direct, &wrapper] {
        let line = outcome
            .document()
            .entities()
            .find_map(|e| match e {
                cadcodec::EntityType::Line(l) => Some(l),
                _ => None,
            })
            .unwrap();
        assert_eq!(line.start, cadcodec::Vector3::new(1., 2., 0.));
        assert_eq!(line.end, cadcodec::Vector3::new(3., 4., 0.));
        assert_eq!(line.common.color, cadcodec::Color::ByBlock);
    }
}
#[test]
fn invalid_logical_input_is_rejected_under_both_policies() {
    let mut d = DrawingBuilder::new(DrawingOptions::new("invalid", "mm"))
        .unwrap()
        .build_document()
        .unwrap();
    d.next_entity_id = 0;
    for policy in [ConversionLossPolicy::Allow, ConversionLossPolicy::Reject] {
        assert!(matches!(
            ocdraw_document_to_cad_document(
                &d,
                ImportOptions {
                    loss_policy: policy,
                    ..Default::default()
                }
            ),
            Err(DirectImportError::InvalidDocument(_))
        ));
    }
}

#[test]
fn logical_import_uses_sparse_scope_identity_and_preserves_source_order() {
    let d = load_drawing_bytes(include_bytes!(
        "../../../conformance/next/ocdraw/valid/sparse-identities.ocdraw.json"
    ))
    .into_validated_drawing()
    .unwrap()
    .into_document();
    let imported = ocdraw_document_to_cad_document(&d, ImportOptions::default()).unwrap();
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
            cadcodec::objects::ObjectType::Layout(l) if l.name == "Sheet" => Some(l),
            _ => None,
        })
        .unwrap();
    assert_eq!(paper.tab_order, 1);
}
#[test]
fn invalid_membership_and_references_never_produce_partial_cad() {
    for mutation in 0..3 {
        let mut d = load_drawing_bytes(include_bytes!(
            "../../../conformance/next/ocdraw/valid/sparse-identities.ocdraw.json"
        ))
        .into_validated_drawing()
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
        for policy in [ConversionLossPolicy::Allow, ConversionLossPolicy::Reject] {
            assert!(matches!(
                ocdraw_document_to_cad_document(
                    &d,
                    ImportOptions {
                        loss_policy: policy,
                        ..Default::default()
                    }
                ),
                Err(DirectImportError::InvalidDocument(_))
            ));
        }
    }
}
#[test]
fn logical_export_keeps_semantic_loss_policy() {
    let mut source = cadcodec::CadDocument::new();
    source.header.project_name = "unrepresented".into();
    let logical = cad_document_to_ocdraw_document(&source, ExportOptions::default()).unwrap();
    let encoded = cad_document_to_drawing(&source, ExportOptions::default()).unwrap();
    assert!(!logical.diagnostics().is_empty());
    assert_eq!(logical.diagnostics(), encoded.diagnostics());
    assert!(matches!(
        cad_document_to_ocdraw_document(
            &source,
            ExportOptions {
                loss_policy: ConversionLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(DirectExportError::LossRejected { .. })
    ));
}
