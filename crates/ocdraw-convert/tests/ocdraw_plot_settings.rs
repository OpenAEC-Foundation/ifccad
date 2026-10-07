use ocdraw_convert::{
    cad_document_to_ocdraw_document,
    opencadcodec::{objects::ObjectType, CadDocument},
    CadToOcdrawOptions,
};
#[test]
fn cad_millimetres_and_inch_output_remain_distinct() {
    let mut cad = CadDocument::new();
    for o in cad.objects.values_mut() {
        if let ObjectType::Layout(l) = o {
            if l.name == "Layout1" {
                l.paper_width = 279.4;
                l.paper_height = 215.9;
                l.plot_paper_units = 0;
                l.plot_type = 1;
                l.plot_scale_type = 0;
                l.plot_flags.use_standard_scale = false;
                l.plot_scale_numerator = 2.;
                l.plot_scale_denominator = 1.;
                l.plot_origin_x = 127.;
            }
        }
    }
    let result = cad_document_to_ocdraw_document(&cad, CadToOcdrawOptions::default()).unwrap();
    let encoded = ocdraw::ocdraw::encode_ocdraw_document(result.document()).unwrap();
    let native = ocdraw::ocdraw::load_ocdraw_bytes(encoded.bytes()).unwrap();
    let paper = native.as_value()["layouts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["kind"] == "paper")
        .unwrap();
    assert_eq!(paper["media"]["unit"], "mm");
    assert_eq!(paper["media"]["width"], 279.4);
    assert_eq!(paper["plotSettings"]["plotUnit"], "in");
    assert_eq!(paper["plotSettings"]["mapping"]["scale"]["mode"], "Fixed");
    assert_eq!(paper["plotSettings"]["mapping"]["placement"]["x"], 5.);
}
#[test]
fn saved_layout_linetype_scaling_is_not_flattened_to_header() {
    let mut cad = CadDocument::new();
    cad.add_layout("Second").unwrap();
    for o in cad.objects.values_mut() {
        if let ObjectType::Layout(l) = o {
            if l.name != "Model" {
                l.paper_width = 420.;
                l.paper_height = 297.;
                l.flags = if l.name == "Second" { 1 } else { 0 };
            }
        }
    }
    let result = cad_document_to_ocdraw_document(&cad, Default::default()).unwrap();
    let doc = result.document();
    assert!(
        !doc.layouts
            .iter()
            .find(|l| l.name == "Layout1")
            .unwrap()
            .settings
            .paper_space_linetype_scaling
    );
    assert!(
        doc.layouts
            .iter()
            .find(|l| l.name == "Second")
            .unwrap()
            .settings
            .paper_space_linetype_scaling
    );
}

#[test]
fn contradictory_active_standard_scale_is_diagnosed_with_medium_retained() {
    let mut cad = CadDocument::new();
    for o in cad.objects.values_mut() {
        if let ObjectType::Layout(l) = o {
            if l.name == "Layout1" {
                l.paper_width = 420.;
                l.paper_height = 297.;
                l.plot_paper_units = 1;
                l.plot_flags.use_standard_scale = true;
                l.plot_scale_type = 16;
                l.plot_scale_factor = 2.;
            }
        }
    }
    let out = cad_document_to_ocdraw_document(&cad, Default::default()).unwrap();
    let sheet = out
        .document()
        .layouts
        .iter()
        .find(|l| l.name == "Layout1")
        .unwrap();
    assert!(sheet.settings.media.is_some());
    assert!(sheet.settings.plot_settings.is_none());
    assert!(!out.diagnostics().is_empty());
}

#[test]
fn active_paper_header_does_not_replace_inactive_model_scaling() {
    use ocdraw_convert::{
        ocdraw_document_to_cad_document,
        opencadcodec::{entities::Viewport, EntityType},
    };
    let mut cad = CadDocument::new();
    cad.header.show_model_space = false;
    cad.header.paper_space_linetype_scaling = false;
    for o in cad.objects.values_mut() {
        if let ObjectType::Layout(l) = o {
            if l.name == "Model" {
                l.flags |= 1;
            } else {
                l.paper_width = 420.;
                l.paper_height = 297.;
                l.flags &= !1;
            }
        }
    }
    let mut canvas = Viewport::new();
    canvas.id = 1;
    cad.add_entity_to_layout(EntityType::Viewport(canvas), "Layout1")
        .unwrap();
    let native = cad_document_to_ocdraw_document(&cad, Default::default()).unwrap();
    assert!(
        native
            .document()
            .layouts
            .iter()
            .find(|l| l.name == "Model")
            .unwrap()
            .settings
            .paper_space_linetype_scaling
    );
    let back = ocdraw_document_to_cad_document(native.document(), Default::default()).unwrap();
    assert!(!back.document().header.paper_space_linetype_scaling);
}

#[test]
fn unrepresentable_printable_corner_is_not_silently_rounded() {
    let mut cad = CadDocument::new();
    for o in cad.objects.values_mut() {
        if let ObjectType::Layout(l) = o {
            if l.name == "Layout1" {
                l.paper_width = 420.;
                l.paper_height = 297.;
                l.plot_paper_units = 1;
                l.plot_margin_right = 0.1;
            }
        }
    }
    let out = cad_document_to_ocdraw_document(&cad, Default::default()).unwrap();
    let sheet = out
        .document()
        .layouts
        .iter()
        .find(|l| l.name == "Layout1")
        .unwrap();
    assert!(sheet.settings.media.is_some());
    assert!(sheet.settings.plot_settings.is_none());
    assert!(out.diagnostics().iter().any(|d|d.reasons().iter().any(|r|matches!(r,ocdraw_convert::CadToOcdrawLossReason::UnsupportedSemantic{name} if name.contains("exact printable")))));
}

#[test]
fn authored_plot_identical_to_cad_defaults_reports_absence_ambiguity() {
    let mut cad = CadDocument::new();
    for o in cad.objects.values_mut() {
        if let ObjectType::Layout(l) = o {
            if l.name == "Layout1" {
                l.paper_width = 420.;
                l.paper_height = 297.;
                l.shade_plot_resolution = 2;
            }
        }
    }
    let native = cad_document_to_ocdraw_document(&cad, Default::default()).unwrap();
    let mut doc = native.into_document();
    doc.layouts[1]
        .settings
        .plot_settings
        .as_mut()
        .unwrap()
        .output
        .shaded_plot
        .quality
        .mode = ocdraw::plot_kernel::ShadedPlotQualityMode::Draft;
    let out = ocdraw_convert::ocdraw_document_to_cad_document(&doc, Default::default()).unwrap();
    assert!(out
        .diagnostics()
        .iter()
        .any(|d| d.code == "LAYOUT_DEFAULT_PLOT_AMBIGUOUS"));
}
