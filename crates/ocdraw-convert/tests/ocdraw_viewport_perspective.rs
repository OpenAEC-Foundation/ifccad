#[path = "support/perspective_reference.rs"]
mod reference;
use ocdraw::ocdraw::*;
use ocdraw_convert::*;
use opencadcodec::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType, Handle};
use std::io::Cursor;

fn source(unit: i16, perspective: bool) -> CadDocument {
    let reference = DxfReader::from_reader(Cursor::new(include_bytes!(
        "fixtures/viewports/reference-perspective.dxf"
    )))
    .unwrap()
    .read()
    .unwrap();
    let mut document = CadDocument::new();
    document.header.insertion_units = unit;
    // Construct an explicit overall Paper canvas before the two authored cameras.
    // The reference DXF fragment only specifies the authored viewport entities.
    let mut canvas = opencadcodec::entities::Viewport::new();
    canvas.id = 1;
    canvas.width = 300.;
    canvas.height = 200.;
    canvas.view_height = 200.;
    document
        .add_entity_to_layout(EntityType::Viewport(canvas), "Layout1")
        .unwrap();
    for entity in reference.entities() {
        if let EntityType::Viewport(v) = entity {
            let mut v = v.clone();
            v.common.handle = Handle::NULL;
            v.common.owner_handle = Handle::NULL;
            v.status.perspective = perspective;
            document
                .add_entity_to_layout(EntityType::Viewport(v), "Layout1")
                .unwrap();
        }
    }
    document
}
fn physical(d: &CadDocument, kind: u8) -> CadDocument {
    match kind {
        0 => d.clone(),
        1 => DxfReader::from_reader(Cursor::new(DxfWriter::new(d).write_to_vec().unwrap()))
            .unwrap()
            .read()
            .unwrap(),
        _ => DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(d).unwrap()))
            .read()
            .unwrap(),
    }
}
#[test]
fn perspective_source_and_strict_native_readback_preserve_camera_in_all_transports_and_units() {
    let cases = reference::load_perspective_reference_cases();
    for unit in [0, 1, 4] {
        for transport in [0, 1, 2] {
            let source = physical(&source(unit, true), transport);
            for case in &cases {
                reference::assert_reference_view(case, &source);
            }
            let native = cad_document_to_encoded_ocdraw(&source, Default::default())
                .unwrap_or_else(|e| {
                    panic!(
                        "unit={unit},transport={transport},error={e:?}, viewports={:?}",
                        source
                            .entities()
                            .filter_map(|e| match e {
                                EntityType::Viewport(v) =>
                                    Some((v.id, v.status.perspective, v.width, v.common.handle)),
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                    )
                });
            let loaded = load_ocdraw_bytes(native.encoded().bytes()).unwrap();
            assert_eq!(loaded.viewports().len(), 2);
            for case in &cases {
                let view = &loaded
                    .viewports()
                    .iter()
                    .find(|v| v.frame.width == case.width)
                    .unwrap()
                    .view;
                assert_eq!(view.projection, DrawingProjection::Perspective);
                assert_eq!(view.direction.components(), case.direction);
                assert_eq!(view.target.components(), case.target);
                assert_eq!(view.lens_length, Some(case.lens));
                assert_eq!([view.center.x(), view.center.y()], case.view_center);
                assert_eq!(view.height, case.view_height);
            }
            let target = ocdraw_source_to_cad_document(&loaded, Default::default()).unwrap();
            let returned = physical(target.document(), transport);
            for case in &cases {
                reference::assert_reference_view(case, &returned);
            }
        }
    }
}
#[test]
fn native_authored_perspective_maps_without_using_perspective_import() {
    let mut drawing = cad_document_to_ocdraw_document(&source(4, false), Default::default())
        .unwrap()
        .into_document();
    for v in &mut drawing.viewports {
        v.view.projection = DrawingProjection::Perspective;
    }
    validate_ocdraw_document(&drawing).unwrap();
    let target = ocdraw_document_to_cad_document(
        &drawing,
        OcdrawToCadOptions {
            loss_policy: OcdrawLossPolicy::Reject,
            ..Default::default()
        },
    )
    .unwrap();
    for case in reference::load_perspective_reference_cases() {
        reference::assert_reference_view(&case, target.document());
    }
}
#[test]
fn dormant_zero_lens_and_independent_visibility_lock_and_on_state_survive() {
    let mut d = source(4, false);
    for handle in d
        .entities()
        .filter_map(|e| matches!(e, EntityType::Viewport(_)).then_some(e.common().handle))
        .collect::<Vec<_>>()
    {
        let EntityType::Viewport(v) = d.get_entity_mut(handle).unwrap() else {
            panic!()
        };
        v.lens_length = 0.;
        v.common.invisible = true;
        v.status.locked = true;
        v.turn_off();
    }
    let native = cad_document_to_encoded_ocdraw(&d, Default::default()).unwrap();
    let loaded = load_ocdraw_bytes(native.encoded().bytes()).unwrap();
    assert_eq!(loaded.viewports().len(), 2);
    assert!(loaded
        .viewports()
        .iter()
        .all(|v| v.view.lens_length == Some(0.) && !v.visible && !v.view_enabled && v.view_locked));
    let target = ocdraw_source_to_cad_document(&loaded, Default::default()).unwrap();
    let returned = physical(target.document(), 2);
    let back = cad_document_to_ocdraw_document(&returned, Default::default()).unwrap();
    assert!(back
        .document()
        .viewports
        .iter()
        .all(|v| v.view.lens_length == Some(0.) && !v.visible && !v.view_enabled && v.view_locked));
}
#[test]
fn invalid_native_perspective_camera_is_rejected_under_both_policies() {
    let base = cad_document_to_ocdraw_document(&source(4, false), Default::default())
        .unwrap()
        .into_document();
    for bad in 0..3 {
        let mut drawing = base.clone();
        let v = &mut drawing.viewports[0];
        v.view.projection = DrawingProjection::Perspective;
        match bad {
            0 => v.view.lens_length = Some(0.),
            1 => v.view.direction = ocdraw::ocdraw::Vector3::new(0., 0., 0.),
            _ => {
                v.view.front_clip = DrawingClip {
                    mode: DrawingClipMode::AtDistance,
                    distance: Some(1.),
                };
                v.view.back_clip = DrawingClip {
                    mode: DrawingClipMode::AtDistance,
                    distance: Some(2.),
                };
            }
        }
        for loss_policy in [OcdrawLossPolicy::Allow, OcdrawLossPolicy::Reject] {
            assert!(ocdraw_document_to_cad_document(
                &drawing,
                OcdrawToCadOptions {
                    loss_policy,
                    ..Default::default()
                }
            )
            .is_err());
        }
    }
}

#[test]
fn invalid_source_perspective_is_diagnosed_and_skipped_without_workspace_refs() {
    for bad in 0..3 {
        let mut d = source(4, true);
        let handle = d
            .entities()
            .find_map(|e| match e {
                EntityType::Viewport(v) if v.width == 160. => Some(v.common.handle),
                _ => None,
            })
            .unwrap();
        let EntityType::Viewport(v) = d.get_entity_mut(handle).unwrap() else {
            panic!()
        };
        match bad {
            0 => v.lens_length = 0.,
            1 => v.view_direction = opencadcodec::Vector3::ZERO,
            _ => {
                v.status.front_clipping = true;
                v.status.front_clip_not_at_eye = true;
                v.status.back_clipping = true;
                v.front_clip_z = 1.;
                v.back_clip_z = 2.;
            }
        }
        let native = cad_document_to_ocdraw_document(&d, Default::default()).unwrap();
        assert!(!native
            .document()
            .viewports
            .iter()
            .any(|v| v.frame.width == 160.));
        assert!(native.document().viewport_workspaces.iter().all(|w| native
            .document()
            .viewports
            .iter()
            .any(|v| v.id == w.viewport_entity_id)));
        assert!(!native.diagnostics().is_empty());
        assert!(cad_document_to_ocdraw_document(
            &d,
            CadToOcdrawOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        )
        .is_err());
    }
}
