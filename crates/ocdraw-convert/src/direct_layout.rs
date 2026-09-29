//! CAD Layout plot fields and the standalone drawing Layout contract.

use cadcodec::objects::Layout;
use ocdraw::drawing::{
    LayoutRect, PlotArea, PlotMapping, PlotMedia, PlotOffsetReference, PlotOptions, PlotOutput,
    PlotPlacement, PlotRotation, PlotScale, PlotSettings, PlotUnit, ShadedPlot, ShadedPlotMode,
    ShadedPlotQuality, ShadedPlotQualityMode,
};
use serde_json::Value;

// cadcodec may retain DXF integer plot codes only in this raw list. Apply
// parsed values to a copy so the direct route sees the same authored state.
pub(crate) fn normalized_plot_integers(layout: &Layout) -> Layout {
    let mut normalized = layout.clone();
    if let Some(codes) = &layout.raw_plot_settings_codes {
        for (code, raw) in codes {
            let value = raw.trim();
            match *code {
                70 => {
                    if let Ok(bits) = value.parse::<i32>() {
                        normalized.plot_flags = cadcodec::objects::PlotFlags::from_bits(bits);
                    }
                }
                72 => {
                    if let Ok(parsed) = value.parse() {
                        normalized.plot_paper_units = parsed;
                    }
                }
                73 => {
                    if let Ok(parsed) = value.parse() {
                        normalized.plot_rotation = parsed;
                    }
                }
                74 => {
                    if let Ok(parsed) = value.parse() {
                        normalized.plot_type = parsed;
                    }
                }
                75 => {
                    if let Ok(parsed) = value.parse() {
                        normalized.plot_scale_type = parsed;
                    }
                }
                76 => {
                    if let Ok(parsed) = value.parse() {
                        normalized.shade_plot_mode = parsed;
                    }
                }
                77 => {
                    if let Ok(parsed) = value.parse() {
                        normalized.shade_plot_resolution = parsed;
                    }
                }
                78 => {
                    if let Ok(parsed) = value.parse() {
                        normalized.shade_plot_dpi = parsed;
                    }
                }
                _ => {}
            }
        }
    }
    normalized
}

pub(crate) fn unrepresented_fields(layout: &Layout) -> Vec<&'static str> {
    let default = Layout::new(&layout.name);
    let unconfigured_plot_state = layout.paper_width == 0.0
        && layout.paper_height == 0.0
        && (layout.plot_type != default.plot_type
            || layout.plot_flags != default.plot_flags
            || layout.plot_rotation != default.plot_rotation
            || layout.plot_paper_units != default.plot_paper_units
            || layout.plot_margin_left != default.plot_margin_left
            || layout.plot_margin_bottom != default.plot_margin_bottom
            || layout.plot_margin_right != default.plot_margin_right
            || layout.plot_margin_top != default.plot_margin_top
            || layout.plot_origin_x != default.plot_origin_x
            || layout.plot_origin_y != default.plot_origin_y
            || layout.plot_window_min_x != default.plot_window_min_x
            || layout.plot_window_min_y != default.plot_window_min_y
            || layout.plot_window_max_x != default.plot_window_max_x
            || layout.plot_window_max_y != default.plot_window_max_y
            || layout.plot_scale_numerator != default.plot_scale_numerator
            || layout.plot_scale_denominator != default.plot_scale_denominator
            || layout.plot_scale_type != default.plot_scale_type
            || layout.shade_plot_mode != default.shade_plot_mode
            || layout.shade_plot_resolution != default.shade_plot_resolution
            || layout.shade_plot_dpi != default.shade_plot_dpi
            || !layout.plot_style_sheet.is_empty()
            || !layout.plot_printer_name.is_empty()
            || !layout.paper_size.is_empty());
    let checks = [
        (
            unconfigured_plot_state,
            "unconfigured plot medium has meaningful plot fields",
        ),
        (
            layout.flags & !3 != 0,
            "layout flags beyond kind and limits checking",
        ),
        (
            layout.insertion_base != default.insertion_base,
            "layout insertion base",
        ),
        (layout.elevation != default.elevation, "layout elevation"),
        (
            layout.ucs_origin != default.ucs_origin
                || layout.ucs_x_axis != default.ucs_x_axis
                || layout.ucs_y_axis != default.ucs_y_axis
                || layout.ucs_ortho_type != default.ucs_ortho_type
                || layout.base_ucs != cadcodec::Handle::NULL
                || layout.named_ucs != cadcodec::Handle::NULL,
            "layout UCS",
        ),
        (
            !layout.reactors.is_empty() || layout.xdictionary_handle.is_some(),
            "layout object attachments",
        ),
        (!layout.plot_page_name.is_empty(), "named page setup source"),
        (
            !layout.plot_view_name.is_empty() || layout.plot_view_handle != cadcodec::Handle::NULL,
            "saved plot view reference",
        ),
        (
            layout.visual_style_handle != cadcodec::Handle::NULL,
            "shade-plot visual style reference",
        ),
        (
            layout.paper_image_origin_x != 0.0 || layout.paper_image_origin_y != 0.0,
            "paper image origin",
        ),
        (
            layout.plot_flags.use_standard_scale
                && layout.plot_scale_type != 0
                && layout.plot_scale_type != 1,
            "standard plot-scale preset",
        ),
        (
            layout.plot_flags.model_type && layout.flags & 1 == 0,
            "plot model-type flag",
        ),
        (
            layout.plot_flags.unknown_bits != 0
                || layout.plot_flags.show_plot_styles
                || layout.plot_flags.update_paper
                || layout.plot_flags.zoom_to_paper_on_update
                || layout.plot_flags.initializing
                || layout.plot_flags.prev_plot_init,
            "unrepresented plot flags",
        ),
        (
            layout.plot_flags.plot_plot_styles && !layout.plot_style_sheet.is_empty(),
            "active external CTB/STB contents are unavailable",
        ),
    ];
    checks
        .into_iter()
        .filter_map(|(present, name)| present.then_some(name))
        .collect()
}

