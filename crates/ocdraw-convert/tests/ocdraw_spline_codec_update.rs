use ocdraw::ocdraw::*;
use ocdraw_convert::*;
use opencadcodec::entities::Spline;
use opencadcodec::{CadDocument, EntityType, Handle, Line, Vector3};

const LEGACY: &str = "fe69506cb99dea6f4c4a73b690a27fdf04403ea0";

#[test]
fn previous_version_two_snapshot_remains_readable_after_pin_update() {
    let previous = "063c10671fe7833d562f772159771318c7a0ebb9";
    let mut doc = captured();
    let preservation = doc.preservation.as_mut().unwrap();
    preservation.sources[0].provider_revision = previous.into();
    let record = &mut preservation.records[0];
    let mut body: serde_json::Value = serde_json::from_slice(&record.payload.bytes).unwrap();
    body["codecRevision"] = serde_json::json!(previous);
    record.payload.bytes = serde_json::to_vec(&body).unwrap();
    let restored = ocdraw_document_to_cad_document(&reopened(&doc), Default::default()).unwrap();
    assert!(restored
        .document()
        .entities()
        .any(|e| matches!(e, EntityType::Spline(_))));
}
fn captured() -> OcdrawDocument {
    let mut source = CadDocument::new();
    source.layers.get_mut("0").unwrap().handle = Handle::new(0xf001);
    let mut spline = Spline::from_control_points(
        3,
        vec![
            Vector3::ZERO,
            Vector3::new(1., 2., 0.),
            Vector3::new(2., 2., 0.),
            Vector3::new(3., 0., 0.),
        ],
    );
    spline.common.layer_handle = Some(source.layers.get("0").unwrap().handle);
    spline.dwg_scenario = Some(1);
    source.add_entity(EntityType::Spline(spline)).unwrap();
    cad_document_to_ocdraw_document(
        &source,
        CadToOcdrawOptions {
            preservation_capture: OcdrawPreservationCapture::SupportedTyped,
            ..Default::default()
        },
    )
    .unwrap()
    .into_document()
}
fn reopened(doc: &OcdrawDocument) -> OcdrawDocument {
    load_ocdraw_bytes(encode_ocdraw_document(doc).unwrap().bytes())
        .unwrap()
        .into_document()
}

#[test]
fn new_codec_skipped_fields_are_durable_and_layer_handle_is_rebound() {
    let doc = reopened(&captured());
    let record = &doc.preservation.as_ref().unwrap().records[0];
    assert_eq!(record.payload.version, 2);
    let payload: serde_json::Value = serde_json::from_slice(&record.payload.bytes).unwrap();
    assert_eq!(payload["dwgScenario"], 1);
    assert!(payload["common"]["layerHandle"].is_string());
    let out = ocdraw_document_to_cad_document(&doc, OcdrawToCadOptions::default()).unwrap();
    let spline = out
        .document()
        .entities()
        .find_map(|e| match e {
            EntityType::Spline(s) => Some(s),
            _ => None,
        })
        .unwrap();
    assert_eq!(spline.dwg_scenario, Some(1));
    assert_eq!(
        spline.common.layer_handle,
        Some(
            out.document()
                .layers
                .get(&spline.common.layer)
                .unwrap()
                .handle
        )
    );
}

