mod common;
use common::*;
use ifccad_convert::*;
use ocdraw::ifccad::*;
use opencadcodec::{EntityType, Handle, Vector3};

fn source() -> (opencadcodec::CadDocument, Handle, Handle) {
    let mut d = cad();
    d.add_layout("Sheet").unwrap();
    let mut v = opencadcodec::entities::Viewport::new();
    v.id = 2;
    v.center = Vector3::new(100., 75., 0.);
    v.width = 160.;
    v.height = 100.;
    v.view_height = 200.;
    v.view_direction = Vector3::new(0., 0., 100.);
    v.status = opencadcodec::entities::ViewportStatusFlags::from_bits(0x10000 | 0x4000 | 1);
    v.common.invisible = true;
    v.render_mode = opencadcodec::entities::ViewportRenderMode::Wireframe3D;
    v.frozen_layers = vec![d.layers.get("Notes").unwrap().handle];
    let viewport = d
        .add_entity_to_layout(EntityType::Viewport(v), "Sheet")
        .unwrap();
    let circle = d
        .add_entity_to_layout(
            EntityType::Circle(opencadcodec::Circle::from_center_radius(
                Vector3::new(100., 75., 0.),
                50.,
            )),
            "Sheet",
        )
        .unwrap();
    let EntityType::Viewport(v) = d.get_entity_mut(viewport).unwrap() else {
        panic!()
    };
    v.clip_boundary_handle = circle;
    (d, viewport, circle)
}

#[test]
fn cad_import_resolves_later_circle_without_reordering_and_preserves_display_states() {
    let (d, handle, boundary) = source();
    let out = from_cad(&d, metadata()).unwrap();
    let sheet = &out.validated_source().document().paper_layouts[0];
    assert_eq!(sheet.entities.len(), 2);
    assert_eq!(
        out.mappings().entities.cad_handle(sheet.entities[0].id),
        Some(handle)
    );
    assert_eq!(
        out.mappings().entities.cad_handle(sheet.entities[1].id),
        Some(boundary)
    );
    let IfccadEntityKind::Viewport(v) = &sheet.entities[0].kind else {
        panic!()
    };
    assert_eq!(v.paper_clip.boundary_entity_id, Some(sheet.entities[1].id));
    assert!(!v.visible);
    assert!(!v.view_enabled);
    assert!(v.view_locked);
    assert_eq!(v.view.direction, [0., 0., 100.]);
    assert_eq!(v.frozen_layers.len(), 1);
}

#[test]
fn export_preallocates_forward_boundary_handles_and_assigns_runtime_numbers() {
    let source = viewport_drawing();
    let out = ifccad_document_to_cad_document(
        &source,
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        },
    )
    .unwrap();
    let handle = out.mappings().entities.cad_handle(1001).unwrap();
    let EntityType::Viewport(v) = out.document().get_entity(handle).unwrap() else {
        panic!()
    };
    assert_eq!(v.id, 2);
    assert_eq!(
        v.clip_boundary_handle,
        out.mappings().entities.cad_handle(1002).unwrap()
    );
    assert_eq!(v.status.to_bits() & 0x10000, 0x10000);
    assert!(v.common.invisible);
    assert_eq!(v.status.to_bits() & 0x20000, 0x20000);
    assert!(v.status.locked);
    let owner = out
        .document()
        .block_records
        .iter()
        .find(|b| b.handle == v.common.owner_handle)
        .unwrap();
    let authored: Vec<_> = owner
        .entity_handles
        .iter()
        .filter(|h| **h == handle || out.mappings().entities.ifccad_id(**h).is_some())
        .copied()
        .collect();
    assert_eq!(
        authored,
        [1000, 1001, 1002].map(|id| out.mappings().entities.cad_handle(id).unwrap())
    );
    let restored = from_cad(out.document(), metadata()).unwrap();
    let IfccadEntityKind::Viewport(v) =
        &restored.validated_source().document().paper_layouts[0].entities[1].kind
    else {
        panic!()
    };
    assert_eq!(v.view, native_viewport().view);
}

