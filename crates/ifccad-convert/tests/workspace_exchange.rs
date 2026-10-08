mod common;
use ifccad_convert::*;
use ocdraw::ifccad::*;
use opencadcodec::{DwgReader, DwgWriter, DxfReader, DxfWriter};
use std::io::Cursor;

#[test]
fn workspace_fields_survive_direct_dxf_and_dwg_exchange() {
    let original = load_ifccad_bytes(
        include_bytes!("../../../examples/ifccad/workspace-state.ifcx"),
        Default::default(),
    )
    .unwrap();
    let target = ifccad_source_to_cad_document(&original, Default::default()).unwrap();
    assert!(target
        .diagnostics()
        .iter()
        .any(|d| d.location.contains("grid.adaptive")));
    for (format, cad) in [
        ("direct", target.document().clone()),
        (
            "dxf",
            DxfReader::from_reader(Cursor::new(
                DxfWriter::new(target.document()).write_to_vec().unwrap(),
            ))
            .unwrap()
            .read()
            .unwrap(),
        ),
        (
            "dwg",
            DwgReader::from_stream(Cursor::new(
                DwgWriter::write_to_vec(target.document()).unwrap(),
            ))
            .read()
            .unwrap(),
        ),
    ] {
        let outcome =
            cad_document_to_encoded_ifccad(&cad, common::metadata(), Default::default()).unwrap();
        let actual = outcome.validated_source().document();
        let expected = original.document();
        assert_eq!(
            actual.model_windows.len(),
            expected.model_windows.len(),
            "{format}"
        );
        for (a, b) in actual.model_windows.iter().zip(&expected.model_windows) {
            assert_eq!(a.rectangle, b.rectangle, "{format}");
            assert_eq!(a.grid, b.grid);
            assert_eq!(a.snap, b.snap);
            assert_eq!(a.use_stored_ucs, b.use_stored_ucs);
        }
        assert_eq!(
            actual
                .ucs_definitions
                .iter()
                .map(|u| (&u.name, u.frame, u.elevation))
                .collect::<Vec<_>>(),
            expected
                .ucs_definitions
                .iter()
                .map(|u| (&u.name, u.frame, u.elevation))
                .collect::<Vec<_>>(),
            "{format}"
        );
        assert_eq!(
            actual.paper_layouts.len(),
            expected.paper_layouts.len(),
            "{format}"
        );
        for p in &expected.paper_layouts {
            let a = actual
                .paper_layouts
                .iter()
                .find(|a| a.name == p.name)
                .unwrap();
            assert_eq!(a.entities.len(), p.entities.len());
            let canvas = a.canvas.as_ref().unwrap();
            let b = p.canvas.as_ref().unwrap();
            assert_eq!(canvas.frame, b.frame, "{format}");
            assert_eq!(canvas.grid, file_grid(b.grid, format), "{format}");
            assert_eq!(canvas.snap, b.snap);
            assert_eq!(canvas.current_ucs, None);
            assert_eq!(canvas.active_context, None);
            assert_eq!(canvas.use_stored_ucs, b.use_stored_ucs);
            for (a, b) in a.entities.iter().zip(&p.entities) {
                if let (IfccadEntityKind::Viewport(a), IfccadEntityKind::Viewport(b)) =
                    (&a.as_native().unwrap().kind, &b.as_native().unwrap().kind)
                {
                    let mut expected_workspace = b.workspace.clone().unwrap();
                    expected_workspace.grid = file_grid(expected_workspace.grid, format);
                    assert_eq!(a.workspace.as_ref(), Some(&expected_workspace), "{format}");
                    assert_eq!(a.view_enabled, b.view_enabled);
                    assert_eq!(a.view_locked, b.view_locked);
                }
            }
        }
    }
}

fn file_grid(
    mut grid: ocdraw::workspace_kernel::WorkspaceGrid,
    format: &str,
) -> ocdraw::workspace_kernel::WorkspaceGrid {
    if format != "direct" {
        grid.beyond_limits = false;
        grid.adaptive = false;
        grid.subdivision = false;
        grid.follows_workplane = false;
    }
    grid
}

