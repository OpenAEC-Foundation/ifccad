use ocdraw::ocdraw::*;
use ocdraw_convert::*;
use opencadcodec::entities::Spline;
use opencadcodec::{CadDocument, EntityType, Vector3};

fn captured() -> OcdrawDocument {
    let mut source = CadDocument::new();
    let mut spline = Spline::new();
    spline.degree = 2;
    spline.control_points = vec![
        Vector3::new(0., 0., 0.),
        Vector3::new(1., 2., 0.),
        Vector3::new(3., 0., 0.),
    ];
    spline.knots = vec![0., 0., 0., 1., 1., 1.];
    source.add_entity(EntityType::Spline(spline)).unwrap();
    let outcome = cad_document_to_encoded_ocdraw(
        &source,
        CadToOcdrawOptions {
            preservation_capture: OcdrawPreservationCapture::SupportedTyped,
            ..Default::default()
        },
    )
    .unwrap();
    drop(source);
    load_ocdraw_bytes(outcome.encoded().bytes())
        .unwrap()
        .into_document()
}

#[test]
fn restore_uses_durable_snapshot_and_authoritative_native_common_properties() {
    let mut doc = captured();
    let snapshot = doc.preservation.clone();
    doc.opaque_entities[0].visible = false;
    doc.opaque_entities[0].appearance.as_mut().unwrap().color =
        AppearanceSelection::Explicit(DrawingColor::rgb(1, 2, 3));
    doc.opaque_entities[0]
        .appearance
        .as_mut()
        .unwrap()
        .line_pattern_scale = 2.;
    doc.layers[0].name = "Renamed".into();
    let outcome = ocdraw_document_to_cad_document(&doc, OcdrawToCadOptions::default()).unwrap();
    let spline = outcome
        .document()
        .entities()
        .find_map(|e| {
            if let EntityType::Spline(s) = e {
                Some(s)
            } else {
                None
            }
        })
        .expect("restored spline");
    assert_eq!(spline.degree, 2);
    assert_eq!(spline.knots, vec![0., 0., 0., 1., 1., 1.]);
    assert_eq!(spline.common.layer, "Renamed");
    assert_eq!(
        spline.common.color,
        opencadcodec::Color::Rgb { r: 1, g: 2, b: 3 }
    );
    assert_eq!(spline.common.linetype_scale, 2.);
    assert!(spline.common.invisible);
    assert!(spline.common.raw_record.is_none());
    assert_eq!(doc.preservation, snapshot);
    assert!(outcome
        .preservation_report()
        .entries()
        .iter()
        .any(|e| e.result == OcdrawPreservationResult::RestoredTyped));
    assert!(!outcome.geometry_assessment().is_complete());
}

#[test]
fn changed_units_and_missing_predicates_are_refused_without_erasing_payload() {
    let mut doc = captured();
    let original = doc.unit.clone();
    doc.unit = "cm".into();
    let out = ocdraw_document_to_cad_document(&doc, OcdrawToCadOptions::default()).unwrap();
    assert!(!out
        .document()
        .entities()
        .any(|e| matches!(e, EntityType::Spline(_))));
    assert!(out
        .preservation_report()
        .entries()
        .iter()
        .any(|e| e.reason == Some(OcdrawPreservationReason::ChangedDependency)));
    assert!(matches!(
        ocdraw_document_to_cad_document(
            &doc,
            OcdrawToCadOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(OcdrawToCadError::LossRejected { .. })
    ));
    doc.unit = original;
    assert!(
        ocdraw_document_to_cad_document(&doc, OcdrawToCadOptions::default())
            .unwrap()
            .document()
            .entities()
            .any(|e| matches!(e, EntityType::Spline(_)))
    );
    doc.preservation.as_mut().unwrap().records[0]
        .conditions
        .clear();
    let out = ocdraw_document_to_cad_document(&doc, OcdrawToCadOptions::default()).unwrap();
    assert!(out
        .preservation_report()
        .entries()
        .iter()
        .any(|e| e.reason == Some(OcdrawPreservationReason::UnsupportedContext)));
}

#[test]
fn explicit_skip_has_real_export_loss_but_detached_archive_has_no_drawable_obligation() {
    let mut doc = captured();
    let options = OcdrawToCadOptions {
        preservation_restore: OcdrawPreservationRestore::Skip,
        ..Default::default()
    };
    let out = ocdraw_document_to_cad_document(&doc, options).unwrap();
    assert!(!out
        .document()
        .entities()
        .any(|e| matches!(e, EntityType::Spline(_))));
    assert!(!out.diagnostics().is_empty());
    assert!(matches!(
        ocdraw_document_to_cad_document(
            &doc,
            OcdrawToCadOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..options
            }
        ),
        Err(OcdrawToCadError::LossRejected { .. })
    ));
    let id = doc.opaque_entities[0].id;
    doc.opaque_entities.clear();
    doc.scopes[0].entities.retain(|x| *x != id);
    recompute_ocdraw_document_bounds(&mut doc).unwrap();
    let out = ocdraw_document_to_cad_document(&doc, OcdrawToCadOptions::default()).unwrap();
    assert!(out.preservation_report().entries().is_empty());
}

