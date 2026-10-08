mod common;
use ifccad_convert::*;
use ocdraw::ifccad::*;
use opencadcodec::{Color, EntityType, Vector3};
mod spline_support;
use spline_support::captured;
#[test]
fn native_edits_remain_authoritative_after_durable_reopen() {
    let mut d = captured();
    let snapshot = d.preservation.clone();
    let e = d.model.entities[0].as_opaque_mut().unwrap();
    e.visible = false;
    let a = e.appearance.as_mut().unwrap();
    a.appearance.color = IfccadMode::Explicit("#aabbcc".into());
    a.line_pattern_scale = 2.;
    d.layers[0].name = "Renamed source layer".into();
    let bytes = encode_ifccad_document(&d).unwrap();
    let d = load_ifccad_bytes(bytes.bytes(), Default::default())
        .unwrap()
        .into_document();
    let restored = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    let spline = restored
        .document()
        .entities()
        .find_map(|e| {
            if let EntityType::Spline(s) = e {
                Some(s)
            } else {
                None
            }
        })
        .expect("opaque spline should restore");
    assert_eq!(spline.common.color, Color::from_rgb(170, 187, 204));
    assert!(spline.common.invisible);
    assert_eq!(spline.common.linetype_scale, 2.);
    assert_eq!(spline.common.layer, "Renamed source layer");
    assert_eq!(
        spline.common.owner_handle,
        restored.document().header.model_space_block_handle
    );
    assert_eq!(spline.control_points[1], Vector3::new(2., 4., 0.));
    assert_eq!(d.preservation, snapshot);
    assert!(restored
        .preservation_report()
        .entries()
        .iter()
        .any(|e| e.result == IfccadPreservationResult::RestoredTyped));
}
#[test]
fn every_restore_rechecks_stored_conditions_and_skip_reports_omission() {
    let original = captured();
    for mutation in 0..6 {
        let mut d = original.clone();
        let r = &mut d.preservation.as_mut().unwrap().records[0];
        match mutation {
            0 => r.conditions.clear(),
            1 => r.conditions[0].version = 2,
            2 => r.conditions[0].baseline = vec![0xff],
            3 => r.conditions.push(r.conditions[0].clone()),
            4 => d.length_unit = "in".into(),
            _ => r.conditions[0].predicate = "future".into(),
        }
        let bytes = encode_ifccad_document(&d).unwrap();
        let d = load_ifccad_bytes(bytes.bytes(), Default::default())
            .unwrap()
            .into_document();
        let result = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
        assert!(!result
            .document()
            .entities()
            .any(|e| matches!(e, EntityType::Spline(_))));
        assert!(result
            .preservation_report()
            .entries()
            .iter()
            .any(|e| e.result == IfccadPreservationResult::NotRestored));
        assert!(ifccad_document_to_cad_document(
            &d,
            IfccadToCadOptions {
                loss_policy: IfccadLossPolicy::Reject,
                ..Default::default()
            }
        )
        .is_err());
    }
    let skipped = ifccad_document_to_cad_document(
        &original,
        IfccadToCadOptions {
            preservation: IfccadPreservationRestore::Skip,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!skipped
        .document()
        .entities()
        .any(|e| matches!(e, EntityType::Spline(_))));
    assert!(!skipped.preservation_report().entries().is_empty());
}

#[test]
fn unavailable_appearance_still_rebinds_qualified_inherited_pattern_handles() {
    let d = captured();
    let mut cad = ifccad_document_to_cad_document(&d, Default::default())
        .unwrap()
        .into_document();
    let handle = cad.line_types.get("ByLayer").unwrap().handle;
    let entity = cad
        .entities()
        .find(|e| matches!(e, EntityType::Spline(_)))
        .unwrap()
        .common()
        .handle;
    let s = cad.get_entity_mut(entity).unwrap().common_mut();
    s.line_weight = opencadcodec::LineWeight::Default;
    s.linetype_handle = Some(handle);
    let native = cad_document_to_ifccad_document(
        &cad,
        IfccadTargetMetadata {
            drawing_id: 1,
            header: d.header.clone(),
        },
        CadToIfccadOptions {
            preservation: IfccadPreservationCapture::SupportedTyped,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(native.document().model.entities[0]
        .as_opaque()
        .unwrap()
        .appearance
        .is_none());
    let restored = ifccad_document_to_cad_document(native.document(), Default::default()).unwrap();
    let s = restored
        .document()
        .entities()
        .find_map(|e| {
            if let EntityType::Spline(s) = e {
                Some(s)
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            panic!(
                "qualified inherited handle should restore: {:?}; bindings {:?}",
                restored.preservation_report().entries(),
                native.document().preservation.as_ref().unwrap().records[0].bindings
            )
        });
    assert_eq!(
        s.common.linetype_handle,
        Some(
            restored
                .document()
                .line_types
                .get("ByLayer")
                .unwrap()
                .handle
        )
    );
    let wrong = cad.line_types.get("ByBlock").unwrap().handle;
    cad.get_entity_mut(entity)
        .unwrap()
        .common_mut()
        .linetype_handle = Some(wrong);
    let native = cad_document_to_ifccad_document(
        &cad,
        IfccadTargetMetadata {
            drawing_id: 1,
            header: d.header.clone(),
        },
        CadToIfccadOptions {
            preservation: IfccadPreservationCapture::SupportedTyped,
            ..Default::default()
        },
    )
    .unwrap();
    let refused = ifccad_document_to_cad_document(native.document(), Default::default()).unwrap();
    assert!(!refused
        .document()
        .entities()
        .any(|e| matches!(e, EntityType::Spline(_))));
}

fn scoped() -> IfccadDocument {
    let mut native = common::nested([0.; 3]);
    native.paper_layouts = vec![
        IfccadPaperLayout {
            bounds_quality: None,
            canvas: None,
            id: 42,
            name: "Physical".into(),
            tab_index: 1,
            bounds: None,
            settings: common::paper_settings(ocdraw::plot_kernel::PlotUnit::Inch, 5.),
            entities: vec![],
        },
        IfccadPaperLayout {
            bounds_quality: None,
            canvas: None,
            id: 43,
            name: "Unknown".into(),
            tab_index: 2,
            bounds: None,
            settings: Default::default(),
            entities: vec![],
        },
    ];
    native.id_counters.next_layout_id = 44;
    let mut cad = ifccad_document_to_cad_document(&native, Default::default())
        .unwrap()
        .into_document();
    cad.add_entity(EntityType::Spline(spline_support::spline()))
        .unwrap();
    for name in ["Physical", "Unknown"] {
        cad.add_entity_to_layout(EntityType::Spline(spline_support::spline()), name)
            .unwrap();
    }
    for name in ["Inner", "Unused"] {
        let owner = cad.block_records.get(name).unwrap().handle;
        let mut s = spline_support::spline();
        s.common.owner_handle = owner;
        cad.add_entity(EntityType::Spline(s)).unwrap();
    }
    let result = cad_document_to_ifccad_document(
        &cad,
        common::metadata(),
        CadToIfccadOptions {
            preservation: IfccadPreservationCapture::SupportedTyped,
            ..Default::default()
        },
    )
    .unwrap();
    let bytes = encode_ifccad_document(result.document()).unwrap();
    load_ifccad_bytes(bytes.bytes(), Default::default())
        .unwrap()
        .into_document()
}
fn restored_count(d: &IfccadDocument) -> usize {
    ifccad_document_to_cad_document(d, Default::default())
        .unwrap()
        .document()
        .entities()
        .filter(|e| matches!(e, EntityType::Spline(_)))
        .count()
}
#[test]
fn all_owners_nested_occurrences_and_exact_paper_contexts_are_separate() {
    let original = scoped();
    assert_eq!(restored_count(&original), 5);
    assert!(original.model.bounds.is_none());
    assert!(original.paper_layouts.iter().all(|p| p.bounds.is_none()));
    let proof = ifccad_document_to_cad_document(&original, Default::default()).unwrap();
    assert!(!proof.geometry_assessment().is_complete());
    assert!(proof
        .geometry_assessment()
        .unassessed_entities()
        .iter()
        .any(
            |e| matches!(e,IfccadGeometryEntitySource::BlockOccurrence{path,..} if path.len()==2)
        ));
    let mut d = original.clone();
    let paper = d
        .paper_layouts
        .iter_mut()
        .find(|p| p.name == "Physical")
        .unwrap();
    let id = paper.entities[0].id();
    let record = d
        .preservation
        .as_ref()
        .unwrap()
        .records
        .iter()
        .find(|r| r.subject == Some(IfccadPreservationTarget::Entity(id)))
        .unwrap();
    let baseline: serde_json::Value = serde_json::from_slice(
        &record
            .conditions
            .iter()
            .find(|c| c.predicate.ends_with("splineCoordinateContext"))
            .unwrap()
            .baseline,
    )
    .unwrap();
    assert_eq!(baseline["meaning"]["numerator"], "127");
    assert_eq!(baseline["meaning"]["denominator"], "1000");
    paper.settings = common::paper_settings(ocdraw::plot_kernel::PlotUnit::Millimetre, 127.);
    assert_eq!(restored_count(&d), 5);
    d.paper_layouts
        .iter_mut()
        .find(|p| p.name == "Physical")
        .unwrap()
        .settings = common::paper_settings(ocdraw::plot_kernel::PlotUnit::Millimetre, 254.);
    assert_eq!(restored_count(&d), 4);
    d.paper_layouts
        .iter_mut()
        .find(|p| p.name == "Physical")
        .unwrap()
        .settings = common::paper_settings(ocdraw::plot_kernel::PlotUnit::Inch, 5.);
    assert_eq!(restored_count(&d), 5);
    d.blocks
        .iter_mut()
        .find(|b| b.name == "Inner")
        .unwrap()
        .base_point = [1., 2., 3.];
    assert_eq!(restored_count(&d), 5);
    d.blocks
        .iter_mut()
        .find(|b| b.name == "Inner")
        .unwrap()
        .insertion_unit = "in".into();
    assert_eq!(restored_count(&d), 4);
    let mut d = original.clone();
    let opaque = d
        .model
        .entities
        .iter()
        .position(|e| e.as_opaque().is_some())
        .unwrap();
    let e = d.model.entities.remove(opaque);
    d.blocks
        .iter_mut()
        .find(|b| b.name == "Unused")
        .unwrap()
        .entities
        .push(e);
    assert_eq!(restored_count(&d), 4);
    assert_eq!(original.preservation, d.preservation);
}

#[test]
fn dormant_opaque_clip_survives_capture_and_failed_boundary_never_leaves_a_dangling_cad_reference()
{
    let d = captured();
    let mut cad = ifccad_document_to_cad_document(&d, Default::default())
        .unwrap()
        .into_document();
    cad.add_layout("Sheet").unwrap();
    let mut v = opencadcodec::entities::Viewport::new();
    v.id = 2;
    v.width = 100.;
    v.height = 100.;
    v.view_height = 100.;
    let viewport = cad
        .add_entity_to_layout(EntityType::Viewport(v), "Sheet")
        .unwrap();
    let boundary = cad
        .add_entity_to_layout(EntityType::Spline(spline_support::spline()), "Sheet")
        .unwrap();
    let EntityType::Viewport(v) = cad.get_entity_mut(viewport).unwrap() else {
        panic!()
    };
    v.clip_boundary_handle = boundary;
    let result = cad_document_to_ifccad_document(
        &cad,
        common::metadata(),
        CadToIfccadOptions {
            preservation: IfccadPreservationCapture::SupportedTyped,
            ..Default::default()
        },
    )
    .unwrap();
    let mut d = result.into_document();
    let paper = d.paper_layouts.iter().find(|p| p.name == "Sheet").unwrap();
    let v = paper
        .entities
        .iter()
        .filter_map(IfccadEntity::as_native)
        .find_map(|e| {
            if let IfccadEntityKind::Viewport(v) = &e.kind {
                Some(v)
            } else {
                None
            }
        })
        .expect("dormant clip viewport should be retained");
    let id = v.paper_clip.boundary_entity_id.unwrap();
    assert!(!v.paper_clip.enabled);
    assert!(paper
        .entities
        .iter()
        .any(|e| e.id() == id && e.as_opaque().is_some()));
    let restored = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    assert!(restored.document().entities().any(|e|matches!(e,EntityType::Viewport(v) if !v.clip_boundary_handle.is_null()&&matches!(restored.document().get_entity(v.clip_boundary_handle),Some(EntityType::Spline(_))))));
    d.preservation
        .as_mut()
        .unwrap()
        .records
        .iter_mut()
        .find(|r| r.subject == Some(IfccadPreservationTarget::Entity(id)))
        .unwrap()
        .conditions
        .clear();
    let refused = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    assert!(!refused
        .document()
        .entities()
        .any(|e| matches!(e,EntityType::Viewport(v) if !v.clip_boundary_handle.is_null())));
}

#[test]
fn typed_forward_references_rebind_after_reorder_and_each_reference_guard_is_required() {
    let d = captured();
    let mut cad = ifccad_document_to_cad_document(&d, Default::default())
        .unwrap()
        .into_document();
    let source_spline = cad
        .entities()
        .find(|e| matches!(e, EntityType::Spline(_)))
        .unwrap()
        .common()
        .handle;
    let line = cad
        .add_entity(EntityType::Line(opencadcodec::Line::from_coords(
            7., 8., 0., 9., 10., 0.,
        )))
        .unwrap();
    let mut data = opencadcodec::xdata::ExtendedDataRecord::new("APP");
    data.values = vec![
        opencadcodec::xdata::XDataValue::Handle(line),
        opencadcodec::xdata::XDataValue::LayerName("0".into()),
    ];
    cad.get_entity_mut(source_spline)
        .unwrap()
        .common_mut()
        .extended_data
        .add_record(data);
    let result = cad_document_to_ifccad_document(
        &cad,
        common::metadata(),
        CadToIfccadOptions {
            preservation: IfccadPreservationCapture::SupportedTyped,
            ..Default::default()
        },
    )
    .unwrap();
    let mut d = result.into_document();
    d.model.entities.reverse();
    let bytes = encode_ifccad_document(&d).unwrap();
    let d = load_ifccad_bytes(bytes.bytes(), Default::default())
        .unwrap()
        .into_document();
    let restored = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    let line = restored
        .document()
        .entities()
        .find(|e| matches!(e, EntityType::Line(_)))
        .unwrap()
        .common()
        .handle;
    let spline = restored
        .document()
        .entities()
        .find_map(|e| {
            if let EntityType::Spline(s) = e {
                Some(s)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(
        spline.common.extended_data.records()[0].values[0],
        opencadcodec::xdata::XDataValue::Handle(line)
    );
    for mutation in 0..5 {
        let mut d = d.clone();
        let r = &mut d.preservation.as_mut().unwrap().records[0];
        let slot = "common.extendedData.records[0].values[0]";
        let binding = r.bindings.iter().position(|b| b.slot == slot).unwrap();
        match mutation {
            0 => {
                r.bindings.remove(binding);
            }
            1 => r.bindings[binding].source_key = "wrong".into(),
            2 => r.bindings[binding].target = IfccadPreservationTarget::Record(r.id),
            3 => {
                let c = r
                    .conditions
                    .iter()
                    .position(|c| {
                        serde_json::from_slice::<serde_json::Value>(&c.baseline)
                            .is_ok_and(|b| b["slot"] == slot)
                    })
                    .unwrap();
                r.conditions.remove(c);
            }
            _ => {
                let target = r.bindings[binding].target;
                let IfccadPreservationTarget::Entity(id) = target else {
                    panic!()
                };
                d.model.entities.retain(|e| e.id() != id);
            }
        }
        let bytes = encode_ifccad_document(&d).unwrap();
        let d = load_ifccad_bytes(bytes.bytes(), Default::default())
            .unwrap()
            .into_document();
        let refused = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
        assert!(
            !refused
                .document()
                .entities()
                .any(|e| matches!(e, EntityType::Spline(_))),
            "mutation {mutation}"
        );
    }
}
#[test]
fn unsupported_attached_context_stays_stored_and_detached_archives_have_no_export_obligation() {
    let original = captured();
    let mut cad = ifccad_document_to_cad_document(&original, Default::default())
        .unwrap()
        .into_document();
    let h = cad
        .entities()
        .find(|e| matches!(e, EntityType::Spline(_)))
        .unwrap()
        .common()
        .handle;
    cad.get_entity_mut(h).unwrap().common_mut().graphic_data = Some(vec![0xff, 0, 3]);
    let captured = cad_document_to_ifccad_document(
        &cad,
        common::metadata(),
        CadToIfccadOptions {
            preservation: IfccadPreservationCapture::SupportedTyped,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(captured
        .preservation_report()
        .entries()
        .iter()
        .any(|e| e.result == IfccadPreservationResult::RestorationUnavailable));
    let mut d = captured.into_document();
    let bytes = encode_ifccad_document(&d).unwrap();
    d = load_ifccad_bytes(bytes.bytes(), Default::default())
        .unwrap()
        .into_document();
    assert_eq!(
        cad_preservation::decode_spline_snapshot(
            &d.preservation.as_ref().unwrap().records[0].payload.bytes
        )
        .unwrap()
        .to_source()
        .common
        .graphic_data,
        Some(vec![0xff, 0, 3])
    );
    let result = ifccad_document_to_cad_document(&d, Default::default()).unwrap();
    assert!(!result
        .document()
        .entities()
        .any(|e| matches!(e, EntityType::Spline(_))));
    d.model.entities.clear();
    d.preservation.as_mut().unwrap().records[0].subject = None;
    let archived = encode_ifccad_document(&d).unwrap();
    let d = load_ifccad_bytes(archived.bytes(), Default::default())
        .unwrap()
        .into_document();
    let out = ifccad_document_to_cad_document(
        &d,
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(out.preservation_report().entries().is_empty());
}
