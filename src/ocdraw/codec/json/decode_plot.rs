use crate::ocdraw::{
    LayoutRect, PlotArea, PlotMapping, PlotMedia, PlotOffsetReference, PlotOptions, PlotOutput,
    PlotPlacement, PlotRotation, PlotScale, PlotSettings, PlotUnit, ShadedPlot, ShadedPlotMode,
    ShadedPlotQuality, ShadedPlotQualityMode,
};
use serde_json::Value;

pub(super) fn rectangle(value: &Value) -> Option<LayoutRect> {
    Some(LayoutRect {
        min_x: value.get("minX")?.as_f64()?,
        min_y: value.get("minY")?.as_f64()?,
        max_x: value.get("maxX")?.as_f64()?,
        max_y: value.get("maxY")?.as_f64()?,
    })
}

pub(super) fn plot_settings(value: &Value) -> Option<PlotSettings> {
    let media = value.get("media")?;
    let area = value.get("area")?;
    let mapping = value.get("mapping")?;
    let scale = mapping.get("scale")?;
    let placement = mapping.get("placement")?;
    let output = value.get("output")?;
    let shading = output.get("shadedPlot")?;
    let quality = shading.get("quality")?;
    let options = value.get("options")?;
    Some(PlotSettings {
        media: PlotMedia {
            unit: match media.get("unit")?.as_str()? {
                "mm" => PlotUnit::Millimetre,
                "in" => PlotUnit::Inch,
                "px" => PlotUnit::Pixel,
                _ => return None,
            },
            width: media.get("width")?.as_f64()?,
            height: media.get("height")?.as_f64()?,
            printable_area: rectangle(media.get("printableArea")?)?,
            rotation: match media.get("rotation")?.as_str()? {
                "none" => PlotRotation::None,
                "counterClockwise90" => PlotRotation::CounterClockwise90,
                "upsideDown" => PlotRotation::UpsideDown,
                "clockwise90" => PlotRotation::Clockwise90,
                _ => return None,
            },
            device_name: media
                .get("deviceName")
                .and_then(Value::as_str)
                .map(str::to_owned),
            media_name: media
                .get("mediaName")
                .and_then(Value::as_str)
                .map(str::to_owned),
        },
        area: match area.get("mode")?.as_str()? {
            "Layout" => PlotArea::Layout,
            "Extents" => PlotArea::Extents,
            "Limits" => PlotArea::Limits,
            "Window" => PlotArea::Window(rectangle(area.get("window")?)?),
            _ => return None,
        },
        mapping: PlotMapping {
            scale: match scale.get("mode")?.as_str()? {
                "Fixed" => PlotScale::Fixed {
                    output_length: scale.get("outputLength")?.as_f64()?,
                    scope_length: scale.get("scopeLength")?.as_f64()?,
                },
                "FitToArea" => PlotScale::FitToArea,
                _ => return None,
            },
            placement: match placement.get("mode")?.as_str()? {
                "Centered" => PlotPlacement::Centered,
                "Offset" => PlotPlacement::Offset {
                    reference: match placement.get("reference")?.as_str()? {
                        "Media" => PlotOffsetReference::Media,
                        "PrintableArea" => PlotOffsetReference::PrintableArea,
                        _ => return None,
                    },
                    x: placement.get("x")?.as_f64()?,
                    y: placement.get("y")?.as_f64()?,
                },
                _ => return None,
            },
        },
        output: PlotOutput {
            shaded_plot: ShadedPlot {
                mode: match shading.get("mode")?.as_str()? {
                    "AsDisplayed" => ShadedPlotMode::AsDisplayed,
                    "Wireframe" => ShadedPlotMode::Wireframe,
                    "Hidden" => ShadedPlotMode::Hidden,
                    "Rendered" => ShadedPlotMode::Rendered,
                    _ => return None,
                },
                quality: ShadedPlotQuality {
                    mode: match quality.get("mode")?.as_str()? {
                        "Draft" => ShadedPlotQualityMode::Draft,
                        "Preview" => ShadedPlotQualityMode::Preview,
                        "Normal" => ShadedPlotQualityMode::Normal,
                        "Presentation" => ShadedPlotQualityMode::Presentation,
                        "Maximum" => ShadedPlotQualityMode::Maximum,
                        "Custom" => ShadedPlotQualityMode::Custom,
                        _ => return None,
                    },
                    dpi: quality
                        .get("dpi")
                        .and_then(Value::as_u64)
                        .and_then(|dpi| u32::try_from(dpi).ok()),
                },
            },
            apply_plot_styles: output.get("applyPlotStyles")?.as_bool()?,
            plot_style_table_name: output
                .get("plotStyleTableName")
                .and_then(Value::as_str)
                .map(str::to_owned),
        },
        options: PlotOptions {
            plot_viewport_borders: options.get("plotViewportBorders")?.as_bool()?,
            plot_paper_space_last: options.get("plotPaperSpaceLast")?.as_bool()?,
            hide_paper_space_objects: options.get("hidePaperSpaceObjects")?.as_bool()?,
            plot_line_weights: options.get("plotLineWeights")?.as_bool()?,
            scale_line_weights: options.get("scaleLineWeights")?.as_bool()?,
            plot_transparency: options.get("plotTransparency")?.as_bool()?,
        },
    })
}
