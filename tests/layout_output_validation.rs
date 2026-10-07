use ocdraw::{geometry_kernel::CoordinateLengthUnit as Unit, plot_kernel::*};

fn medium(unit: MediaUnit) -> LayoutOutputSettings {
    LayoutOutputSettings {
        media: Some(LayoutMedia {
            unit,
            width: 420.,
            height: 297.,
        }),
        ..Default::default()
    }
}
fn plot(unit: PlotUnit) -> PlotSettings {
    PlotSettings {
        plot_unit: unit,
        page: PlotPage {
            printable_area: PlotRect {
                min_x: 5.,
                min_y: 5.,
                max_x: 415.,
                max_y: 292.,
            },
            rotation: PlotRotation::None,
            device_name: None,
            media_name: None,
        },
        area: PlotArea::Layout,
        mapping: PlotMapping {
            scale: PlotScale::Fixed {
                output_length: 1.,
                scope_length: 1.,
            },
            placement: PlotPlacement::Offset {
                reference: PlotOffsetReference::Media,
                x: -2.,
                y: 0.,
            },
        },
        output: PlotOutput {
            shaded_plot: ShadedPlot {
                mode: ShadedPlotMode::AsDisplayed,
                quality: ShadedPlotQuality {
                    mode: ShadedPlotQualityMode::Normal,
                    dpi: None,
                },
            },
            apply_plot_styles: false,
            plot_style_table_name: None,
        },
        options: PlotOptions {
            plot_viewport_borders: false,
            plot_paper_space_last: true,
            hide_paper_space_objects: false,
            plot_line_weights: true,
            scale_line_weights: false,
            plot_transparency: false,
        },
    }
}
#[test]
fn medium_without_plot_is_valid_for_model_and_paper() {
    let s = medium(MediaUnit::Physical(Unit::Millimetre));
    for kind in [LayoutOutputKind::Model, LayoutOutputKind::Paper] {
        assert!(validate_layout_output(&s, kind).is_ok());
    }
    assert!(s.plot_settings.is_none());
    assert!(s.paper_space_linetype_scaling);
    assert!(
        validate_layout_output(&LayoutOutputSettings::default(), LayoutOutputKind::Paper).is_ok()
    );
}
#[test]
fn complete_plot_requires_medium_and_compatible_units() {
    let mut s = medium(MediaUnit::Physical(Unit::Millimetre));
    s.plot_settings = Some(plot(PlotUnit::Inch));
    assert!(validate_layout_output(&s, LayoutOutputKind::Paper).is_ok());
    s.media = None;
    assert!(validate_layout_output(&s, LayoutOutputKind::Paper).is_err());
    s.media = medium(MediaUnit::Physical(Unit::Unitless)).media;
    assert!(validate_layout_output(&s, LayoutOutputKind::Paper).is_err());
    s.media = medium(MediaUnit::Pixel).media;
    assert!(validate_layout_output(&s, LayoutOutputKind::Paper).is_err());
    s.plot_settings.as_mut().unwrap().plot_unit = PlotUnit::Pixel;
    assert!(validate_layout_output(&s, LayoutOutputKind::Paper).is_ok());
    s.media = medium(MediaUnit::Physical(Unit::Millimetre)).media;
    assert!(validate_layout_output(&s, LayoutOutputKind::Paper).is_err());
}
#[test]
fn layout_output_rejects_invalid_values_and_area_rules() {
    for width in [0., -1., f64::NAN, f64::INFINITY] {
        let mut s = medium(MediaUnit::Physical(Unit::Millimetre));
        s.media.as_mut().unwrap().width = width;
        assert!(validate_layout_output(&s, LayoutOutputKind::Paper).is_err());
    }
    let mut s = medium(MediaUnit::Physical(Unit::Millimetre));
    s.plot_settings = Some(plot(PlotUnit::Millimetre));
    assert!(validate_layout_output(&s, LayoutOutputKind::Model).is_err());
    s.plot_settings.as_mut().unwrap().area = PlotArea::Limits;
    assert!(validate_layout_output(&s, LayoutOutputKind::Paper).is_err());
    assert!(validate_layout_output(&s, LayoutOutputKind::Model).is_err());
    s.limits = Some(PlotRect {
        min_x: 0.,
        min_y: 0.,
        max_x: 20.,
        max_y: 20.,
    });
    assert!(validate_layout_output(&s, LayoutOutputKind::Model).is_ok());
    s.plot_settings.as_mut().unwrap().page.printable_area.max_x = 421.;
    assert!(validate_layout_output(&s, LayoutOutputKind::Model).is_err());
    s.plot_settings.as_mut().unwrap().page.printable_area.max_x = 415.;
    for dpi in [99, 32768] {
        let quality = &mut s.plot_settings.as_mut().unwrap().output.shaded_plot.quality;
        quality.mode = ShadedPlotQualityMode::Custom;
        quality.dpi = Some(dpi);
        assert!(validate_layout_output(&s, LayoutOutputKind::Model).is_err());
    }
}
