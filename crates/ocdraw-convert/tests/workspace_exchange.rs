use ocdraw::ocdraw::*;
use ocdraw_convert::*;
use opencadcodec::{CadDocument, DwgReader, DwgWriter, DxfReader, DxfWriter};
use std::io::Cursor;

#[test]
fn workspace_fields_survive_direct_dxf_and_dwg_exchange() {
    let original = load_ocdraw_bytes(include_bytes!(
        "../../../examples/ocdraw/workspace-state.ocdraw.json"
    ))
    .unwrap();
    let target = ocdraw_source_to_cad_document(&original, Default::default()).unwrap();
    assert!(target
        .diagnostics()
        .iter()
        .any(|d| d.location == "/viewState/activeModelWindowId"));
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
        assert_workspace(format, &cad, original.document());
    }
}
fn assert_workspace(format: &str, cad: &CadDocument, expected: &OcdrawDocument) {
    let outcome = cad_document_to_encoded_ocdraw(cad, Default::default()).unwrap();
    let loaded = load_ocdraw_bytes(outcome.encoded().bytes()).unwrap();
    let actual = loaded.document();
    assert_eq!(
        actual.model_windows.len(),
        expected.model_windows.len(),
        "{format}"
    );
    for (a, b) in actual.model_windows.iter().zip(&expected.model_windows) {
        assert_eq!(a.rectangle, b.rectangle, "{format}");
        assert_eq!(a.grid, b.grid, "{format}");
        assert_eq!(a.snap, b.snap, "{format}");
        assert_eq!(a.use_stored_ucs, b.use_stored_ucs, "{format}");
    }
    assert_eq!(
        actual.paper_canvases.len(),
        expected.paper_canvases.len(),
        "{format}"
    );
    for canvas in &expected.paper_canvases {
        let name = &expected
            .layouts
            .iter()
            .find(|l| l.id == canvas.scope_id)
            .unwrap()
            .name;
        let scope = actual.layouts.iter().find(|l| &l.name == name).unwrap().id;
        let a = actual
            .paper_canvases
            .iter()
            .find(|c| c.scope_id == scope)
            .unwrap();
        assert_eq!(a.frame, canvas.frame, "{format}: {name}");
        assert_eq!(a.grid, file_grid(canvas.grid, format), "{format}");
        assert_eq!(a.snap, canvas.snap);
        assert_eq!(a.current_ucs, None);
        assert_eq!(a.active_context, None);
        assert_eq!(a.use_stored_ucs, canvas.use_stored_ucs);
    }
    assert_eq!(
        actual.viewport_workspaces.len(),
        expected.viewport_workspaces.len(),
        "{format}"
    );
    for (a, b) in actual
        .viewport_workspaces
        .iter()
        .zip(&expected.viewport_workspaces)
    {
        assert_eq!(a.grid, file_grid(b.grid, format), "{format}");
        assert_eq!(a.snap, b.snap, "{format}");
        assert_eq!(a.stored_ucs, b.stored_ucs);
        assert_eq!(a.use_stored_ucs, b.use_stored_ucs);
    }
    assert_eq!(
        actual
            .ucs_definitions
            .iter()
            .map(|u| (
                &u.definition.name,
                u.definition.frame,
                u.definition.elevation
            ))
            .collect::<Vec<_>>(),
        expected
            .ucs_definitions
            .iter()
            .map(|u| (
                &u.definition.name,
                u.definition.frame,
                u.definition.elevation
            ))
            .collect::<Vec<_>>()
    );
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
fn active_secondary_paper_layout_does_not_corrupt_roles() {
    let mut source = CadDocument::new();
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
    let mut native = cad_document_to_ocdraw_document(&source, Default::default())
        .unwrap()
        .into_document();
    let selected = native
        .layouts
        .iter()
        .find(|l| l.name == "Second")
        .unwrap()
        .id;
    native.workspace_state.as_mut().unwrap().active_layout_id = Some(selected);
    let target = ocdraw_document_to_cad_document(&native, Default::default()).unwrap();
    assert!(target
        .diagnostics()
        .iter()
        .any(|d| d.location.contains("activeLayout")));
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
        let actual = cad_document_to_ocdraw_document(&cad, Default::default())
            .unwrap()
            .into_document();
        let active = actual.workspace_state.unwrap().active_layout_id;
        assert_eq!(active, None, "dwg={format}");
        assert_eq!(
            actual
                .layouts
                .iter()
                .filter(|l| l.kind == DrawingLayoutKind::Paper)
                .count(),
            2
        );
    }
}

#[test]
fn canvas_display_state_is_diagnosed_without_discarding_aids() {
    let mut source = CadDocument::new();
    source.add_layout("Second").unwrap();
    for v in source.entities_mut() {
        if let opencadcodec::EntityType::Viewport(v) = v {
            v.status.locked = true;
            v.turn_off();
        }
    }
    let out = cad_document_to_ocdraw_document(&source, Default::default()).unwrap();
    assert!(!out.document().paper_canvases.is_empty());
    assert!(out.diagnostics().iter().any(|d|d.reasons().iter().any(|r|matches!(r,CadToOcdrawLossReason::UnsupportedSemantic{name} if name.contains("canvas display")))));
}
