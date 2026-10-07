use crate::ocdraw::{
    LayoutRect, PlotArea, PlotOffsetReference, PlotPlacement, PlotRotation, PlotScale,
    PlotSettings, PlotUnit, ShadedPlotMode, ShadedPlotQualityMode,
};
use serde_json::{json, Value};

pub(crate) fn encode_rect(rect: LayoutRect) -> Option<Value> {
    rect.is_valid()
        .then(|| json!({"minX":rect.min_x,"minY":rect.min_y,"maxX":rect.max_x,"maxY":rect.max_y}))
}

pub(crate) fn encode_plot_settings(plot: &PlotSettings) -> Option<Value> {
    let unit = match plot.plot_unit {
        PlotUnit::Millimetre => "mm",
        PlotUnit::Inch => "in",
        PlotUnit::Pixel => "px",
    };
    let rotation = match plot.page.rotation {
        PlotRotation::None => "none",
        PlotRotation::CounterClockwise90 => "counterClockwise90",
        PlotRotation::UpsideDown => "upsideDown",
        PlotRotation::Clockwise90 => "clockwise90",
    };
    let mut page = json!({"printableArea":encode_rect(plot.page.printable_area).expect("valid layout rectangle"),"rotation":rotation});
    if let Some(name) = &plot.page.device_name {
        page["deviceName"] = json!(name);
    }
    if let Some(name) = &plot.page.media_name {
        page["mediaName"] = json!(name);
    }
    let area = match plot.area {
        PlotArea::Layout => json!({"mode":"Layout"}),
        PlotArea::Extents => json!({"mode":"Extents"}),
        PlotArea::Limits => json!({"mode":"Limits"}),
        PlotArea::Window(rect) => {
            json!({"mode":"Window","window":encode_rect(rect).expect("valid layout rectangle")})
        }
    };
    let scale = match plot.mapping.scale {
        PlotScale::Fixed {
            output_length,
            scope_length,
        } => json!({"mode":"Fixed","outputLength":output_length,"scopeLength":scope_length}),
        PlotScale::FitToArea => json!({"mode":"FitToArea"}),
    };
    let placement = match plot.mapping.placement {
        PlotPlacement::Centered => json!({"mode":"Centered"}),
        PlotPlacement::Offset { reference, x, y } => {
            json!({"mode":"Offset","reference":match reference {PlotOffsetReference::Media=>"Media",PlotOffsetReference::PrintableArea=>"PrintableArea"},"x":x,"y":y})
        }
    };
    let mut output = json!({"shadedPlot":encode_shading(plot.output.shaded_plot),"applyPlotStyles":plot.output.apply_plot_styles});
    if let Some(name) = &plot.output.plot_style_table_name {
        output["plotStyleTableName"] = json!(name);
    }
    let o = plot.options;
    Some(
        json!({"plotUnit":unit,"page":page,"area":area,"mapping":{"scale":scale,"placement":placement},"output":output,
            "options":{"plotViewportBorders":o.plot_viewport_borders,"plotPaperSpaceLast":o.plot_paper_space_last,"hidePaperSpaceObjects":o.hide_paper_space_objects,"plotLineWeights":o.plot_line_weights,"scaleLineWeights":o.scale_line_weights,"plotTransparency":o.plot_transparency}}),
    )
}

pub(super) fn encode_shading(shading: crate::ocdraw::ShadedPlot) -> Value {
    let mut quality = json!({"mode":match shading.quality.mode {ShadedPlotQualityMode::Draft=>"Draft",ShadedPlotQualityMode::Preview=>"Preview",ShadedPlotQualityMode::Normal=>"Normal",ShadedPlotQualityMode::Presentation=>"Presentation",ShadedPlotQualityMode::Maximum=>"Maximum",ShadedPlotQualityMode::Custom=>"Custom"}});
    if let Some(dpi) = shading.quality.dpi {
        quality["dpi"] = json!(dpi);
    }
    json!({"mode":match shading.mode {ShadedPlotMode::AsDisplayed=>"AsDisplayed",ShadedPlotMode::Wireframe=>"Wireframe",ShadedPlotMode::Hidden=>"Hidden",ShadedPlotMode::Rendered=>"Rendered"},"quality":quality})
}
