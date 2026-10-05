#[path = "support/viewport_clips.rs"]
mod support;
use ocdraw::ocdraw::*;
use ocdraw_convert::*;
use opencadcodec::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter};
use std::io::Cursor;

fn physical(d: &CadDocument, dwg: bool) -> CadDocument {
    if dwg {
        DwgReader::from_stream(Cursor::new(DwgWriter::write_to_vec(d).unwrap()))
            .read()
            .unwrap()
    } else {
        DxfReader::from_reader(Cursor::new(DxfWriter::new(d).write_to_vec().unwrap()))
            .unwrap()
            .read()
            .unwrap()
    }
}
fn verify(d: &OcdrawDocument, kind: &str, forward: bool, bulge: f64) {
    validate_ocdraw_document(d).unwrap();
    assert_eq!(d.viewports.len(), 1, "{kind}/{forward}/{bulge}");
    let v = &d.viewports[0];
    assert!(v.paper_clip.enabled);
    assert!(v.view_locked);
    assert!((v.view.twist - std::f64::consts::PI / 6.).abs() < 1e-12);
    assert_eq!(
        (v.frame.width, v.frame.height, v.view.height),
        (12., 12., 20.)
    );
    let b = d
        .geometric_entities
        .iter()
        .find(|e| Some(e.id) == v.paper_clip.boundary_entity_id)
        .unwrap();
    assert!(!b.visible);
    match &b.geometry {
        DrawingGeometry::Circle { placement, radius } => {
            assert_eq!(kind, "circle");
            assert_eq!(*radius, 2.);
            assert_eq!(*placement, CoordinateFrame3::default());
        }
        DrawingGeometry::Ellipse {
            placement,
            semi_major_radius,
            semi_minor_radius,
            arc,
        } => {
            assert_eq!(kind, "ellipse");
            assert!((*semi_major_radius - 5.).abs() < 1e-12);
            assert!((*semi_minor_radius - 2.5).abs() < 1e-12);
            assert_eq!(*arc, None);
            // Independently known major endpoint (3,4), allowing the converter's numerical policy.
            let p = placement.try_to_scope_point(Point2::new(5., 0.)).unwrap();
            assert!((p.x() - 3.).abs() < 1e-12 && (p.y() - 4.).abs() < 1e-12);
        }
        DrawingGeometry::PlanarPolyline {
            vertices, closed, ..
        } => {
            assert!(matches!(kind, "polyline" | "legacy"));
            assert!(*closed);
            assert_eq!(vertices, &vec![[-1., 0., bulge], [1., 0., bulge]]);
        }
        _ => panic!("unsupported returned boundary"),
    }
    let scope = d
        .scopes
        .iter()
        .find(|s| s.entities.contains(&v.id))
        .unwrap();
    let roles = scope
        .entities
        .iter()
        .map(|id| {
            if *id == v.id {
                "viewport"
            } else if *id == b.id {
                "boundary"
            } else {
                "line"
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(
        roles,
        if forward {
            vec!["line", "viewport", "boundary", "line"]
        } else {
            vec!["line", "boundary", "viewport", "line"]
        }
    );
}
fn exchange(dwg: bool) {
    for kind in support::KINDS {
        for forward in [false, true] {
            for bulge in if kind == "polyline" || kind == "legacy" {
                vec![1., -1., 2.]
            } else {
                vec![1.]
            } {
                let (mut source, _, handle) = support::source(kind, forward, bulge);
                let opencadcodec::EntityType::Viewport(viewport) =
                    source.get_entity_mut(handle).unwrap()
                else {
                    panic!()
                };
                viewport.twist_angle = std::f64::consts::PI / 6.;
                let decoded = physical(&source, dwg);
                let exported =
                    cad_document_to_encoded_ocdraw(&decoded, CadToOcdrawOptions::default())
                        .unwrap();
                let loaded = load_ocdraw_bytes(exported.encoded().bytes()).unwrap();
                verify(loaded.document(), kind, forward, bulge);
                let imported =
                    ocdraw_source_to_cad_document(&loaded, OcdrawToCadOptions::default()).unwrap();
                let returned = physical(imported.document(), dwg);
                let encoded =
                    cad_document_to_encoded_ocdraw(&returned, CadToOcdrawOptions::default())
                        .unwrap();
                let read = load_ocdraw_bytes(encoded.encoded().bytes()).unwrap();
                verify(read.document(), kind, forward, bulge);
            }
        }
    }
    // Also prove authored native forward references survive actual CAD IO.
    for kind in ["circle", "ellipse", "polyline"] {
        let native = support::native(kind, true);
        let target =
            ocdraw_document_to_cad_document(&native, OcdrawToCadOptions::default()).unwrap();
        let returned = physical(target.document(), dwg);
        let exported =
            cad_document_to_encoded_ocdraw(&returned, CadToOcdrawOptions::default()).unwrap();
        let loaded = load_ocdraw_bytes(exported.encoded().bytes()).unwrap();
        assert_eq!(loaded.viewports().len(), 1);
        assert!(loaded.viewports()[0].paper_clip.enabled);
        assert_eq!(loaded.viewports()[0].frame, native.viewports[0].frame);
    }
}
#[test]
fn dxf_viewport_clip_families_roundtrip() {
    exchange(false);
}
#[test]
fn dwg_viewport_clip_families_roundtrip() {
    exchange(true);
}

#[test]
fn pinned_dwg_without_overall_canvas_reclassifies_the_authored_viewport() {
    let mut native = support::native("circle", true);
    native.paper_canvases.clear();
    let target = ocdraw_document_to_cad_document(&native, OcdrawToCadOptions::default()).unwrap();
    let decoded = physical(target.document(), true);
    let returned = cad_document_to_encoded_ocdraw(&decoded, CadToOcdrawOptions::default()).unwrap();
    let loaded = load_ocdraw_bytes(returned.encoded().bytes()).unwrap();
    assert!(loaded.viewports().is_empty());
    assert_eq!(loaded.paper_canvases().len(), 1);
}

#[test]
fn dormant_clips_survive_dxf_and_dwg_without_activation() {
    for kind in ["circle", "ellipse", "polyline"] {
        for forward in [false, true] {
            let mut native = support::native(kind, forward);
            native.viewports[0].paper_clip.enabled = false;
            // Dormant geometry is deliberately outside the frame and remains valid.
            native.viewports[0].frame.width = 0.5;
            native.viewports[0].frame.height = 0.5;
            validate_ocdraw_document(&native).unwrap();
            let imported =
                ocdraw_document_to_cad_document(&native, OcdrawToCadOptions::default()).unwrap();
            for dwg in [false, true] {
                let returned = physical(imported.document(), dwg);
                let encoded =
                    cad_document_to_encoded_ocdraw(&returned, CadToOcdrawOptions::default())
                        .unwrap();
                let loaded = load_ocdraw_bytes(encoded.encoded().bytes()).unwrap();
                assert_eq!(loaded.viewports().len(), 1);
                let viewport = &loaded.viewports()[0];
                assert!(!viewport.paper_clip.enabled);
                assert_eq!(viewport.frame, native.viewports[0].frame);
                let boundary = loaded
                    .geometric_entities()
                    .iter()
                    .find(|g| Some(g.id) == viewport.paper_clip.boundary_entity_id)
                    .unwrap();
                assert_eq!(boundary.geometry, native.geometric_entities[0].geometry);
                assert!(!boundary.visible);
            }
        }
    }
}
