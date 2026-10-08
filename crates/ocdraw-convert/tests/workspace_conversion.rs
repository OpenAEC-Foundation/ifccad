use ocdraw::ocdraw::*;
use ocdraw_convert::*;
use opencadcodec::entities::Viewport;
use opencadcodec::{CadDocument, EntityType, VPort, Vector3};

fn reopened(doc: &OcdrawDocument) -> OcdrawDocument {
    load_ocdraw_bytes(encode_ocdraw_document(doc).unwrap().bytes())
        .unwrap()
        .into_document()
}

#[test]
fn several_windows_survive_without_inventing_active_identity() {
    let mut source = CadDocument::new();
    source.vports.clear();
    for (left, right) in [(0., 0.5), (0.5, 1.)] {
        let mut window = VPort::active();
        window.handle = source.allocate_handle();
        window.lower_left.x = left;
        window.upper_right.x = right;
        window.snap_on = false;
        window.snap_spacing = opencadcodec::Vector2::ZERO;
        source.vports.add_allow_duplicate(window);
    }
    let result = cad_document_to_ocdraw_document(&source, Default::default()).unwrap();
    let drawing = reopened(result.document());
    assert_eq!(drawing.model_windows.len(), 2);
    assert_eq!(drawing.view_state.unwrap().active_model_window_id, None);
    assert_eq!(drawing.model_windows[1].snap.spacing, Point2::new(0., 0.));
}

#[test]
fn current_model_ucs_is_retained_without_windows() {
    let mut source = CadDocument::new();
    source.vports.clear();
    source.header.model_space_ucs_origin = Vector3::new(2., 3., 0.);
    let drawing = cad_document_to_ocdraw_document(&source, Default::default())
        .unwrap()
        .into_document();
    assert!(drawing.model_windows.is_empty());
    assert!(matches!(
        drawing.view_state.unwrap().current_model_ucs,
        Some(DrawingUcsSelection::Unnamed(_))
    ));
}

#[test]
fn canvas_preserves_frame_disabled_spacing_and_activation() {
    let mut source = CadDocument::new();
    let mut view = Viewport::new();
    view.id = 1;
    view.center = Vector3::new(100., -50., 3.);
    view.width = 20.;
    view.height = 10.;
    view.status.snap_on = false;
    view.snap_spacing = Vector3::ZERO;
    view.ucs_per_viewport = false;
    let handle = source
        .add_entity_to_layout(EntityType::Viewport(view), "Layout1")
        .unwrap();
    for object in source.objects.values_mut() {
        if let opencadcodec::objects::ObjectType::Layout(layout) = object {
            if layout.name == "Layout1" {
                layout.viewport = handle;
            }
        }
    }
    let drawing = reopened(
        cad_document_to_ocdraw_document(&source, Default::default())
            .unwrap()
            .document(),
    );
    let canvas = drawing.paper_canvases.first().unwrap();
    assert_eq!(canvas.frame.unwrap().center, Point3::new(100., -50., 3.));
    assert_eq!(canvas.snap.spacing, Point2::new(0., 0.));
    assert!(!canvas.use_stored_ucs);
    assert_eq!(canvas.active_context, None);
    assert_eq!(canvas.current_ucs, None);
}

#[test]
fn viewport_workspace_maps_both_directions_without_touching_geometry() {
    let mut source = CadDocument::new();
    let mut view = Viewport::new();
    view.id = 2;
    view.center = Vector3::new(90., 70., 0.);
    view.width = 160.;
    view.height = 100.;
    view.status.grid_on = true;
    view.grid_spacing = Vector3::new(2., 3., 0.);
    view.status.snap_on = false;
    view.snap_spacing = Vector3::ZERO;
    view.snap_base = Vector3::new(4., 5., 0.);
    view.snap_angle = 0.25;
    view.ucs_origin = Vector3::new(6., 7., 0.);
    view.ucs_per_viewport = false;
    let handle = source
        .add_entity_to_layout(EntityType::Viewport(view), "Layout1")
        .unwrap();
    let result = cad_document_to_ocdraw_document(&source, Default::default()).unwrap();
    let id = result.entity_mapping()[&handle];
    let drawing = reopened(result.document());
    let workspace = drawing
        .viewport_workspaces
        .iter()
        .find(|w| w.viewport_entity_id == id)
        .unwrap();
    assert_eq!(workspace.grid.spacing, Point2::new(2., 3.));
    assert_eq!(workspace.snap.spacing, Point2::new(0., 0.));
    assert!(!workspace.use_stored_ucs);
    let target = ocdraw_document_to_cad_document(&drawing, Default::default()).unwrap();
    let view = match target
        .document()
        .get_entity(target.entity_mapping()[&id])
        .unwrap()
    {
        EntityType::Viewport(v) => v,
        _ => panic!("viewport"),
    };
    assert_eq!(view.center, Vector3::new(90., 70., 0.));
    assert_eq!(view.grid_spacing, Vector3::new(2., 3., 0.));
    assert_eq!(view.snap_spacing, Vector3::ZERO);
    assert_eq!(view.ucs_origin, Vector3::new(6., 7., 0.));
}

