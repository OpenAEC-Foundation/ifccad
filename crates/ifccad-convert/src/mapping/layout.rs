//! CAD Layout plot fields and the standalone drawing Layout contract.

use ocdraw::plot_kernel::{
    LayoutMedia, MediaUnit, PlotArea, PlotMapping, PlotOffsetReference, PlotOptions, PlotOutput,
    PlotPage, PlotPlacement, PlotRect as LayoutRect, PlotRotation, PlotScale, PlotSettings,
    PlotUnit, ShadedPlot, ShadedPlotMode, ShadedPlotQuality, ShadedPlotQualityMode,
};
use opencadcodec::objects::Layout;

pub(crate) fn unrepresented_fields(layout: &Layout, model: bool) -> Vec<&'static str> {
    let default = Layout::new(&layout.name);
    let mut plot_flags = layout.plot_flags;
    if model {
        plot_flags.model_type = default.plot_flags.model_type;
    }
    let unconfigured_plot_state = layout.paper_width == 0.0
        && layout.paper_height == 0.0
        && (layout.plot_type != default.plot_type
            || plot_flags != default.plot_flags
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
                || layout.base_ucs != opencadcodec::Handle::NULL
                || layout.named_ucs != opencadcodec::Handle::NULL,
            "layout UCS",
        ),
        (
            !layout.reactors.is_empty() || layout.xdictionary_handle.is_some(),
            "layout object attachments",
        ),
        (!layout.plot_page_name.is_empty(), "named page setup source"),
        (
            !layout.plot_view_name.is_empty()
                || layout.plot_view_handle != opencadcodec::Handle::NULL,
            "saved plot view reference",
        ),
        (
            layout.visual_style_handle != opencadcodec::Handle::NULL,
            "shade-plot visual style reference",
        ),
        (
            layout.paper_image_origin_x != 0.0 || layout.paper_image_origin_y != 0.0,
            "paper image origin",
        ),
        (
            layout.plot_flags.model_type && !model,
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
) -> Result<Option<PlotSettings>, PlotMappingError> {
    if layout.paper_width == 0.0 && layout.paper_height == 0.0 {
        return Ok(None);
    }
    if !layout.paper_width.is_finite()
        || layout.paper_width <= 0.0
        || !layout.paper_height.is_finite()
        || layout.paper_height <= 0.0
    {
        return Err("invalid plot medium".into());
    }
    if plot_is_default(layout) {
        return Ok(None);
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
        _ => return Err("unsupported plot area".into()),
    };
    let unit = match layout.plot_paper_units {
        0 => PlotUnit::Inch,
        1 => PlotUnit::Millimetre,
        2 => PlotUnit::Pixel,
        _ => return Err("unsupported plot paper unit".into()),
    };
    if unit == PlotUnit::Pixel {
        return Err("pixel output has no qualified physical-to-raster mapping".into());
    }
    let rotation = match layout.plot_rotation {
        0 => PlotRotation::None,
        1 => PlotRotation::CounterClockwise90,
        2 => PlotRotation::UpsideDown,
        3 => PlotRotation::Clockwise90,
        _ => return Err("unsupported plot rotation".into()),
    };
    let printable_area = rect(
        layout.plot_margin_left,
        layout.plot_margin_bottom,
        subtract_plot_lengths_exact(layout.paper_width, layout.plot_margin_right).map_err(
            |_| PlotMappingError::Unsupported("no exact printable area corner representation"),
        )?,
        subtract_plot_lengths_exact(layout.paper_height, layout.plot_margin_top).map_err(|_| {
            PlotMappingError::Unsupported("no exact printable area corner representation")
        })?,
    )
    .ok_or("invalid plot margins")?;
    let scale = effective_scale(layout)?;
    let placement = if layout.plot_flags.plot_centered {
        PlotPlacement::Centered
    } else {
        PlotPlacement::Offset {
            reference: PlotOffsetReference::Media,
            x: convert_plot_length_exact(layout.plot_origin_x, mm(), output_unit(unit))?,
            y: convert_plot_length_exact(layout.plot_origin_y, mm(), output_unit(unit))?,
        }
    };
    if matches!(area, PlotArea::Layout)
        && (!matches!(scale, PlotScale::Fixed { .. })
            || matches!(placement, PlotPlacement::Centered))
    {
        return Err("unsupported Layout plot mapping".into());
    }
    let mode = match layout.shade_plot_mode {
        0 => ShadedPlotMode::AsDisplayed,
        1 => ShadedPlotMode::Wireframe,
        2 => ShadedPlotMode::Hidden,
        3 => ShadedPlotMode::Rendered,
        _ => return Err("unsupported shaded plot mode".into()),
    };
    let quality_mode = match layout.shade_plot_resolution {
        0 => ShadedPlotQualityMode::Draft,
        1 => ShadedPlotQualityMode::Preview,
        2 => ShadedPlotQualityMode::Normal,
        3 => ShadedPlotQualityMode::Presentation,
        4 => ShadedPlotQualityMode::Maximum,
        5 => ShadedPlotQualityMode::Custom,
        _ => return Err("unsupported shaded plot quality".into()),
    };
    let dpi = if quality_mode == ShadedPlotQualityMode::Custom {
        let dpi = u32::try_from(layout.shade_plot_dpi).map_err(|_| "invalid shaded plot dpi")?;
        if !(100..=32767).contains(&dpi) {
            return Err("invalid shaded plot dpi".into());
        }
        Some(dpi)
    } else {
        None
    };
    Ok(Some(PlotSettings {
        plot_unit: unit,
        page: PlotPage {
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

pub(crate) fn apply_plot_to_cad(
    target: &mut Layout,
    medium: &LayoutMedia,
    plot: &PlotSettings,
) -> Result<(), PlotNumericError> {
    let media = &plot.page;
    if plot.plot_unit == PlotUnit::Pixel {
        return Err(PlotNumericError::UnsupportedUnit);
    }
    target.plot_paper_units = match plot.plot_unit {
        PlotUnit::Inch => 0,
        PlotUnit::Millimetre => 1,
        PlotUnit::Pixel => 2,
    };
    target.plot_rotation = match media.rotation {
        PlotRotation::None => 0,
        PlotRotation::CounterClockwise90 => 1,
        PlotRotation::UpsideDown => 2,
        PlotRotation::Clockwise90 => 3,
    };
    target.plot_printer_name = media.device_name.clone().unwrap_or_default();
    target.paper_size = media.media_name.clone().unwrap_or_default();
    target.plot_margin_left =
        convert_plot_length_exact(media.printable_area.min_x, medium.unit, mm())?;
    target.plot_margin_bottom =
        convert_plot_length_exact(media.printable_area.min_y, medium.unit, mm())?;
    target.plot_margin_right = convert_plot_length_exact(
        subtract_plot_lengths_exact(medium.width, media.printable_area.max_x)?,
        medium.unit,
        mm(),
    )?;
    target.plot_margin_top = convert_plot_length_exact(
        subtract_plot_lengths_exact(medium.height, media.printable_area.max_y)?,
        medium.unit,
        mm(),
    )?;
    target.plot_type = match plot.area {
        PlotArea::Layout => 5,
        PlotArea::Extents => 1,
        PlotArea::Limits => 2,
        PlotArea::Window(window) => {
            target.plot_window_min_x = window.min_x;
            target.plot_window_min_y = window.min_y;
            target.plot_window_max_x = window.max_x;
            target.plot_window_max_y = window.max_y;
            4
        }
    };
    match plot.mapping.scale {
        PlotScale::FitToArea => {
            target.plot_scale_type = 0;
            target.plot_flags.use_standard_scale = true;
        }
        PlotScale::Fixed {
            output_length,
            scope_length,
        } => {
            target.plot_scale_type = 0;
            target.plot_flags.use_standard_scale = false;
            target.plot_scale_numerator = output_length;
            target.plot_scale_denominator = scope_length;
            let factor = output_length / scope_length;
            if !factor.is_finite() || factor <= 0. {
                return Err(PlotNumericError::OutOfRange);
            }
            target.plot_scale_factor = factor;
        }
    }
    target.plot_flags.plot_centered = matches!(plot.mapping.placement, PlotPlacement::Centered);
    if let PlotPlacement::Offset { x, y, .. } = plot.mapping.placement {
        target.plot_origin_x = convert_plot_length_exact(x, output_unit(plot.plot_unit), mm())?;
        target.plot_origin_y = convert_plot_length_exact(y, output_unit(plot.plot_unit), mm())?;
    }
    target.shade_plot_mode = match plot.output.shaded_plot.mode {
        ShadedPlotMode::AsDisplayed => 0,
        ShadedPlotMode::Wireframe => 1,
        ShadedPlotMode::Hidden => 2,
        ShadedPlotMode::Rendered => 3,
    };
    target.shade_plot_resolution = match plot.output.shaded_plot.quality.mode {
        ShadedPlotQualityMode::Draft => 0,
        ShadedPlotQualityMode::Preview => 1,
        ShadedPlotQualityMode::Normal => 2,
        ShadedPlotQualityMode::Presentation => 3,
        ShadedPlotQualityMode::Maximum => 4,
        ShadedPlotQualityMode::Custom => 5,
    };
    if let Some(dpi) = plot.output.shaded_plot.quality.dpi {
        target.shade_plot_dpi = dpi as i16;
    }
    target.plot_flags.plot_plot_styles = plot.output.apply_plot_styles;
    target.plot_style_sheet = plot
        .output
        .plot_style_table_name
        .clone()
        .unwrap_or_default();
    let options = plot.options;
    target.plot_flags.plot_viewport_borders = options.plot_viewport_borders;
    target.plot_flags.draw_viewports_first = options.plot_paper_space_last;
    target.plot_flags.plot_hidden = options.hide_paper_space_objects;
    target.plot_flags.print_lineweights = options.plot_line_weights;
    target.plot_flags.scale_lineweights = options.scale_line_weights;
    Ok(())
}
use cad_geometry_convert::plot_units::{
    convert_plot_length_exact, subtract_plot_lengths_exact, PlotNumericError,
};
use ocdraw::geometry_kernel::CoordinateLengthUnit;
#[derive(Debug)]
pub(crate) enum PlotMappingError {
    Unsupported(&'static str),
    Numeric(PlotNumericError),
}
impl From<&'static str> for PlotMappingError {
    fn from(v: &'static str) -> Self {
        Self::Unsupported(v)
    }
}
impl From<PlotNumericError> for PlotMappingError {
    fn from(v: PlotNumericError) -> Self {
        Self::Numeric(v)
    }
}
impl std::fmt::Display for PlotMappingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported(s) => f.write_str(s),
            Self::Numeric(e) => e.fmt(f),
        }
    }
}
fn mm() -> MediaUnit {
    MediaUnit::Physical(CoordinateLengthUnit::Millimetre)
}
fn output_unit(unit: PlotUnit) -> MediaUnit {
    match unit {
        PlotUnit::Millimetre => mm(),
        PlotUnit::Inch => MediaUnit::Physical(CoordinateLengthUnit::Inch),
        PlotUnit::Pixel => MediaUnit::Pixel,
    }
}
pub(crate) fn medium_from_cad(l: &Layout) -> Option<LayoutMedia> {
    (l.paper_width.is_finite()
        && l.paper_width > 0.
        && l.paper_height.is_finite()
        && l.paper_height > 0.)
        .then_some(LayoutMedia {
            unit: mm(),
            width: l.paper_width,
            height: l.paper_height,
        })
}
pub(crate) fn apply_medium_to_cad(
    l: &mut Layout,
    media: &LayoutMedia,
) -> Result<(), PlotNumericError> {
    let w = convert_plot_length_exact(media.width, media.unit, mm())?;
    let h = convert_plot_length_exact(media.height, media.unit, mm())?;
    l.paper_width = w;
    l.paper_height = h;
    Ok(())
}
pub(crate) fn effective_scale(l: &Layout) -> Result<PlotScale, PlotMappingError> {
    if l.plot_flags.use_standard_scale {
        if l.plot_scale_type == 0 {
            return Ok(PlotScale::FitToArea);
        }
        let (n, d) = match l.plot_scale_type {
            1 => (1., 1536.),
            2 => (1., 768.),
            3 => (1., 384.),
            4 => (1., 192.),
            5 => (1., 128.),
            6 => (1., 96.),
            7 => (1., 64.),
            8 => (1., 48.),
            9 => (1., 32.),
            10 => (1., 24.),
            11 => (1., 16.),
            12 => (1., 12.),
            13 => (1., 4.),
            14 => (1., 2.),
            15 | 16 => (1., 1.),
            17 => (1., 2.),
            18 => (1., 4.),
            19 => (1., 8.),
            20 => (1., 10.),
            21 => (1., 16.),
            22 => (1., 20.),
            23 => (1., 30.),
            24 => (1., 40.),
            25 => (1., 50.),
            26 => (1., 100.),
            27 => (2., 1.),
            28 => (4., 1.),
            29 => (8., 1.),
            30 => (10., 1.),
            31 => (100., 1.),
            32 => (1000., 1.),
            _ => return Err("unsupported standard plot scale".into()),
        };
        if !l.plot_scale_factor.is_finite() || l.plot_scale_factor != n / d {
            return Err("inconsistent active standard plot scale".into());
        }
        return Ok(PlotScale::Fixed {
            output_length: n,
            scope_length: d,
        });
    }
    if l.plot_scale_numerator.is_finite()
        && l.plot_scale_numerator > 0.
        && l.plot_scale_denominator.is_finite()
        && l.plot_scale_denominator > 0.
    {
        Ok(PlotScale::Fixed {
            output_length: l.plot_scale_numerator,
            scope_length: l.plot_scale_denominator,
        })
    } else {
        Err("invalid custom plot scale".into())
    }
}
pub(crate) fn plot_is_default(l: &Layout) -> bool {
    let d = Layout::new(&l.name);
    let mut flags = l.plot_flags;
    flags.model_type = d.plot_flags.model_type;
    flags == d.plot_flags
        && l.plot_type == d.plot_type
        && l.plot_rotation == d.plot_rotation
        && l.plot_paper_units == d.plot_paper_units
        && l.plot_scale_type == d.plot_scale_type
        && l.plot_scale_numerator == d.plot_scale_numerator
        && l.plot_scale_denominator == d.plot_scale_denominator
        && l.plot_origin_x == 0.
        && l.plot_origin_y == 0.
        && l.plot_margin_left == 0.
        && l.plot_margin_bottom == 0.
        && l.plot_margin_right == 0.
        && l.plot_margin_top == 0.
        && l.plot_window_min_x == 0.
        && l.plot_window_min_y == 0.
        && l.plot_window_max_x == 0.
        && l.plot_window_max_y == 0.
        && l.shade_plot_mode == d.shade_plot_mode
        && l.shade_plot_resolution == d.shade_plot_resolution
        && l.shade_plot_dpi == d.shade_plot_dpi
        && l.plot_printer_name.is_empty()
        && l.paper_size.is_empty()
        && l.plot_style_sheet.is_empty()
}

use crate::{IfccadConversionError, IfccadDiagnostic};
pub(crate) fn settings_from_cad(
    l: &Layout,
    model: bool,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<ocdraw::ifccad::IfccadLayoutSettings, IfccadConversionError> {
    let location = format!("layout/{}", l.name);
    if medium_from_cad(l).is_none() && (l.paper_width != 0. || l.paper_height != 0.) {
        issues.push(crate::diagnostics::diagnostic(
            "paper-medium",
            format!("{location}.media"),
            "invalid or partial medium omitted; layout geometry retained",
        ));
    }
    let mut settings = ocdraw::ifccad::IfccadLayoutSettings {
        media: medium_from_cad(l),
        limits: if l.min_limits != l.max_limits {
            rect(
                l.min_limits.0,
                l.min_limits.1,
                l.max_limits.0,
                l.max_limits.1,
            )
        } else {
            None
        },
        limits_checking: l.flags & 2 != 0,
        paper_space_linetype_scaling: l.flags & 1 != 0,
        plot_settings: match plot_from_cad(l, model) {
            Ok(v) => v,
            Err(PlotMappingError::Numeric(e)) => return Err(e.into()),
            Err(e) => {
                issues.push(crate::diagnostics::diagnostic(
                    "plot-settings",
                    &location,
                    e.to_string(),
                ));
                None
            }
        },
    };
    if ocdraw::plot_kernel::validate_layout_output(
        &settings,
        if model {
            ocdraw::plot_kernel::LayoutOutputKind::Model
        } else {
            ocdraw::plot_kernel::LayoutOutputKind::Paper
        },
    )
    .is_err()
    {
        settings.plot_settings = None;
        issues.push(crate::diagnostics::diagnostic(
            "plot-settings",
            &location,
            "invalid plot record omitted; valid medium retained",
        ));
    }
    for reason in unrepresented_fields(l, model) {
        issues.push(crate::diagnostics::diagnostic(
            "layout-field",
            &location,
            reason,
        ));
    }
    Ok(settings)
}
pub(crate) fn apply_settings(
    l: &mut Layout,
    settings: &ocdraw::ifccad::IfccadLayoutSettings,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<(), IfccadConversionError> {
    if settings.paper_space_linetype_scaling {
        l.flags |= 1;
    } else {
        l.flags &= !1;
    }
    if settings.limits_checking {
        l.flags |= 2;
    } else {
        l.flags &= !2;
    }
    if let Some(r) = settings.limits {
        l.min_limits = (r.min_x, r.min_y);
        l.max_limits = (r.max_x, r.max_y);
    }
    if let Some(media) = &settings.media {
        match apply_medium_to_cad(l, media) {
            Ok(()) => (),
            Err(PlotNumericError::UnsupportedUnit) => {
                issues.push(crate::diagnostics::diagnostic(
                    "paper-medium",
                    format!("layout/{}", l.name),
                    "unsupported exact CAD medium unit; medium and plot configuration omitted",
                ));
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        }
    }
    if let Some(plot) = &settings.plot_settings {
        apply_plot_to_cad(l, settings.media.as_ref().expect("validated medium"), plot)?;
        if plot_is_default(l) {
            issues.push(crate::diagnostics::diagnostic(
                "plot-default-ambiguity",
                format!("layout/{}", l.name),
                "authored plot equals CAD defaults; absence cannot be distinguished on reimport",
            ));
        }
        if matches!(
            plot.mapping.placement,
            PlotPlacement::Offset {
                reference: PlotOffsetReference::PrintableArea,
                ..
            }
        ) {
            issues.push(crate::diagnostics::diagnostic(
                "plot-settings",
                format!("layout/{}", l.name),
                "CAD cannot assert printable-area-relative offset",
            ));
        }
        if plot.options.plot_transparency {
            issues.push(crate::diagnostics::diagnostic(
                "plot-settings",
                format!("layout/{}", l.name),
                "CAD plot transparency unsupported",
            ));
        }
        if plot.output.apply_plot_styles && plot.output.plot_style_table_name.is_some() {
            issues.push(crate::diagnostics::diagnostic(
                "plot-settings",
                format!("layout/{}", l.name),
                "active external plot style table contents unavailable",
            ));
        }
    }
    Ok(())
}
