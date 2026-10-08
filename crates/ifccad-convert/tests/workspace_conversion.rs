mod common;
use ifccad_convert::*;
use ocdraw::ifccad::*;
use opencadcodec::entities::Viewport;
use opencadcodec::{CadDocument, EntityType, VPort, Vector3};

#[test]
fn available_windows_and_ucs_survive_unknown_current_window() {
    let mut source = CadDocument::new();
    source.vports.clear();
    let mut ucs = opencadcodec::Ucs::new("Unused");
    ucs.handle = source.allocate_handle();
    ucs.origin = Vector3::new(1., 2., 0.);
    ucs.elevation = 3.;
    source.ucss.add(ucs).unwrap();
    for (left, right) in [(0., 0.5), (0.5, 1.)] {
        let mut window = VPort::active();
        window.handle = source.allocate_handle();
        window.lower_left.x = left;
        window.upper_right.x = right;
        window.snap_on = false;
        window.snap_spacing = opencadcodec::Vector2::ZERO;
        source.vports.add_allow_duplicate(window);
    }
    let result =
        cad_document_to_encoded_ifccad(&source, common::metadata(), Default::default()).unwrap();
    let drawing = result.validated_source().document();
    assert_eq!(drawing.ucs_definitions.len(), 1);
    assert_eq!(drawing.model_windows.len(), 2);
    assert_eq!(
        drawing
            .model_view_state
            .as_ref()
            .unwrap()
            .active_model_window_id,
        None
    );
    assert_eq!(
        drawing.model_windows[1].snap.spacing,
        ocdraw::geometry_kernel::Point2::new(0., 0.)
    );
    assert!(drawing.id_counters.next_model_window_id > drawing.model_windows[1].id.0);
    assert!(drawing.id_counters.next_ucs_id > drawing.ucs_definitions[0].id.0);
}
#[test]
fn paper_canvas_and_authored_viewport_workspace_are_distinct() {
    let mut source = CadDocument::new();
    source.header.insertion_units = 6;
    let mut canvas = Viewport::new();
    canvas.id = 1;
    canvas.center = Vector3::new(100., -50., 3.);
    canvas.width = 20.;
    canvas.height = 10.;
    canvas.snap_spacing = Vector3::ZERO;
    canvas.status.snap_on = false;
    canvas.ucs_per_viewport = false;
    let overall = source
        .add_entity_to_layout(EntityType::Viewport(canvas), "Layout1")
        .unwrap();
    let mut view = Viewport::new();
    view.id = 2;
    view.width = 160.;
    view.height = 100.;
    view.grid_spacing = Vector3::new(2., 3., 0.);
    view.ucs_origin = Vector3::new(4., 5., 0.);
    view.ucs_per_viewport = false;
    view.snap_spacing = Vector3::ZERO;
    view.status.snap_on = false;
    let authored = source
        .add_entity_to_layout(EntityType::Viewport(view), "Layout1")
        .unwrap();
    for object in source.objects.values_mut() {
        if let opencadcodec::objects::ObjectType::Layout(layout) = object {
            if layout.name == "Layout1" {
                layout.viewport = overall;
                layout.plot_paper_units = 0;
                layout.paper_width = 8.5;
                layout.paper_height = 11.;
            }
        }
    }
    let result =
        cad_document_to_encoded_ifccad(&source, common::metadata(), Default::default()).unwrap();
    let drawing = result.validated_source().document();
    assert_eq!(drawing.length_unit, "m");
    let canvas = drawing.paper_layouts[0].canvas.as_ref().unwrap();
    assert_eq!(canvas.frame.unwrap().center.components(), [100., -50., 3.]);
    assert_eq!(canvas.current_ucs, None);
    assert_eq!(canvas.active_context, None);
    assert!(!canvas.use_stored_ucs);
    let id = result.mappings().entities.ifccad_id(authored).unwrap();
    let IfccadEntityKind::Viewport(view) = &drawing.paper_layouts[0]
        .entities
        .iter()
        .find(|e| e.id() == id)
        .unwrap()
        .as_native()
        .unwrap()
        .kind
    else {
        panic!("viewport")
    };
    let workspace = view.workspace.as_ref().unwrap();
    assert_eq!(
        workspace.grid.spacing,
        ocdraw::geometry_kernel::Point2::new(2., 3.)
    );
    assert_eq!(
        workspace.snap.spacing,
        ocdraw::geometry_kernel::Point2::new(0., 0.)
    );
    let target =
        ifccad_source_to_cad_document(result.validated_source(), Default::default()).unwrap();
    let EntityType::Viewport(view) = target
        .document()
        .get_entity(target.mappings().entities.cad_handle(id).unwrap())
        .unwrap()
    else {
        panic!("viewport")
    };
    assert_eq!(view.grid_spacing, Vector3::new(2., 3., 0.));
    assert_eq!(view.ucs_origin, Vector3::new(4., 5., 0.));
    assert_eq!(view.snap_spacing, Vector3::ZERO);
    assert!(!view.ucs_per_viewport);
}
#[test]
fn drawing_choices_and_header_ucs_are_independent_of_stored_snapshots() {
    let mut source = CadDocument::new();
    source.vports.clear();
    source.header.model_space_ucs_origin = Vector3::new(2., 3., 0.);
    let drawing =
        cad_document_to_encoded_ifccad(&source, common::metadata(), Default::default()).unwrap();
    let doc = drawing.validated_source().document();
    assert!(doc.model_windows.is_empty());
    assert!(matches!(
        doc.model_view_state.as_ref().unwrap().current_model_ucs,
        Some(IfccadUcsSelection::Unnamed(_))
    ));
    assert!(doc
        .workspace_state
        .as_ref()
        .unwrap()
        .current_layer_id
        .is_some());
    assert_eq!(
        doc.workspace_state.as_ref().unwrap().active_layout_id,
        Some(doc.model.id)
    );
}
#[test]
fn numeric_workspace_failures_are_not_loss_policy_choices() {
    for loss_policy in [IfccadLossPolicy::Allow, IfccadLossPolicy::Reject] {
        let mut source = CadDocument::new();
        source.vports.get_mut("*Active").unwrap().snap_rotation = f64::NAN;
        assert!(cad_document_to_ifccad_document(
            &source,
            common::metadata(),
            CadToIfccadOptions {
                loss_policy,
                ..Default::default()
            }
        )
        .is_err());
    }
}

