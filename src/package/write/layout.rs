use crate::ifcdr::ShadedPlot;
use serde_json::{json, Value};

/// Rectangle in the selected layout's own coordinate domain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlotRect {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}
impl PlotRect {
    pub(crate) fn json(self) -> Value {
        json!({"minX":self.min_x,"minY":self.min_y,"maxX":self.max_x,"maxY":self.max_y})
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlotStyleMode {
    ColorDependent,
    Named,
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
    pub media: PlotMedia,
    pub area: PlotArea,
    pub mapping: PlotMapping,
    pub output: PlotOutput,
    pub options: PlotOptions,
}
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutSettings {
    pub limits: Option<PlotRect>,
    pub limits_checking: bool,
    pub paper_space_linetype_scaling: bool,
    pub plot_settings: Option<PlotSettings>,
}
impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
            limits: None,
            limits_checking: false,
            paper_space_linetype_scaling: true,
            plot_settings: None,
        }
    }
}
impl LayoutSettings {
    pub(crate) fn json(&self, name: &str, kind: &str, scope_id: u32) -> Value {
        let mut result = json!({"name":name,"kind":kind,"scopeId":scope_id,
            "limitsChecking":self.limits_checking,"paperSpaceLinetypeScaling":self.paper_space_linetype_scaling});
        if let Some(rect) = self.limits {
            result["limits"] = rect.json();
        }
        if let Some(plot) = &self.plot_settings {
            result["plotSettings"] = plot.json();
        }
        result
    }
}
impl PlotSettings {
    fn json(&self) -> Value {
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
        let mut media = json!({"unit":unit,"width":self.media.width,"height":self.media.height,"printableArea":self.media.printable_area.json(),"rotation":rotation});
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
            PlotArea::Window(rect) => json!({"mode":"Window","window":rect.json()}),
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
        let mut quality = json!({"mode":match shading.quality.mode {crate::ifcdr::ShadedPlotQualityMode::Draft=>"Draft",crate::ifcdr::ShadedPlotQualityMode::Preview=>"Preview",crate::ifcdr::ShadedPlotQualityMode::Normal=>"Normal",crate::ifcdr::ShadedPlotQualityMode::Presentation=>"Presentation",crate::ifcdr::ShadedPlotQualityMode::Maximum=>"Maximum",crate::ifcdr::ShadedPlotQualityMode::Custom=>"Custom"}});
        if let Some(dpi) = shading.quality.dpi {
            quality["dpi"] = json!(dpi);
        }
        let mut output = json!({"shadedPlot":{"mode":match shading.mode {crate::ifcdr::ShadedPlotMode::AsDisplayed=>"AsDisplayed",crate::ifcdr::ShadedPlotMode::Wireframe=>"Wireframe",crate::ifcdr::ShadedPlotMode::Hidden=>"Hidden",crate::ifcdr::ShadedPlotMode::Rendered=>"Rendered"},"quality":quality},"applyPlotStyles":self.output.apply_plot_styles});
        if let Some(name) = &self.output.plot_style_table_name {
            output["plotStyleTableName"] = json!(name);
        }
        let o = self.options;
        json!({"media":media,"area":area,"mapping":{"scale":scale,"placement":placement},"output":output,
            "options":{"plotViewportBorders":o.plot_viewport_borders,"plotPaperSpaceLast":o.plot_paper_space_last,"hidePaperSpaceObjects":o.hide_paper_space_objects,"plotLineWeights":o.plot_line_weights,"scaleLineWeights":o.scale_line_weights,"plotTransparency":o.plot_transparency}})
    }
}