fn rect(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Option<LayoutRect> {
    [min_x, min_y, max_x, max_y]
        .into_iter()
        .all(f64::is_finite)
        .then_some(LayoutRect {
            min_x,
            min_y,
            max_x,
            max_y,
        })
        .filter(|r| r.min_x <= r.max_x && r.min_y <= r.max_y)
}

fn nonempty(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.to_owned())
}

pub(crate) fn plot_from_cad(
    layout: &Layout,
    is_model: bool,
) -> Result<Option<PlotSettings>, &'static str> {
    if layout.paper_width == 0.0 && layout.paper_height == 0.0 {
        return Ok(None);
    }
    if !layout.paper_width.is_finite()
        || layout.paper_width <= 0.0
        || !layout.paper_height.is_finite()
        || layout.paper_height <= 0.0
    {
        return Err("invalid plot medium");
    }
    let area = match layout.plot_type {
        1 => PlotArea::Extents,
        2 if is_model => PlotArea::Limits,
        4 => PlotArea::Window(
            rect(
                layout.plot_window_min_x,
                layout.plot_window_min_y,
                layout.plot_window_max_x,
                layout.plot_window_max_y,
            )
            .ok_or("invalid plot window")?,
        ),
        5 if !is_model => PlotArea::Layout,
        _ => return Err("unsupported plot area"),
    };
    let unit = match layout.plot_paper_units {
        0 => PlotUnit::Inch,
        1 => PlotUnit::Millimetre,
        2 => PlotUnit::Pixel,
        _ => return Err("unsupported plot paper unit"),
    };
    let rotation = match layout.plot_rotation {
        0 => PlotRotation::None,
        1 => PlotRotation::CounterClockwise90,
        2 => PlotRotation::UpsideDown,
        3 => PlotRotation::Clockwise90,
        _ => return Err("unsupported plot rotation"),
    };
    let printable_area = rect(
        layout.plot_margin_left,
        layout.plot_margin_bottom,
        layout.paper_width - layout.plot_margin_right,
        layout.paper_height - layout.plot_margin_top,
    )
    .ok_or("invalid plot margins")?;
    let scale = if layout.plot_scale_type == 0 {
        PlotScale::FitToArea
    } else if layout.plot_scale_numerator.is_finite()
        && layout.plot_scale_numerator > 0.0
        && layout.plot_scale_denominator.is_finite()
        && layout.plot_scale_denominator > 0.0
    {
        PlotScale::Fixed {
            output_length: layout.plot_scale_numerator,
            scope_length: layout.plot_scale_denominator,
        }
    } else {
        return Err("invalid plot scale");
    };
    let placement = if layout.plot_flags.plot_centered {
        PlotPlacement::Centered
    } else {
        PlotPlacement::Offset {
            reference: PlotOffsetReference::Media,
            x: layout.plot_origin_x,
            y: layout.plot_origin_y,
        }
    };
    if matches!(area, PlotArea::Layout)
        && (!matches!(scale, PlotScale::Fixed { .. })
            || matches!(placement, PlotPlacement::Centered))
    {
        return Err("unsupported Layout plot mapping");
    }
    let mode = match layout.shade_plot_mode {
        0 => ShadedPlotMode::AsDisplayed,
        1 => ShadedPlotMode::Wireframe,
        2 => ShadedPlotMode::Hidden,
        3 => ShadedPlotMode::Rendered,
        _ => return Err("unsupported shaded plot mode"),
    };
    let quality_mode = match layout.shade_plot_resolution {
        0 => ShadedPlotQualityMode::Draft,
        1 => ShadedPlotQualityMode::Preview,
        2 => ShadedPlotQualityMode::Normal,
        3 => ShadedPlotQualityMode::Presentation,
        4 => ShadedPlotQualityMode::Maximum,
        5 => ShadedPlotQualityMode::Custom,
        _ => return Err("unsupported shaded plot quality"),
    };
    let dpi = if quality_mode == ShadedPlotQualityMode::Custom {
        let dpi = u32::try_from(layout.shade_plot_dpi).map_err(|_| "invalid shaded plot dpi")?;
        if !(100..=32767).contains(&dpi) {
            return Err("invalid shaded plot dpi");
        }
        Some(dpi)
    } else {
        None
    };
    Ok(Some(PlotSettings {
        media: PlotMedia {
            unit,
            width: layout.paper_width,
            height: layout.paper_height,
            printable_area,
            rotation,
            device_name: nonempty(&layout.plot_printer_name),
            media_name: nonempty(&layout.paper_size),
        },
        area,
        mapping: PlotMapping { scale, placement },
        output: PlotOutput {
            shaded_plot: ShadedPlot {
                mode,
                quality: ShadedPlotQuality {
                    mode: quality_mode,
                    dpi,
                },
            },
            apply_plot_styles: layout.plot_flags.plot_plot_styles,
            plot_style_table_name: nonempty(&layout.plot_style_sheet),
        },
        options: PlotOptions {
            plot_viewport_borders: layout.plot_flags.plot_viewport_borders,
            plot_paper_space_last: layout.plot_flags.draw_viewports_first,
            hide_paper_space_objects: layout.plot_flags.plot_hidden,
            plot_line_weights: layout.plot_flags.print_lineweights,
            scale_line_weights: layout.plot_flags.scale_lineweights,
            plot_transparency: false,
        },
    }))
}