#[test]
fn nonfinite_workspace_is_a_hard_error_for_both_policies() {
    for policy in [OcdrawLossPolicy::Allow, OcdrawLossPolicy::Reject] {
        let mut source = CadDocument::new();
        source.vports.get_mut("*Active").unwrap().snap_rotation = f64::NAN;
        assert!(cad_document_to_ocdraw_document(
            &source,
            CadToOcdrawOptions {
                loss_policy: policy,
                ..Default::default()
            }
        )
        .is_err());
    }
}

#[test]
fn null_ucs_handle_does_not_retarget_world_to_unused_definition() {
    let mut source = CadDocument::new();
    source.ucss.add(opencadcodec::Ucs::new("Unused")).unwrap();
    let drawing = cad_document_to_ocdraw_document(&source, Default::default())
        .unwrap()
        .into_document();
    assert_eq!(drawing.ucs_definitions.len(), 1);
    assert_eq!(
        drawing.model_windows[0].stored_ucs,
        DrawingUcsSelection::World
    );
}
#[test]
fn unused_nonfinite_ucs_is_a_hard_error() {
    let mut source = CadDocument::new();
    let mut ucs = opencadcodec::Ucs::new("Invalid");
    ucs.origin.x = f64::NAN;
    source.ucss.add(ucs).unwrap();
    assert!(cad_document_to_ocdraw_document(&source, Default::default()).is_err());
}
#[test]
fn model_viewport_aids_keep_model_units_inside_inch_paper() {
    let mut source = CadDocument::new();
    source.header.insertion_units = 6;
    let mut view = Viewport::new();
    view.id = 2;
    view.width = 160.;
    view.height = 100.;
    view.grid_spacing = Vector3::new(2., 3., 0.);
    view.snap_base = Vector3::new(4., 5., 0.);
    view.ucs_origin = Vector3::new(6., 7., 0.);
    source
        .add_entity_to_layout(EntityType::Viewport(view), "Layout1")
        .unwrap();
    for object in source.objects.values_mut() {
        if let opencadcodec::objects::ObjectType::Layout(layout) = object {
            if layout.name == "Layout1" {
                layout.plot_paper_units = 0;
                layout.paper_width = 8.5;
                layout.paper_height = 11.;
            }
        }
    }
    let drawing = reopened(
        cad_document_to_ocdraw_document(&source, Default::default())
            .unwrap()
            .document(),
    );
    assert_eq!(drawing.unit, "m");
    assert_eq!(
        drawing.viewport_workspaces[0].grid.spacing,
        Point2::new(2., 3.)
    );
    assert_eq!(
        drawing.viewport_workspaces[0].snap.base,
        Point2::new(4., 5.)
    );
    let DrawingUcsSelection::Unnamed(frame) = drawing.viewport_workspaces[0].stored_ucs else {
        panic!("stored frame")
    };
    assert_eq!(frame.origin(), Point3::new(6., 7., 0.));
}
#[test]
fn perspective_target_viewport_keeps_its_workspace() {
    let mut source = CadDocument::new();
    let mut view = Viewport::new();
    view.id = 2;
    view.width = 160.;
    view.height = 100.;
    let handle = source
        .add_entity_to_layout(EntityType::Viewport(view), "Layout1")
        .unwrap();
    let result = cad_document_to_ocdraw_document(&source, Default::default()).unwrap();
    let id = result.entity_mapping()[&handle];
    let mut drawing = reopened(result.document());
    drawing
        .viewports
        .iter_mut()
        .find(|v| v.id == id)
        .unwrap()
        .view
        .projection = DrawingProjection::Perspective;
    let target = ocdraw_document_to_cad_document(&drawing, Default::default()).unwrap();
    let mapped = target.entity_mapping()[&id];
    assert!(
        matches!(target.document().get_entity(mapped),Some(EntityType::Viewport(v)) if v.status.perspective)
    );
    assert!(!target
        .diagnostics()
        .iter()
        .any(|d| d.location == format!("/viewportWorkspaces/{id}")
            && d.message.contains("not constructed")));
}
#[test]
fn grid_style_loss_is_located_and_rejectable() {
    let mut drawing = cad_document_to_ocdraw_document(&CadDocument::new(), Default::default())
        .unwrap()
        .into_document();
    drawing.model_windows[0].grid.style = DrawingGridStyle::Dots;
    let target = ocdraw_document_to_cad_document(&drawing, Default::default()).unwrap();
    assert!(target
        .diagnostics()
        .iter()
        .any(|d| d.location == "/modelWindows/0/grid.style"));
    assert!(matches!(
        ocdraw_document_to_cad_document(
            &drawing,
            OcdrawToCadOptions {
                loss_policy: OcdrawLossPolicy::Reject,
                ..Default::default()
            }
        ),
        Err(OcdrawToCadError::LossRejected { .. })
    ));
}

#[test]
fn omitted_viewport_cannot_hide_nonfinite_workspace() {
    let mut source = CadDocument::new();
    let mut viewport = Viewport::new();
    viewport.id = 2;
    viewport.width = 0.;
    viewport.grid_spacing.x = f64::NAN;
    source
        .add_entity_to_layout(EntityType::Viewport(viewport), "Layout1")
        .unwrap();
    assert!(matches!(
        cad_document_to_ocdraw_document(&source, Default::default()),
        Err(CadToOcdrawError::WorkspaceNumeric(_))
    ));
}
