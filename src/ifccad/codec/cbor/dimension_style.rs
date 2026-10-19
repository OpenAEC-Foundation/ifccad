use super::*;
use crate::dimension::{
    DimensionAlternateChoice, DimensionAlternateNumberSettings, DimensionAngleFormat,
    DimensionAppearanceChoice, DimensionArcSymbol, DimensionArrowOverrides, DimensionArrowSettings,
    DimensionBackground, DimensionCenterMark, DimensionComponentAppearance,
    DimensionComponentOverrides, DimensionDistanceFormat, DimensionExtensionLength,
    DimensionExtensionLines, DimensionExtensionLinesOverrides, DimensionExtensionOverrides,
    DimensionExtensionSettings, DimensionFactorApplication, DimensionFitOverrides,
    DimensionFitPlacement, DimensionFitSettings, DimensionFractionLayout, DimensionKind,
    DimensionLineOverrides, DimensionLineSettings, DimensionLineSpacing, DimensionMarker,
    DimensionNumberOverrides, DimensionNumberSettings, DimensionPrimaryNumberOverrides,
    DimensionPrimaryNumberSettings, DimensionStyleOverrides, DimensionStyleSettings,
    DimensionTextMovement, DimensionTextOrientation, DimensionTextOverrides, DimensionTextSettings,
    DimensionToleranceSettings, DimensionZeroSuppression,
};
use crate::text::MTextAttachment;
use std::collections::BTreeMap;
record!(IfccadDimensionStyle;
0 => id: IfccadDimensionStyleId = None,
1 => name: String = None,
2 => settings: DimensionStyleSettings<IfccadTextStyleId,u64,IfccadColor,IfccadLinePatternId> = None,
3 => type_overrides: BTreeMap<DimensionKind,IfccadDimensionOverrides> = None,
);
impl CborValue for IfccadDimensionStyleId {
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        self.0.encode(w)
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        Ok(Self(CborValue::decode(r)?))
    }
    fn exact(&self, o: &Self) -> bool {
        self.0.exact(&o.0)
    }
}
record!(DimensionStyleSettings<IfccadTextStyleId,u64,IfccadColor,IfccadLinePatternId>;
0 => dimension_line: DimensionLineSettings<IfccadColor,IfccadLinePatternId> = None,
1 => extension_lines: DimensionExtensionLines<IfccadColor,IfccadLinePatternId> = None,
2 => arrows: DimensionArrowSettings<u64> = None,
3 => text: DimensionTextSettings<IfccadTextStyleId,IfccadColor,IfccadLinePatternId> = None,
4 => fit: DimensionFitSettings = None,
5 => numbers: DimensionNumberSettings = None,
6 => presentation_scale: f64 = None,
);
record!(DimensionLineSettings<IfccadColor,IfccadLinePatternId>;
0 => appearance: DimensionComponentAppearance<IfccadColor,IfccadLinePatternId> = None,
1 => extension: f64 = None,
2 => suppress_first: bool = None,
3 => suppress_second: bool = None,
);
record!(DimensionArrowSettings<u64>;
0 => first: DimensionMarker<u64> = None,
1 => second: DimensionMarker<u64> = None,
2 => size: f64 = None,
3 => tick_size: f64 = None,
4 => flip_first: bool = None,
5 => flip_second: bool = None,
6 => center_mark: DimensionCenterMark = None,
);
record!(DimensionTextSettings<IfccadTextStyleId,IfccadColor,IfccadLinePatternId>;
0 => appearance: DimensionComponentAppearance<IfccadColor,IfccadLinePatternId> = None,
1 => style: IfccadTextStyleId = None,
2 => height: f64 = None,
3 => attachment: MTextAttachment = None,
4 => inside_orientation: DimensionTextOrientation = None,
5 => outside_orientation: DimensionTextOrientation = None,
6 => gap: f64 = None,
7 => movement: DimensionTextMovement = None,
8 => line_spacing: DimensionLineSpacing = None,
9 => background: DimensionBackground<IfccadColor> = None,
);
record!(DimensionFitSettings;
0 => preferred_placement: DimensionFitPlacement = None,
1 => force_text_inside: bool = None,
2 => keep_dimension_line_outside: bool = None,
);
record!(DimensionNumberSettings;
0 => primary: DimensionPrimaryNumberSettings = None,
1 => alternate: Option<DimensionAlternateNumberSettings> = Some(None),
2 => tolerance: DimensionToleranceSettings = None,
);
record!(DimensionStyleOverrides<IfccadTextStyleId,u64,IfccadColor,IfccadLinePatternId>;
0 => dimension_line: DimensionLineOverrides<IfccadColor,IfccadLinePatternId> = None,
1 => extension_lines: DimensionExtensionLinesOverrides<IfccadColor,IfccadLinePatternId> = None,
2 => arrows: DimensionArrowOverrides<u64> = None,
3 => text: DimensionTextOverrides<IfccadTextStyleId,IfccadColor,IfccadLinePatternId> = None,
4 => fit: DimensionFitOverrides = None,
5 => numbers: DimensionNumberOverrides = None,
6 => presentation_scale: Option<f64> = Some(None),
);
record!(DimensionComponentAppearance<IfccadColor,IfccadLinePatternId>;
0 => color: DimensionAppearanceChoice<IfccadColor> = None,
1 => opacity: DimensionAppearanceChoice<f64> = None,
2 => line_pattern: DimensionAppearanceChoice<IfccadLinePatternId> = None,
3 => line_weight: DimensionAppearanceChoice<f64> = None,
);
record!(DimensionExtensionSettings<IfccadColor,IfccadLinePatternId>;
0 => appearance: DimensionComponentAppearance<IfccadColor,IfccadLinePatternId> = None,
1 => offset: f64 = None,
2 => extension: f64 = None,
3 => suppressed: bool = None,
4 => length: DimensionExtensionLength = None,
);
record!(DimensionPrimaryNumberSettings;
0 => measurement_factor: f64 = None,
1 => application: DimensionFactorApplication = None,
2 => format: DimensionDistanceFormat = None,
3 => angle_format: DimensionAngleFormat = None,
4 => precision: u8 = None,
5 => decimal_separator: char = None,
6 => rounding_increment: f64 = None,
7 => zero_suppression: DimensionZeroSuppression = None,
8 => prefix: String = None,
9 => suffix: String = None,
10 => arc_symbol: DimensionArcSymbol = None,
11 => fraction_layout: DimensionFractionLayout = Some(DimensionFractionLayout::Inline),
12 => fraction_scale: f64 = Some(1.0),
);
impl CborValue for DimensionToleranceSettings {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "None"), (1, "Deviations"), (2, "Limits")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::None => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::Deviations {
                upper,
                lower,
                precision,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if upper.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !upper.exact(d))
                {
                    __cbor_fields += 1;
                }
                if lower.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !lower.exact(d))
                {
                    __cbor_fields += 1;
                }
                if precision.present()
                    && (None as Option<u8>)
                        .as_ref()
                        .is_none_or(|d| !precision.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if upper.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !upper.exact(d))
                {
                    w.u64(0)?;
                    upper.encode(w)?;
                }
                if lower.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !lower.exact(d))
                {
                    w.u64(1)?;
                    lower.encode(w)?;
                }
                if precision.present()
                    && (None as Option<u8>)
                        .as_ref()
                        .is_none_or(|d| !precision.exact(d))
                {
                    w.u64(2)?;
                    precision.encode(w)?;
                }
                Ok(())
            }
            Self::Limits {
                upper,
                lower,
                precision,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if upper.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !upper.exact(d))
                {
                    __cbor_fields += 1;
                }
                if lower.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !lower.exact(d))
                {
                    __cbor_fields += 1;
                }
                if precision.present()
                    && (None as Option<u8>)
                        .as_ref()
                        .is_none_or(|d| !precision.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if upper.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !upper.exact(d))
                {
                    w.u64(0)?;
                    upper.encode(w)?;
                }
                if lower.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !lower.exact(d))
                {
                    w.u64(1)?;
                    lower.encode(w)?;
                }
                if precision.present()
                    && (None as Option<u8>)
                        .as_ref()
                        .is_none_or(|d| !precision.exact(d))
                {
                    w.u64(2)?;
                    precision.encode(w)?;
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
                    RecordFields::new(&body, &[(0, "upper"), (1, "lower"), (2, "precision")])?;
                let upper: f64 = b.take(0, None)?;
                let lower: f64 = b.take(1, None)?;
                let precision: u8 = b.take(2, None)?;
                b.finish()?;
                Self::Deviations {
                    upper,
                    lower,
                    precision,
                }
            }
            2 => {
                let body = f.span(1)?;
                let mut b =
                    RecordFields::new(&body, &[(0, "upper"), (1, "lower"), (2, "precision")])?;
                let upper: f64 = b.take(0, None)?;
                let lower: f64 = b.take(1, None)?;
                let precision: u8 = b.take(2, None)?;
                b.finish()?;
                Self::Limits {
                    upper,
                    lower,
                    precision,
                }
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
                Self::Deviations {
                    upper,
                    lower,
                    precision,
                },
                Self::Deviations {
                    upper: upper_other,
                    lower: lower_other,
                    precision: precision_other,
                },
            ) => {
                upper.exact(upper_other)
                    && lower.exact(lower_other)
                    && precision.exact(precision_other)
            }
            (
                Self::Limits {
                    upper,
                    lower,
                    precision,
                },
                Self::Limits {
                    upper: upper_other,
                    lower: lower_other,
                    precision: precision_other,
                },
            ) => {
                upper.exact(upper_other)
                    && lower.exact(lower_other)
                    && precision.exact(precision_other)
            }
            _ => false,
        }
    }
}
record!(DimensionLineOverrides<IfccadColor,IfccadLinePatternId>;
0 => appearance: DimensionComponentOverrides<IfccadColor,IfccadLinePatternId> = None,
1 => extension: Option<f64> = Some(None),
2 => suppress_first: Option<bool> = Some(None),
3 => suppress_second: Option<bool> = Some(None),
);
record!(DimensionExtensionLinesOverrides<IfccadColor,IfccadLinePatternId>;
0 => first: DimensionExtensionOverrides<IfccadColor,IfccadLinePatternId> = None,
1 => second: DimensionExtensionOverrides<IfccadColor,IfccadLinePatternId> = None,
);
record!(DimensionArrowOverrides<u64>;
0 => first: Option<DimensionMarker<u64>> = Some(None),
1 => second: Option<DimensionMarker<u64>> = Some(None),
2 => size: Option<f64> = Some(None),
3 => tick_size: Option<f64> = Some(None),
4 => flip_first: Option<bool> = Some(None),
5 => flip_second: Option<bool> = Some(None),
6 => center_mark: Option<DimensionCenterMark> = Some(None),
);
record!(DimensionTextOverrides<IfccadTextStyleId,IfccadColor,IfccadLinePatternId>;
0 => appearance: DimensionComponentOverrides<IfccadColor,IfccadLinePatternId> = None,
1 => style: Option<IfccadTextStyleId> = Some(None),
2 => height: Option<f64> = Some(None),
3 => attachment: Option<MTextAttachment> = Some(None),
4 => inside_orientation: Option<DimensionTextOrientation> = Some(None),
5 => outside_orientation: Option<DimensionTextOrientation> = Some(None),
6 => gap: Option<f64> = Some(None),
7 => movement: Option<DimensionTextMovement> = Some(None),
8 => line_spacing: Option<DimensionLineSpacing> = Some(None),
9 => background: Option<DimensionBackground<IfccadColor>> = Some(None),
);
record!(DimensionFitOverrides;
0 => preferred_placement: Option<DimensionFitPlacement> = Some(None),
1 => force_text_inside: Option<bool> = Some(None),
2 => keep_dimension_line_outside: Option<bool> = Some(None),
);
record!(DimensionNumberOverrides;
0 => primary: DimensionPrimaryNumberOverrides = None,
1 => alternate: Option<DimensionAlternateChoice> = Some(None),
2 => tolerance: Option<DimensionToleranceSettings> = Some(None),
);
impl CborValue for DimensionAppearanceChoice<IfccadColor> {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "InheritEntity"),
            (1, "ByLayer"),
            (2, "ByBlock"),
            (3, "Explicit"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::InheritEntity => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::ByLayer => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(1)?;
                Ok(())
            }
            Self::ByBlock => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(2)?;
                Ok(())
            }
            Self::Explicit(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(3)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<IfccadColor>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<IfccadColor>)
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
            0 => Self::InheritEntity,
            1 => Self::ByLayer,
            2 => Self::ByBlock,
            3 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: IfccadColor = b.take(0, None)?;
                b.finish()?;
                Self::Explicit(value_0)
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::InheritEntity, Self::InheritEntity) => true,
            (Self::ByLayer, Self::ByLayer) => true,
            (Self::ByBlock, Self::ByBlock) => true,
            (Self::Explicit(value_0), Self::Explicit(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            _ => false,
        }
    }
}
impl CborValue for DimensionAppearanceChoice<f64> {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "InheritEntity"),
            (1, "ByLayer"),
            (2, "ByBlock"),
            (3, "Explicit"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::InheritEntity => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::ByLayer => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(1)?;
                Ok(())
            }
            Self::ByBlock => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(2)?;
                Ok(())
            }
            Self::Explicit(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(3)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<f64>)
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
            0 => Self::InheritEntity,
            1 => Self::ByLayer,
            2 => Self::ByBlock,
            3 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: f64 = b.take(0, None)?;
                b.finish()?;
                Self::Explicit(value_0)
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::InheritEntity, Self::InheritEntity) => true,
            (Self::ByLayer, Self::ByLayer) => true,
            (Self::ByBlock, Self::ByBlock) => true,
            (Self::Explicit(value_0), Self::Explicit(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            _ => false,
        }
    }
}
impl CborValue for DimensionAppearanceChoice<IfccadLinePatternId> {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "InheritEntity"),
            (1, "ByLayer"),
            (2, "ByBlock"),
            (3, "Explicit"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::InheritEntity => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::ByLayer => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(1)?;
                Ok(())
            }
            Self::ByBlock => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(2)?;
                Ok(())
            }
            Self::Explicit(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(3)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if value_0.present()
                    && (None as Option<IfccadLinePatternId>)
                        .as_ref()
                        .is_none_or(|d| !value_0.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if value_0.present()
                    && (None as Option<IfccadLinePatternId>)
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
            0 => Self::InheritEntity,
            1 => Self::ByLayer,
            2 => Self::ByBlock,
            3 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: IfccadLinePatternId = b.take(0, None)?;
                b.finish()?;
                Self::Explicit(value_0)
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::InheritEntity, Self::InheritEntity) => true,
            (Self::ByLayer, Self::ByLayer) => true,
            (Self::ByBlock, Self::ByBlock) => true,
            (Self::Explicit(value_0), Self::Explicit(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            _ => false,
        }
    }
}
record!(DimensionAlternateNumberSettings;
0 => factor: f64 = None,
1 => format: DimensionDistanceFormat = None,
2 => precision: u8 = None,
3 => decimal_separator: char = None,
4 => rounding_increment: f64 = None,
5 => zero_suppression: DimensionZeroSuppression = None,
6 => prefix: String = None,
7 => suffix: String = None,
8 => fraction_layout: DimensionFractionLayout = Some(DimensionFractionLayout::Inline),
9 => fraction_scale: f64 = Some(1.0),
);
record!(DimensionComponentOverrides<IfccadColor,IfccadLinePatternId>;
0 => color: Option<DimensionAppearanceChoice<IfccadColor>> = Some(None),
1 => opacity: Option<DimensionAppearanceChoice<f64>> = Some(None),
2 => line_pattern: Option<DimensionAppearanceChoice<IfccadLinePatternId>> = Some(None),
3 => line_weight: Option<DimensionAppearanceChoice<f64>> = Some(None),
);
record!(DimensionExtensionOverrides<IfccadColor,IfccadLinePatternId>;
0 => appearance: DimensionComponentOverrides<IfccadColor,IfccadLinePatternId> = None,
1 => offset: Option<f64> = Some(None),
2 => extension: Option<f64> = Some(None),
3 => suppressed: Option<bool> = Some(None),
4 => length: Option<DimensionExtensionLength> = Some(None),
);
record!(DimensionPrimaryNumberOverrides;
0 => measurement_factor: Option<f64> = Some(None),
1 => application: Option<DimensionFactorApplication> = Some(None),
2 => format: Option<DimensionDistanceFormat> = Some(None),
3 => angle_format: Option<DimensionAngleFormat> = Some(None),
4 => precision: Option<u8> = Some(None),
5 => decimal_separator: Option<char> = Some(None),
6 => rounding_increment: Option<f64> = Some(None),
7 => zero_suppression: Option<DimensionZeroSuppression> = Some(None),
8 => prefix: Option<String> = Some(None),
9 => suffix: Option<String> = Some(None),
10 => arc_symbol: Option<DimensionArcSymbol> = Some(None),
11 => fraction_layout: Option<DimensionFractionLayout> = Some(None),
12 => fraction_scale: Option<f64> = Some(None),
);

unit_enum!(DimensionFractionLayout;
0 => Inline,
1 => Horizontal,
2 => Diagonal,
);