#[test]
fn all_transparency_bytes_and_source_default_weight_survive_common_projection() {
    let mut source = CadDocument::new();
    for alpha in 0..=255_u16 {
        let mut s = Spline::new();
        s.common.transparency = opencadcodec::Transparency::Explicit(alpha as u8);
        source.add_entity(EntityType::Spline(s)).unwrap();
    }
    let mut default = Spline::new();
    default.common.line_weight = opencadcodec::LineWeight::Default;
    source.add_entity(EntityType::Spline(default)).unwrap();
    let doc = cad_document_to_ocdraw_document(
        &source,
        CadToOcdrawOptions {
            preservation_capture: OcdrawPreservationCapture::SupportedTyped,
            ..Default::default()
        },
    )
    .unwrap()
    .into_document();
    assert!(doc.opaque_entities.last().unwrap().appearance.is_none());
    let out = ocdraw_document_to_cad_document(&doc, OcdrawToCadOptions::default()).unwrap();
    let splines = out
        .document()
        .entities()
        .filter_map(|e| {
            if let EntityType::Spline(s) = e {
                Some(s)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(splines.len(), 257);
    for (alpha, s) in (0..=255_u16).zip(&splines) {
        assert_eq!(
            s.common.transparency,
            opencadcodec::Transparency::Explicit(alpha as u8)
        );
    }
    assert_eq!(
        splines.last().unwrap().common.line_weight,
        opencadcodec::LineWeight::Default
    );
}

fn with_reference() -> OcdrawDocument {
    let mut source = CadDocument::new();
    let line = source
        .add_entity(EntityType::Line(opencadcodec::Line::from_coords(
            0., 0., 0., 1., 1., 0.,
        )))
        .unwrap();
    let mut spline = Spline::new();
    let mut data = opencadcodec::xdata::ExtendedDataRecord::new("ACAD");
    data.values = vec![
        opencadcodec::xdata::XDataValue::Handle(line),
        opencadcodec::xdata::XDataValue::LayerName("0".into()),
    ];
    spline.common.extended_data.add_record(data);
    source.add_entity(EntityType::Spline(spline)).unwrap();
    let mut doc = cad_document_to_ocdraw_document(
        &source,
        CadToOcdrawOptions {
            preservation_capture: OcdrawPreservationCapture::SupportedTyped,
            ..Default::default()
        },
    )
    .unwrap()
    .into_document();
    doc.scopes[0].entities.reverse(); // Make the required target a forward reference.
    doc.layers[0].name = "Renamed".into();
    doc
}

#[test]
fn typed_xdata_forward_handles_and_layer_names_use_constructed_current_targets() {
    let doc = with_reference();
    let out = ocdraw_document_to_cad_document(&doc, OcdrawToCadOptions::default()).unwrap();
    let s = out
        .document()
        .entities()
        .find_map(|e| {
            if let EntityType::Spline(s) = e {
                Some(s)
            } else {
                None
            }
        })
        .expect("qualified typed metadata restores");
    let line = out
        .document()
        .entities()
        .find(|e| matches!(e, EntityType::Line(_)))
        .unwrap()
        .common()
        .handle;
    let values = &s.common.extended_data.records()[0].values;
    assert_eq!(values[0], opencadcodec::xdata::XDataValue::Handle(line));
    assert_eq!(
        values[1],
        opencadcodec::xdata::XDataValue::LayerName("Renamed".into())
    );
    let owner = out
        .document()
        .block_records
        .iter()
        .find(|r| r.handle == s.common.owner_handle)
        .unwrap();
    assert_eq!(owner.entity_handles[0], s.common.handle);
    assert_eq!(owner.entity_handles[1], line);
}

#[test]
fn missing_reference_conditions_cannot_be_hidden_by_qualified_label() {
    let mut doc = with_reference();
    let r = &mut doc.preservation.as_mut().unwrap().records[0];
    r.dependency_coverage = OcdrawPreservationDependencyCoverage::Qualified;
    r.conditions
        .retain(|c| c.predicate != "openaec.ocdraw.sourceReferenceBinding");
    let out = ocdraw_document_to_cad_document(&doc, OcdrawToCadOptions::default()).unwrap();
    assert!(!out
        .document()
        .entities()
        .any(|e| matches!(e, EntityType::Spline(_))));
    assert!(out
        .preservation_report()
        .entries()
        .iter()
        .any(|e| e.reason == Some(OcdrawPreservationReason::UnsupportedContext)));
}

#[test]
fn generic_live_nonentity_roots_are_reported_and_rejected_but_archives_are_not() {
    let mut doc = captured();
    let p = doc.preservation.as_mut().unwrap();
    let mut r = p.records[0].clone();
    r.id = OcdrawPreservationRecordId(2);
    r.category = OcdrawPreservationCategory::Object;
    r.subject = None;
    r.payload.schema = "unknown.object".into();
    p.next_record_id = 3;
    p.records.push(r);
    let out = ocdraw_document_to_cad_document(&doc, OcdrawToCadOptions::default()).unwrap();
    assert!(out
        .preservation_report()
        .entries()
        .iter()
        .any(|e| e.record_id == Some(OcdrawPreservationRecordId(2))
            && e.reason == Some(OcdrawPreservationReason::UnsupportedPayload)));
    assert!(matches!(
        ocdraw_document_to_cad_document(
            &doc,
            OcdrawToCadOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(OcdrawToCadError::LossRejected { .. })
    ));
}

#[test]
fn malformed_coordinate_baseline_has_a_payload_reason_not_a_native_edit_reason() {
    let mut doc = captured();
    let record = &mut doc.preservation.as_mut().unwrap().records[0];
    let c = record
        .conditions
        .iter_mut()
        .find(|c| c.predicate == "openaec.ocdraw.splineCoordinateContext")
        .unwrap();
    c.baseline = br#"{"kind":"NoSuchScope","coordinateUnit":"unitless"}"#.to_vec();
    let out = ocdraw_document_to_cad_document(&doc, OcdrawToCadOptions::default()).unwrap();
    assert!(out
        .preservation_report()
        .entries()
        .iter()
        .any(|e| e.reason == Some(OcdrawPreservationReason::MalformedPayload)));
}

#[test]
fn deleting_a_soft_reference_keeps_payload_but_refuses_live_restore() {
    let mut doc = with_reference();
    let before = doc.preservation.clone();
    let id = doc.geometric_entities[0].id;
    doc.geometric_entities.clear();
    for scope in &mut doc.scopes {
        scope.entities.retain(|e| *e != id);
    }
    recompute_ocdraw_document_bounds(&mut doc).unwrap();
    validate_ocdraw_document(&doc).unwrap();
    let out = ocdraw_document_to_cad_document(&doc, OcdrawToCadOptions::default()).unwrap();
    assert!(!out
        .document()
        .entities()
        .any(|e| matches!(e, EntityType::Spline(_))));
    assert!(out
        .preservation_report()
        .entries()
        .iter()
        .any(|e| e.reason == Some(OcdrawPreservationReason::MissingDependency)));
    assert_eq!(doc.preservation, before);
}

#[test]
fn owner_move_unknown_predicates_and_record_groups_never_grant_eligibility() {
    let mut doc = captured();
    let id = doc.opaque_entities[0].id;
    doc.scopes[0].entities.clear();
    doc.scopes.push(DrawingScope {
        id: 1,
        kind: DrawingScopeKind::Paper,
        bounds: None,
        bounds_quality: None,
        entities: vec![id],
    });
    doc.layouts.push(DrawingLayout {
        id: 1,
        kind: DrawingLayoutKind::Paper,
        scope_id: 1,
        tab_index: 1,
        name: "Sheet".into(),
        settings: LayoutSettings::default(),
    });
    doc.next_layout_id = 2;
    let out = ocdraw_document_to_cad_document(&doc, OcdrawToCadOptions::default()).unwrap();
    assert!(out
        .preservation_report()
        .entries()
        .iter()
        .any(|e| e.reason == Some(OcdrawPreservationReason::ChangedDependency)));
    let mut unknown = captured();
    unknown.preservation.as_mut().unwrap().records[0]
        .conditions
        .push(OcdrawPreservationCondition {
            target: OcdrawPreservationTarget::Drawing,
            predicate: "future.unknown".into(),
            version: 1,
            baseline: vec![],
        });
    let out = ocdraw_document_to_cad_document(&unknown, OcdrawToCadOptions::default()).unwrap();
    assert!(out
        .preservation_report()
        .entries()
        .iter()
        .any(|e| e.reason == Some(OcdrawPreservationReason::UnsupportedPredicate)));
    let mut cycle = captured();
    let r = &mut cycle.preservation.as_mut().unwrap().records[0];
    r.bindings.push(OcdrawPreservationBinding {
        slot: "shared".into(),
        source_key: "x".into(),
        target: OcdrawPreservationTarget::Record(r.id),
    });
    r.conditions.push(OcdrawPreservationCondition {
        target: OcdrawPreservationTarget::Record(r.id),
        predicate: "openaec.ocdraw.sourceReferenceBinding".into(),
        version: 1,
        baseline: br#"{"slot":"shared","sourceKey":"x","targetRole":"record"}"#.to_vec(),
    });
    validate_ocdraw_document(&cycle).unwrap();
    let out = ocdraw_document_to_cad_document(&cycle, OcdrawToCadOptions::default()).unwrap();
    assert!(out
        .preservation_report()
        .entries()
        .iter()
        .any(|e| e.reason == Some(OcdrawPreservationReason::UnsupportedContext)));
}

#[test]
fn model_scope_reference_uses_role_identity_after_layout_rename() {
    let mut source = CadDocument::new();
    let mut spline = Spline::new();
    let mut data = opencadcodec::xdata::ExtendedDataRecord::new("ACAD");
    data.values.push(opencadcodec::xdata::XDataValue::Handle(
        source.header.model_space_block_handle,
    ));
    spline.common.extended_data.add_record(data);
    source.add_entity(EntityType::Spline(spline)).unwrap();
    let mut doc = cad_document_to_ocdraw_document(
        &source,
        CadToOcdrawOptions {
            preservation_capture: OcdrawPreservationCapture::SupportedTyped,
            ..Default::default()
        },
    )
    .unwrap()
    .into_document();
    doc.layouts
        .iter_mut()
        .find(|l| l.kind == DrawingLayoutKind::Model)
        .unwrap()
        .name = "Renamed model tab".into();
    let out = ocdraw_document_to_cad_document(&doc, OcdrawToCadOptions::default()).unwrap();
    let spline = out
        .document()
        .entities()
        .find_map(|e| {
            if let EntityType::Spline(s) = e {
                Some(s)
            } else {
                None
            }
        })
        .expect("layout name is not an opaque geometry dependency");
    assert_eq!(
        spline.common.extended_data.records()[0].values[0],
        opencadcodec::xdata::XDataValue::Handle(out.document().header.model_space_block_handle)
    );
}
