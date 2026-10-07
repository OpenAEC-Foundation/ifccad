use super::*;
use crate::geometry_kernel::CoordinateLengthUnit;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlotValidationReason {
    NonPositiveOrNonFinite,
    UnitlessMedium,
    MissingMedium,
    IncompatibleOutputUnit,
    InvalidRectangle,
    PrintableOutsideMedium,
    InvalidAreaOrMapping,
    EmptyName,
    InvalidDpi,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlotValidationError {
    pub field: &'static str,
    pub reason: PlotValidationReason,
}

pub fn validate_layout_output(
    settings: &LayoutOutputSettings,
    kind: LayoutOutputKind,
) -> Result<(), Vec<PlotValidationError>> {
    use PlotValidationReason::*;
    let mut errors = Vec::new();
    let mut check = |valid: bool, field, reason| {
        if !valid {
            errors.push(PlotValidationError { field, reason });
        }
    };
    if let Some(media) = &settings.media {
        check(
            media.width.is_finite()
                && media.width > 0.
                && media.height.is_finite()
                && media.height > 0.,
            "media",
            NonPositiveOrNonFinite,
        );
        check(
            media.unit != MediaUnit::Physical(CoordinateLengthUnit::Unitless),
            "media.unit",
            UnitlessMedium,
        );
    }
    if let Some(limits) = settings.limits {
        check(limits.is_valid(), "limits", InvalidRectangle);
    }
    if let Some(plot) = &settings.plot_settings {
        check(settings.media.is_some(), "plotSettings", MissingMedium);
        if let Some(media) = &settings.media {
            check(
                matches!(
                    (media.unit, plot.plot_unit),
                    (MediaUnit::Pixel, PlotUnit::Pixel)
                        | (
                            MediaUnit::Physical(_),
                            PlotUnit::Inch | PlotUnit::Millimetre
                        )
                ),
                "plotSettings.plotUnit",
                IncompatibleOutputUnit,
            );
            let r = plot.page.printable_area;
            check(
                r.min_x >= 0. && r.min_y >= 0. && r.max_x <= media.width && r.max_y <= media.height,
                "plotSettings.page.printableArea",
                PrintableOutsideMedium,
            );
        }
        check(
            plot.page.printable_area.has_area(),
            "plotSettings.page.printableArea",
            InvalidRectangle,
        );
        for (name, field) in [
            (&plot.page.device_name, "plotSettings.page.deviceName"),
            (&plot.page.media_name, "plotSettings.page.mediaName"),
            (
                &plot.output.plot_style_table_name,
                "plotSettings.output.plotStyleTableName",
            ),
        ] {
            check(
                name.as_ref().is_none_or(|n| !n.is_empty()),
                field,
                EmptyName,
            );
        }
        if let PlotScale::Fixed {
            output_length,
            scope_length,
        } = plot.mapping.scale
        {
            check(
                output_length.is_finite()
                    && output_length > 0.
                    && scope_length.is_finite()
                    && scope_length > 0.,
                "plotSettings.mapping.scale",
                NonPositiveOrNonFinite,
            );
        }
        if let PlotPlacement::Offset { x, y, .. } = plot.mapping.placement {
            check(
                x.is_finite() && y.is_finite(),
                "plotSettings.mapping.placement",
                NonPositiveOrNonFinite,
            );
        }
        match plot.area {
            PlotArea::Layout => check(
                kind == LayoutOutputKind::Paper
                    && matches!(plot.mapping.scale, PlotScale::Fixed { .. })
                    && matches!(plot.mapping.placement, PlotPlacement::Offset { .. }),
                "plotSettings.area",
                InvalidAreaOrMapping,
            ),
            PlotArea::Limits => check(
                kind == LayoutOutputKind::Model && settings.limits.is_some(),
                "plotSettings.area",
                InvalidAreaOrMapping,
            ),
            PlotArea::Window(r) => {
                check(r.has_area(), "plotSettings.area.window", InvalidRectangle)
            }
            PlotArea::Extents => (),
        }
        let quality = plot.output.shaded_plot.quality;
        check(
            matches!(
                (quality.mode, quality.dpi),
                (ShadedPlotQualityMode::Custom, Some(100..=32767))
            ) || (quality.mode != ShadedPlotQualityMode::Custom && quality.dpi.is_none()),
            "plotSettings.output.shadedPlot.quality",
            InvalidDpi,
        );
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
