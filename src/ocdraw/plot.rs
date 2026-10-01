//! Effective plot settings embedded in a Layout.
use super::LayoutRect;

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
    pub(crate) fn is_valid(&self) -> bool {
        if !self.media.width.is_finite()
            || self.media.width <= 0.0
            || !self.media.height.is_finite()
            || self.media.height <= 0.0
            || !self.media.printable_area.is_valid()
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
            return false;
        }
        if let PlotArea::Window(rect) = self.area {
            if !rect.is_valid() {
                return false;
            }
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
                return false;
            }
        }
        if let PlotPlacement::Offset { x, y, .. } = self.mapping.placement {
            if !x.is_finite() || !y.is_finite() {
                return false;
            }
        }
        match (
            self.output.shaded_plot.quality.mode,
            self.output.shaded_plot.quality.dpi,
        ) {
            (ShadedPlotQualityMode::Custom, Some(100..=32767)) => {}
            (ShadedPlotQualityMode::Custom, _) => return false,
            (_, Some(_)) => return false,
            (_, None) => {}
        }
        true
    }
}
