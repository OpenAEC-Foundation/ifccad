use opencadcodec::objects::{KnownXRecordKind as K, XRecordValue as V};
use opencadcodec::{CadDocument, EntityType, Handle};
fn source() -> (CadDocument, Handle, Handle) {
    let mut d = CadDocument::new();
    d.add_layout("Sheet").unwrap();
    let layer = d.layers.get("0").unwrap().handle;
    let mut v = opencadcodec::entities::Viewport::new();
    v.id = 2;
    v.width = 100.;
    v.height = 80.;
    v.view_height = 120.;
    v.frozen_layers = vec![layer];
    let viewport = d
        .add_entity_to_layout(EntityType::Viewport(v), "Sheet")
        .unwrap();
    let literal: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/viewports/presentation-xrecord-scalars.json"
    ))
    .unwrap();
    for (kind, value) in [
        (
            K::LayerViewportColorOverride,
            V::Int32(literal["color420"].as_i64().unwrap() as i32),
        ),
        (
            K::LayerViewportAlphaOverride,
            V::Int32(literal["alpha440"].as_i64().unwrap() as i32),
        ),
        (
            K::LayerViewportLineweightOverride,
            V::Int32(literal["lineweight91"].as_i64().unwrap() as i32),
        ),
        (
            K::LayerViewportLinetypeOverride,
            V::Handle(d.line_types.get("Continuous").unwrap().handle),
        ),
    ] {
        assert!(d.set_layer_viewport_override(layer, kind, viewport, value));
    }
    (d, layer, viewport)
}
use ocdraw_convert::*;
#[test]
fn all_viewport_overrides_merge_with_freezing_and_export_again() {
    let (d, _, _) = source();
    let native = cad_document_to_ocdraw_document(&d, Default::default()).unwrap();
    let row = &native.document().viewports[0].layer_overrides[0];
    assert!(row.frozen);
    assert_eq!(row.color.as_ref().unwrap().indexed.as_ref().unwrap().1, 1);
    assert_eq!(row.opacity, Some(127. / 255.));
    assert_eq!(row.line_weight, Some(0.25));
    assert!(row.line_pattern_id.is_some());
    let target = ocdraw_document_to_cad_document(native.document(), Default::default()).unwrap();
    let layer = target.document().layers.get("0").unwrap().handle;
    assert_eq!(
        target
            .document()
            .layer_viewport_overrides(layer, K::LayerViewportColorOverride)
            .len(),
        1
    );
    assert_eq!(
        target
            .document()
            .layer_viewport_overrides(layer, K::LayerViewportAlphaOverride)[0]
            .1,
        V::Int32(33554559)
    );
}

#[test]
fn literal_source_overrides_survive_actual_dxf_and_dwg() {
    let (d, _, _) = source();
    for dwg in [false, true] {
        let d = if dwg {
            opencadcodec::DwgReader::from_stream(std::io::Cursor::new(
                opencadcodec::DwgWriter::write_to_vec(&d).unwrap(),
            ))
            .read()
            .unwrap()
        } else {
            opencadcodec::DxfReader::from_reader(std::io::Cursor::new(
                opencadcodec::DxfWriter::new(&d).write_to_vec().unwrap(),
            ))
            .unwrap()
            .read()
            .unwrap()
        };
        let native = cad_document_to_ocdraw_document(&d, Default::default()).unwrap();
        let view = native
            .document()
            .viewports
            .iter()
            .find(|v| !v.layer_overrides.is_empty())
            .unwrap_or_else(|| {
                panic!(
                    "DWG={dwg}: {:?}, source={:?}, diagnostics={:?}",
                    native.document().viewports,
                    d.entities()
                        .filter(|e| matches!(e, EntityType::Viewport(_)))
                        .collect::<Vec<_>>(),
                    native.diagnostics()
                )
            });
        assert_eq!(
            view.layer_overrides[0].opacity,
            Some(127. / 255.),
            "DWG={dwg}"
        );
        assert_eq!(view.layer_overrides[0].line_weight, Some(0.25), "DWG={dwg}");
        assert!(
            view.layer_overrides[0].line_pattern_id.is_some(),
            "DWG={dwg}"
        );
        let target =
            ocdraw_document_to_cad_document(native.document(), Default::default()).unwrap();
        let layer = target.document().layers.get("0").unwrap().handle;
        assert_eq!(
            target
                .document()
                .layer_viewport_overrides(layer, K::LayerViewportColorOverride)[0]
                .1,
            V::Int32(-1023410175)
        );
    }
}
#[test]
fn shade_plot_is_independent_from_render_mode_and_survives_dwg() {
    use ocdraw::plot_kernel::ShadedPlotMode as Mode;
    for (value, mode) in [(1, Mode::Wireframe), (2, Mode::Hidden), (3, Mode::Rendered)] {
        let (mut d, _, handle) = source();
        let EntityType::Viewport(view) = d.get_entity_mut(handle).unwrap() else {
            panic!()
        };
        view.shade_plot_mode = value;
        view.render_mode = opencadcodec::entities::ViewportRenderMode::Wireframe3D;
        let native = cad_document_to_ocdraw_document(&d, Default::default()).unwrap();
        let view = &native.document().viewports[0];
        assert_eq!(view.plot_shading_override, Some(mode));
        let target =
            ocdraw_document_to_cad_document(native.document(), Default::default()).unwrap();
        let read = opencadcodec::DwgReader::from_stream(std::io::Cursor::new(
            opencadcodec::DwgWriter::write_to_vec(target.document()).unwrap(),
        ))
        .read()
        .unwrap();
        assert!(read.entities().any(|e|matches!(e,EntityType::Viewport(v) if v.shade_plot_mode==value && v.render_mode==opencadcodec::entities::ViewportRenderMode::Wireframe3D)));
    }
}

