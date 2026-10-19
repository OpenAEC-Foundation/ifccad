use super::*;
use crate::dimension::{
    DimensionAlternateChoice, DimensionAlternateNumberSettings, DimensionAngleFormat,
    DimensionArcSymbol, DimensionBackground, DimensionCenterMark, DimensionDistanceFormat,
    DimensionExtensionLength, DimensionExtensionLines, DimensionExtensionSettings,
    DimensionFactorApplication, DimensionFitPlacement, DimensionLineSpacing, DimensionMarker,
    DimensionPadding, DimensionTextMovement, DimensionTextOrientation, DimensionZeroSuppression,
};
record!(DimensionExtensionLines<IfccadColor,IfccadLinePatternId>;
0 => first: DimensionExtensionSettings<IfccadColor,IfccadLinePatternId> = None,
1 => second: DimensionExtensionSettings<IfccadColor,IfccadLinePatternId> = None,
);
impl CborValue for DimensionMarker<u64> {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "ClosedFilled"),
            (1, "ClosedOpen"),
            (2, "Open"),
            (3, "Dot"),
            (4, "Tick"),
            (5, "None"),
            (6, "LocalBlock"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::ClosedFilled => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::ClosedOpen => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(1)?;
                Ok(())
            }
            Self::Open => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(2)?;
                Ok(())
            }
            Self::Dot => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(3)?;
                Ok(())
            }
            Self::Tick => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(4)?;
                Ok(())
            }
            Self::None => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(5)?;
                Ok(())
            }
            Self::LocalBlock { block } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(6)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if block.present()
                    && (None as Option<u64>)
                        .as_ref()
                        .is_none_or(|d| !block.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if block.present()
                    && (None as Option<u64>)
                        .as_ref()
                        .is_none_or(|d| !block.exact(d))
                {
                    w.u64(0)?;
                    block.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => Self::ClosedFilled,
            1 => Self::ClosedOpen,
            2 => Self::Open,
            3 => Self::Dot,
            4 => Self::Tick,
            5 => Self::None,
            6 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "block")])?;
                let block: u64 = b.take(0, None)?;
                b.finish()?;
                Self::LocalBlock { block }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::ClosedFilled, Self::ClosedFilled) => true,
            (Self::ClosedOpen, Self::ClosedOpen) => true,
            (Self::Open, Self::Open) => true,
            (Self::Dot, Self::Dot) => true,
            (Self::Tick, Self::Tick) => true,
            (Self::None, Self::None) => true,
            (Self::LocalBlock { block }, Self::LocalBlock { block: block_other }) => {
                block.exact(block_other)
            }
            _ => false,
        }
    }
}
impl CborValue for DimensionCenterMark {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "None"), (1, "Mark"), (2, "Lines")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::None => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::Mark { size } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if size.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !size.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if size.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !size.exact(d))
                {
                    w.u64(0)?;
                    size.encode(w)?;
                }
                Ok(())
            }
            Self::Lines { size } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if size.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !size.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if size.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !size.exact(d))
                {
                    w.u64(0)?;
                    size.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => Self::None,
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "size")])?;
                let size: f64 = b.take(0, None)?;
                b.finish()?;
                Self::Mark { size }
            }
            2 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "size")])?;
                let size: f64 = b.take(0, None)?;
                b.finish()?;
                Self::Lines { size }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::None, Self::None) => true,
            (Self::Mark { size }, Self::Mark { size: size_other }) => size.exact(size_other),
            (Self::Lines { size }, Self::Lines { size: size_other }) => size.exact(size_other),
            _ => false,
        }
    }
}
unit_enum!(DimensionTextOrientation;
0 => Aligned,
1 => Horizontal,
);
unit_enum!(DimensionTextMovement;
0 => MoveDimensionLine,
1 => AddLeader,
2 => Free,
);
impl CborValue for DimensionLineSpacing {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "AtLeast"), (1, "Exact")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::AtLeast { factor } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if factor.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !factor.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if factor.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !factor.exact(d))
                {
                    w.u64(0)?;
                    factor.encode(w)?;
                }
                Ok(())
            }
            Self::Exact { factor } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if factor.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !factor.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if factor.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !factor.exact(d))
                {
                    w.u64(0)?;
                    factor.encode(w)?;
                }
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
                let mut b = RecordFields::new(&body, &[(0, "factor")])?;
                let factor: f64 = b.take(0, None)?;
                b.finish()?;
                Self::AtLeast { factor }
            }
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "factor")])?;
                let factor: f64 = b.take(0, None)?;
                b.finish()?;
                Self::Exact { factor }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (
                Self::AtLeast { factor },
                Self::AtLeast {
                    factor: factor_other,
                },
            ) => factor.exact(factor_other),
            (
                Self::Exact { factor },
                Self::Exact {
                    factor: factor_other,
                },
            ) => factor.exact(factor_other),
            _ => false,
        }
    }
}
impl CborValue for DimensionBackground<IfccadColor> {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "None"), (1, "Color"), (2, "Canvas")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::None => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::Color {
                color,
                opacity,
                padding,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if color.present()
                    && (None as Option<IfccadColor>)
                        .as_ref()
                        .is_none_or(|d| !color.exact(d))
                {
                    __cbor_fields += 1;
                }
                if opacity.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !opacity.exact(d))
                {
                    __cbor_fields += 1;
                }
                if padding.present()
                    && (None as Option<DimensionPadding>)
                        .as_ref()
                        .is_none_or(|d| !padding.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if color.present()
                    && (None as Option<IfccadColor>)
                        .as_ref()
                        .is_none_or(|d| !color.exact(d))
                {
                    w.u64(0)?;
                    color.encode(w)?;
                }
                if opacity.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !opacity.exact(d))
                {
                    w.u64(1)?;
                    opacity.encode(w)?;
                }
                if padding.present()
                    && (None as Option<DimensionPadding>)
                        .as_ref()
                        .is_none_or(|d| !padding.exact(d))
                {
                    w.u64(2)?;
                    padding.encode(w)?;
                }
                Ok(())
            }
            Self::Canvas { opacity, padding } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if opacity.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !opacity.exact(d))
                {
                    __cbor_fields += 1;
                }
                if padding.present()
                    && (None as Option<DimensionPadding>)
                        .as_ref()
                        .is_none_or(|d| !padding.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if opacity.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !opacity.exact(d))
                {
                    w.u64(0)?;
                    opacity.encode(w)?;
                }
                if padding.present()
                    && (None as Option<DimensionPadding>)
                        .as_ref()
                        .is_none_or(|d| !padding.exact(d))
                {
                    w.u64(1)?;
                    padding.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => Self::None,
            1 => {
                let body = f.span(1)?;
                let mut b =
                    RecordFields::new(&body, &[(0, "color"), (1, "opacity"), (2, "padding")])?;
                let color: IfccadColor = b.take(0, None)?;
                let opacity: f64 = b.take(1, None)?;
                let padding: DimensionPadding = b.take(2, None)?;
                b.finish()?;
                Self::Color {
                    color,
                    opacity,
                    padding,
                }
            }
            2 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "opacity"), (1, "padding")])?;
                let opacity: f64 = b.take(0, None)?;
                let padding: DimensionPadding = b.take(1, None)?;
                b.finish()?;
                Self::Canvas { opacity, padding }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::None, Self::None) => true,
            (
                Self::Color {
                    color,
                    opacity,
                    padding,
                },
                Self::Color {
                    color: color_other,
                    opacity: opacity_other,
                    padding: padding_other,
                },
            ) => {
                color.exact(color_other)
                    && opacity.exact(opacity_other)
                    && padding.exact(padding_other)
            }
            (
                Self::Canvas { opacity, padding },
                Self::Canvas {
                    opacity: opacity_other,
                    padding: padding_other,
                },
            ) => opacity.exact(opacity_other) && padding.exact(padding_other),
            _ => false,
        }
    }
}
unit_enum!(DimensionFitPlacement;
0 => TextInside,
1 => ArrowsInside,
2 => BestFit,
);
impl CborValue for DimensionExtensionLength {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Automatic"), (1, "Fixed")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Automatic => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::Fixed { length } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if length.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !length.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if length.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !length.exact(d))
                {
                    w.u64(0)?;
                    length.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => Self::Automatic,
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "length")])?;
                let length: f64 = b.take(0, None)?;
                b.finish()?;
                Self::Fixed { length }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::Automatic, Self::Automatic) => true,
            (
                Self::Fixed { length },
                Self::Fixed {
                    length: length_other,
                },
            ) => length.exact(length_other),
            _ => false,
        }
    }
}
impl CborValue for DimensionPadding {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Absolute"), (1, "Relative")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Absolute { distance } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if distance.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !distance.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if distance.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !distance.exact(d))
                {
                    w.u64(0)?;
                    distance.encode(w)?;
                }
                Ok(())
            }
            Self::Relative { factor } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if factor.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !factor.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if factor.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !factor.exact(d))
                {
                    w.u64(0)?;
                    factor.encode(w)?;
                }
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
                let mut b = RecordFields::new(&body, &[(0, "distance")])?;
                let distance: f64 = b.take(0, None)?;
                b.finish()?;
                Self::Absolute { distance }
            }
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "factor")])?;
                let factor: f64 = b.take(0, None)?;
                b.finish()?;
                Self::Relative { factor }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (
                Self::Absolute { distance },
                Self::Absolute {
                    distance: distance_other,
                },
            ) => distance.exact(distance_other),
            (
                Self::Relative { factor },
                Self::Relative {
                    factor: factor_other,
                },
            ) => factor.exact(factor_other),
            _ => false,
        }
    }
}
unit_enum!(DimensionFactorApplication;
0 => AllOwners,
1 => PaperOnly,
);
unit_enum!(DimensionDistanceFormat;
0 => Decimal,
1 => Fractional,
2 => Engineering,
3 => Architectural,
4 => Scientific,
);
unit_enum!(DimensionAngleFormat;
0 => DecimalDegrees,
1 => DegreesMinutesSeconds,
2 => Grads,
3 => Radians,
);
record!(DimensionZeroSuppression;
0 => leading: bool = None,
1 => trailing: bool = None,
2 => zero_feet: bool = None,
3 => zero_inches: bool = None,
);
unit_enum!(DimensionArcSymbol;
0 => None,
1 => Before,
2 => Above,
);
impl CborValue for DimensionAlternateChoice {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "None"), (1, "Enabled")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::None => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::Enabled(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<DimensionAlternateNumberSettings>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<DimensionAlternateNumberSettings>)
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
            0 => Self::None,
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: DimensionAlternateNumberSettings = b.take(0, None)?;
                b.finish()?;
                Self::Enabled(value_0)
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::None, Self::None) => true,
            (Self::Enabled(value_0), Self::Enabled(value_0_other)) => value_0.exact(value_0_other),
            _ => false,
        }
    }
}
