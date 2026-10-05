mod common;
#[path = "common/perspective_reference.rs"]
mod perspective_reference;
use ifcx_cad_convert::*;
use ocdraw::ifcx_cad::*;
use opencadcodec::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter, EntityType};
use perspective_reference::*;
use std::io::Cursor;

#[test]
fn independent_perspective_fixture_establishes_camera_convention() {
    let reference = DxfReader::from_reader(Cursor::new(include_bytes!(
        "fixtures/viewports/reference-perspective.dxf"
    )))
    .unwrap()
    .read()
    .unwrap();
    let cases = load_perspective_reference_cases();
    for case in &cases {
        assert_reference_view(case, &reference);
    }
    for unit in [0, 1, 4] {
        let mut document = CadDocument::new();
        document.header.insertion_units = unit;
        for entity in reference.entities() {
            if let EntityType::Viewport(v) = entity {
                let mut v = v.clone();
                v.common.handle = opencadcodec::Handle::NULL;
                v.common.owner_handle = opencadcodec::Handle::NULL;
                document
                    .add_entity_to_layout(EntityType::Viewport(v), "Layout1")
                    .unwrap();
            }
        }
        for dwg in [false, true] {
            let restored = if dwg {
                DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(&document).unwrap()))
                    .read()
                    .unwrap()
            } else {
                DxfReader::from_reader(Cursor::new(
                    DxfWriter::new(&document).write_to_vec().unwrap(),
                ))
                .unwrap()
                .read()
                .unwrap()
            };
            for case in &cases {
                assert_reference_view(case, &restored);
            }
        }
    }
}

#[test]
fn multiple_paper_viewports_exchange_in_memory_dxf_dwg() {
    let mut drawing = common::nested([0.; 3]);
    drawing.paper_layouts = common::viewport_drawing().paper_layouts;
    let mut second = drawing.paper_layouts[0].clone();
    second.id = 43;
    second.name = "Other".into();
    second.tab_index = 2;
    second.length_unit = "in".into();
    for e in &mut second.entities {
        e.id += 1000;
    }
    let IfcxCadEntityKind::Viewport(v) = &mut second.entities[1].kind else {
        panic!()
    };
    v.paper_clip.boundary_entity_id = Some(2002);
    v.view.projection = IfcxCadViewportProjection::Orthographic;
    v.view.lens_length_mm = Some(0.);
    v.view.front_clip = IfcxCadViewportDepthClip {
        mode: IfcxCadViewportClipMode::AtDistance,
        distance: Some(-10.),
    };
    v.view.back_clip = IfcxCadViewportDepthClip {
        mode: IfcxCadViewportClipMode::AtDistance,
        distance: Some(-20.),
    };
    v.visible = true;
    v.view_enabled = true;
    v.view_locked = false;
    v.render_mode = IfcxCadViewportRenderMode::FlatShadedWithEdges;
    second.entities[2].kind = IfcxCadEntityKind::PlanarPolyline {
        vertices: vec![[20., 25.], [180., 25.], [180., 125.], [20., 125.]],
        closed: true,
        placement: IfcxCadPlacement {
            origin: [0.; 3],
            x_axis: [1., 0., 0.],
            y_axis: [0., 1., 0.],
        },
        line_pattern_generation: IfcxCadLinePatternGeneration::PerSegment,
    };
    drawing.paper_layouts.push(second);
    drawing.paper_layouts.push(IfcxCadPaperLayout {
        id: 44,
        name: "Empty".into(),
        tab_index: 3,
        length_unit: "unitless".into(),
        paper: None,
        entities: vec![],
    });
    drawing.id_counters.next_layout_id = 45;
    let encoded = encode_ifcx_cad_document(&drawing).unwrap();
    let source = load_ifcx_cad_bytes(encoded.bytes(), Default::default()).unwrap();
    let cad = ifcx_cad_source_to_cad_document(
        &source,
        IfcxCadToCadOptions {
            loss_policy: IfcxCadLossPolicy::Reject,
        },
    )
    .unwrap();
    for format in [0, 1, 2] {
        let restored = match format {
            0 => cad.document().clone(),
            1 => DxfReader::from_reader(Cursor::new(
                DxfWriter::new(cad.document()).write_to_vec().unwrap(),
            ))
            .unwrap()
            .read()
            .unwrap(),
            _ => DwgReader::from_stream(Cursor::new(
                DwgWriter::write_to_vec(cad.document()).unwrap(),
            ))
            .read()
            .unwrap(),
        };
        let native = common::from_cad(&restored, common::metadata()).unwrap();
        let out = native.validated_source().document();
        assert_eq!(out.paper_layouts.len(), 3);
        for (expected, actual) in drawing.paper_layouts.iter().zip(&out.paper_layouts) {
            assert_eq!(expected.name, actual.name);
            assert_eq!(expected.entities.len(), actual.entities.len());
            if actual.entities.is_empty() {
                continue;
            }
            let IfcxCadEntityKind::Viewport(ev) = &expected.entities[1].kind else {
                panic!()
            };
            let IfcxCadEntityKind::Viewport(av) = &actual.entities[1].kind else {
                panic!()
            };
            assert_eq!(ev.frame, av.frame);
            assert_eq!(ev.view, av.view);
            assert_eq!(ev.render_mode, av.render_mode);
            assert_eq!(
                (ev.visible, ev.view_enabled, ev.view_locked),
                (av.visible, av.view_enabled, av.view_locked)
            );
            assert_eq!(
                av.paper_clip.boundary_entity_id,
                Some(actual.entities[2].id)
            );
            assert_eq!(av.model_id, out.model.id);
            assert_eq!(expected.entities[2].kind, actual.entities[2].kind);
            let names = |d: &IfcxCadDocument, ids: &[u64]| {
                ids.iter()
                    .map(|id| d.layers.iter().find(|l| l.id == *id).unwrap().name.clone())
                    .collect::<Vec<_>>()
            };
            assert_eq!(
                names(&drawing, &ev.frozen_layers),
                names(out, &av.frozen_layers)
            );
        }
    }
}

