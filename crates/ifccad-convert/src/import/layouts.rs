use super::diagnostic::DiagnosticAccumulator;
use crate::{ImportDiagnostic, ImportError};
use cadcodec::objects::{Layout, ObjectType};
use cadcodec::{CadDocument, Handle};
use ifccad::ifcdr::{ScopeId, ShadedPlotMode, ShadedPlotQualityMode};
use ifccad::package::{
    DrawingLayoutKind, DrawingLayoutRef, LayoutSettings, PlotArea, PlotPlacement, PlotRotation,
    PlotScale, PlotUnit,
};
use std::collections::BTreeMap;

pub(crate) fn allocate(
    document: &mut CadDocument,
    layouts: &[DrawingLayoutRef<'_>],
    diagnostics: &mut DiagnosticAccumulator,
) -> Result<BTreeMap<ScopeId, Handle>, ImportError> {
    let mut owners = BTreeMap::new();
    for source in layouts {
        let model = source.kind() == DrawingLayoutKind::Model;
        let handle = if model {
            document.objects.iter().find_map(|(&handle, object)| match object {
                ObjectType::Layout(layout) if layout.block_record == document.header.model_space_block_handle => Some(handle),
                _ => None,
            }).ok_or_else(|| ImportError::InternalInvariant { message: "fresh CadDocument has no model layout".into() })?
        } else if let Some((&handle, _)) = document.objects.iter().find(|(_, object)| matches!(object, ObjectType::Layout(layout) if layout.name == source.name())) {
            handle
        } else {
            document.add_layout(source.name()).map_err(|error| ImportError::InternalInvariant { message: format!("could not create paper layout: {error}") })?
        };
        let ObjectType::Layout(target) =
            document.objects.get_mut(&handle).expect("allocated layout")
        else {
            unreachable!()
        };
        target.name = source.name().into();
        let settings = source.settings();
        if document.header.paper_space_linetype_scaling != settings.paper_space_linetype_scaling
            && !model
        {
            diagnostics.record(ImportDiagnostic::LayoutFieldUnsupported {
                layout: source.name().into(),
                field: "paperSpaceLinetypeScaling differs from document-wide CAD header".into(),
            });
        } else {
            document.header.paper_space_linetype_scaling = settings.paper_space_linetype_scaling;
        }
        apply_settings(target, settings, source.name(), diagnostics);
        if !model {
            owners.insert(source.scope().id(), target.block_record);
        } else if source.name() != "Model" {
            if let Some(ObjectType::Dictionary(dictionary)) = document
                .objects
                .get_mut(&document.header.acad_layout_dict_handle)
            {
                if let Some((name, _)) = dictionary
                    .entries
                    .iter_mut()
                    .find(|(_, linked)| *linked == handle)
                {
                    *name = source.name().into();
                }
            }
        }
    }
    Ok(owners)
}

fn apply_settings(
    target: &mut Layout,
    settings: LayoutSettings,
    name: &str,
    diagnostics: &mut DiagnosticAccumulator,
) {
    target.flags = if settings.limits_checking {
        target.flags | 2
    } else {
        target.flags & !2
    };
    if let Some(rect) = settings.limits {
        target.min_limits = (rect.min_x, rect.min_y);
        target.max_limits = (rect.max_x, rect.max_y);
    }
    let Some(plot) = settings.plot_settings else {
        return;
    };
    target.paper_width = plot.media.width;
    target.paper_height = plot.media.height;
    target.plot_paper_units = match plot.media.unit {
        PlotUnit::Inch => 0,
        PlotUnit::Millimetre => 1,
        PlotUnit::Pixel => 2,
    };
    target.plot_rotation = match plot.media.rotation {
        PlotRotation::None => 0,
        PlotRotation::CounterClockwise90 => 1,
        PlotRotation::UpsideDown => 2,
        PlotRotation::Clockwise90 => 3,
    };
    target.plot_printer_name = plot.media.device_name.unwrap_or_default();
    target.paper_size = plot.media.media_name.unwrap_or_default();
    target.plot_margin_left = plot.media.printable_area.min_x;
    target.plot_margin_bottom = plot.media.printable_area.min_y;
    target.plot_margin_right = plot.media.width - plot.media.printable_area.max_x;
    target.plot_margin_top = plot.media.height - plot.media.printable_area.max_y;
    target.plot_type = match plot.area {
        PlotArea::Layout => 5,
        PlotArea::Extents => 1,
        PlotArea::Limits => 2,
        PlotArea::Window(rect) => {
            target.plot_window_min_x = rect.min_x;
            target.plot_window_min_y = rect.min_y;
            target.plot_window_max_x = rect.max_x;
            target.plot_window_max_y = rect.max_y;
            4
        }
    };
    match plot.mapping.scale {
        PlotScale::FitToArea => target.plot_scale_type = 0,
        PlotScale::Fixed {
            output_length,
            scope_length,
        } => {
            target.plot_scale_type = 1;
            target.plot_scale_numerator = output_length;
            target.plot_scale_denominator = scope_length;
            target.plot_scale_factor = output_length / scope_length;
        }
    }
    match plot.mapping.placement {
        PlotPlacement::Centered => target.plot_flags.plot_centered = true,
        PlotPlacement::Offset { reference, x, y } => {
            target.plot_flags.plot_centered = false;
            target.plot_origin_x = x;
            target.plot_origin_y = y;
            if reference != ifccad::package::PlotOffsetReference::Media {
                diagnostics.record(ImportDiagnostic::LayoutFieldUnsupported {
                    layout: name.into(),
                    field: "printable-area-relative plot offset".into(),
                });
            }
        }
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
    target.plot_style_sheet = plot.output.plot_style_table_name.unwrap_or_default();
    target.plot_flags.plot_viewport_borders = plot.options.plot_viewport_borders;
    target.plot_flags.draw_viewports_first = plot.options.plot_paper_space_last;
    target.plot_flags.plot_hidden = plot.options.hide_paper_space_objects;
    target.plot_flags.print_lineweights = plot.options.plot_line_weights;
    target.plot_flags.scale_lineweights = plot.options.scale_line_weights;
    if plot.options.plot_transparency {
        diagnostics.record(ImportDiagnostic::LayoutFieldUnsupported {
            layout: name.into(),
            field: "plotTransparency".into(),
        });
    }
}
