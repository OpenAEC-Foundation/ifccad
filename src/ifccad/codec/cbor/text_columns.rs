use super::*;
use crate::text::{MTextBackground, MTextColumnHeight, MTextColumns, MTextFill, TextPadding};
impl CborValue for MTextColumns {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[
            (0, "Static"),
            (1, "DynamicAutoHeight"),
            (2, "DynamicManualHeight"),
        ]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Static {
                count,
                column_width,
                gutter,
                column_height,
                flow_reversed,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(0)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if count.present()
                    && (None as Option<u32>)
                        .as_ref()
                        .is_none_or(|d| !count.exact(d))
                {
                    __cbor_fields += 1;
                }
                if column_width.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !column_width.exact(d))
                {
                    __cbor_fields += 1;
                }
                if gutter.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !gutter.exact(d))
                {
                    __cbor_fields += 1;
                }
                if column_height.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !column_height.exact(d))
                {
                    __cbor_fields += 1;
                }
                if flow_reversed.present()
                    && (None as Option<bool>)
                        .as_ref()
                        .is_none_or(|d| !flow_reversed.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if count.present()
                    && (None as Option<u32>)
                        .as_ref()
                        .is_none_or(|d| !count.exact(d))
                {
                    w.u64(0)?;
                    count.encode(w)?;
                }
                if column_width.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !column_width.exact(d))
                {
                    w.u64(1)?;
                    column_width.encode(w)?;
                }
                if gutter.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !gutter.exact(d))
                {
                    w.u64(2)?;
                    gutter.encode(w)?;
                }
                if column_height.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !column_height.exact(d))
                {
                    w.u64(3)?;
                    column_height.encode(w)?;
                }
                if flow_reversed.present()
                    && (None as Option<bool>)
                        .as_ref()
                        .is_none_or(|d| !flow_reversed.exact(d))
                {
                    w.u64(4)?;
                    flow_reversed.encode(w)?;
                }
                Ok(())
            }
            Self::DynamicAutoHeight {
                column_width,
                gutter,
                column_height,
                current_column_count,
                flow_reversed,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if column_width.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !column_width.exact(d))
                {
                    __cbor_fields += 1;
                }
                if gutter.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !gutter.exact(d))
                {
                    __cbor_fields += 1;
                }
                if column_height.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !column_height.exact(d))
                {
                    __cbor_fields += 1;
                }
                if current_column_count.present()
                    && (None as Option<u32>)
                        .as_ref()
                        .is_none_or(|d| !current_column_count.exact(d))
                {
                    __cbor_fields += 1;
                }
                if flow_reversed.present()
                    && (None as Option<bool>)
                        .as_ref()
                        .is_none_or(|d| !flow_reversed.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if column_width.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !column_width.exact(d))
                {
                    w.u64(0)?;
                    column_width.encode(w)?;
                }
                if gutter.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !gutter.exact(d))
                {
                    w.u64(1)?;
                    gutter.encode(w)?;
                }
                if column_height.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !column_height.exact(d))
                {
                    w.u64(2)?;
                    column_height.encode(w)?;
                }
                if current_column_count.present()
                    && (None as Option<u32>)
                        .as_ref()
                        .is_none_or(|d| !current_column_count.exact(d))
                {
                    w.u64(3)?;
                    current_column_count.encode(w)?;
                }
                if flow_reversed.present()
                    && (None as Option<bool>)
                        .as_ref()
                        .is_none_or(|d| !flow_reversed.exact(d))
                {
                    w.u64(4)?;
                    flow_reversed.encode(w)?;
                }
                Ok(())
            }
            Self::DynamicManualHeight {
                column_width,
                gutter,
                column_heights,
                flow_reversed,
            } => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(2)?;
                w.u64(1)?;
                let mut __cbor_fields = 0_u64;
                if column_width.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !column_width.exact(d))
                {
                    __cbor_fields += 1;
                }
                if gutter.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !gutter.exact(d))
                {
                    __cbor_fields += 1;
                }
                if column_heights.present()
                    && (None as Option<Vec<MTextColumnHeight>>)
                        .as_ref()
                        .is_none_or(|d| !column_heights.exact(d))
                {
                    __cbor_fields += 1;
                }
                if flow_reversed.present()
                    && (None as Option<bool>)
                        .as_ref()
                        .is_none_or(|d| !flow_reversed.exact(d))
                {
                    __cbor_fields += 1;
                }
                w.map(__cbor_fields)?;
                if column_width.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !column_width.exact(d))
                {
                    w.u64(0)?;
                    column_width.encode(w)?;
                }
                if gutter.present()
                    && (None as Option<f64>)
                        .as_ref()
                        .is_none_or(|d| !gutter.exact(d))
                {
                    w.u64(1)?;
                    gutter.encode(w)?;
                }
                if column_heights.present()
                    && (None as Option<Vec<MTextColumnHeight>>)
                        .as_ref()
                        .is_none_or(|d| !column_heights.exact(d))
                {
                    w.u64(2)?;
                    column_heights.encode(w)?;
                }
                if flow_reversed.present()
                    && (None as Option<bool>)
                        .as_ref()
                        .is_none_or(|d| !flow_reversed.exact(d))
                {
                    w.u64(3)?;
                    flow_reversed.encode(w)?;
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
                    &[
                        (0, "count"),
                        (1, "column_width"),
                        (2, "gutter"),
                        (3, "column_height"),
                        (4, "flow_reversed"),
                    ],
                )?;
                let count: u32 = b.take(0, None)?;
                let column_width: f64 = b.take(1, None)?;
                let gutter: f64 = b.take(2, None)?;
                let column_height: f64 = b.take(3, None)?;
                let flow_reversed: bool = b.take(4, None)?;
                b.finish()?;
                Self::Static {
                    count,
                    column_width,
                    gutter,
                    column_height,
                    flow_reversed,
                }
            }
            1 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(
                    &body,
                    &[
                        (0, "column_width"),
                        (1, "gutter"),
                        (2, "column_height"),
                        (3, "current_column_count"),
                        (4, "flow_reversed"),
                    ],
                )?;
                let column_width: f64 = b.take(0, None)?;
                let gutter: f64 = b.take(1, None)?;
                let column_height: f64 = b.take(2, None)?;
                let current_column_count: u32 = b.take(3, None)?;
                let flow_reversed: bool = b.take(4, None)?;
                b.finish()?;
                Self::DynamicAutoHeight {
                    column_width,
                    gutter,
                    column_height,
                    current_column_count,
                    flow_reversed,
                }
            }
            2 => {
                let body = f.span(1)?;
                let mut b = RecordFields::new(
                    &body,
                    &[
                        (0, "column_width"),
                        (1, "gutter"),
                        (2, "column_heights"),
                        (3, "flow_reversed"),
                    ],
                )?;
                let column_width: f64 = b.take(0, None)?;
                let gutter: f64 = b.take(1, None)?;
                let column_heights: Vec<MTextColumnHeight> = b.take(2, None)?;
                let flow_reversed: bool = b.take(3, None)?;
                b.finish()?;
                Self::DynamicManualHeight {
                    column_width,
                    gutter,
                    column_heights,
                    flow_reversed,
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
                Self::Static {
                    count,
                    column_width,
                    gutter,
                    column_height,
                    flow_reversed,
                },
                Self::Static {
                    count: count_other,
                    column_width: column_width_other,
                    gutter: gutter_other,
                    column_height: column_height_other,
                    flow_reversed: flow_reversed_other,
                },
            ) => {
                count.exact(count_other)
                    && column_width.exact(column_width_other)
                    && gutter.exact(gutter_other)
                    && column_height.exact(column_height_other)
                    && flow_reversed.exact(flow_reversed_other)
            }
            (
                Self::DynamicAutoHeight {
                    column_width,
                    gutter,
                    column_height,
                    current_column_count,
                    flow_reversed,
                },
                Self::DynamicAutoHeight {
                    column_width: column_width_other,
                    gutter: gutter_other,
                    column_height: column_height_other,
                    current_column_count: current_column_count_other,
                    flow_reversed: flow_reversed_other,
                },
            ) => {
                column_width.exact(column_width_other)
                    && gutter.exact(gutter_other)
                    && column_height.exact(column_height_other)
                    && current_column_count.exact(current_column_count_other)
                    && flow_reversed.exact(flow_reversed_other)
            }
            (
                Self::DynamicManualHeight {
                    column_width,
                    gutter,
                    column_heights,
                    flow_reversed,
                },
                Self::DynamicManualHeight {
                    column_width: column_width_other,
                    gutter: gutter_other,
                    column_heights: column_heights_other,
                    flow_reversed: flow_reversed_other,
                },
            ) => {
                column_width.exact(column_width_other)
                    && gutter.exact(gutter_other)
                    && column_heights.exact(column_heights_other)
                    && flow_reversed.exact(flow_reversed_other)
            }
            _ => false,
        }
    }
}
record!(MTextBackground<IfccadColor>;
0 => fill: MTextFill<IfccadColor> = None,
1 => padding: TextPadding = None,
2 => opacity: f64 = None,
3 => frame: bool = None,
);
impl CborValue for MTextFill<IfccadColor> {
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
            Self::Color(value_0) => {
                w.map(2)?;
                w.u64(0)?;
                w.u64(1)?;
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
            Self::Canvas => {
                w.map(1)?;
                w.u64(0)?;
                w.u64(2)?;
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
                let value_0: IfccadColor = b.take(0, None)?;
                b.finish()?;
                Self::Color(value_0)
            }
            2 => Self::Canvas,
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (Self::None, Self::None) => true,
            (Self::Color(value_0), Self::Color(value_0_other)) => value_0.exact(value_0_other),
            (Self::Canvas, Self::Canvas) => true,
            _ => false,
        }
    }
}
impl CborValue for TextPadding {
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
impl CborValue for MTextColumnHeight {
    #[cfg(test)]
    fn registered_codes() -> &'static [(u64, &'static str)] {
        &[(0, "Fixed"), (1, "Auto")]
    }
    fn encode(&self, w: &mut CborWriter) -> Result<(), IfccadCborEncodeError> {
        match self {
            Self::Fixed { distance } => {
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
            Self::Auto => {
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
                let mut b = RecordFields::new(&body, &[(0, "distance")])?;
                let distance: f64 = b.take(0, None)?;
                b.finish()?;
                Self::Fixed { distance }
            }
            1 => Self::Auto,
            _ => return Err(r.error("unknown variant code")),
        };
        f.finish()?;
        Ok(value)
    }
    fn exact(&self, o: &Self) -> bool {
        match (self, o) {
            (
                Self::Fixed { distance },
                Self::Fixed {
                    distance: distance_other,
                },
            ) => distance.exact(distance_other),
            (Self::Auto, Self::Auto) => true,
            _ => false,
        }
    }
}