#[test]
fn perspective_exchange_matches_independent_reference() {
    let literal = DxfReader::from_reader(Cursor::new(include_bytes!(
        "fixtures/viewports/reference-perspective.dxf"
    )))
    .unwrap()
    .read()
    .unwrap();
    let cases = load_perspective_reference_cases();
    for unit in ["mm", "in", "unitless"] {
        let mut cad = common::cad();
        cad.add_layout("Sheet").unwrap();
        for e in literal.entities() {
            if let EntityType::Viewport(v) = e {
                let mut v = v.clone();
                v.common.handle = opencadcodec::Handle::NULL;
                cad.add_entity_to_layout(EntityType::Viewport(v), "Sheet")
                    .unwrap();
            }
        }
        let native = common::from_cad(&cad, common::metadata()).unwrap();
        let mut drawing = native.validated_source().document().clone();
        drawing.length_unit = unit.into();
        drawing.paper_layouts[0].length_unit = unit.into();
        let target = ifcx_cad_document_to_cad_document(
            &drawing,
            IfcxCadToCadOptions {
                loss_policy: IfcxCadLossPolicy::Reject,
            },
        )
        .unwrap();
        for dwg in [false, true] {
            let restored = if dwg {
                DwgReader::from_stream(Cursor::new(
                    DwgWriter::write_to_vec(target.document()).unwrap(),
                ))
                .read()
                .unwrap()
            } else {
                DxfReader::from_reader(Cursor::new(
                    DxfWriter::new(target.document()).write_to_vec().unwrap(),
                ))
                .unwrap()
                .read()
                .unwrap()
            };
            for case in &cases {
                assert_reference_view(case, &restored);
            }
            common::from_cad(&restored, common::metadata()).unwrap();
        }
    }
}
