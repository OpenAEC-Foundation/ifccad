use super::*;
use crate::geometry_kernel::CoordinateLengthUnit;
record!(IfccadHeader;
0 => id: String = None,
1 => data_version: String = None,
2 => author: String = None,
3 => timestamp: String = None,
);
record!(IfccadIdCounters;
0 => next_entity_id: u64 = None,
1 => next_layer_id: u64 = None,
2 => next_layout_id: u64 = None,
3 => next_block_id: u64 = None,
4 => next_line_pattern_id: u64 = None,
5 => next_text_style_id: u64 = None,
6 => next_symbol_source_id: u64 = None,
7 => next_preservation_record_id: u64 = None,
8 => next_ucs_id: u64 = None,
9 => next_model_window_id: u64 = None,
10 => next_dimension_style_id: u64 = None,
);
record!(IfccadLayer;
0 => id: u64 = None,
1 => name: String = None,
2 => description: Option<String> = Some(None),
3 => visible: bool = Some(true),
4 => frozen: bool = Some(false),
5 => locked: bool = Some(false),
6 => plottable: bool = Some(true),
7 => frozen_in_new_viewports: bool = Some(false),
8 => appearance: IfccadLayerAppearance = None,
);
record!(IfccadLinePattern;
0 => id: IfccadLinePatternId = None,
1 => name: String = None,
2 => description: Option<String> = Some(None),
3 => pattern: Vec<IfccadLinePatternElement> = None,
);
record!(IfccadPointDisplay;
0 => glyph: IfccadPointGlyph = None,
1 => circle: bool = None,
2 => square: bool = None,
3 => size: IfccadPointSize = None,
);
unit_enum!(IfccadBoundsQuality;
0 => Enclosing,
1 => Estimated,
2 => Partial,
);
record!(IfccadBounds3d;
0 => min: [f64;3] = None,
1 => max: [f64;3] = None,
);
impl CborValue for IfccadEntity {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Native"), (1, "Opaque")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Native(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                value_0.encode(w)?;
                Ok(())
            }
            Self::Opaque(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                value_0.encode(w)?;
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => {
                let mut body = f.span(1)?;
                let value_0 = CborValue::decode(&mut body)?;
                Self::Native(value_0)
            }
            1 => {
                let mut body = f.span(1)?;
                let value_0 = CborValue::decode(&mut body)?;
                Self::Opaque(value_0)
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::Native(value_0), Self::Native(value_0_other)) => value_0.exact(value_0_other),
            (Self::Opaque(value_0), Self::Opaque(value_0_other)) => value_0.exact(value_0_other),
            _ => false,
        }
    }
}
record!(IfccadLayerAppearance;
0 => color: IfccadColor = None,
1 => opacity: f64 = Some(1.0),
2 => line_pattern: IfccadLinePatternId = None,
3 => line_weight: f64 = None,
);
impl CborValue for IfccadLinePatternId {
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
unit_enum!(IfccadPointGlyph;
0 => Dot,
1 => Hidden,
2 => Plus,
3 => Cross,
4 => ShortLine,
);
impl CborValue for IfccadPointSize {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "DefaultFivePercent"),
            (1, "Absolute"),
            (2, "ViewportPercent"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::DefaultFivePercent => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::Absolute(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
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
            Self::ViewportPercent(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
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
            0 => Self::DefaultFivePercent,
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: f64 = b.take(0, None)?;
                b.finish()?;
                Self::Absolute(value_0)
            }
            2 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: f64 = b.take(0, None)?;
                b.finish()?;
                Self::ViewportPercent(value_0)
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::DefaultFivePercent, Self::DefaultFivePercent) => true,
            (Self::Absolute(value_0), Self::Absolute(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            (Self::ViewportPercent(value_0), Self::ViewportPercent(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            _ => false,
        }
    }
}
record!(IfccadOpaqueEntity;
0 => id: u64 = None,
1 => preservation_record_id: IfccadPreservationRecordId = None,
2 => layer_id: Option<u64> = Some(None),
3 => appearance: Option<IfccadOpaqueAppearance> = Some(None),
4 => visible: bool = None,
);
impl CborValue for IfccadColor {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Foreground"), (1, "Concrete")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Foreground => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::Concrete {
                rgb,
                indexed,
                named,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if rgb.present()
                    && (None as Option<[u8; 3]>)
                        .as_ref()
                        .is_none_or(|d| !rgb.exact(d))
                {
                    __cbor_fields += 1;
                }
                if indexed.present()
                    && (Some(None) as Option<Option<(String, u64)>>)
                        .as_ref()
                        .is_none_or(|d| !indexed.exact(d))
                {
                    __cbor_fields += 1;
                }
                if named.present()
                    && (Some(None) as Option<Option<(String, String)>>)
                        .as_ref()
                        .is_none_or(|d| !named.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if rgb.present()
                    && (None as Option<[u8; 3]>)
                        .as_ref()
                        .is_none_or(|d| !rgb.exact(d))
                {
                    w.u64(0)?;
                    rgb.encode(w)?;
                }
                if indexed.present()
                    && (Some(None) as Option<Option<(String, u64)>>)
                        .as_ref()
                        .is_none_or(|d| !indexed.exact(d))
                {
                    w.u64(1)?;
                    indexed.encode(w)?;
                }
                if named.present()
                    && (Some(None) as Option<Option<(String, String)>>)
                        .as_ref()
                        .is_none_or(|d| !named.exact(d))
                {
                    w.u64(2)?;
                    named.encode(w)?;
                }
                Ok(())
            }
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => Self::Foreground,
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "rgb"), (1, "indexed"), (2, "named")])?;
                let rgb: [u8; 3] = b.take(0, None)?;
                let indexed: Option<(String, u64)> = b.take(1, Some(None))?;
                let named: Option<(String, String)> = b.take(2, Some(None))?;
                b.finish()?;
                Self::Concrete {
                    rgb,
                    indexed,
                    named,
                }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::Foreground, Self::Foreground) => true,
            (
                Self::Concrete {
                    rgb,
                    indexed,
                    named,
                },
                Self::Concrete {
                    rgb: rgb_other,
                    indexed: indexed_other,
                    named: named_other,
                },
            ) => rgb.exact(rgb_other) && indexed.exact(indexed_other) && named.exact(named_other),
            _ => false,
        }
    }
}
record!(IfccadEntityAppearance;
0 => color: IfccadMode<IfccadColor> = Some(IfccadMode::ByLayer),
1 => opacity: IfccadMode<f64> = Some(IfccadMode::ByLayer),
2 => line_pattern: IfccadMode<IfccadLinePatternId> = Some(IfccadMode::ByLayer),
3 => line_weight: IfccadMode<f64> = Some(IfccadMode::ByLayer),
);
unit_enum!(CoordinateLengthUnit;
0 => Unitless,
1 => Millimetre,
2 => Centimetre,
3 => Metre,
4 => Kilometre,
5 => Inch,
6 => Foot,
7 => Mile,
8 => Microinch,
9 => Mil,
10 => Yard,
11 => Angstrom,
12 => Nanometre,
13 => Micrometre,
14 => Decimetre,
15 => Decametre,
16 => Hectometre,
17 => Gigametre,
18 => AstronomicalUnit,
19 => LightYear,
20 => Parsec,
21 => UsSurveyFoot,
22 => UsSurveyInch,
23 => UsSurveyYard,
24 => UsSurveyMile,
);
record!(IfccadPlacement;
0 => origin: [f64;3] = None,
1 => x_axis: [f64;3] = None,
2 => y_axis: [f64;3] = None,
);
unit_enum!(IfccadLinePatternGeneration;
0 => PerSegment,
1 => Continuous,
);
record!(IfccadBlockTransform;
0 => placement: IfccadPlacement = None,
1 => rotation: f64 = Some(0.0),
2 => scale: [f64;3] = Some([1.0, 1.0, 1.0]),
);
impl CborValue for IfccadMode<IfccadColor> {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "ByLayer"), (1, "ByBlock"), (2, "Explicit")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::ByLayer => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::ByBlock => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(1)?;
                Ok(())
            }
            Self::Explicit(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
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
            0 => Self::ByLayer,
            1 => Self::ByBlock,
            2 => {
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
            (Self::ByLayer, Self::ByLayer) => true,
            (Self::ByBlock, Self::ByBlock) => true,
            (Self::Explicit(value_0), Self::Explicit(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            _ => false,
        }
    }
}
impl CborValue for IfccadMode<f64> {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "ByLayer"), (1, "ByBlock"), (2, "Explicit")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::ByLayer => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::ByBlock => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(1)?;
                Ok(())
            }
            Self::Explicit(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
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
            0 => Self::ByLayer,
            1 => Self::ByBlock,
            2 => {
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
            (Self::ByLayer, Self::ByLayer) => true,
            (Self::ByBlock, Self::ByBlock) => true,
            (Self::Explicit(value_0), Self::Explicit(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            _ => false,
        }
    }
}
impl CborValue for IfccadMode<IfccadLinePatternId> {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "ByLayer"), (1, "ByBlock"), (2, "Explicit")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::ByLayer => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(0)?;
                Ok(())
            }
            Self::ByBlock => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(1)?;
                Ok(())
            }
            Self::Explicit(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
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
            0 => Self::ByLayer,
            1 => Self::ByBlock,
            2 => {
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
            (Self::ByLayer, Self::ByLayer) => true,
            (Self::ByBlock, Self::ByBlock) => true,
            (Self::Explicit(value_0), Self::Explicit(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            _ => false,
        }
    }
}
