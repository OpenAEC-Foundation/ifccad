//! Format-neutral effective layout output values.
use crate::geometry_kernel::CoordinateLengthUnit;

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
pub struct PlotPage {
    pub printable_area: PlotRect,
    pub rotation: PlotRotation,
    pub device_name: Option<String>,
    pub media_name: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlotArea {
    Layout,
    Extents,
    Limits,
    Window(PlotRect),
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
    pub plot_unit: PlotUnit,
    pub page: PlotPage,
    pub area: PlotArea,
    pub mapping: PlotMapping,
    pub output: PlotOutput,
    pub options: PlotOptions,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaUnit {
    Physical(CoordinateLengthUnit),
    Pixel,
}
impl MediaUnit {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Physical(u) => u.as_str(),
            Self::Pixel => "px",
        }
    }
    pub fn from_token(token: &str) -> Option<Self> {
        if token == "px" {
            Some(Self::Pixel)
        } else {
            CoordinateLengthUnit::from_token(token).map(Self::Physical)
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutMedia {
    pub unit: MediaUnit,
    pub width: f64,
    pub height: f64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlotRect {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}
impl PlotRect {
    pub(crate) fn is_valid(self) -> bool {
        [self.min_x, self.min_y, self.max_x, self.max_y]
            .into_iter()
            .all(f64::is_finite)
            && self.min_x <= self.max_x
            && self.min_y <= self.max_y
    }
    pub fn has_area(self) -> bool {
        self.is_valid() && self.min_x < self.max_x && self.min_y < self.max_y
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutOutputKind {
    Model,
    Paper,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PlotStyleMode {
    #[default]
    ColorDependent,
    Named,
}
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutOutputSettings {
    pub media: Option<LayoutMedia>,
    pub limits: Option<PlotRect>,
    pub limits_checking: bool,
    pub paper_space_linetype_scaling: bool,
    pub plot_settings: Option<PlotSettings>,
}
impl Default for LayoutOutputSettings {
    fn default() -> Self {
        Self {
            media: None,
            limits: None,
            limits_checking: false,
            paper_space_linetype_scaling: true,
            plot_settings: None,
        }
    }
}