#[test]
fn previous_revision_payload_one_still_restores_after_save_and_native_edit() {
    let mut doc = captured();
    let preservation = doc.preservation.as_mut().unwrap();
    preservation.sources[0].provider_revision = LEGACY.into();
    let record = &mut preservation.records[0];
    record.payload.version = 1;
    let mut payload: serde_json::Value = serde_json::from_slice(&record.payload.bytes).unwrap();
    payload["codecRevision"] = LEGACY.into();
    payload.as_object_mut().unwrap().remove("dwgScenario");
    payload["common"]
        .as_object_mut()
        .unwrap()
        .remove("layerHandle");
    record.payload.bytes = serde_json::to_vec(&payload).unwrap();
    doc.opaque_entities[0].visible = false;
    let out =
        ocdraw_document_to_cad_document(&reopened(&doc), OcdrawToCadOptions::default()).unwrap();
    let spline = out
        .document()
        .entities()
        .find_map(|e| match e {
            EntityType::Spline(s) => Some(s),
            _ => None,
        })
        .unwrap();
    assert!(spline.common.invisible);
    assert_eq!(spline.dwg_scenario, None);
    assert_eq!(spline.degree, 3);
    assert!(out
        .preservation_report()
        .entries()
        .iter()
        .any(|e| e.result == OcdrawPreservationResult::RestoredTyped));
}

#[test]
fn new_payload_requires_both_explicit_nullable_source_fields() {
    for missing in ["dwgScenario", "layerHandle"] {
        let mut doc = captured();
        let record = &mut doc.preservation.as_mut().unwrap().records[0];
        let mut payload: serde_json::Value = serde_json::from_slice(&record.payload.bytes).unwrap();
        if missing == "dwgScenario" {
            payload.as_object_mut().unwrap().remove(missing);
        } else {
            payload["common"].as_object_mut().unwrap().remove(missing);
        }
        record.payload.bytes = serde_json::to_vec(&payload).unwrap();
        let out = ocdraw_document_to_cad_document(&reopened(&doc), OcdrawToCadOptions::default())
            .unwrap();
        assert!(
            out.preservation_report()
                .entries()
                .iter()
                .any(|e| e.reason == Some(OcdrawPreservationReason::MalformedPayload)),
            "{missing}"
        );
    }
}

#[test]
fn unresolved_layer_handle_never_becomes_the_readers_layer_zero_fallback() {
    let mut source = CadDocument::new();
    let mut spline = Spline::new();
    spline.common.layer_handle = Some(Handle::new(0xdead));
    source.add_entity(EntityType::Spline(spline)).unwrap();
    let doc = cad_document_to_ocdraw_document(
        &source,
        CadToOcdrawOptions {
            preservation_capture: OcdrawPreservationCapture::SupportedTyped,
            ..Default::default()
        },
    )
    .unwrap()
    .into_document();
    assert!(doc.opaque_entities[0].layer_id.is_none());
    let out =
        ocdraw_document_to_cad_document(&reopened(&doc), OcdrawToCadOptions::default()).unwrap();
    assert!(out
        .preservation_report()
        .entries()
        .iter()
        .any(|e| e.reason == Some(OcdrawPreservationReason::UnresolvedReference)));

    let mut source = CadDocument::new();
    let mut line = Line::new();
    line.end = Vector3::new(1., 0., 0.);
    line.common.layer_handle = Some(Handle::new(0xdead));
    source.add_entity(EntityType::Line(line)).unwrap();
    let out = cad_document_to_ocdraw_document(&source, CadToOcdrawOptions::default()).unwrap();
    assert!(out.document().geometric_entities.is_empty());
    assert!(!out.diagnostics().is_empty());
}

#[test]
fn resolved_source_layer_name_handle_contradiction_is_structurally_rejected() {
    let mut source = CadDocument::new();
    let mut other = opencadcodec::Layer::new("Other");
    other.handle = Handle::new(0xabc);
    source.layers.add(other).unwrap();
    let mut spline = Spline::new();
    spline.common.layer_handle = Some(source.layers.get("Other").unwrap().handle);
    source.add_entity(EntityType::Spline(spline)).unwrap();
    let result = cad_document_to_ocdraw_document(
        &source,
        CadToOcdrawOptions {
            preservation_capture: OcdrawPreservationCapture::SupportedTyped,
            ..Default::default()
        },
    );
    assert!(matches!(
        result,
        Err(CadToOcdrawError::InvalidSourceStructure { .. })
    ));
}