#[test]
fn paper_mode_with_two_layouts_retains_the_active_block_owner() {
    let mut source = opencadcodec::CadDocument::new();
    source.add_layout("Second").unwrap();
    source.header.show_model_space = false;
    let native = cad_document_to_ifccad_document(&source, common::metadata(), Default::default())
        .unwrap()
        .into_document();
    let encoded = encode_ifccad_document(&native).unwrap();
    let loaded = load_ifccad_bytes(encoded.bytes(), Default::default()).unwrap();
    let native = loaded.document();
    let expected = native
        .paper_layouts
        .iter()
        .find(|p| p.name == "Layout1")
        .unwrap()
        .id;
    assert_eq!(
        native.workspace_state.as_ref().unwrap().active_layout_id,
        Some(expected)
    );
}

#[test]
fn active_secondary_paper_layout_does_not_corrupt_roles() {
    let mut source = opencadcodec::CadDocument::new();
    source.add_layout("Second").unwrap();
    source
        .add_entity_to_layout(
            opencadcodec::EntityType::Line(opencadcodec::Line::from_points(
                opencadcodec::Vector3::ZERO,
                opencadcodec::Vector3::new(1., 0., 0.),
            )),
            "Layout1",
        )
        .unwrap();
    source
        .add_entity_to_layout(
            opencadcodec::EntityType::Line(opencadcodec::Line::from_coords(2., 0., 0., 3., 0., 0.)),
            "Second",
        )
        .unwrap();
    let mut native =
        cad_document_to_ifccad_document(&source, common::metadata(), Default::default())
            .unwrap()
            .into_document();
    let selected = native
        .paper_layouts
        .iter()
        .find(|l| l.name == "Second")
        .unwrap()
        .id;
    native.workspace_state.as_mut().unwrap().active_layout_id = Some(selected);
    let target = ifccad_document_to_cad_document(&native, Default::default()).unwrap();
    assert!(!target
        .diagnostics()
        .iter()
        .any(|d| d.location.contains("activeLayout")));
    let selected_block = target
        .document()
        .objects
        .values()
        .find_map(|o| match o {
            opencadcodec::objects::ObjectType::Layout(l) if l.name == "Second" => {
                Some(l.block_record)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(
        target
            .document()
            .block_records
            .get("*Paper_Space")
            .unwrap()
            .handle,
        selected_block
    );
    assert_eq!(
        target.document().header.paper_space_block_handle,
        selected_block
    );
    for format in [false, true] {
        let cad = if format {
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
        for (name, start_x) in [("Layout1", 0.), ("Second", 2.)] {
            let owner = cad
                .objects
                .values()
                .find_map(|o| match o {
                    opencadcodec::objects::ObjectType::Layout(l) if l.name == name => {
                        Some(l.block_record)
                    }
                    _ => None,
                })
                .unwrap();
            assert!(cad.entities().any(|e|matches!(e,opencadcodec::EntityType::Line(l) if l.common.owner_handle == owner && l.start.x == start_x)));
        }
        let actual = cad_document_to_ifccad_document(&cad, common::metadata(), Default::default())
            .unwrap()
            .into_document();
        let active = actual.workspace_state.unwrap().active_layout_id;
        let expected = actual
            .paper_layouts
            .iter()
            .find(|p| p.name == "Second")
            .unwrap()
            .id;
        assert_eq!(active, Some(expected), "dwg={format}");
        assert_eq!(actual.paper_layouts.len(), 2);
    }
}

#[test]
fn canvas_display_state_is_diagnosed_without_discarding_aids() {
    let mut source = opencadcodec::CadDocument::new();
    source.add_layout("Second").unwrap();
    for v in source.entities_mut() {
        if let opencadcodec::EntityType::Viewport(v) = v {
            v.status.locked = true;
            v.turn_off();
        }
    }
    let out =
        cad_document_to_ifccad_document(&source, common::metadata(), Default::default()).unwrap();
    assert!(out.document().paper_layouts[0].canvas.is_some());
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.location.ends_with(".canvas.status")));
}