pub(crate) fn apply_plot_to_cad(target: &mut Layout, plot: &Value) {
    let media = &plot["media"];
    let number = |value: &Value| value.as_f64().expect("validated plot number");
    target.paper_width = number(&media["width"]);
    target.paper_height = number(&media["height"]);
    target.plot_paper_units = match media["unit"].as_str().unwrap() {
        "in" => 0,
        "mm" => 1,
        "px" => 2,
        _ => unreachable!(),
    };
    target.plot_rotation = match media["rotation"].as_str().unwrap() {
        "none" => 0,
        "counterClockwise90" => 1,
        "upsideDown" => 2,
        "clockwise90" => 3,
        _ => unreachable!(),
    };
    target.plot_printer_name = media["deviceName"].as_str().unwrap_or("").into();
    target.paper_size = media["mediaName"].as_str().unwrap_or("").into();
    let printable = &media["printableArea"];
    target.plot_margin_left = number(&printable["minX"]);
    target.plot_margin_bottom = number(&printable["minY"]);
    target.plot_margin_right = target.paper_width - number(&printable["maxX"]);
    target.plot_margin_top = target.paper_height - number(&printable["maxY"]);
    let area = &plot["area"];
    target.plot_type = match area["mode"].as_str().unwrap() {
        "Layout" => 5,
        "Extents" => 1,
        "Limits" => 2,
        "Window" => {
            let window = &area["window"];
            target.plot_window_min_x = number(&window["minX"]);
            target.plot_window_min_y = number(&window["minY"]);
            target.plot_window_max_x = number(&window["maxX"]);
            target.plot_window_max_y = number(&window["maxY"]);
            4
        }
        _ => unreachable!(),
    };
    let scale = &plot["mapping"]["scale"];
    if scale["mode"] == "FitToArea" {
        target.plot_scale_type = 0;
    } else {
        target.plot_scale_type = 1;
        target.plot_scale_numerator = number(&scale["outputLength"]);
        target.plot_scale_denominator = number(&scale["scopeLength"]);
        target.plot_scale_factor = target.plot_scale_numerator / target.plot_scale_denominator;
    }
    let placement = &plot["mapping"]["placement"];
    target.plot_flags.plot_centered = placement["mode"] == "Centered";
    if placement["mode"] == "Offset" {
        target.plot_origin_x = number(&placement["x"]);
        target.plot_origin_y = number(&placement["y"]);
    }
    let shaded = &plot["output"]["shadedPlot"];
    target.shade_plot_mode = match shaded["mode"].as_str().unwrap() {
        "AsDisplayed" => 0,
        "Wireframe" => 1,
        "Hidden" => 2,
        "Rendered" => 3,
        _ => unreachable!(),
    };
    target.shade_plot_resolution = match shaded["quality"]["mode"].as_str().unwrap() {
        "Draft" => 0,
        "Preview" => 1,
        "Normal" => 2,
        "Presentation" => 3,
        "Maximum" => 4,
        "Custom" => 5,
        _ => unreachable!(),
    };
    if let Some(dpi) = shaded["quality"]["dpi"].as_i64() {
        target.shade_plot_dpi = dpi as i16;
    }
    target.plot_flags.plot_plot_styles = plot["output"]["applyPlotStyles"] == true;
    target.plot_style_sheet = plot["output"]["plotStyleTableName"]
        .as_str()
        .unwrap_or("")
        .into();
    let options = &plot["options"];
    target.plot_flags.plot_viewport_borders = options["plotViewportBorders"] == true;
    target.plot_flags.draw_viewports_first = options["plotPaperSpaceLast"] == true;
    target.plot_flags.plot_hidden = options["hidePaperSpaceObjects"] == true;
    target.plot_flags.print_lineweights = options["plotLineWeights"] == true;
    target.plot_flags.scale_lineweights = options["scaleLineWeights"] == true;
}
