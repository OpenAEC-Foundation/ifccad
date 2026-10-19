use super::*;
use crate::line_pattern::{
    LinePatternElement, LinePatternRotation, LinePatternTransform, SymbolSource,
};
record!(IfccadSymbolSource;
0 => id: IfccadSymbolSourceId = None,
1 => source: SymbolSource = None,
);
impl CborValue for IfccadSymbolSourceId {
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
impl CborValue for SymbolSource {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Shx")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Shx { file_name } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if file_name.present()
                    && (None as Option<String>)
                        .as_ref()
                        .is_none_or(|d| !file_name.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if file_name.present()
                    && (None as Option<String>)
                        .as_ref()
                        .is_none_or(|d| !file_name.exact(d))
                {
                    w.u64(0)?;
                    file_name.encode(w)?;
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
                let mut b = RecordFields::new(&body, &[(0, "file_name")])?;
                let file_name: String = b.take(0, None)?;
                b.finish()?;
                Self::Shx { file_name }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (
                Self::Shx { file_name },
                Self::Shx {
                    file_name: file_name_other,
                },
            ) => file_name.exact(file_name_other),
        }
    }
}
record!(LinePatternElement<IfccadLinePatternMarker>;
0 => length: f64 = None,
1 => marker: Option<IfccadLinePatternMarker> = Some(None),
);
impl CborValue for IfccadLinePatternMarker {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Text"), (1, "Symbol")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Text {
                text,
                text_style_id,
                transform,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if text.present()
                    && (None as Option<String>)
                        .as_ref()
                        .is_none_or(|d| !text.exact(d))
                {
                    __cbor_fields += 1;
                }
                if text_style_id.present()
                    && (None as Option<IfccadTextStyleId>)
                        .as_ref()
                        .is_none_or(|d| !text_style_id.exact(d))
                {
                    __cbor_fields += 1;
                }
                if transform.present()
                    && (None as Option<LinePatternTransform>)
                        .as_ref()
                        .is_none_or(|d| !transform.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if text.present()
                    && (None as Option<String>)
                        .as_ref()
                        .is_none_or(|d| !text.exact(d))
                {
                    w.u64(0)?;
                    text.encode(w)?;
                }
                if text_style_id.present()
                    && (None as Option<IfccadTextStyleId>)
                        .as_ref()
                        .is_none_or(|d| !text_style_id.exact(d))
                {
                    w.u64(1)?;
                    text_style_id.encode(w)?;
                }
                if transform.present()
                    && (None as Option<LinePatternTransform>)
                        .as_ref()
                        .is_none_or(|d| !transform.exact(d))
                {
                    w.u64(2)?;
                    transform.encode(w)?;
                }
                Ok(())
            }
            Self::Symbol {
                symbol_source_id,
                shape_number,
                transform,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if symbol_source_id.present()
                    && (None as Option<IfccadSymbolSourceId>)
                        .as_ref()
                        .is_none_or(|d| !symbol_source_id.exact(d))
                {
                    __cbor_fields += 1;
                }
                if shape_number.present()
                    && (None as Option<u32>)
                        .as_ref()
                        .is_none_or(|d| !shape_number.exact(d))
                {
                    __cbor_fields += 1;
                }
                if transform.present()
                    && (None as Option<LinePatternTransform>)
                        .as_ref()
                        .is_none_or(|d| !transform.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if symbol_source_id.present()
                    && (None as Option<IfccadSymbolSourceId>)
                        .as_ref()
                        .is_none_or(|d| !symbol_source_id.exact(d))
                {
                    w.u64(0)?;
                    symbol_source_id.encode(w)?;
                }
                if shape_number.present()
                    && (None as Option<u32>)
                        .as_ref()
                        .is_none_or(|d| !shape_number.exact(d))
                {
                    w.u64(1)?;
                    shape_number.encode(w)?;
                }
                if transform.present()
                    && (None as Option<LinePatternTransform>)
                        .as_ref()
                        .is_none_or(|d| !transform.exact(d))
                {
                    w.u64(2)?;
                    transform.encode(w)?;
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
                let mut b = RecordFields::new(
                    &body,
                    &[(0, "text"), (1, "text_style_id"), (2, "transform")],
                )?;
                let text: String = b.take(0, None)?;
                let text_style_id: IfccadTextStyleId = b.take(1, None)?;
                let transform: LinePatternTransform = b.take(2, None)?;
                b.finish()?;
                Self::Text {
                    text,
                    text_style_id,
                    transform,
                }
            }
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(
                    &body,
                    &[
                        (0, "symbol_source_id"),
                        (1, "shape_number"),
                        (2, "transform"),
                    ],
                )?;
                let symbol_source_id: IfccadSymbolSourceId = b.take(0, None)?;
                let shape_number: u32 = b.take(1, None)?;
                let transform: LinePatternTransform = b.take(2, None)?;
                b.finish()?;
                Self::Symbol {
                    symbol_source_id,
                    shape_number,
                    transform,
                }
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (
                Self::Text {
                    text,
                    text_style_id,
                    transform,
                },
                Self::Text {
                    text: text_other,
                    text_style_id: text_style_id_other,
                    transform: transform_other,
                },
            ) => {
                text.exact(text_other)
                    && text_style_id.exact(text_style_id_other)
                    && transform.exact(transform_other)
            }
            (
                Self::Symbol {
                    symbol_source_id,
                    shape_number,
                    transform,
                },
                Self::Symbol {
                    symbol_source_id: symbol_source_id_other,
                    shape_number: shape_number_other,
                    transform: transform_other,
                },
            ) => {
                symbol_source_id.exact(symbol_source_id_other)
                    && shape_number.exact(shape_number_other)
                    && transform.exact(transform_other)
            }
            _ => false,
        }
    }
}
record!(LinePatternTransform;
0 => scale: f64 = Some(1.0),
1 => offset: [f64;2] = Some([0.0, 0.0]),
2 => rotation: LinePatternRotation = Some(LinePatternRotation::Relative(0.0)),
);
impl CborValue for LinePatternRotation {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Relative"), (1, "Absolute")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Relative(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
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
        }
    }
    fn decode(r: &mut CborReader<'_>) -> Result<Self, IfccadCborReadError> {
        let mut f = RecordFields::new(r, &[(0, "kind"), (1, "body")])?;
        let code: u64 = f.take(0, None)?;
        let value = match code {
            0 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: f64 = b.take(0, None)?;
                b.finish()?;
                Self::Relative(value_0)
            }
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(&body, &[(0, "value_0")])?;
                let value_0: f64 = b.take(0, None)?;
                b.finish()?;
                Self::Absolute(value_0)
            }
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::Relative(value_0), Self::Relative(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            (Self::Absolute(value_0), Self::Absolute(value_0_other)) => {
                value_0.exact(value_0_other)
            }
            _ => false,
        }
    }
}