fn canvas_native_json() -> serde_json::Value {
    let mut source = CadDocument::new();
    source.add_layout("Second").unwrap();
    let native =
        cad_document_to_encoded_ifccad(&source, common::metadata(), Default::default()).unwrap();
    serde_json::from_slice(native.encoded().bytes()).unwrap()
}
#[test]
fn omitted_activation_default_is_not_foreign_ifcx() {
    let mut raw = canvas_native_json();
    for node in raw["data"].as_array_mut().unwrap() {
        if let Some(canvas) = node["attributes"].get_mut("ifccad::paperCanvas") {
            canvas.as_object_mut().unwrap().remove("useStoredUcs");
        }
    }
    let native = load_ifccad_bytes(&serde_json::to_vec(&raw).unwrap(), Default::default()).unwrap();
    let target = ifccad_source_to_cad_document(&native, Default::default()).unwrap();
    assert!(!target
        .diagnostics()
        .iter()
        .any(|d| d.code == "foreign-ifcx"));
}
#[test]
fn workspace_scalar_precision_is_not_certified_as_geometry_rounding() {
    let mut raw = canvas_native_json();
    for node in raw["data"].as_array_mut().unwrap() {
        if let Some(canvas) = node["attributes"].get_mut("ifccad::paperCanvas") {
            canvas["frame"]["center"][0] = serde_json::json!(9007199254740993u64);
        }
    }
    let native = load_ifccad_bytes(&serde_json::to_vec(&raw).unwrap(), Default::default()).unwrap();
    assert!(
        matches!(ifccad_source_to_cad_document(&native,Default::default()),Err(IfccadConversionError::Unsupported(issues)) if issues.iter().any(|d|d.code == "precision" && d.location.contains("paperCanvas")))
    );
}

#[test]
fn ucs_definition_scalar_precision_is_not_silently_rounded() {
    let mut raw: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../examples/ifccad/workspace-state.ifcx"
    ))
    .unwrap();
    for node in raw["data"].as_array_mut().unwrap() {
        if let Some(ucs) = node["attributes"].get_mut("ifccad::ucsDefinition") {
            ucs["frame"]["origin"][0] = serde_json::json!(9007199254740993u64);
        }
    }
    let native = load_ifccad_bytes(&serde_json::to_vec(&raw).unwrap(), Default::default()).unwrap();
    assert!(
        matches!(ifccad_source_to_cad_document(&native,Default::default()),Err(IfccadConversionError::Unsupported(issues)) if issues.iter().any(|d|d.code == "precision" && d.location.contains("ucsDefinition")))
    );
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
        cad_document_to_ifccad_document(&source, common::metadata(), Default::default()),
        Err(IfccadConversionError::WorkspaceNumeric(_))
    ));
}