#[test]
fn supported_fields_do_not_hide_unrelated_record_data() {
    let (mut d, layer, _) = source();
    d.xrecord_mut(layer, "ADSK_XREC_LAYER_COLOR_OVR")
        .unwrap()
        .add_string(1, "future application payload");
    let native = cad_document_to_ocdraw_document(&d, Default::default()).unwrap();
    let view = &native.document().viewports[0];
    assert_eq!(
        view.layer_overrides[0].color.as_ref().unwrap().rgb,
        [255, 0, 0]
    );
    assert!(native.diagnostics().iter().any(|d|matches!(d.source(), CadToOcdrawDiagnosticSource::Collection{kind,..} if kind=="objects")));
}
#[test]
fn unknown_packing_and_missing_pattern_leave_supported_siblings() {
    let (mut d, layer, viewport) = source();
    d.set_layer_viewport_override(
        layer,
        K::LayerViewportColorOverride,
        viewport,
        V::Int32(-1006632960),
    ); // c4000000 unsupported method
    d.set_layer_viewport_override(
        layer,
        K::LayerViewportLinetypeOverride,
        viewport,
        V::Handle(Handle::new(0xdead)),
    );
    let native = cad_document_to_ocdraw_document(&d, Default::default()).unwrap();
    let view = &native.document().viewports[0];
    assert!(view.layer_overrides[0].color.is_none());
    assert!(view.layer_overrides[0].line_pattern_id.is_none());
    assert_eq!(view.layer_overrides[0].opacity, Some(127. / 255.));
    assert!(!native.diagnostics().is_empty());
}
#[test]
fn duplicate_equal_values_are_consumed_but_conflicts_are_invalid_structure() {
    let (mut d, layer, _) = source();
    let record = d.xrecord_mut(layer, "ADSK_XREC_LAYER_COLOR_OVR").unwrap();
    record.entries.extend(record.entries.clone());
    record.synchronize_object_references();
    assert!(cad_document_to_ocdraw_document(&d, Default::default()).is_ok());
    let record = d.xrecord_mut(layer, "ADSK_XREC_LAYER_COLOR_OVR").unwrap();
    record.entries[6].value = V::Int32(-1023410174); // second section: ACI 2
    assert!(cad_document_to_ocdraw_document(&d, Default::default()).is_err());
}
#[test]
fn unavailable_viewport_never_creates_an_override_row() {
    let (mut d, layer, _) = source();
    d.set_layer_viewport_override(
        layer,
        K::LayerViewportColorOverride,
        Handle::new(0xbeef),
        V::Int32(-1023410175),
    );
    let native = cad_document_to_ocdraw_document(&d, Default::default()).unwrap();
    let view = &native.document().viewports[0];
    assert_eq!(view.layer_overrides.len(), 1);
    assert!(!native.diagnostics().is_empty());
}