#[test]
fn contradictory_clip_ownership_and_shared_boundaries_are_structural_errors() {
    for cross_owner in [false, true] {
        let (mut document, handle, _) = source();
        if cross_owner {
            let foreign = document
                .add_entity(EntityType::Circle(
                    opencadcodec::Circle::from_center_radius(Vector3::new(100., 75., 0.), 50.),
                ))
                .unwrap();
            let EntityType::Viewport(v) = document.get_entity_mut(handle).unwrap() else {
                unreachable!()
            };
            v.clip_boundary_handle = foreign;
        } else {
            let EntityType::Viewport(v) = document.get_entity(handle).unwrap() else {
                unreachable!()
            };
            let mut copy = v.clone();
            copy.common.handle = Handle::NULL;
            copy.id = 3;
            document
                .add_entity_to_layout(EntityType::Viewport(copy), "Sheet")
                .unwrap();
        }
        for loss_policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
            assert!(matches!(
                cad_document_to_ifccad_document(
                    &document,
                    metadata(),
                    CadToIfccadOptions {
                        loss_policy,
                        ..Default::default()
                    },
                ),
                Err(IfccadConversionError::InvalidStructure(_))
            ));
        }
    }
}

#[test]
fn invalid_active_clip_omits_whole_viewport_but_retains_independent_geometry() {
    for missing in [false, true] {
        let (mut d, handle, _) = source();
        let EntityType::Viewport(v) = d.get_entity_mut(handle).unwrap() else {
            panic!()
        };
        if missing {
            v.clip_boundary_handle = Handle::new(0xFFFFFF);
        } else {
            v.width = 1.;
        }
        let out = cad_document_to_encoded_ifccad(&d, metadata(), Default::default()).unwrap();
        assert!(out.mappings().entities.ifccad_id(handle).is_none());
        assert_eq!(
            out.validated_source().document().paper_layouts[0]
                .entities
                .len(),
            1
        );
        assert!(out.diagnostics().iter().any(|d| d.code == "viewport-clip"));
        assert!(matches!(
            from_cad(&d, metadata()),
            Err(IfccadConversionError::Unsupported(_))
        ));
    }
}

#[test]
fn dormant_boundary_and_missing_frozen_layer_are_independent() {
    let (mut d, handle, boundary) = source();
    let EntityType::Viewport(v) = d.get_entity_mut(handle).unwrap() else {
        panic!()
    };
    v.status = opencadcodec::entities::ViewportStatusFlags::from_bits(0x8000);
    v.frozen_layers.push(Handle::new(0xABCDEF));
    let out = cad_document_to_encoded_ifccad(&d, metadata(), Default::default()).unwrap();
    let sheet = &out.validated_source().document().paper_layouts[0];
    let IfccadEntityKind::Viewport(v) = &sheet.entities[0].kind else {
        panic!()
    };
    assert!(!v.paper_clip.enabled);
    assert_eq!(
        v.paper_clip.boundary_entity_id,
        out.mappings().entities.ifccad_id(boundary)
    );
    assert_eq!(v.frozen_layers.len(), 1);
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "viewport-frozen-layer"));
    assert!(matches!(
        from_cad(&d, metadata()),
        Err(IfccadConversionError::Unsupported(_))
    ));
}

