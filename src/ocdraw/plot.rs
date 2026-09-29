//! Effective plot settings embedded in a Layout.
use super::LayoutRect;
use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShadedPlotMode {
    AsDisplayed,
    Wireframe,
    Hidden,
    Rendered,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShadedPlotQualityMode {
    Draft,
    Preview,
    Normal,
    Presentation,
    Maximum,
    Custom,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadedPlotQuality {
    pub mode: ShadedPlotQualityMode,
    pub dpi: Option<u32>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadedPlot {
    pub mode: ShadedPlotMode,
    pub quality: ShadedPlotQuality,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlotUnit {
    Millimetre,
    Inch,
    Pixel,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlotRotation {
    None,
    CounterClockwise90,
    UpsideDown,
    Clockwise90,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PlotMedia {
    pub unit: PlotUnit,
    pub width: f64,
    pub height: f64,
    pub printable_area: LayoutRect,
    pub rotation: PlotRotation,
    pub device_name: Option<String>,
    pub media_name: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlotArea {
    Layout,
    Extents,
    Limits,
    Window(LayoutRect),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlotScale {
    Fixed {
        output_length: f64,
        scope_length: f64,
    },
    FitToArea,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlotPlacement {
    Centered,
    Offset {
        reference: PlotOffsetReference,
        x: f64,
        y: f64,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlotOffsetReference {
    Media,
    PrintableArea,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlotMapping {
    pub scale: PlotScale,
    pub placement: PlotPlacement,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PlotOutput {
    pub shaded_plot: ShadedPlot,
    pub apply_plot_styles: bool,
    pub plot_style_table_name: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlotOptions {
    pub plot_viewport_borders: bool,
    pub plot_paper_space_last: bool,
    pub hide_paper_space_objects: bool,
    pub plot_line_weights: bool,
    pub scale_line_weights: bool,
    pub plot_transparency: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PlotSettings {
    pub media: PlotMedia,
    pub area: PlotArea,
    pub mapping: PlotMapping,
    pub output: PlotOutput,
    pub options: PlotOptions,
}
impl PlotSettings {
    pub(crate) fn to_json(&self) -> Option<Value> {
        if !self.media.width.is_finite()
            || self.media.width <= 0.0
            || !self.media.height.is_finite()
            || self.media.height <= 0.0
            || self.media.printable_area.to_json().is_none()
            || self
                .media
                .device_name
                .as_ref()
                .is_some_and(String::is_empty)
            || self.media.media_name.as_ref().is_some_and(String::is_empty)
            || self
                .output
                .plot_style_table_name
                .as_ref()
                .is_some_and(String::is_empty)
        {
            return None;
        }
        if let PlotArea::Window(rect) = self.area {
            rect.to_json()?;
        }
        if let PlotScale::Fixed {
            output_length,
            scope_length,
        } = self.mapping.scale
        {
            if !output_length.is_finite()
                || output_length <= 0.0
                || !scope_length.is_finite()
                || scope_length <= 0.0
            {
                return None;
            }
        }
        if let PlotPlacement::Offset { x, y, .. } = self.mapping.placement {
            if !x.is_finite() || !y.is_finite() {
                return None;
            }
        }
        match (
            self.output.shaded_plot.quality.mode,
            self.output.shaded_plot.quality.dpi,
        ) {
            (ShadedPlotQualityMode::Custom, Some(100..=32767)) => {}
            (ShadedPlotQualityMode::Custom, _) => return None,
            (_, Some(_)) => return None,
            (_, None) => {}
        }
        let unit = match self.media.unit {
            PlotUnit::Millimetre => "mm",
            PlotUnit::Inch => "in",
            PlotUnit::Pixel => "px",
        };
        let rotation = match self.media.rotation {
            PlotRotation::None => "none",
            PlotRotation::CounterClockwise90 => "counterClockwise90",
            PlotRotation::UpsideDown => "upsideDown",
            PlotRotation::Clockwise90 => "clockwise90",
        };
        let mut media = json!({"unit":unit,"width":self.media.width,"height":self.media.height,"printableArea":self.media.printable_area.to_json().expect("valid layout rectangle"),"rotation":rotation});
        if let Some(name) = &self.media.device_name {
            media["deviceName"] = json!(name);
        }
        if let Some(name) = &self.media.media_name {
            media["mediaName"] = json!(name);
        }
        let area = match self.area {
            PlotArea::Layout => json!({"mode":"Layout"}),
            PlotArea::Extents => json!({"mode":"Extents"}),
            PlotArea::Limits => json!({"mode":"Limits"}),
            PlotArea::Window(rect) => {
                json!({"mode":"Window","window":rect.to_json().expect("valid layout rectangle")})
            }
        };
        let scale = match self.mapping.scale {
            PlotScale::Fixed {
                output_length,
                scope_length,
            } => json!({"mode":"Fixed","outputLength":output_length,"scopeLength":scope_length}),
            PlotScale::FitToArea => json!({"mode":"FitToArea"}),
        };
        let placement = match self.mapping.placement {
            PlotPlacement::Centered => json!({"mode":"Centered"}),
            PlotPlacement::Offset { reference, x, y } => {
                json!({"mode":"Offset","reference":match reference {PlotOffsetReference::Media=>"Media",PlotOffsetReference::PrintableArea=>"PrintableArea"},"x":x,"y":y})
            }
        };
        let shading = self.output.shaded_plot;
        let mut quality = json!({"mode":match shading.quality.mode {ShadedPlotQualityMode::Draft=>"Draft",ShadedPlotQualityMode::Preview=>"Preview",ShadedPlotQualityMode::Normal=>"Normal",ShadedPlotQualityMode::Presentation=>"Presentation",ShadedPlotQualityMode::Maximum=>"Maximum",ShadedPlotQualityMode::Custom=>"Custom"}});
        if let Some(dpi) = shading.quality.dpi {
            quality["dpi"] = json!(dpi);
        }
        let mut output = json!({"shadedPlot":{"mode":match shading.mode {ShadedPlotMode::AsDisplayed=>"AsDisplayed",ShadedPlotMode::Wireframe=>"Wireframe",ShadedPlotMode::Hidden=>"Hidden",ShadedPlotMode::Rendered=>"Rendered"},"quality":quality},"applyPlotStyles":self.output.apply_plot_styles});
        if let Some(name) = &self.output.plot_style_table_name {
            output["plotStyleTableName"] = json!(name);
        }
        let o = self.options;
        Some(
            json!({"media":media,"area":area,"mapping":{"scale":scale,"placement":placement},"output":output,
            "options":{"plotViewportBorders":o.plot_viewport_borders,"plotPaperSpaceLast":o.plot_paper_space_last,"hidePaperSpaceObjects":o.hide_paper_space_objects,"plotLineWeights":o.plot_line_weights,"scaleLineWeights":o.scale_line_weights,"plotTransparency":o.plot_transparency}}),
        )
    }
}
