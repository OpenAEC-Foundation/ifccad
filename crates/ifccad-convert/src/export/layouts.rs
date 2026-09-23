use super::conversion::ExportContext;
use super::{
    ExportAction, ExportDiagnostic, ExportDiagnosticSource, ExportError, ExportLossReason,
    SourceStructureProblem,
};
use cadcodec::objects::{Layout, ObjectType};
use cadcodec::{CadDocument, Handle};
use ifccad::ifcdr::{ShadedPlot, ShadedPlotMode, ShadedPlotQuality, ShadedPlotQualityMode};
use ifccad::package::{
    DrawingBuilder, LayoutSettings, PlotArea, PlotMapping, PlotMedia, PlotOffsetReference,
    PlotOptions, PlotOutput, PlotPlacement, PlotRect, PlotRotation, PlotScale, PlotSettings,
    PlotUnit,
};

pub(crate) fn add_layouts(
    document: &CadDocument,
    drawing: &mut DrawingBuilder<'_>,
    context: &mut ExportContext,
) -> Result<(), ExportError> {
    let mut layouts = document
        .objects
        .values()
        .filter_map(|object| match object {
            ObjectType::Layout(layout) => Some(layout),
            _ => None,
        })
        .collect::<Vec<_>>();
    layouts.sort_by(|a, b| {
        a.tab_order
            .cmp(&b.tab_order)
            .then_with(|| a.name.cmp(&b.name))
    });
    let mut used_blocks = std::collections::BTreeSet::new();
    for layout in layouts {
        let model = layout.block_record == document.header.model_space_block_handle;
        if !model && is_untouched_scaffold(layout, document) {
            continue;
        }
        if !used_blocks.insert(layout.block_record) {
            return Err(ExportError::InvalidSourceStructure {
                problems: vec![SourceStructureProblem::InconsistentRelationship {
                    description: format!(
                        "multiple layouts select block record {:?}",
                        layout.block_record
                    ),
                }],
            });
        }
        if !model
            && document
                .block_records
                .iter()
                .all(|record| record.handle != layout.block_record)
        {
            return Err(ExportError::InvalidSourceStructure {
                problems: vec![SourceStructureProblem::InconsistentRelationship {
                    description: format!("layout {} has no block record", layout.name),
                }],
            });
        }
        diagnose_layout_extras(layout, context);
        let settings = settings_from_cad(layout, model, document, context);
        if model {
            drawing.set_model_layout_settings(settings);
        } else {
            let key = drawing.add_paper_space(layout.name.clone())?;
            drawing.set_paper_layout_settings(key, settings)?;
            context.paper_scopes.insert(layout.block_record, key);
        }
    }
    Ok(())
}

fn is_untouched_scaffold(layout: &Layout, document: &CadDocument) -> bool {
    if layout.name != "Layout1" || layout.block_record != document.header.paper_space_block_handle {
        return false;
    }
    if document
        .entities()
        .any(|entity| entity.common().owner_handle == layout.block_record)
    {
        return false;
    }
    let mut candidate = layout.clone();
    let mut baseline = Layout::new("Layout1");
    candidate.handle = Handle::NULL;
    candidate.owner = Handle::NULL;
    candidate.block_record = Handle::NULL;
    candidate.tab_order = 0;
    if candidate.raw_plot_settings_codes.as_ref().is_some_and(|codes| {
        codes.iter().any(|(code, _)| !matches!(code, 1 | 2 | 4 | 6 | 7 | 40..=49 | 140..=143 | 147..=149 | 70 | 72..=78))
    }) {
        return false;
    }
    candidate.raw_plot_settings_codes = None;
    baseline.tab_order = 0;
    candidate == baseline
}

fn diagnose_layout_extras(layout: &Layout, context: &mut ExportContext) {
    let default = Layout::new(&layout.name);
    let unsupported = [
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
                || layout.base_ucs != Handle::NULL
                || layout.named_ucs != Handle::NULL,
            "layout UCS",
        ),
        (
            !layout.reactors.is_empty() || layout.xdictionary_handle.is_some(),
            "layout object attachments",
        ),
        (!layout.plot_page_name.is_empty(), "named page setup source"),
        (
            !layout.plot_view_name.is_empty() || layout.plot_view_handle != Handle::NULL,
            "saved plot view reference",
        ),
        (
            layout.visual_style_handle != Handle::NULL,
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
    ];
    for (present, name) in unsupported {
        if present {
            loss(context, layout, name, ExportAction::PartiallyExported);
        }
    }
}