#[test]
fn viewport_visibility_and_all_render_modes_are_independent() {
    use opencadcodec::entities::ViewportRenderMode;
    for (mode, expected) in [
        (
            ViewportRenderMode::Wireframe2D,
            IfccadViewportRenderMode::TwoDimensional,
        ),
        (
            ViewportRenderMode::Wireframe3D,
            IfccadViewportRenderMode::Wireframe,
        ),
        (
            ViewportRenderMode::HiddenLine,
            IfccadViewportRenderMode::HiddenLine,
        ),
        (
            ViewportRenderMode::FlatShaded,
            IfccadViewportRenderMode::FlatShadedWithoutEdges,
        ),
        (
            ViewportRenderMode::GouraudShaded,
            IfccadViewportRenderMode::SmoothShadedWithoutEdges,
        ),
        (
            ViewportRenderMode::FlatShadedWithEdges,
            IfccadViewportRenderMode::FlatShadedWithEdges,
        ),
        (
            ViewportRenderMode::GouraudShadedWithEdges,
            IfccadViewportRenderMode::SmoothShadedWithEdges,
        ),
    ] {
        for bits in [0, 0x8000, 0x4000, 0xC000, 0x28000, 0x2C000] {
            for visible in [false, true] {
                let (mut d, handle, _) = source();
                let EntityType::Viewport(v) = d.get_entity_mut(handle).unwrap() else {
                    panic!()
                };
                v.render_mode = mode;
                v.status = opencadcodec::entities::ViewportStatusFlags::from_bits(bits);
                v.common.invisible = !visible;
                v.lens_length = 0.;
                v.front_clip_z = -20.;
                v.back_clip_z = -30.;
                let imported = from_cad(&d, metadata()).unwrap();
                let doc = imported.validated_source().document();
                let IfccadEntityKind::Viewport(v) = &doc.paper_layouts[0].entities[0].kind else {
                    panic!()
                };
                assert_eq!(v.render_mode, expected);
                assert_eq!(v.visible, visible);
                assert_eq!(v.view_enabled, bits & 0x8000 != 0 && bits & 0x20000 == 0);
                assert_eq!(v.view_locked, bits & 0x4000 != 0);
                assert_eq!(v.view.lens_length_mm, Some(0.));
                assert_eq!(v.view.front_clip.distance, Some(-20.));
                let exported = ifccad_document_to_cad_document(
                    doc,
                    IfccadToCadOptions {
                        loss_policy: IfccadLossPolicy::Reject,
                        ..Default::default()
                    },
                )
                .unwrap();
                let restored = from_cad(exported.document(), metadata()).unwrap();
                let IfccadEntityKind::Viewport(rv) =
                    &restored.validated_source().document().paper_layouts[0].entities[0].kind
                else {
                    panic!()
                };
                assert_eq!(v.view, rv.view);
                assert_eq!(v.render_mode, rv.render_mode);
                assert_eq!(
                    (v.visible, v.view_enabled, v.view_locked),
                    (rv.visible, rv.view_enabled, rv.view_locked)
                );
            }
        }
    }
}

