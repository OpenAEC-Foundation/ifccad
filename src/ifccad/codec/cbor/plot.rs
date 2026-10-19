use super::*;
use crate::geometry_kernel::CoordinateLengthUnit;
use crate::plot_kernel::{
    LayoutMedia, LayoutOutputSettings, MediaUnit, PlotArea, PlotMapping, PlotOffsetReference,
    PlotOptions, PlotOutput, PlotPage, PlotPlacement, PlotRect, PlotRotation, PlotScale,
    PlotSettings, PlotStyleMode, PlotUnit, ShadedPlot, ShadedPlotMode, ShadedPlotQuality,
    ShadedPlotQualityMode,
};
unit_enum!(PlotStyleMode;
0 => ColorDependent,
1 => Named,
);
record!(LayoutOutputSettings;
0 => media: Option<LayoutMedia> = Some(None),
1 => limits: Option<PlotRect> = Some(None),
2 => limits_checking: bool = None,
3 => paper_space_linetype_scaling: bool = None,
4 => plot_settings: Option<PlotSettings> = Some(None),
);
record!(LayoutMedia;
0 => unit: MediaUnit = None,
1 => width: f64 = None,
2 => height: f64 = None,
);
record!(PlotRect;
0 => min_x: f64 = None,
1 => min_y: f64 = None,
2 => max_x: f64 = None,
3 => max_y: f64 = None,
);
record!(PlotSettings;
0 => plot_unit: PlotUnit = None,
1 => page: PlotPage = None,
2 => area: PlotArea = None,
3 => mapping: PlotMapping = None,
4 => output: PlotOutput = None,
5 => options: PlotOptions = None,
);
impl CborValue for MediaUnit {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Physical"), (1, "Pixel")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Physical(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<CoordinateLengthUnit>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<CoordinateLengthUnit>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
            Self::Pixel => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(1)?;
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: CoordinateLengthUnit = b.take(0, None)?;
                b.finish()?;
                Self::Physical(value_0)
            }
            1 => Self::Pixel,
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::Physical(value_0), Self::Physical(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            (Self::Pixel, Self::Pixel) => true,
            _ => false,
        }
    }
}
unit_enum!(PlotUnit;
0 => Millimetre,
1 => Inch,
2 => Pixel,
);
record!(PlotPage;
0 => printable_area: PlotRect = None,
1 => rotation: PlotRotation = None,
2 => device_name: Option<String> = Some(None),
3 => media_name: Option<String> = Some(None),
);
impl CborValue for PlotArea {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Layout"), (1, "Extents"), (2, "Limits"), (3, "Window")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Layout => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::Extents => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(1)?;
                Ok(())
            }
            Self::Limits => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(2)?;
                Ok(())
            }
            Self::Window(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(3)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<PlotRect>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<PlotRect>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    w.u64(0)?;
                    value_0.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => Self::Layout,
            1 => Self::Extents,
            2 => Self::Limits,
            3 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: PlotRect = b.take(0, None)?;
                b.finish()?;
                Self::Window(value_0)
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::Layout, Self::Layout) => true,
            (Self::Extents, Self::Extents) => true,
            (Self::Limits, Self::Limits) => true,
            (Self::Window(value_0), Self::Window(value_0_other)) => value_0.exact(value_0_other),
            _ => false,
        }
    }
}
record!(PlotMapping;
0 => scale: PlotScale = None,
1 => placement: PlotPlacement = None,
);
record!(PlotOutput;
0 => shaded_plot: ShadedPlot = None,
1 => apply_plot_styles: bool = None,
2 => plot_style_table_name: Option<String> = Some(None),
);
record!(PlotOptions;
0 => plot_viewport_borders: bool = None,
1 => plot_paper_space_last: bool = None,
2 => hide_paper_space_objects: bool = None,
3 => plot_line_weights: bool = None,
4 => scale_line_weights: bool = None,
5 => plot_transparency: bool = None,
);
unit_enum!(PlotRotation;
0 => None,
1 => CounterClockwise90,
2 => UpsideDown,
3 => Clockwise90,
);
impl CborValue for PlotScale {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Fixed"), (1, "FitToArea")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Fixed {
                output_length,
                scope_length,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if output_length.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !output_length.exact(d))
                {
                    __cbor_fields += 1;
                }
                if scope_length.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !scope_length.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if output_length.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !output_length.exact(d))
                {
                    w.u64(0)?;
                    output_length.encode(w)?;
                }
                if scope_length.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !scope_length.exact(d))
                {
                    w.u64(1)?;
                    scope_length.encode(w)?;
                }
                Ok(())
            }
            Self::FitToArea => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(1)?;
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "output_length"), (1, "scope_length")])?;
                let output_length: f64 = b.take(0, None)?;
                let scope_length: f64 = b.take(1, None)?;
                b.finish()?;
                Self::Fixed {
                    output_length,
                    scope_length,
                }
            }
            1 => Self::FitToArea,
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (
                Self::Fixed {
                    output_length,
                    scope_length,
                },
                Self::Fixed {
                    output_length: output_length_other,
                    scope_length: scope_length_other,
                },
            ) => output_length.exact(output_length_other) && scope_length.exact(scope_length_other),
            (Self::FitToArea, Self::FitToArea) => true,
            _ => false,
        }
    }
}
impl CborValue for PlotPlacement {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Centered"), (1, "Offset")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Centered => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::Offset { reference, x, y } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if reference.present()
                    && (None as Option<PlotOffsetReference>)
                        .as_ref()
                        .is_none_or(|d| !reference.exact(d))
                {
                    __cbor_fields += 1;
                }
                if x.present() && (None as Option<f64>).as_ref().is_none_or(|d| !x.exact(d)) {
                    __cbor_fields += 1;
                }
                if y.present() && (None as Option<f64>).as_ref().is_none_or(|d| !y.exact(d)) {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if reference.present()
                    && (None as Option<PlotOffsetReference>)
                        .as_ref()
                        .is_none_or(|d| !reference.exact(d))
                {
                    w.u64(0)?;
                    reference.encode(w)?;
                }
                if x.present() && (None as Option<f64>).as_ref().is_none_or(|d| !x.exact(d)) {
                    w.u64(1)?;
                    x.encode(w)?;
                }
                if y.present() && (None as Option<f64>).as_ref().is_none_or(|d| !y.exact(d)) {
                    w.u64(2)?;
                    y.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => Self::Centered,
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "reference"), (1, "x"), (2, "y")])?;
                let reference: PlotOffsetReference = b.take(0, None)?;
                let x: f64 = b.take(1, None)?;
                let y: f64 = b.take(2, None)?;
                b.finish()?;
                Self::Offset { reference, x, y }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::Centered, Self::Centered) => true,
            (
                Self::Offset { reference, x, y },
                Self::Offset {
                    reference: reference_other,
                    x: x_other,
                    y: y_other,
                },
            ) => reference.exact(reference_other) && x.exact(x_other) && y.exact(y_other),
            _ => false,
        }
    }
}
record!(ShadedPlot;
0 => mode: ShadedPlotMode = None,
1 => quality: ShadedPlotQuality = None,
);
unit_enum!(PlotOffsetReference;
0 => Media,
1 => PrintableArea,
);
unit_enum!(ShadedPlotMode;
0 => AsDisplayed,
1 => Wireframe,
2 => Hidden,
3 => Rendered,
);
record!(ShadedPlotQuality;
0 => mode: ShadedPlotQualityMode = None,
1 => dpi: Option<u32> = Some(None),
);
unit_enum!(ShadedPlotQualityMode;
0 => Draft,
1 => Preview,
2 => Normal,
3 => Presentation,
4 => Maximum,
5 => Custom,
);