fn settings_from_cad(
    layout: &Layout,
    model: bool,
    document: &CadDocument,
    context: &mut ExportContext,
) -> LayoutSettings {
    let limits = rect(
        layout.min_limits.0,
        layout.min_limits.1,
        layout.max_limits.0,
        layout.max_limits.1,
    );
    let mut settings = LayoutSettings {
        limits,
        limits_checking: (layout.flags & 2) != 0,
        paper_space_linetype_scaling: document.header.paper_space_linetype_scaling,
        plot_settings: None,
    };
    if layout.min_limits != layout.max_limits && limits.is_none() {
        loss(
            context,
            layout,
            "invalid layout limits",
            ExportAction::PartiallyExported,
        );
    }
    if layout.paper_width == 0.0 && layout.paper_height == 0.0 {
        let default = Layout::new(&layout.name);
        let mut plot_flags = layout.plot_flags;
        // DWG readers set this redundant flag on the model layout even when
        // no plot medium has ever been configured.
        if model {
            plot_flags.model_type = default.plot_flags.model_type;
        }
        if layout.plot_type != default.plot_type
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
            || !layout.paper_size.is_empty()
        {
            loss(
                context,
                layout,
                "unconfigured plot medium has meaningful plot fields",
                ExportAction::Skipped,
            );
        }
        return settings;
    }
    let area = match layout.plot_type {
        1 => PlotArea::Extents,
        2 if model && limits.is_some() => PlotArea::Limits,
        4 => match rect(
            layout.plot_window_min_x,
            layout.plot_window_min_y,
            layout.plot_window_max_x,
            layout.plot_window_max_y,
        ) {
            Some(window) => PlotArea::Window(window),
            None => {
                loss(
                    context,
                    layout,
                    "invalid plot window",
                    ExportAction::Skipped,
                );
                return settings;
            }
        },
        5 if !model => PlotArea::Layout,
        0 | 3 => {
            loss(
                context,
                layout,
                "deferred Display or NamedView plot area",
                ExportAction::Skipped,
            );
            return settings;
        }
        _ => {
            loss(
                context,
                layout,
                "unsupported plot area",
                ExportAction::Skipped,
            );
            return settings;
        }
    };
    let unit = match layout.plot_paper_units {
        0 => PlotUnit::Inch,
        1 => PlotUnit::Millimetre,
        2 => PlotUnit::Pixel,
        _ => {
            loss(
                context,
                layout,
                "unsupported plot paper unit",
                ExportAction::Skipped,
            );
            return settings;
        }
    };
    let rotation = match layout.plot_rotation {
        0 => PlotRotation::None,
        1 => PlotRotation::CounterClockwise90,
        2 => PlotRotation::UpsideDown,
        3 => PlotRotation::Clockwise90,
        _ => {
            loss(
                context,
                layout,
                "unsupported plot rotation",
                ExportAction::Skipped,
            );
            return settings;
        }
    };
    let Some(printable_area) = rect(
        layout.plot_margin_left,
        layout.plot_margin_bottom,
        layout.paper_width - layout.plot_margin_right,
        layout.paper_height - layout.plot_margin_top,
    ) else {
        loss(
            context,
            layout,
            "invalid plot media or margins",
            ExportAction::Skipped,
        );
        return settings;
    };
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
        loss(context, layout, "invalid plot scale", ExportAction::Skipped);
        return settings;
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
        loss(
            context,
            layout,
            "unsupported Layout plot mapping",
            ExportAction::Skipped,
        );
        return settings;
    }
    let mode = match layout.shade_plot_mode {
        0 => ShadedPlotMode::AsDisplayed,
        1 => ShadedPlotMode::Wireframe,
        2 => ShadedPlotMode::Hidden,
        3 => ShadedPlotMode::Rendered,
        _ => {
            loss(
                context,
                layout,
                "unsupported shade plot mode",
                ExportAction::Skipped,
            );
            return settings;
        }
    };
    let quality_mode = match layout.shade_plot_resolution {
        0 => ShadedPlotQualityMode::Draft,
        1 => ShadedPlotQualityMode::Preview,
        2 => ShadedPlotQualityMode::Normal,
        3 => ShadedPlotQualityMode::Presentation,
        4 => ShadedPlotQualityMode::Maximum,
        5 => ShadedPlotQualityMode::Custom,
        _ => {
            loss(
                context,
                layout,
                "unsupported shade plot resolution",
                ExportAction::Skipped,
            );
            return settings;
        }
    };
    let dpi = if quality_mode == ShadedPlotQualityMode::Custom {
        match u32::try_from(layout.shade_plot_dpi) {
            Ok(dpi) if (100..=32767).contains(&dpi) => Some(dpi),
            _ => {
                loss(
                    context,
                    layout,
                    "invalid custom shade plot dpi",
                    ExportAction::Skipped,
                );
                return settings;
            }
        }
    } else {
        None
    };
    settings.plot_settings = Some(PlotSettings {
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
    });
    if layout.plot_flags.plot_plot_styles && !layout.plot_style_sheet.is_empty() {
        loss(
            context,
            layout,
            "active external CTB/STB contents are unavailable",
            ExportAction::PartiallyExported,
        );
    }
    if layout.plot_flags.unknown_bits != 0
        || layout.plot_flags.show_plot_styles
        || layout.plot_flags.update_paper
        || layout.plot_flags.zoom_to_paper_on_update
        || layout.plot_flags.initializing
        || layout.plot_flags.prev_plot_init
    {
        loss(
            context,
            layout,
            "unrepresented plot flags",
            ExportAction::PartiallyExported,
        );
    }
    settings
}

fn rect(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Option<PlotRect> {
    if [min_x, min_y, max_x, max_y].into_iter().all(f64::is_finite)
        && min_x < max_x
        && min_y < max_y
    {
        Some(PlotRect {
            min_x,
            min_y,
            max_x,
            max_y,
        })
    } else {
        None
    }
}
fn nonempty(s: &str) -> Option<String> {
    (!s.is_empty()).then(|| s.to_owned())
}
fn loss(context: &mut ExportContext, layout: &Layout, name: &str, action: ExportAction) {
    context.diagnostics.push(ExportDiagnostic::loss(
        ExportDiagnosticSource::Object {
            handle: layout.handle,
            kind: "LAYOUT".into(),
        },
        action,
        vec![ExportLossReason::UnsupportedSemantic { name: name.into() }],
    ));
}