#[test]
fn two_viewports_and_layers_keep_overrides_separate_without_thawing_global_freeze() {
    let (mut d, _, _) = source();
    let mut layer = opencadcodec::Layer::new("GloballyFrozen");
    layer.handle = d.allocate_handle();
    layer.flags.frozen = true;
    let layer_handle = layer.handle;
    d.layers.add(layer).unwrap();
    let mut v = opencadcodec::entities::Viewport::new();
    v.id = 3;
    v.width = 120.;
    v.height = 70.;
    v.view_height = 150.;
    let second = d
        .add_entity_to_layout(EntityType::Viewport(v), "Sheet")
        .unwrap();
    d.set_layer_viewport_override(
        layer_handle,
        K::LayerViewportColorOverride,
        second,
        V::Int32(-1039523821),
    );
    let native = cad_document_to_ocdraw_document(&d, Default::default()).unwrap();
    let views = native.document().viewports.iter().collect::<Vec<_>>();
    assert_eq!(views.len(), 2);
    assert!(views.iter().any(|v| v
        .layer_overrides
        .iter()
        .any(|r| r.frozen && r.color.as_ref().is_some_and(|c| c.rgb == [255, 0, 0]))));
    assert!(views.iter().any(|v| v
        .layer_overrides
        .iter()
        .any(|r| !r.frozen && r.color.as_ref().is_some_and(|c| c.rgb == [10, 32, 19]))));
    let target = ocdraw_document_to_cad_document(native.document(), Default::default()).unwrap();
    let layer = target.document().layers.get("GloballyFrozen").unwrap();
    assert!(layer.flags.frozen);
    assert_eq!(
        target
            .document()
            .layer_viewport_overrides(layer.handle, K::LayerViewportColorOverride)
            .len(),
        1
    );
}
#[test]
fn skipped_viewport_does_not_receive_late_override_binding() {
    let (mut d, _, viewport) = source();
    let EntityType::Viewport(v) = d.get_entity_mut(viewport).unwrap() else {
        panic!()
    };
    v.width = 0.;
    let native = cad_document_to_ocdraw_document(&d, Default::default()).unwrap();
    let views = native.document().viewports.iter().collect::<Vec<_>>();
    assert!(views.is_empty());
    let target = ocdraw_document_to_cad_document(native.document(), Default::default()).unwrap();
    let layer = target.document().layers.get("0").unwrap().handle;
    assert!(target
        .document()
        .layer_viewport_overrides(layer, K::LayerViewportColorOverride)
        .is_empty());
    assert!(!native.diagnostics().is_empty());
}

#[test]
fn forward_pattern_handle_registered_after_xrecord_resolves_in_both_transports() {
    let (mut d, layer, viewport) = source();
    let future = d.allocate_handle();
    d.set_layer_viewport_override(
        layer,
        K::LayerViewportLinetypeOverride,
        viewport,
        V::Handle(future),
    );
    let mut pattern = opencadcodec::LineType::new("ForwardPattern");
    pattern.handle = future;
    pattern.elements = vec![
        opencadcodec::tables::LineTypeElement {
            length: 2.,
            complex: None,
        },
        opencadcodec::tables::LineTypeElement {
            length: -1.,
            complex: None,
        },
    ];
    pattern.pattern_length = 3.;
    d.line_types.add(pattern).unwrap();
    for dwg in [false, true] {
        let d = if dwg {
            opencadcodec::DwgReader::from_stream(std::io::Cursor::new(
                opencadcodec::DwgWriter::write_to_vec(&d).unwrap(),
            ))
            .read()
            .unwrap()
        } else {
            opencadcodec::DxfReader::from_reader(std::io::Cursor::new(
                opencadcodec::DxfWriter::new(&d).write_to_vec().unwrap(),
            ))
            .unwrap()
            .read()
            .unwrap()
        };
        let native = cad_document_to_ocdraw_document(&d, Default::default()).unwrap();
        let v = native
            .document()
            .viewports
            .iter()
            .find(|v| !v.layer_overrides.is_empty())
            .unwrap();
        let pattern = native
            .document()
            .line_patterns
            .iter()
            .find(|p| p.name == "ForwardPattern")
            .unwrap();
        assert_eq!(v.layer_overrides[0].line_pattern_id, Some(pattern.id));
    }
}