#[test]
fn unsupported_boundary_families_and_camera_planes_omit_only_the_viewport() {
    for case in 0..6 {
        let (mut d, handle, boundary) = source();
        let EntityType::Viewport(v) = d.get_entity_mut(handle).unwrap() else {
            panic!()
        };
        match case {
            0 => v.center.z = 1.,
            1 => v.view_center.z = 1.,
            2 => v.view_direction = Vector3::ZERO,
            3 => v.lens_length = 0.,
            4 => {
                v.front_clip_z = -20.;
                v.back_clip_z = -10.;
                v.status = opencadcodec::entities::ViewportStatusFlags::from_bits(0x10017);
            }
            _ => {
                let EntityType::Circle(c) = d.get_entity_mut(boundary).unwrap() else {
                    panic!()
                };
                c.radius = 50.1;
            }
        }
        let out = cad_document_to_encoded_ifccad(&d, metadata(), Default::default()).unwrap();
        assert!(out.mappings().entities.ifccad_id(handle).is_none());
        assert!(out.mappings().entities.ifccad_id(boundary).is_some());
        assert!(matches!(
            from_cad(&d, metadata()),
            Err(IfccadConversionError::Unsupported(_))
        ));
    }
    for family in 0..4 {
        let (mut d, handle, _) = source();
        let boundary = match family {
            0 => EntityType::Ellipse(opencadcodec::Ellipse::new()),
            1 => EntityType::Spline(opencadcodec::Spline::new()),
            2 => {
                let mut p = opencadcodec::LwPolyline::new();
                p.vertices = vec![
                    opencadcodec::entities::LwVertex::new(opencadcodec::Vector2::new(100., 75.)),
                    opencadcodec::entities::LwVertex::new(opencadcodec::Vector2::new(150., 75.)),
                    opencadcodec::entities::LwVertex::new(opencadcodec::Vector2::new(150., 100.)),
                ];
                p.is_closed = true;
                p.vertices[0].bulge = 0.5;
                EntityType::LwPolyline(p)
            }
            _ => EntityType::Line(opencadcodec::Line::from_coords(
                100., 75., 0., 150., 75., 0.,
            )),
        };
        let boundary = d.add_entity_to_layout(boundary, "Sheet").unwrap();
        let EntityType::Viewport(v) = d.get_entity_mut(handle).unwrap() else {
            panic!()
        };
        v.clip_boundary_handle = boundary;
        let out = cad_document_to_encoded_ifccad(&d, metadata(), Default::default()).unwrap();
        if family == 2 {
            assert!(out.mappings().entities.ifccad_id(handle).is_some());
            assert!(!out.diagnostics().iter().any(|d| d.code == "viewport-clip"));
            continue;
        }
        assert!(out.mappings().entities.ifccad_id(handle).is_none());
        assert!(out.diagnostics().iter().any(|d| d.code == "viewport-clip"));
        assert!(matches!(
            from_cad(&d, metadata()),
            Err(IfccadConversionError::Unsupported(_))
        ));
    }
}

#[test]
fn omitted_export_boundary_never_leaves_viewport_mapping() {
    let mut drawing = viewport_drawing();
    let id = drawing.id_counters.allocate_block_id().unwrap();
    drawing.blocks.push(IfccadBlockDefinition {
        id,
        name: "*Unsupported clip reference".into(),
        base_point: [0.; 3],
        insertion_unit: "mm".into(),
        bounds: None,
        entities: vec![],
    });
    drawing.paper_layouts[0].entities[2].kind = IfccadEntityKind::BlockInstance {
        definition_id: id,
        transform: IfccadBlockTransform {
            placement: IfccadPlacement {
                origin: [0.; 3],
                x_axis: [1., 0., 0.],
                y_axis: [0., 1., 0.],
            },
            rotation: 0.,
            scale: [1.; 3],
        },
    };
    for e in &mut drawing.paper_layouts[0].entities {
        if let IfccadEntityKind::Viewport(v) = &mut e.kind {
            v.paper_clip.enabled = false;
        }
    }
    validate_ifccad_document(&drawing).unwrap();
    let out = ifccad_document_to_cad_document(&drawing, Default::default()).unwrap();
    assert!(out.mappings().entities.cad_handle(1001).is_none());
    assert!(out.mappings().entities.cad_handle(1002).is_none());
    assert!(out.mappings().entities.cad_handle(1000).is_some());
    assert!(matches!(
        ifccad_document_to_cad_document(
            &drawing,
            IfccadToCadOptions {
                loss_policy: IfccadLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(IfccadConversionError::Unsupported(_))
    ));
}

#[test]
fn zero_runtime_number_does_not_make_an_authored_viewport_scaffold() {
    let d = ifccad_document_to_cad_document(
        &viewport_drawing(),
        IfccadToCadOptions {
            loss_policy: IfccadLossPolicy::Reject,
            ..Default::default()
        },
    )
    .unwrap()
    .into_document();
    let mut d = opencadcodec::DwgReader::from_stream(std::io::Cursor::new(
        opencadcodec::DwgWriter::write_to_vec(&d).unwrap(),
    ))
    .read()
    .unwrap();
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
    v.id = 0;
    let out = from_cad(&d, metadata()).unwrap();
    assert!(out.mappings().entities.ifccad_id(handle).is_some());
}
