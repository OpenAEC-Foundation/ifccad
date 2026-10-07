use crate::plot_kernel::{
    PlotArea, PlotMapping, PlotOffsetReference, PlotOptions, PlotOutput, PlotPage, PlotPlacement,
    PlotRect as LayoutRect, PlotRotation, PlotScale, PlotSettings, PlotUnit, ShadedPlot,
    ShadedPlotMode, ShadedPlotQuality, ShadedPlotQualityMode,
};
use serde_json::Value;

fn rectangle(value: &Value) -> Option<LayoutRect> {
    Some(LayoutRect {
        min_x: value.get("minX")?.as_f64()?,
        min_y: value.get("minY")?.as_f64()?,
        max_x: value.get("maxX")?.as_f64()?,
        max_y: value.get("maxY")?.as_f64()?,
    })
}

fn plot_settings(value: &Value) -> Option<PlotSettings> {
    let page = value.get("page")?;
    let area = value.get("area")?;
    let mapping = value.get("mapping")?;
    let scale = mapping.get("scale")?;
    let placement = mapping.get("placement")?;
    let output = value.get("output")?;
    let shading = output.get("shadedPlot")?;
    let quality = shading.get("quality")?;
    let options = value.get("options")?;
    Some(PlotSettings {
        plot_unit: match value.get("plotUnit")?.as_str()? {
            "mm" => PlotUnit::Millimetre,
            "in" => PlotUnit::Inch,
            "px" => PlotUnit::Pixel,
            _ => return None,
        },
        page: PlotPage {
            printable_area: rectangle(page.get("printableArea")?)?,
            rotation: match page.get("rotation")?.as_str()? {
                "none" => PlotRotation::None,
                "counterClockwise90" => PlotRotation::CounterClockwise90,
                "upsideDown" => PlotRotation::UpsideDown,
                "clockwise90" => PlotRotation::Clockwise90,
                _ => return None,
            },
            device_name: page
                .get("deviceName")
                .and_then(Value::as_str)
                .map(str::to_owned),
            media_name: page
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

fn layout_media(value: &Value) -> Option<crate::plot_kernel::LayoutMedia> {
    Some(crate::plot_kernel::LayoutMedia {
        unit: crate::plot_kernel::MediaUnit::from_token(value.get("unit")?.as_str()?)?,
        width: value.get("width")?.as_f64()?,
        height: value.get("height")?.as_f64()?,
    })
}

use serde_json::json;
pub(crate) fn encode_rect(rect: LayoutRect) -> Option<Value> {
    rect.is_valid()
        .then(|| json!({"minX":rect.min_x,"minY":rect.min_y,"maxX":rect.max_x,"maxY":rect.max_y}))
}

fn encode_plot_settings(plot: &PlotSettings) -> Option<Value> {
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

fn encode_shading(shading: ShadedPlot) -> Value {
    let mut quality = json!({"mode":match shading.quality.mode {ShadedPlotQualityMode::Draft=>"Draft",ShadedPlotQualityMode::Preview=>"Preview",ShadedPlotQualityMode::Normal=>"Normal",ShadedPlotQualityMode::Presentation=>"Presentation",ShadedPlotQualityMode::Maximum=>"Maximum",ShadedPlotQualityMode::Custom=>"Custom"}});
    if let Some(dpi) = shading.quality.dpi {
        quality["dpi"] = json!(dpi);
    }
    json!({"mode":match shading.mode {ShadedPlotMode::AsDisplayed=>"AsDisplayed",ShadedPlotMode::Wireframe=>"Wireframe",ShadedPlotMode::Hidden=>"Hidden",ShadedPlotMode::Rendered=>"Rendered"},"quality":quality})
}

fn keys(v: &Value, allowed: &[&str]) -> bool {
    v.as_object()
        .is_some_and(|m| m.keys().all(|k| allowed.contains(&k.as_str())))
}
fn plot_keys(v: &Value) -> bool {
    if [&v["page"], &v["output"]].into_iter().any(|o| {
        ["deviceName", "mediaName", "plotStyleTableName"]
            .into_iter()
            .any(|key| o.get(key).is_some_and(|name| name.as_str().is_none()))
    }) {
        return false;
    }
    let only_mode = |o: &Value| keys(o, &["mode"]);
    if v["area"]["mode"] != "Window" && !only_mode(&v["area"]) {
        return false;
    }
    if v["mapping"]["scale"]["mode"] == "FitToArea" && !only_mode(&v["mapping"]["scale"]) {
        return false;
    }
    if v["mapping"]["placement"]["mode"] == "Centered" && !only_mode(&v["mapping"]["placement"]) {
        return false;
    }
    if v["output"]["shadedPlot"]["quality"]["mode"] != "Custom"
        && !only_mode(&v["output"]["shadedPlot"]["quality"])
    {
        return false;
    }
    keys(
        v,
        &["plotUnit", "page", "area", "mapping", "output", "options"],
    ) && keys(
        &v["page"],
        &["printableArea", "rotation", "deviceName", "mediaName"],
    ) && keys(
        &v["page"]["printableArea"],
        &["minX", "minY", "maxX", "maxY"],
    ) && keys(&v["area"], &["mode", "window"])
        && v["area"]
            .get("window")
            .is_none_or(|r| keys(r, &["minX", "minY", "maxX", "maxY"]))
        && keys(&v["mapping"], &["scale", "placement"])
        && keys(
            &v["mapping"]["scale"],
            &["mode", "outputLength", "scopeLength"],
        )
        && keys(&v["mapping"]["placement"], &["mode", "reference", "x", "y"])
        && keys(
            &v["output"],
            &["shadedPlot", "applyPlotStyles", "plotStyleTableName"],
        )
        && keys(&v["output"]["shadedPlot"], &["mode", "quality"])
        && keys(&v["output"]["shadedPlot"]["quality"], &["mode", "dpi"])
        && keys(
            &v["options"],
            &[
                "plotViewportBorders",
                "plotPaperSpaceLast",
                "hidePaperSpaceObjects",
                "plotLineWeights",
                "scaleLineWeights",
                "plotTransparency",
            ],
        )
}
pub(super) fn decode_output(v: &Value) -> Option<crate::ifccad::IfccadLayoutSettings> {
    let mut out = crate::ifccad::IfccadLayoutSettings::default();
    if let Some(media) = v.get("media") {
        if !keys(media, &["unit", "width", "height"]) {
            return None;
        }
        out.media = Some(layout_media(media)?);
    }
    if let Some(limits) = v.get("limits") {
        if !keys(limits, &["minX", "minY", "maxX", "maxY"]) {
            return None;
        }
        out.limits = Some(rectangle(limits)?);
    }
    if let Some(b) = v.get("limitsChecking") {
        out.limits_checking = b.as_bool()?;
    }
    if let Some(b) = v.get("paperSpaceLinetypeScaling") {
        out.paper_space_linetype_scaling = b.as_bool()?;
    }
    if let Some(plot) = v.get("plotSettings") {
        if !plot_keys(plot) {
            return None;
        }
        out.plot_settings = Some(plot_settings(plot)?);
    }
    Some(out)
}
pub(super) fn with_output(mut v: Value, s: &crate::ifccad::IfccadLayoutSettings) -> Value {
    if let Some(m) = &s.media {
        v["media"] = json!({"unit":m.unit.as_str(),"width":m.width,"height":m.height});
    }
    if let Some(l) = s.limits {
        v["limits"] = encode_rect(l).expect("validated limits");
    }
    if s.limits_checking {
        v["limitsChecking"] = json!(true);
    }
    if !s.paper_space_linetype_scaling {
        v["paperSpaceLinetypeScaling"] = json!(false);
    }
    if let Some(p) = &s.plot_settings {
        v["plotSettings"] = encode_plot_settings(p).expect("validated plot");
    }
    v
}
