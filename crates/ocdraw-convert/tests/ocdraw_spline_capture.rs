use ocdraw_convert::*;
use opencadcodec::entities::Spline;
use opencadcodec::{BlockRecord, CadDocument, Color, EntityType, Handle, Line, LineWeight};

#[test]
fn original_dwg_records_are_reported_as_omitted_storage_supplements() {
    let mut source = CadDocument::new();
    source
        .add_entity(EntityType::Spline(Spline::new()))
        .unwrap();
    let bytes = opencadcodec::DwgWriter::write_to_vec(&source).unwrap();
    let read = opencadcodec::DwgReader::from_stream(std::io::Cursor::new(bytes))
        .read()
        .unwrap();
    assert!(read
        .entities()
        .any(|e| matches!(e, EntityType::Spline(_)) && e.common().raw_record.is_some()));
    let out = cad_document_to_ocdraw_document(&read, options()).unwrap();
    assert!(out
        .preservation_report()
        .entries()
        .iter()
        .any(|e| e.result == OcdrawPreservationResult::StorageSupplementOmitted));
    let payload: serde_json::Value = serde_json::from_slice(
        &out.document().preservation.as_ref().unwrap().records[0]
            .payload
            .bytes,
    )
    .unwrap();
    assert!(payload["common"].get("rawRecord").is_none());
}

#[test]
fn unresolved_common_pattern_is_stored_but_known_name_handle_contradiction_is_fatal() {
    let mut source = CadDocument::new();
    let mut s = Spline::new();
    s.common.linetype = "MissingPattern".into();
    s.common.linetype_handle = Some(Handle::new(0x777));
    let h = source.add_entity(EntityType::Spline(s)).unwrap();
    let out = cad_document_to_ocdraw_document(&source, options()).unwrap();
    assert!(out.document().opaque_entities[0].appearance.is_none());
    let payload: serde_json::Value = serde_json::from_slice(
        &out.document().preservation.as_ref().unwrap().records[0]
            .payload
            .bytes,
    )
    .unwrap();
    assert_eq!(payload["common"]["linetype"], "MissingPattern");
    assert_eq!(payload["common"]["linetypeHandle"], "777");
    let restored =
        ocdraw_document_to_cad_document(out.document(), OcdrawToCadOptions::default()).unwrap();
    assert!(!restored
        .document()
        .entities()
        .any(|e| matches!(e, EntityType::Spline(_))));
    let continuous = source.line_types.get("Continuous").unwrap().handle;
    let EntityType::Spline(s) = source.get_entity_mut(h).unwrap() else {
        unreachable!()
    };
    s.common.linetype = "ByLayer".into();
    s.common.linetype_handle = Some(continuous);
    assert!(matches!(
        cad_document_to_ocdraw_document(&source, options()),
        Err(CadToOcdrawError::InvalidSourceStructure { .. })
    ));
}

fn options() -> CadToOcdrawOptions {
    CadToOcdrawOptions {
        preservation_capture: OcdrawPreservationCapture::SupportedTyped,
        ..Default::default()
    }
}
#[test]
fn capture_keeps_explicit_owner_order_and_native_common_capability_separate() {
    let mut source = CadDocument::new();
    let mut first = Spline::new();
    first.common.color = Color::None;
    first.common.line_weight = LineWeight::Default;
    first.common.linetype_scale = f64::NAN;
    let a = source.add_entity(EntityType::Spline(first)).unwrap();
    let b = source
        .add_entity(EntityType::Line(Line::from_coords(0., 0., 0., 1., 1., 0.)))
        .unwrap();
    let c = source
        .add_entity(EntityType::Spline(Spline::new()))
        .unwrap();
    let model = source.header.model_space_block_handle;
    source
        .block_records
        .iter_mut()
        .find(|r| r.handle == model)
        .unwrap()
        .entity_handles = vec![c, b, a];
    let out = cad_document_to_ocdraw_document(&source, options()).unwrap();
    let ids = [c, b, a].map(|h| out.entity_mapping()[&h]);
    assert_eq!(out.document().scopes[0].entities, ids);
    let opaque = out
        .document()
        .opaque_entities
        .iter()
        .find(|e| e.id == ids[2])
        .unwrap();
    assert!(opaque.appearance.is_none());
    assert!(out
        .preservation_report()
        .entries()
        .iter()
        .any(|e| e.entity_id == Some(opaque.id)
            && e.result == OcdrawPreservationResult::NativePropertyUnrepresented));
    assert!(!out.geometry_assessment().is_complete());
    assert_eq!(out.geometry_assessment().assessed_entities(), 1);
    assert!(out.document().scopes[0].bounds.is_none());
    let disabled = cad_document_to_ocdraw_document(&source, CadToOcdrawOptions::default()).unwrap();
    assert!(disabled.document().preservation.is_none());
    assert!(disabled.document().opaque_entities.is_empty());
}

#[test]
fn unsupported_valid_owner_is_archived_and_corrupt_owner_is_fatal() {
    let mut source = CadDocument::new();
    let mut block = BlockRecord::new("External");
    block.handle = source.allocate_handle();
    block.flags.is_xref = true;
    let owner = block.handle;
    source.block_records.add(block).unwrap();
    let mut spline = Spline::new();
    spline.common.owner_handle = owner;
    let h = source.add_entity(EntityType::Spline(spline)).unwrap();
    let out = cad_document_to_ocdraw_document(&source, options()).unwrap();
    assert!(out.document().opaque_entities.is_empty());
    let p = out.document().preservation.as_ref().unwrap();
    assert_eq!(p.records.len(), 1);
    assert!(p.records[0].subject.is_none());
    let payload: serde_json::Value = serde_json::from_slice(&p.records[0].payload.bytes).unwrap();
    assert_eq!(payload["sourceOwnerHandle"], format!("{owner:x}"));
    assert_eq!(payload["sourceOrderIndex"], 0);
    assert!(!out.diagnostics().is_empty());
    source.get_entity_mut(h).unwrap().common_mut().owner_handle = Handle::new(u64::MAX - 1);
    assert!(matches!(
        cad_document_to_ocdraw_document(&source, options()),
        Err(CadToOcdrawError::InvalidSourceStructure { .. })
    ));
}

#[test]
fn capture_reject_allows_saved_typed_data_but_keeps_unrelated_losses_and_report() {
    let mut source = CadDocument::new();
    source
        .add_entity(EntityType::Spline(Spline::new()))
        .unwrap();
    let opts = CadToOcdrawOptions {
        loss_policy: OcdrawLossPolicy::Reject,
        ..options()
    };
    let out = cad_document_to_ocdraw_document(&source, opts).unwrap();
    assert_eq!(out.document().opaque_entities.len(), 1);
    source.header.project_name = "Unmapped project".into();
    let Err(CadToOcdrawError::LossRejected {
        diagnostics,
        preservation,
    }) = cad_document_to_ocdraw_document(&source, opts)
    else {
        panic!("unrelated actual loss must reject");
    };
    assert!(!diagnostics.is_empty());
    assert!(preservation
        .entries()
        .iter()
        .any(|e| e.result == OcdrawPreservationResult::CapturedTyped));
}
